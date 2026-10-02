use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::assets::{ASSETS, copy_embedded_dir_all};

#[derive(Args, Clone, Debug, PartialEq)]
pub struct InitCommand {}

impl InitCommand {
    pub fn run(self, sandbox_dir: PathBuf) -> Result<()> {
        if !sandbox_dir.exists() {
            std::fs::create_dir(&sandbox_dir).context("error creating sandbox directory")?;
        }

        let assets = ASSETS
            .get_dir(".sandbox")
            .context("error getting .sandbox directory from assets")?;

        copy_embedded_dir_all(assets, &sandbox_dir)
            .context("error copying embedded directory to sandbox directory")
    }
}
