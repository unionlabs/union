use crate::network::Network;
use color_eyre::Result;
use std::{fs, io, path::Path};
use tracing::{debug, field::display as as_display};



pub fn set_seeds(network: Network, file: impl AsRef<Path>) -> Result<()> {
    let file = file.as_ref();
    debug!(target: "unionvisor", "reading config.toml at {} to replace seeds",  as_display(file.display()));
    let contents = fs::read_to_string(file)?;
    let new = contents.replace(r#"seeds = """#, &format!(r#"seeds="{}""#, network.seeds()));
    debug!(target: "unionvisor", "replacing contents by deleting and writing file");
    fs::remove_file(file)?;
    fs::write(file, new)?;
    Ok(())
}

pub fn download_genesis(network: Network, to: impl AsRef<Path>) -> Result<()> {
    let to = to.as_ref();
    debug!(target: "unionvisor", "fetching genesis.json for {} at {}", network, network.genesis_url());
    let mut resp = reqwest::blocking::get(network.genesis_url())?;
    debug!(target: "unionvisor", "writing genesis.json to {}",  as_display(to.display()));
    let mut out = fs::File::create(to)?;
    io::copy(&mut resp, &mut out)?;
    Ok(())
}
