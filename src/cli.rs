use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};

use crate::app::App;

#[derive(Debug, Parser)]
#[command(name = "sandbox", version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        let home_dir = dirs::home_dir().context("error getting home directory")?;

        let sandbox_dir = home_dir.join(".sandbox");

        let app = App::new(sandbox_dir, self.command);

        app.run()
    }
}

#[derive(Args, Debug, PartialEq)]
pub struct AttachArgs {
    pub name: String,
}

#[derive(Args, Debug, PartialEq)]
pub struct CreateArgs {
    /// The name of the sandbox to delete.
    pub name: Option<String>,
}

#[derive(Args, Debug, PartialEq)]
pub struct DeleteArgs {
    /// The name of the sandbox to delete.
    pub name: String,
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum Commands {
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
