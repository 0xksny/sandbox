use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};

use crate::config::Config;

mod attach;
mod create;
mod delete;
mod herdr;
mod init;
mod list;
mod open;

pub use attach::AttachCommand;
pub use create::CreateCommand;
pub use delete::DeleteCommand;
pub use herdr::HerdrCommand;
pub use init::InitCommand;
pub use list::ListCommand;
pub use open::OpenCommand;

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
            Commands::Open(command) => command.run(sandbox_dir),
            Commands::Init(_) => unreachable!(),
        }
    }

    fn sandbox_dir(&self) -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("error getting home directory")?;

        Ok(home_dir.join(".sandbox"))
    }

    fn is_init(sandbox_dir: &Path) -> Result<bool> {
        fs::exists(sandbox_dir).context("error checking if sandbox directory exists")
    }

    fn load_config(sandbox_dir: &Path) -> Result<Config> {
        let config_path = sandbox_dir.join("config.toml");

        let contents = fs::read_to_string(&config_path)
            .context(format!("error reading {}", config_path.display()))?;

        toml::from_str(&contents).context(format!("error parsing {}", config_path.display()))
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
    /// Open a sandbox.
    Open(OpenCommand),
}
