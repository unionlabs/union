use std::{fs, io, path::Path};

use thiserror::Error;
use tracing::{debug, field::display as as_display};

use crate::network::Network;

pub fn set_seeds(network: Network, file: impl AsRef<Path>) -> Result<(), SetSeedsError> {
    use SetSeedsError::*;
    let file = file.as_ref();
    debug!(target: "unionvisor", "reading config.toml at {} to replace seeds",  as_display(file.display()));
    let contents = fs::read_to_string(file).map_err(|source| CantReadContents { source })?;
    let new = contents.replace(r#"seeds = """#, &format!(r#"seeds="{}""#, network.seeds()));
    debug!(target: "unionvisor", "replacing contents by deleting and writing file");
    fs::remove_file(file).map_err(|source| CantRemoveFile { source })?;
    fs::write(file, new).map_err(|source| CantWriteFile { source })?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum SetSeedsError {
    #[error("cannot read file contents of config.toml")]
    CantReadContents {
        #[backtrace]
        source: io::Error,
    },
    #[error("cannot remove old config.toml")]
    CantRemoveFile {
        #[backtrace]
        source: io::Error,
    },
    #[error("cannot write new config.toml")]
    CantWriteFile {
        #[backtrace]
        source: io::Error,
    },
}

pub fn download_genesis(
    network: Network,
    to: impl AsRef<Path>,
) -> Result<(), DownloadGenesisError> {
    use DownloadGenesisError::*;
    let to = to.as_ref();
    let url = network.genesis_url();
    debug!(target: "unionvisor", "fetching genesis.json for {} at {}", network, url);
    let mut resp = reqwest::blocking::get(url).map_err(|source| CantDownload {
        source,
        url: url.to_owned(),
    })?;
    debug!(target: "unionvisor", "writing genesis.json to {}",  as_display(to.display()));
    let mut out = fs::File::create(to).map_err(|source| CantCreateFile { source })?;
    io::copy(&mut resp, &mut out).map_err(|source| CantWriteFile { source })?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum DownloadGenesisError {
    #[error("cannot download genesis.json from {url}")]
    CantDownload {
        #[backtrace]
        source: reqwest::Error,
        url: String,
    },
    #[error("cannot create genesis.json")]
    CantCreateFile {
        #[backtrace]
        source: io::Error,
    },
    #[error("cannot write genesis.json")]
    CantWriteFile {
        #[backtrace]
        source: io::Error,
    },
}
