mod app;
mod cli;
mod config;

#[cfg(test)]
mod tests;

use std::env;
use std::process::ExitCode;

use app::{run_add, run_misterio, run_mv, run_rm};
use cli::{parse_add_args, parse_misterio_args, parse_mv_args, parse_rm_args, Parsed};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn misterio_entry() -> ExitCode {
    handle_parsed(parse_misterio_args(env::args_os().skip(1)), run_misterio)
}

pub fn misterio_add_entry() -> ExitCode {
    handle_parsed(parse_add_args(env::args_os().skip(1)), run_add)
}

pub fn misterio_mv_entry() -> ExitCode {
    handle_parsed(parse_mv_args(env::args_os().skip(1)), run_mv)
}

pub fn misterio_rm_entry() -> ExitCode {
    handle_parsed(parse_rm_args(env::args_os().skip(1)), run_rm)
}

fn handle_parsed<T>(
    parsed: Result<Parsed<T>, String>,
    run: impl FnOnce(T) -> Result<(), String>,
) -> ExitCode {
    match parsed {
        Ok(Parsed::Run(args)) => exit_code(run(args)),
        Ok(Parsed::Done) => ExitCode::SUCCESS,
        Err(message) => error_exit(message),
    }
}

fn exit_code(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => error_exit(message),
    }
}

fn error_exit(message: String) -> ExitCode {
    eprintln!("{message}");
    ExitCode::from(1)
}
