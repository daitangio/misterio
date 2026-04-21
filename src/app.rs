use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

use crate::cli::{AddArgs, MisterioArgs, MoveArgs, RemoveArgs};
use crate::config::{load_host_config, HostConfig};

pub(crate) fn run_misterio(args: MisterioArgs) -> Result<(), String> {
    run_misterio_command(
        &args.home,
        &args.hosts,
        args.list_flag,
        args.single_role.as_deref(),
        &args.docker_command,
    )
}

pub(crate) fn run_add(args: AddArgs) -> Result<(), String> {
    misterio_add(&args.home, &args.target_host, &args.role_list, args.build_flag)
}

pub(crate) fn run_mv(args: MoveArgs) -> Result<(), String> {
    misterio_mv(
        &args.home,
        &args.role,
        &args.source_host,
        &args.destination_host,
    )
}

pub(crate) fn run_rm(args: RemoveArgs) -> Result<(), String> {
    misterio_rm(&args.home, &args.source_host, &args.role_list)
}

pub(crate) fn run_misterio_command(
    home: &Path,
    requested_hosts: &[String],
    list_flag: bool,
    single_role: Option<&str>,
    docker_command: &[String],
) -> Result<(), String> {
    verify_misterio_home(home)?;

    let hosts = if requested_hosts.is_empty() {
        list_hosts(home)?
    } else {
        requested_hosts.to_vec()
    };

    println!("HOSTS:{hosts:?} MISTERIO HOME:{}", home.display());

    if list_flag {
        for host in hosts {
            list_roles_for_host(home, &host);
        }
        return Ok(());
    }

    let config_path = home.join("hosts").join("misterio.toml");
    for host in hosts {
        if host.contains("localhost") {
            return Err("Use misterio.toml not localhost for special needs".to_string());
        }

        let config = load_host_config(&config_path, &host)?;
        apply_host_environment(&host, &config);

        if config != HostConfig::default() {
            println!("Config for {host}: {config:?}");
        }

        let host_path = home.join("hosts").join(&host);
        for filename in read_sorted_entry_names(&host_path)? {
            if !filename.ends_with(".env") {
                println!("WARN Ignored not-env file: {filename}");
                continue;
            }

            if let Some(role) = single_role {
                if !role_matches(&filename, role) {
                    continue;
                }
            }

            process_role(home, &host_path.join(filename), docker_command)?;
        }
    }

    Ok(())
}

fn misterio_add(
    home: &Path,
    target_host: &str,
    role_list: &[String],
    build_flag: bool,
) -> Result<(), String> {
    verify_misterio_home(home)?;

    let target_dir = home.join("hosts").join(target_host);
    fs::create_dir_all(&target_dir)
        .map_err(|error| format!("failed to create {}: {error}", target_dir.display()))?;

    let mut base_port = 7000_u16;
    for role in role_list {
        let env_path = target_dir.join(format!("{role}.env"));
        if env_path.exists() {
            return Err(format!("FATAL: Role {role} already exists as {}", env_path.display()));
        }

        let mut file = fs::File::create(&env_path)
            .map_err(|error| format!("failed to create {}: {error}", env_path.display()))?;
        write_prop(
            "MISTERIO_CREATION_USER",
            &env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            &mut file,
        )?;
        write_prop("MISTERIO_CREATION_DATE", &timestamp_string(), &mut file)?;

        let instance_name = determine_instance_name(role);
        let upper = instance_name.to_uppercase();
        let lower = instance_name.to_lowercase();
        write_prop(&format!("{upper}_HOME"), &format!("/opt/{lower}"), &mut file)?;

        let port = determine_fixed_port(role, base_port);
        write_prop("MISTERIO_MAGIPORT", &port.to_string(), &mut file)?;
        base_port = port.saturating_add(1);

        if build_flag {
            run_misterio_command(
                home,
                &[target_host.to_string()],
                false,
                Some(role),
                &["build".to_string()],
            )?;
        }
    }

    Ok(())
}

