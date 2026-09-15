use std::{path::PathBuf, process::Command};

use anyhow::{Context, Result};

pub struct Session {
    name: String,
    path: PathBuf,
}

impl Session {
    pub fn new(name: String, path: PathBuf) -> Self {
        Session { name, path }
    }

    pub fn exists(&self) -> Result<bool> {
        let target = format!("={}", self.name);

        let status = Command::new("tmux")
            .args(["has-session", "-t", &target])
            .status()
            .context("error checking if tmux session exists")?;

        Ok(status.success())
    }

    pub fn create(&self, agent: &str) -> Result<()> {
        let path_str = self
            .path
            .to_str()
            .context("error converting sandbox path to string")?;

        Command::new("tmux")
            .args(["new-session", "-d", "-s", &self.name, "-c", path_str, agent])
            .status()
            .context("error creating tmux session")?;

        Ok(())
    }

    pub fn attach(&self) -> Result<()> {
        Command::new("tmux")
            .args(["attach-session", "-t", &self.name])
            .status()
            .context("error attaching to tmux session")?;

        Ok(())
    }
}
