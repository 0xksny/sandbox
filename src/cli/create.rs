use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::Args;

use crate::{config::Config, sandbox::Sandbox};

#[derive(Args, Clone, Debug, PartialEq)]
pub struct CreateCommand {
    /// The name of the sandbox to create.
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
