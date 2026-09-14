use clap::{Args, Parser, Subcommand};
use include_dir::{Dir, DirEntry, include_dir};
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

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
        return get_sandbox_dir().join("sandboxes").join(&self.name);
    }

    fn exists(&self) -> bool {
        fs::exists(self.path()).unwrap()
    }

    fn create(&self) {
        fs::create_dir(&self.path()).unwrap();
        copy_dir_all(
            get_sandbox_dir().join("templates").join("default"),
            &self.path(),
        )
        .unwrap();
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
    let sandbox_dir = get_sandbox_dir();

    fs::create_dir(&sandbox_dir)?;

    let assets = ASSETS.get_dir(".sandbox").unwrap();

    copy_embedded_dir_all(assets, &sandbox_dir)
}

fn copy_embedded_dir_all(source: &Dir<'_>, destination: &PathBuf) -> io::Result<()> {
    for entry in source.entries() {
        let relative_path = entry.path().strip_prefix(source.path()).unwrap();
        let destination_path = destination.join(relative_path);

        match entry {
            DirEntry::Dir(directory) => {
                fs::create_dir_all(&destination_path)?;
                copy_embedded_dir_all(directory, &destination_path)?;
            }
            DirEntry::File(file) => {
                if destination_path
                    .file_name()
                    .is_some_and(|file_name| file_name == ".gitkeep")
                {
                    continue;
                }

                fs::write(destination_path, file.contents())?;
            }
        }
    }

    Ok(())
}

fn copy_dir_all(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> io::Result<()> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let destination_path = destination.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(entry.path(), destination_path)?;
        } else {
            fs::copy(entry.path(), destination_path)?;
        }
    }

    Ok(())
}

fn list() {
    let mut items: Vec<String> = fs::read_dir(get_sandbox_dir().join("sandboxes"))
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
