use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::Args;

use crate::sandbox::Sandbox;

#[derive(Args, Clone, Debug, PartialEq)]
pub struct OpenCommand {
    /// The name of the sandbox to open.
    pub name: String,
}

impl OpenCommand {
    pub fn run(self, sandbox_dir: PathBuf) -> Result<()> {
        let sandbox = Sandbox::new(sandbox_dir, self.name);

        if !sandbox.exists()? {
            return Err(anyhow!("sandbox does not exist"));
        }

        sandbox.open().context("error opening sandbox")
    }
}
