use anyhow::Result;
use clap::{Args, Subcommand};

use crate::herdr::Herdr;

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
