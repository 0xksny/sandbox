use std::{fs, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::Args;

#[derive(Args, Clone, Debug, PartialEq)]
pub struct ListCommand {}

impl ListCommand {
    pub fn run(self, sandbox_dir: PathBuf) -> Result<()> {
        let read_dir_result = fs::read_dir(sandbox_dir.join("sandboxes"))
            .context("error reading sandboxes directory")?;
        let mut items: Vec<String> = read_dir_result
            .map(|item| -> Result<Option<String>> {
                let entry = item.context("error getting sandboxes directory entry")?;

                let entry_file_type = entry
                    .file_type()
                    .context("error getting sandbox entry file type")?;
                if !entry_file_type.is_dir() {
                    return Ok(None);
                }

                let entry_file_name = entry.file_name().into_string().map_err(|error| {
                    anyhow!("error converting file name to string: {:?}", error)
                })?;

                Ok(Some(entry_file_name))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();

        items.sort();
        for item in items {
            println!("{}", item);
        }

        Ok(())
    }
}
