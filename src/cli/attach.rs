use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::Args;

use crate::{config::Config, sandbox::Sandbox};

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
