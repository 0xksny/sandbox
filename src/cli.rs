use std::{fs, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{Args, Parser, Subcommand};

use crate::{
    assets::{ASSETS, copy_embedded_dir_all},
    config::Config,
    herdr::Herdr,
    sandbox::Sandbox,
};

#[derive(Debug, Parser)]
#[command(name = "sandbox", version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        let sandbox_dir = self
            .sandbox_dir()
            .context("error getting sandbox directory")?;

        if let Commands::Init(command) = self.command {
            return command.run(sandbox_dir);
        }

        let is_init =
            Cli::is_init(&sandbox_dir).context("error checking if sandbox is initialized")?;

        if !is_init {
            return Err(anyhow!("sandbox is not initialized"));
        }

        let config = Cli::load_config(&sandbox_dir).context("error loading configuration")?;

        match self.command {
            Commands::Attach(command) => command.run(sandbox_dir, config),
            Commands::Create(command) => command.run(sandbox_dir, config),
            Commands::Delete(command) => command.run(sandbox_dir),
            Commands::Herdr(command) => command.run(),
            Commands::List(command) => command.run(sandbox_dir),
            Commands::Init(_) => unreachable!(),
        }
    }

    fn sandbox_dir(&self) -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("error getting home directory")?;

        Ok(home_dir.join(".sandbox"))
    }

    fn is_init(sandbox_dir: &PathBuf) -> Result<bool> {
        fs::exists(sandbox_dir).context("error checking if sandbox directory exists")
    }

    fn load_config(sandbox_dir: &PathBuf) -> Result<Config> {
        let config_path = sandbox_dir.join("config.toml");

        let contents = fs::read_to_string(&config_path)
            .context(format!("error reading {}", config_path.display()))?;

        toml::from_str(&contents).context(format!("error parsing {}", config_path.display()))
    }
}

#[derive(Args, Clone, Debug, PartialEq)]
pub struct AttachCommand {
    /// The name of the sandbox to attach to.
    pub name: String,
}

impl AttachCommand {
    pub fn run(self, sandbox_dir: PathBuf, config: Config) -> Result<()> {
        let sandbox = Sandbox::new(sandbox_dir, self.name);

        if !sandbox.exists()? {
            return Err(anyhow!("sandbox does not exist"));
        }

        let session = sandbox.session();

        let exists = session
            .exists()
            .context("error checking if session exists")?;

        if !exists {
            session
                .create(&config.agent)
                .context("error creating session")?;
        }

        session.attach().context("error attaching to session")
    }
}

#[derive(Args, Clone, Debug, PartialEq)]
pub struct CreateCommand {
    /// The name of the sandbox to delete.
    pub name: Option<String>,
}

impl CreateCommand {
    pub fn run(self, sandbox_dir: PathBuf, config: Config) -> Result<()> {
        let name = match self.name {
            Some(name) => name,
            None => petname::petname(3, "-").context("petname did not generate a name")?,
        };

        let sandbox = Sandbox::new(sandbox_dir, name);

        if sandbox.exists()? {
            return Err(anyhow!("sandbox already exists"));
        }

        sandbox
            .create(&config.template)
            .context("error creating sandbox")
    }
}

#[derive(Args, Clone, Debug, PartialEq)]
pub struct DeleteCommand {
    /// The name of the sandbox to delete.
    pub name: String,
}

impl DeleteCommand {
    pub fn run(self, sandbox_dir: PathBuf) -> Result<()> {
        let sandbox = Sandbox::new(sandbox_dir, self.name);

        if !sandbox.exists()? {
            return Err(anyhow!("sandbox does not exist"));
        }

        sandbox.delete().context("error deleting sandbox")
    }
}

#[derive(Args, Clone, Debug, PartialEq)]
pub struct HerdrCommand {
    #[command(subcommand)]
    command: HerdrCommands,
}

impl HerdrCommand {
    pub fn run(self) -> Result<()> {
        let herdr = Herdr::new();

        match self.command {
            HerdrCommands::List(command) => command.run(herdr),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Subcommand)]
enum HerdrCommands {
    /// List workspaces.
    List(HerdrListCommand),
}

#[derive(Args, Clone, Debug, PartialEq)]
struct HerdrListCommand {}

impl HerdrListCommand {
    fn run(self, herdr: Herdr) -> Result<()> {
        for workspace in herdr.list_workspaces()? {
            println!("{}", workspace);
        }

        Ok(())
    }
}

#[derive(Args, Clone, Debug, PartialEq)]
pub struct InitCommand {}

impl InitCommand {
    pub fn run(self, sandbox_dir: PathBuf) -> Result<()> {
        fs::create_dir(&sandbox_dir).context("error creating sandbox directory")?;

        let assets = ASSETS
            .get_dir(".sandbox")
            .context("error getting .sandbox directory from assets")?;

        copy_embedded_dir_all(assets, &sandbox_dir)
            .context("error copying embedded directory to sandbox directory")
    }
}

#[derive(Args, Clone, Debug, PartialEq)]
pub struct ListCommand {}

impl ListCommand {
    pub fn run(self, sandbox_dir: PathBuf) -> Result<()> {
        let read_dir_result = fs::read_dir(sandbox_dir.join("sandboxes"))
            .context("error reading sandboxes directory")?;

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
}

#[derive(Clone, Debug, PartialEq, Subcommand)]
pub enum Commands {
    /// Attach to a sandbox.
    Attach(AttachCommand),
    /// Create a sandbox.
    Create(CreateCommand),
    /// Delete a sandbox.
    Delete(DeleteCommand),
    /// Commands related to herdr.
    Herdr(HerdrCommand),
    /// Initialize the CLI.
    Init(InitCommand),
    /// List sandboxes.
    List(ListCommand),
}