fn misterio_mv(
    home: &Path,
    role: &str,
    source_host: &str,
    destination_host: &str,
) -> Result<(), String> {
    run_misterio_command(
        home,
        &[source_host.to_string()],
        false,
        Some(role),
        &["down".to_string()],
    )?;

    let src = home.join("hosts").join(source_host).join(format!("{role}.env"));
    let dst = home
        .join("hosts")
        .join(destination_host)
        .join(format!("{role}.env"));
    println!("{} -> {}", src.display(), dst.display());

    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }

    fs::rename(&src, &dst).map_err(|error| {
        format!(
            "failed to move {} to {}: {error}",
            src.display(),
            dst.display()
        )
    })?;

    run_misterio_command(
        home,
        &[destination_host.to_string()],
        false,
        Some(role),
        &["up".to_string(), "-d".to_string()],
    )
}

fn misterio_rm(home: &Path, source_host: &str, role_list: &[String]) -> Result<(), String> {
    for role in role_list {
        run_misterio_command(
            home,
            &[source_host.to_string()],
            false,
            Some(role),
            &["down".to_string()],
        )?;

        let src = home.join("hosts").join(source_host).join(format!("{role}.env"));
        println!("Moving {} to the attic", src.display());

        let attic_dir = home.join("attic").join(source_host);
        fs::create_dir_all(&attic_dir)
            .map_err(|error| format!("failed to create {}: {error}", attic_dir.display()))?;
        let dst = attic_dir.join(format!("{role}.env"));
        fs::rename(&src, &dst).map_err(|error| {
            format!(
                "failed to move {} to {}: {error}",
                src.display(),
                dst.display()
            )
        })?;

        run_misterio_command(
            home,
            &[source_host.to_string()],
            false,
            None,
            &["@rebuild".to_string()],
        )?;
    }

    Ok(())
}

fn process_role(
    home: &Path,
    env_full_path: &Path,
    docker_command: &[String],
) -> Result<(), String> {
    if docker_command.len() == 1 && docker_command[0].starts_with('@') {
        match docker_command[0].as_str() {
            "@rebuild" => {
                low_level_process_role(home, env_full_path, &["down"])?;
                low_level_process_role(home, env_full_path, &["up", "--build", "-d"])?;
            }
            "@upgrade" => {
                low_level_process_role(home, env_full_path, &["pull"])?;
                low_level_process_role(home, env_full_path, &["down"])?;
                low_level_process_role(home, env_full_path, &["up", "--build", "-d"])?;
            }
            alias => return Err(format!("unknown alias: {alias}")),
        }
    } else {
        let command: Vec<&str> = docker_command.iter().map(String::as_str).collect();
        low_level_process_role(home, env_full_path, &command)?;
    }

    Ok(())
}

fn low_level_process_role(
    home: &Path,
    env_full_path: &Path,
    docker_command: &[&str],
) -> Result<(), String> {
    let env_file = env_full_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("invalid env path: {}", env_full_path.display()))?;
    let role_name = role_name_from_env(env_file);
    let role_dir = home.join("roles").join(&role_name);

    if !role_dir.is_dir() {
        return Err(format!("missing role directory {}", role_dir.display()));
    }

    let docker_host = env::var("DOCKER_HOST").unwrap_or_default();
    let printable_command = std::iter::once("docker")
        .chain(std::iter::once("compose"))
        .chain(docker_command.iter().copied())
        .collect::<Vec<_>>()
        .join(" ");
    println!("==== {docker_host} {role_name}\t-> {printable_command}");

    fs::copy(env_full_path, role_dir.join(".env")).map_err(|error| {
        format!(
            "failed to copy {} into {}: {error}",
            env_full_path.display(),
            role_dir.display()
        )
    })?;

    let status = Command::new("docker")
        .arg("compose")
        .args(docker_command)
        .current_dir(&role_dir)
        .status()
        .map_err(|error| {
            format!(
                "failed to start docker compose in {}: {error}",
                role_dir.display()
            )
        })?;

    if status.success() {
        return Ok(());
    }

    Err(format!(
        "{docker_host}::{role_name} failed with return code {}",
        status.code().unwrap_or(1)
    ))
}

