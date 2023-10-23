use std::{fs, io, path::Path};

use thiserror::Error;
use tracing::{debug, field::display as as_display, info};

pub fn set_seeds(seeds: &str, file: impl AsRef<Path>) -> Result<(), SetSeedsError> {
    let file = file.as_ref();
    info!(target: "unionvisor", "reading config.toml at {} to replace seeds", as_display(file.display()));
    let contents = fs::read_to_string(file).map_err(SetSeedsError::ConfigRead)?;
    let new = contents.replace(r#"seeds = """#, &format!(r#"seeds="{seeds}""#));
    debug!(target: "unionvisor", "replacing contents by deleting and writing file");
    fs::remove_file(file).map_err(SetSeedsError::RemoveOldConfig)?;
    fs::write(file, new).map_err(SetSeedsError::ConfigWrite)?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum SetSeedsError {
    #[error("cannot read file contents of config.toml")]
    ConfigRead(#[source] io::Error),
    #[error("cannot remove old config.toml")]
    RemoveOldConfig(#[source] io::Error),
    #[error("cannot write new config.toml")]
    ConfigWrite(#[source] io::Error),
}
