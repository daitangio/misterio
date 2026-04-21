use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct HostConfig {
    docker_context: Option<String>,
    docker_host: Option<String>,
}

enum Parsed<T> {
    Run(T),
    Done,
}

pub fn misterio_entry() -> ExitCode {
    match parse_misterio_args(env::args_os().skip(1)) {
        Ok(Parsed::Run(args)) => exit_code(run_misterio(args)),
        Ok(Parsed::Done) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

pub fn misterio_add_entry() -> ExitCode {
    match parse_add_args(env::args_os().skip(1)) {
        Ok(Parsed::Run(args)) => exit_code(run_add(args)),
        Ok(Parsed::Done) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

pub fn misterio_mv_entry() -> ExitCode {
    match parse_mv_args(env::args_os().skip(1)) {
        Ok(Parsed::Run(args)) => exit_code(run_mv(args)),
        Ok(Parsed::Done) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

pub fn misterio_rm_entry() -> ExitCode {
    match parse_rm_args(env::args_os().skip(1)) {
        Ok(Parsed::Run(args)) => exit_code(run_rm(args)),
        Ok(Parsed::Done) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

fn exit_code(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

#[derive(Debug)]
struct MisterioArgs {
    home: PathBuf,
    hosts: Vec<String>,
    list_flag: bool,
    single_role: Option<String>,
    docker_command: Vec<String>,
}

#[derive(Debug)]
struct AddArgs {
    home: PathBuf,
    build_flag: bool,
    target_host: String,
    role_list: Vec<String>,
}

#[derive(Debug)]
struct MoveArgs {
    home: PathBuf,
    role: String,
    source_host: String,
    destination_host: String,
}

#[derive(Debug)]
struct RemoveArgs {
    home: PathBuf,
    source_host: String,
    role_list: Vec<String>,
}

fn parse_misterio_args(args: impl Iterator<Item = OsString>) -> Result<Parsed<MisterioArgs>, String> {
    let mut home = default_home();
    let mut hosts = Vec::new();
    let mut list_flag = false;
    let mut single_role = env::var("MISTERIO_SINGLE_ROLE").ok();
    let mut docker_command = Vec::new();
    let mut collecting_command = false;

    let mut args = args.map(os_to_string).peekable();
    while let Some(arg) = args.next() {
        if collecting_command {
            docker_command.push(arg);
            continue;
        }

        match arg.as_str() {
            "--help" => {
                print!("{}", misterio_help());
                return Ok(Parsed::Done);
            }
            "--version" | "-V" => {
                println!("misterio {VERSION}");
                return Ok(Parsed::Done);
            }
            "--" => {
                collecting_command = true;
            }
            "--list" => {
                list_flag = true;
            }
            "--no-list" => {
                list_flag = false;
            }
            "--home" => {
                home = PathBuf::from(next_value(&mut args, "--home")?);
            }
            "--misterio-host" | "-h" => {
                hosts.push(next_value(&mut args, "--misterio-host")?);
            }
            "--single-role" | "-r" => {
                single_role = Some(next_value(&mut args, "--single-role")?);
            }
            _ if arg.starts_with("--home=") => {
                home = PathBuf::from(value_after_equals(&arg, "--home")?);
            }
            _ if arg.starts_with("--misterio-host=") => {
                hosts.push(value_after_equals(&arg, "--misterio-host")?);
            }
            _ if arg.starts_with("--single-role=") => {
                single_role = Some(value_after_equals(&arg, "--single-role")?);
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown option for misterio: {arg}"));
            }
            _ => {
                collecting_command = true;
                docker_command.push(arg);
            }
        }
    }

    Ok(Parsed::Run(MisterioArgs {
        home,
        hosts,
        list_flag,
        single_role,
        docker_command,
    }))
}

fn parse_add_args(args: impl Iterator<Item = OsString>) -> Result<Parsed<AddArgs>, String> {
    let mut home = default_home();
    let mut build_flag = false;
    let mut positional = Vec::new();

    let mut args = args.map(os_to_string).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                print!("{}", add_help());
                return Ok(Parsed::Done);
            }
            "--version" | "-V" => {
                println!("misterio-add {VERSION}");
                return Ok(Parsed::Done);
            }
            "--home" => {
                home = PathBuf::from(next_value(&mut args, "--home")?);
            }
            "--build" => {
                build_flag = true;
            }
            "--no-build" => {
                build_flag = false;
            }
            _ if arg.starts_with("--home=") => {
                home = PathBuf::from(value_after_equals(&arg, "--home")?);
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown option for misterio-add: {arg}"));
            }
            _ => positional.push(arg),
        }
    }

    if positional.len() < 2 {
        return Err("usage: misterio-add [--home PATH] [--build] TARGET_HOST ROLE...".to_string());
    }

    let target_host = positional.remove(0);
    Ok(Parsed::Run(AddArgs {
        home,
        build_flag,
        target_host,
        role_list: positional,
    }))
}

fn parse_mv_args(args: impl Iterator<Item = OsString>) -> Result<Parsed<MoveArgs>, String> {
    let mut home = default_home();
    let mut positional = Vec::new();

    let mut args = args.map(os_to_string).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                print!("{}", mv_help());
                return Ok(Parsed::Done);
            }
            "--version" | "-V" => {
                println!("misterio-mv {VERSION}");
                return Ok(Parsed::Done);
            }
            "--home" => {
                home = PathBuf::from(next_value(&mut args, "--home")?);
            }
            _ if arg.starts_with("--home=") => {
                home = PathBuf::from(value_after_equals(&arg, "--home")?);
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown option for misterio-mv: {arg}"));
            }
            _ => positional.push(arg),
        }
    }

    if positional.len() != 3 {
        return Err(
            "usage: misterio-mv [--home PATH] ROLE SOURCE_HOST DESTINATION_HOST".to_string(),
        );
    }

    Ok(Parsed::Run(MoveArgs {
        home,
        role: positional.remove(0),
        source_host: positional.remove(0),
        destination_host: positional.remove(0),
    }))
}

fn parse_rm_args(args: impl Iterator<Item = OsString>) -> Result<Parsed<RemoveArgs>, String> {
    let mut home = default_home();
    let mut positional = Vec::new();

    let mut args = args.map(os_to_string).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                print!("{}", rm_help());
                return Ok(Parsed::Done);
            }
            "--version" | "-V" => {
                println!("misterio-rm {VERSION}");
                return Ok(Parsed::Done);
            }
            "--home" => {
                home = PathBuf::from(next_value(&mut args, "--home")?);
            }
            _ if arg.starts_with("--home=") => {
                home = PathBuf::from(value_after_equals(&arg, "--home")?);
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown option for misterio-rm: {arg}"));
            }
            _ => positional.push(arg),
        }
    }

    if positional.len() < 2 {
        return Err("usage: misterio-rm [--home PATH] SOURCE_HOST ROLE...".to_string());
    }

    let source_host = positional.remove(0);
    Ok(Parsed::Run(RemoveArgs {
        home,
        source_host,
        role_list: positional,
    }))
}

