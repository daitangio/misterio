use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use crate::VERSION;

pub(crate) enum Parsed<T> {
    Run(T),
    Done,
}

#[derive(Debug)]
pub(crate) struct MisterioArgs {
    pub(crate) home: PathBuf,
    pub(crate) hosts: Vec<String>,
    pub(crate) list_flag: bool,
    pub(crate) single_role: Option<String>,
    pub(crate) docker_command: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct AddArgs {
    pub(crate) home: PathBuf,
    pub(crate) build_flag: bool,
    pub(crate) target_host: String,
    pub(crate) role_list: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct MoveArgs {
    pub(crate) home: PathBuf,
    pub(crate) role: String,
    pub(crate) source_host: String,
    pub(crate) destination_host: String,
}

#[derive(Debug)]
pub(crate) struct RemoveArgs {
    pub(crate) home: PathBuf,
    pub(crate) source_host: String,
    pub(crate) role_list: Vec<String>,
}

pub(crate) fn parse_misterio_args(
    args: impl Iterator<Item = OsString>,
) -> Result<Parsed<MisterioArgs>, String> {
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
            "--" => collecting_command = true,
            "--list" => list_flag = true,
            "--no-list" => list_flag = false,
            "--home" => home = PathBuf::from(next_value(&mut args, "--home")?),
            "--misterio-host" | "-h" => hosts.push(next_value(&mut args, "--misterio-host")?),
            "--single-role" | "-r" => {
                single_role = Some(next_value(&mut args, "--single-role")?)
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

pub(crate) fn parse_add_args(
    args: impl Iterator<Item = OsString>,
) -> Result<Parsed<AddArgs>, String> {
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
            "--home" => home = PathBuf::from(next_value(&mut args, "--home")?),
            "--build" => build_flag = true,
            "--no-build" => build_flag = false,
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
        return Err(
            "usage: misterio-add [--home PATH] [--build] TARGET_HOST ROLE...".to_string(),
        );
    }

    let target_host = positional.remove(0);
    Ok(Parsed::Run(AddArgs {
        home,
        build_flag,
        target_host,
        role_list: positional,
    }))
}

pub(crate) fn parse_mv_args(
    args: impl Iterator<Item = OsString>,
) -> Result<Parsed<MoveArgs>, String> {
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
            "--home" => home = PathBuf::from(next_value(&mut args, "--home")?),
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

pub(crate) fn parse_rm_args(
    args: impl Iterator<Item = OsString>,
) -> Result<Parsed<RemoveArgs>, String> {
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
            "--home" => home = PathBuf::from(next_value(&mut args, "--home")?),
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
