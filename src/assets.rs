use std::{fs, io, path::Path};

use anyhow::{Context, Result};
use include_dir::{Dir, DirEntry, include_dir};

pub static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

pub fn copy_embedded_dir_all(source: &Dir<'_>, destination: &Path) -> Result<()> {
    for entry in source.entries() {
        let relative_path = entry
            .path()
            .strip_prefix(source.path())
            .context("error stripping prefix from source entry path")?;
        let destination_path = destination.join(relative_path);

        match entry {
            DirEntry::Dir(directory) => {
                fs::create_dir_all(&destination_path)?;
                copy_embedded_dir_all(directory, &destination_path)?;
            }
            DirEntry::File(file) => {
                if destination_path
                    .file_name()
                    .is_some_and(|file_name| file_name == ".gitkeep")
                {
                    continue;
                }

                fs::write(destination_path, file.contents())?;
            }
        }
    }

    Ok(())
}

pub fn copy_dir_all(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> io::Result<()> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let destination_path = destination.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(entry.path(), destination_path)?;
        } else {
            fs::copy(entry.path(), destination_path)?;
        }
    }

    Ok(())
}
