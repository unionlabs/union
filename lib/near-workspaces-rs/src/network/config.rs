//! Network specific configurations used to modify behavior inside a chain.
//! This is so far only useable with sandbox networks since it would require
//! direct access to a node to change the config. Each network like mainnet
//! and testnet already have pre-configured settings; meanwhile sandbox can
//! have additional settings on top of them to facilitate custom behavior
//! such as sending large requests to the sandbox network.
//
// NOTE: nearcore has many, many configs which can easily change in the future
// so this config.rs file just purely modifies the data and does not try to
// replicate all the structs from nearcore side; which can be a huge maintenance
// churn if we were to.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde_json::Value;

use crate::error::ErrorKind;
use crate::Result;

/// Overwrite the $home_dir/config.json file over a set of entries. `value` will be used per (key, value) pair
/// where value can also be another dict. This recursively sets all entry in `value` dict to the config
/// dict, and saves back into `home_dir` at the end of the day.
fn overwrite(home_dir: impl AsRef<Path>, value: Value) -> Result<()> {
    let home_dir = home_dir.as_ref();
    let config_file =
        File::open(home_dir.join("config.json")).map_err(|err| ErrorKind::Io.custom(err))?;
    let config = BufReader::new(config_file);
    let mut config: Value =
        serde_json::from_reader(config).map_err(|err| ErrorKind::DataConversion.custom(err))?;

    json_patch::merge(&mut config, &value);
    let config_file =
        File::create(home_dir.join("config.json")).map_err(|err| ErrorKind::Io.custom(err))?;
    serde_json::to_writer(config_file, &config).map_err(|err| ErrorKind::Io.custom(err))?;

    Ok(())
}

/// Parse an environment variable or return a default value.
fn parse_env<T>(env_var: &str) -> Result<Option<T>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match std::env::var(env_var) {
        Ok(val) => {
            let val = val
                .parse::<T>()
                .map_err(|err| ErrorKind::DataConversion.custom(err))?;

            Ok(Some(val))
        }
        Err(_err) => Ok(None),
    }
}

/// Set extra configs for the sandbox defined by workspaces.
pub(crate) fn set_sandbox_configs(home_dir: impl AsRef<Path>) -> Result<()> {
    overwrite(
        home_dir,
        serde_json::json!({
            "rpc": {
                "limits_config": {
                    // default to 1GB payload size so that large state patches can work.
                    "json_payload_max_size": parse_env("NEAR_SANDBOX_MAX_PAYLOAD_SIZE")?.unwrap_or(1024 * 1024 * 1024),
                },
            },
            "store": {
                // default to 3,000 files open at a time so that windows WSL can work without configuring.
                "max_open_files": parse_env("NEAR_SANDBOX_MAX_FILES")?.unwrap_or(3000),
            }
        }),
    )
}
