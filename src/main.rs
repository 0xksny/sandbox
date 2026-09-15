mod app;
mod assets;
mod cli;
mod config;
mod sandbox;
mod session;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {:#}", error);
            ExitCode::FAILURE
        }
    }
}
