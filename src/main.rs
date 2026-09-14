use anyhow::{Context, Result, anyhow};
use clap::{Args, Parser, Subcommand};
use include_dir::{Dir, DirEntry, include_dir};
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
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

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {}", error);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let is_init = is_init().context("error checking if sandbox is initialized")?;

    if cli.command != Commands::Init && !is_init {
        return Err(anyhow!("sandbox is not initialized"));
    }

    match cli.command {
        Commands::Attach(AttachArgs { name }) => attach(name),
        Commands::Create(CreateArgs { name }) => create(name),
        Commands::Delete(DeleteArgs { name }) => delete(name),
        Commands::Init => init(),
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

    fn path(&self) -> Result<PathBuf> {
        let sandbox_dir = get_sandbox_dir().context("error getting sandbox directory")?;

        Ok(sandbox_dir.join("sandboxes").join(&self.name))
    }

    fn exists(&self) -> Result<bool> {
        let path = self.path().context("error getting sandbox path")?;

        fs::exists(path).context("error checking if sandbox path exists")
    }

    fn create(&self) -> Result<()> {
        let sandbox_dir = get_sandbox_dir().context("error getting sandbox directory")?;
        let path = self.path().context("error getting sandbox path")?;

        fs::create_dir(&path).context("error creating sandbox directory")?;

        copy_dir_all(sandbox_dir.join("templates").join("default"), path)
            .context("error copying template to sandbox")
    }

    fn delete(&self) -> Result<()> {
        let path = self.path().context("error getting sandbox path")?;

        fs::remove_dir_all(path).context("error removing sandbox directory")
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

    fn exists(&self) -> Result<bool> {
        let target = format!("={}", self.sandbox.name);

        let status = Command::new("tmux")
            .args(["has-session", "-t", &target])
            .status()
            .context("error checking if tmux session exists")?;

        Ok(status.success())
    }

    fn create(&self) -> Result<()> {
        let path = self.sandbox.path().context("error getting sandbox path")?;

        let path_str = path
            .to_str()
            .context("error converting sandbox path to string")?;

        Command::new("tmux")
            .args([
                "new-session",
                "-d",
                "-s",
                &self.sandbox.name,
                "-c",
                path_str,
                "codex",
            ])
            .status()
            .context("error creating tmux session")?;

        Ok(())
    }

    fn attach(&self) -> Result<()> {
        Command::new("tmux")
            .args(["attach-session", "-t", &self.sandbox.name])
            .status()
            .context("error attaching to tmux session")?;

        Ok(())
    }
}

fn get_sandbox_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("error getting home directory")?;

    Ok(home.join(".sandbox"))
}

fn is_init() -> Result<bool> {
    let sandbox_dir = get_sandbox_dir().context("error getting sandbox directory")?;

    fs::exists(sandbox_dir).context("error checking if sandbox directory exists")
}

fn attach(name: String) -> Result<()> {
    let sandbox = Sandbox::new(name);

    if !sandbox.exists()? {
        return Err(anyhow!("sandbox does not exist"));
    }

    let session = sandbox.session();

    let exists = session
        .exists()
        .context("error checking if session exists")?;

    if !exists {
        session.create().context("error creating session")?;
    }

    session.attach().context("error attaching to session")
}

fn create(name: Option<String>) -> Result<()> {
    let name = match name {
        Some(name) => name,
        None => petname::petname(3, "-").context("petname did not generate a name")?,
    };

    let sandbox = Sandbox::new(name);

    if sandbox.exists()? {
        return Err(anyhow!("sandbox already exists"));
    }

    sandbox.create().context("error creating sandbox")
}

fn init() -> Result<()> {
    let sandbox_dir = get_sandbox_dir().context("error getting sandbox directory")?;

    fs::create_dir(&sandbox_dir).context("error creating sandbox directory")?;

    let assets = ASSETS
        .get_dir(".sandbox")
        .context("error getting .sandbox directory from assets")?;

    copy_embedded_dir_all(assets, &sandbox_dir)
        .context("error copying embedded directory to sandbox directory")
}

fn copy_embedded_dir_all(source: &Dir<'_>, destination: &Path) -> Result<()> {
    for entry in source.entries() {
        let relative_path = entry
            .path()
            .strip_prefix(source.path())
            .context("error stripping prefix from source entry path")?;
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

fn list() -> Result<()> {
    let sandbox_dir = get_sandbox_dir().context("error getting sandbox directory")?;

    let read_dir_result =
        fs::read_dir(sandbox_dir.join("sandboxes")).context("error reading sandboxes directory")?;

    let mut items: Vec<String> = read_dir_result
        .map(|item| -> Result<String> {
            let entry = item.context("error getting sandboxes directory entry")?;

            entry
                .file_name()
                .into_string()
                .map_err(|error| anyhow!("error converting file name to string: {:?}", error))
        })
        .collect::<Result<_>>()?;

    items.sort();

    for item in items {
        println!("{}", item)
    }

    Ok(())
}

fn delete(name: String) -> Result<()> {
    let sandbox = Sandbox::new(name);

    if !sandbox.exists()? {
        return Err(anyhow!("sandbox does not exist"));
    }

    sandbox.delete().context("error deleting sandbox")
}
