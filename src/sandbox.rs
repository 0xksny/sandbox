use std::{fs, path::PathBuf};

use anyhow::{Context, Result};

use crate::assets::copy_dir_all;
use crate::session::Session;

pub struct Sandbox {
    sandbox_dir: PathBuf,

    name: String,
}

impl Sandbox {
    pub fn new(sandbox_dir: PathBuf, name: String) -> Self {
        Sandbox { sandbox_dir, name }
    }

    fn path(&self) -> PathBuf {
        self.sandbox_dir.join("sandboxes").join(&self.name)
    }

    pub fn exists(&self) -> Result<bool> {
        let path = self.path();

        fs::exists(path).context("error checking if sandbox path exists")
    }

    pub fn create(&self, template: &str) -> Result<()> {
        let path = self.path();

        fs::create_dir(&path).context("error creating sandbox directory")?;

        copy_dir_all(self.sandbox_dir.join("templates").join(template), path)
            .context("error copying template to sandbox")
    }

    pub fn delete(&self) -> Result<()> {
        let path = self.path();

        fs::remove_dir_all(path).context("error removing sandbox directory")
    }

    pub fn session(&self) -> Session {
        Session::new(self.name.clone(), self.path())
    }
}
