use clap::{Args, Parser, Subcommand};
use std::{fs, io, path::PathBuf, process::Command};

#[derive(Debug, Parser)]
#[command(name = "sandbox", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug, PartialEq)]
struct AttachArgs {
    name: String,
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
enum Commands {
    /// Attach to a sandbox.
    Attach(AttachArgs),
    /// Create a sandbox.
    Create(CreateArgs),
    /// Initialize the CLI.
    Init,
    /// List sandboxes.
    List,
    /// Delete a sandbox.
    Delete(DeleteArgs),
}

fn main() {
    let cli = Cli::parse();

    if cli.command != Commands::Init && !is_init() {
        panic!("sandbox is not initialized")
    }

    match cli.command {
        Commands::Attach(AttachArgs { name }) => attach(name),
        Commands::Create(CreateArgs { name }) => create(name),
        Commands::Delete(DeleteArgs { name }) => delete(name),
        Commands::Init => init().unwrap(),
        Commands::List => list(),
    }
}

struct Sandbox {
    name: String,
}

impl Sandbox {
    fn new(name: String) -> Self {
        Sandbox { name }
    }

    fn path(&self) -> PathBuf {
        return get_sandbox_dir().join(&self.name);
    }

    fn exists(&self) -> bool {
        fs::exists(self.path()).unwrap()
    }

    fn create(&self) {
        fs::create_dir(&self.path()).unwrap();
    }

    fn delete(&self) {
        fs::remove_dir_all(&self.path()).unwrap();
    }

    fn session(&self) -> Session<'_> {
        Session::new(self)
    }
}

struct Session<'a> {
    sandbox: &'a Sandbox,
}

impl<'a> Session<'a> {
    fn new(sandbox: &'a Sandbox) -> Self {
        Session { sandbox }
    }

    fn exists(&self) -> bool {
        let target = format!("={}", self.sandbox.name);

        Command::new("tmux")
            .args(["has-session", "-t", &target])
            .status()
            .unwrap()
            .success()
    }

    fn create(&self) {
        Command::new("tmux")
            .args([
                "new-session",
                "-d",
                "-s",
                &self.sandbox.name,
                "-c",
                self.sandbox.path().to_str().unwrap(),
                "codex",
            ])
            .status()
            .unwrap();
    }

    fn attach(&self) {
        Command::new("tmux")
            .args(["attach-session", "-t", &self.sandbox.name])
            .status()
            .unwrap();
    }
}

fn get_sandbox_dir() -> PathBuf {
    let home = dirs::home_dir().map(PathBuf::from).unwrap();

    home.join(".sandbox")
}

fn is_init() -> bool {
    fs::exists(get_sandbox_dir()).unwrap()
}

fn attach(name: String) {
    let sandbox = Sandbox::new(name);

    if !sandbox.exists() {
        panic!("sandbox does not exist")
    }

    let session = sandbox.session();

    if !session.exists() {
        session.create();
    }

    session.attach();
}

fn create(name: Option<String>) {
    let name = match name {
        Some(name) => name,
        None => petname::petname(3, "-").unwrap(),
    };

    let sandbox = Sandbox::new(name);

    if sandbox.exists() {
        panic!("sandbox already exists")
    }

    sandbox.create();
}

fn init() -> io::Result<()> {
    fs::create_dir(get_sandbox_dir())
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
    let sandbox = Sandbox::new(name);

    if !sandbox.exists() {
        panic!("sandbox does not exist")
    }

    sandbox.delete();
}