fn run_misterio(args: MisterioArgs) -> Result<(), String> {
    run_misterio_command(
        &args.home,
        &args.hosts,
        args.list_flag,
        args.single_role.as_deref(),
        &args.docker_command,
    )
}

fn run_add(args: AddArgs) -> Result<(), String> {
    misterio_add(&args.home, &args.target_host, &args.role_list, args.build_flag)
}

fn run_mv(args: MoveArgs) -> Result<(), String> {
    misterio_mv(
        &args.home,
        &args.role,
        &args.source_host,
        &args.destination_host,
    )
}

fn run_rm(args: RemoveArgs) -> Result<(), String> {
    misterio_rm(&args.home, &args.source_host, &args.role_list)
}

pub fn run_misterio_command(
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
            println!("Roles for {host}");
            let host_path = home.join("hosts").join(&host);
            match fs::read_dir(host_path) {
                Ok(entries) => {
                    let mut names = collect_entry_names(entries)?;
                    names.sort();
                    for name in names {
                        println!("\t{name}");
                    }
                }
                Err(_) => {
                    println!("No roles for {host}");
                }
            }
        }
        return Ok(());
    }

    for host in hosts {
        if host.contains("localhost") {
            return Err("Use misterio.toml not localhost for special needs".to_string());
        }

        env::set_var("DOCKER_HOST", format!("ssh://{host}"));
        load_misterio_config(&home.join("hosts").join("misterio.toml"), &host)?;

        let host_path = home.join("hosts").join(&host);
        let entries = fs::read_dir(&host_path)
            .map_err(|error| format!("failed to read {}: {error}", host_path.display()))?;
        let mut names = collect_entry_names(entries)?;
        names.sort();

        for filename in names {
            if !filename.ends_with(".env") {
                println!("WARN Ignored not-env file: {filename}");
                continue;
            }

            if let Some(role) = single_role {
                if !filename.contains(role) {
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

    let mut base_port = 7000_u16;
    for role in role_list {
        let target_dir = home.join("hosts").join(target_host);
        fs::create_dir_all(&target_dir)
            .map_err(|error| format!("failed to create {}: {error}", target_dir.display()))?;

        let env_path = target_dir.join(format!("{role}.env"));
        if env_path.exists() {
            return Err(format!("FATAL: Role {role} already exists as {}", env_path.display()));
        }

        let mut file = fs::File::create(&env_path)
            .map_err(|error| format!("failed to create {}: {error}", env_path.display()))?;
        write_prop("MISTERIO_CREATION_USER", &env::var("USER").unwrap_or_else(|_| "unknown".to_string()), &mut file)?;
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

fn process_role(home: &Path, env_full_path: &Path, docker_command: &[String]) -> Result<(), String> {
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
            alias => return Err(format!("Unknown alias:{alias}")),
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
        .map_err(|error| format!("failed to start docker compose in {}: {error}", role_dir.display()))?;

    if !status.success() {
        println!(
            "{docker_host}::{role_name} Failed with return code {}",
            status.code().unwrap_or(1)
        );
    }

    Ok(())
}

fn verify_misterio_home(home: &Path) -> Result<(), String> {
    let mut errors = 0;
    for required in ["hosts", "roles"] {
        let path = home.join(required);
        if !path.is_dir() {
            println!("FATAL: Missed required directory {}", path.display());
            errors += 1;
        }
    }

    if errors == 0 {
        Ok(())
    } else {
        Err(format!("home dir has {errors} validation errors"))
    }
}

fn list_hosts(home: &Path) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(home.join("hosts"))
        .map_err(|error| format!("failed to read {}: {error}", home.join("hosts").display()))?;
    let mut hosts = Vec::new();

    for entry in entries {
        let entry = entry.map_err(io_error)?;
        let name = entry.file_name();
        let name = os_to_string(name);
        if name != "misterio.toml" {
            hosts.push(name);
        }
    }

    hosts.sort();
    Ok(hosts)
}

fn load_misterio_config(config_path: &Path, host_target: &str) -> Result<(), String> {
    if !config_path.exists() {
        println!(
            "{} not defined (see documentation for happy features)",
            config_path.display()
        );
        return Ok(());
    }

    let raw = fs::read_to_string(config_path)
        .map_err(|error| format!("failed to read {}: {error}", config_path.display()))?;
    let config = parse_host_config(&raw, host_target);

    if config == HostConfig::default() {
        return Ok(());
    }

    println!("Config for {host_target}: {config:?}");
    if let Some(context) = config.docker_context {
        env::set_var("DOCKER_CONTEXT", context);
    }
    if let Some(host) = config.docker_host {
        env::set_var("DOCKER_HOST", host);
    }

    Ok(())
}

fn parse_host_config(raw: &str, host_target: &str) -> HostConfig {
    let mut current_section = String::new();
    let mut current_subsection = String::new();
    let mut config = HostConfig::default();

    for line in raw.lines() {
        let line = strip_comment(line).trim();
        if line.is_empty() {
            continue;
        }

        if let Some(section) = line.strip_prefix('[').and_then(|value| value.strip_suffix(']')) {
            current_section.clear();
            current_subsection.clear();

            let mut parts = section.split('.');
            current_section.push_str(parts.next().unwrap_or_default().trim());
            current_subsection.push_str(parts.next().unwrap_or_default().trim());
            continue;
        }

        if current_section != host_target || current_subsection != "docker" {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = parse_toml_string(value.trim());

        match key {
            "context" => config.docker_context = Some(value),
            "host" => config.docker_host = Some(value),
            _ => {}
        }
    }

    config
}

fn strip_comment(line: &str) -> &str {
    let mut in_quotes = false;

    for (index, ch) in line.char_indices() {
        match ch {
            '"' => in_quotes = !in_quotes,
            '#' if !in_quotes => return &line[..index],
            _ => {}
        }
    }

    line
}

fn parse_toml_string(value: &str) -> String {
    if let Some(inner) = value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
        inner.replace("\\\"", "\"")
    } else {
        value.to_string()
    }
}

fn role_name_from_env(env_file: &str) -> String {
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

fn determine_instance_name(role: &str) -> String {
    if let Some((name, instance)) = role.split_once('@') {
        format!("{name}_{instance}").to_lowercase()
    } else {
        role.to_lowercase()
    }
}

fn determine_fixed_port(role: &str, base_port: u16) -> u16 {
    let mut idx = 0_u16;
    if let Some((_, instance)) = role.split_once('@') {
        if let Ok(value) = u32::from_str_radix(instance, 36) {
            let digits = value.to_string();
            let prefix = if digits.len() > 2 { &digits[..2] } else { &digits };
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
        names.push(os_to_string(entry.file_name()));
    }
    Ok(names)
}

fn next_value<I>(args: &mut std::iter::Peekable<I>, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn value_after_equals(arg: &str, flag: &str) -> Result<String, String> {
    arg.split_once('=')
        .map(|(_, value)| value.to_string())
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn os_to_string(value: OsString) -> String {
    value.to_string_lossy().into_owned()
}

fn default_home() -> PathBuf {
    if let Ok(home) = env::var("MISTERIO_HOME") {
        return PathBuf::from(home);
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
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

fn misterio_help() -> &'static str {
    "misterio\n\
\n\
Minimal docker compose orchestration for a set of hosts.\n\
\n\
Usage:\n\
  misterio [OPTIONS] [--] [DOCKER_COMMAND...]\n\
\n\
Options:\n\
  --home PATH\n\
  -h, --misterio-host HOST   Repeat to target multiple hosts\n\
  --list / --no-list\n\
  -r, --single-role ROLE\n\
  -V, --version\n\
  --help\n\
\n\
Aliases:\n\
  @rebuild   Run: down, then up --build -d\n\
  @upgrade   Run: pull, down, then up --build -d\n"
}

fn add_help() -> &'static str {
    "misterio-add\n\
\n\
Usage:\n\
  misterio-add [OPTIONS] TARGET_HOST ROLE...\n\
\n\
Options:\n\
  --home PATH\n\
  --build / --no-build\n\
  -V, --version\n\
  --help\n"
}

fn mv_help() -> &'static str {
    "misterio-mv\n\
\n\
Usage:\n\
  misterio-mv [OPTIONS] ROLE SOURCE_HOST DESTINATION_HOST\n\
\n\
Options:\n\
  --home PATH\n\
  -V, --version\n\
  --help\n"
}

fn rm_help() -> &'static str {
    "misterio-rm\n\
\n\
Usage:\n\
  misterio-rm [OPTIONS] SOURCE_HOST ROLE...\n\
\n\
Options:\n\
  --home PATH\n\
  -V, --version\n\
  --help\n"
}

#[cfg(test)]
mod tests {
    use super::{determine_fixed_port, determine_instance_name, parse_host_config, role_name_from_env};

    #[test]
    fn instance_names_match_previous_rules() {
        assert_eq!(determine_instance_name("pgvector"), "pgvector");
        assert_eq!(determine_instance_name("pgvector@1"), "pgvector_1");
    }

    #[test]
    fn fixed_port_stays_stable_for_instances() {
        assert_eq!(determine_fixed_port("pgvector", 7000), 7008);
        assert_eq!(determine_fixed_port("pgvector@1", 7000), 7011);
    }

    #[test]
    fn role_name_ignores_instance_suffix() {
        assert_eq!(role_name_from_env("pgvector.env"), "pgvector");
        assert_eq!(role_name_from_env("pgvector@2.env"), "pgvector");
    }

    #[test]
    fn minimal_host_config_parser_reads_docker_block() {
        let raw = r#"
        [alice.docker]
        context = "orbstack"
        host = "ssh://alice"

        [bob.docker]
        host = "ssh://bob"
        "#;

        let config = parse_host_config(raw, "alice");
        assert_eq!(config.docker_context.as_deref(), Some("orbstack"));
        assert_eq!(config.docker_host.as_deref(), Some("ssh://alice"));
    }
}
