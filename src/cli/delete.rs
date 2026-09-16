use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::Args;

use crate::sandbox::Sandbox;

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
