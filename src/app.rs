use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};

use crate::{
    assets::{ASSETS, copy_embedded_dir_all},
    cli::{AttachArgs, Commands, CreateArgs, DeleteArgs},
    config::Config,
    sandbox::Sandbox,
};

pub struct App {
    sandbox_dir: PathBuf,
    command: Commands,
}

impl App {
    pub fn new(sandbox_dir: PathBuf, command: Commands) -> Self {
        App {
            sandbox_dir,
            command,
        }
    }

    fn is_init(&self) -> Result<bool> {
        fs::exists(&self.sandbox_dir).context("error checking if sandbox directory exists")
    }

    fn load_config(&self) -> Result<Config> {
        let config_path = self.sandbox_dir.join("config.toml");

        let contents = fs::read_to_string(&config_path)
            .context(format!("error reading {}", config_path.display()))?;

        toml::from_str(&contents).context(format!("error parsing {}", config_path.display()))
    }

    fn attach(&self, name: String, config: &Config) -> Result<()> {
        let sandbox = Sandbox::new(self.sandbox_dir.clone(), name);

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

    fn create(&self, name: Option<String>, config: &Config) -> Result<()> {
        let name = match name {
            Some(name) => name,
            None => petname::petname(3, "-").context("petname did not generate a name")?,
        };

        let sandbox = Sandbox::new(self.sandbox_dir.clone(), name);

        if sandbox.exists()? {
            return Err(anyhow!("sandbox already exists"));
        }

        sandbox
            .create(&config.template)
            .context("error creating sandbox")
    }

    fn delete(&self, name: String) -> Result<()> {
        let sandbox = Sandbox::new(self.sandbox_dir.clone(), name);

        if !sandbox.exists()? {
            return Err(anyhow!("sandbox does not exist"));
        }

        sandbox.delete().context("error deleting sandbox")
    }

    fn init(&self) -> Result<()> {
        fs::create_dir(&self.sandbox_dir).context("error creating sandbox directory")?;

        let assets = ASSETS
            .get_dir(".sandbox")
            .context("error getting .sandbox directory from assets")?;

        copy_embedded_dir_all(assets, &self.sandbox_dir)
            .context("error copying embedded directory to sandbox directory")
    }

    fn list(&self) -> Result<()> {
        let read_dir_result = fs::read_dir(self.sandbox_dir.join("sandboxes"))
            .context("error reading sandboxes directory")?;

        let mut items: Vec<String> = read_dir_result
            .map(|item| -> Result<String> {
                let entry = item.context("error getting sandboxes directory entry")?;

                entry
                    .file_name()
                    .into_string()
                    .map_err(|error| anyhow!("error converting file name to string: {:?}", error))
            })
            .collect::<Result<_>>()?;

        items.sort();

        for item in items {
            println!("{}", item)
        }

        Ok(())
    }

    pub fn run(self) -> Result<()> {
        let is_init = self
            .is_init()
            .context("error checking if sandbox is initialized")?;

        if self.command != Commands::Init && !is_init {
            return Err(anyhow!("sandbox is not initialized"));
        }

        let config = match &self.command {
            Commands::Init => None,
            _ => Some(self.load_config().context("error loading configuration")?),
        };

        match &self.command {
            Commands::Attach(AttachArgs { name }) => {
                let config = config.as_ref().context("error converting config to ref")?;
                self.attach(name.clone(), config)
            }
            Commands::Create(CreateArgs { name }) => {
                let config = config.as_ref().context("error converting config to ref")?;
                self.create(name.clone(), config)
            }
            Commands::Delete(DeleteArgs { name }) => self.delete(name.clone()),
            Commands::Init => self.init(),
            Commands::List => self.list(),
        }
    }
}
