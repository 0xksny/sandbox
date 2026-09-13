use clap::{Args, Parser, Subcommand};
use std::{fs, io, path::PathBuf};

const NAME: &'static str = "sandbox";

#[derive(Debug, Parser)]
#[command(name = NAME, version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Args, Debug, PartialEq)]
struct CreateArgs {
    /// The name of the sandbox to delete.
    name: Option<String>,
}

#[derive(Args, Debug, PartialEq)]
struct DeleteArgs {
    /// The name of the sandbox to delete.
    name: String,
}

#[derive(Debug, PartialEq, Subcommand)]
enum Command {
    /// Initialize the CLI.
    Init,
    /// Create a sandbox.
    Create(CreateArgs),
    /// List sandboxes.
    List,
    /// Delete a sandbox.
    Delete(DeleteArgs),
}

fn main() {
    let cli = Cli::parse();

    if cli.command != Command::Init && !is_init() {
        panic!("{} is not initialized", NAME)
    }

    match cli.command {
        Command::Init => init().unwrap(),
        Command::Create(CreateArgs { name }) => create(name),
        Command::List => list(),
        Command::Delete(DeleteArgs { name }) => delete(name),
    }
}

fn get_sandbox_dir() -> PathBuf {
    let home = dirs::home_dir().map(PathBuf::from).unwrap();

    home.join(".sandbox")
}

fn is_init() -> bool {
    fs::exists(get_sandbox_dir()).unwrap()
}

fn init() -> io::Result<()> {
    fs::create_dir(get_sandbox_dir())
}

fn create(name: Option<String>) {
    let name = match name {
        Some(name) => name,
        None => petname::petname(3, "-").unwrap(),
    };

    fs::create_dir(get_sandbox_dir().join(&name)).unwrap();

    println!("Created {}", name);
}

fn list() {
    let mut items: Vec<String> = fs::read_dir(get_sandbox_dir())
        .unwrap()
        .map(|item| item.unwrap().file_name().into_string().unwrap())
        .collect();

    items.sort();

    for item in items {
        println!("{}", item)
    }
}

fn delete(name: String) {
    fs::remove_dir_all(get_sandbox_dir().join(name)).unwrap()
}