fn verify_misterio_home(home: &Path) -> Result<(), String> {
    let missing = ["hosts", "roles"]
        .into_iter()
        .map(|required| home.join(required))
        .filter(|path| !path.is_dir())
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!("missing required directories: {}", missing.join(", ")))
    }
}

fn list_hosts(home: &Path) -> Result<Vec<String>, String> {
    let mut hosts = read_sorted_entry_names(&home.join("hosts"))?;
    hosts.retain(|name| name != "misterio.toml");
    Ok(hosts)
}

fn list_roles_for_host(home: &Path, host: &str) {
    println!("Roles for {host}");
    let host_path = home.join("hosts").join(host);

    match read_sorted_entry_names(&host_path) {
        Ok(names) => {
            for name in names {
                println!("\t{name}");
            }
        }
        Err(_) => println!("No roles for {host}"),
    }
}

fn read_sorted_entry_names(path: &Path) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let mut names = collect_entry_names(entries)?;
    names.sort();
    Ok(names)
}

fn apply_host_environment(host: &str, config: &HostConfig) {
    env::remove_var("DOCKER_CONTEXT");
    env::set_var("DOCKER_HOST", format!("ssh://{host}"));

    if let Some(context) = config.docker_context.as_deref() {
        env::set_var("DOCKER_CONTEXT", context);
    }

    if let Some(remote_host) = config.docker_host.as_deref() {
        env::set_var("DOCKER_HOST", remote_host);
    }
}

pub(crate) fn role_matches(env_file: &str, target_role: &str) -> bool {
    env_file.strip_suffix(".env") == Some(target_role)
        || role_name_from_env(env_file) == target_role
}

pub(crate) fn role_name_from_env(env_file: &str) -> String {
    if let Some((role, _)) = env_file.split_once('@') {
        role.to_string()
    } else {
        env_file.trim_end_matches(".env").to_string()
    }
}

fn write_prop(key: &str, value: &str, file: &mut fs::File) -> Result<(), String> {
    let rendered = if value.contains(' ') {
        format!("{}=\"{}\"", key.to_uppercase(), value)
    } else {
        format!("{}={value}", key.to_uppercase())
    };
    writeln!(file, "{rendered}").map_err(io_error)?;
    println!("Defining:: {rendered}");
    Ok(())
}

pub(crate) fn determine_instance_name(role: &str) -> String {
    if let Some((name, instance)) = role.split_once('@') {
        format!("{name}_{instance}").to_lowercase()
    } else {
        role.to_lowercase()
    }
}

pub(crate) fn determine_fixed_port(role: &str, base_port: u16) -> u16 {
    let mut idx = 0_u16;
    if let Some((_, instance)) = role.split_once('@') {
        if let Ok(value) = u32::from_str_radix(instance, 36) {
            let digits = value.to_string();
            let prefix = if digits.len() > 2 {
                &digits[..2]
            } else {
                &digits
            };
            idx = prefix.parse::<u16>().unwrap_or(0);
        }
    }

    base_port
        .saturating_add(role.len() as u16)
        .saturating_add(idx)
}

fn collect_entry_names(entries: fs::ReadDir) -> Result<Vec<String>, String> {
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(io_error)?;
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    Ok(names)
}

fn timestamp_string() -> String {
    match Command::new("date").args(["-u", "+%Y-%m-%d %H:%M:%S"]).output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "1970-01-01 00:00:00".to_string(),
    }
}

fn io_error(error: io::Error) -> String {
    error.to_string()
}
