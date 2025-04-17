use std::{
    fs::{self, File},
    io::Read,
    process::Command,
};

use serde_json::{json, Value};
use tempfile::tempdir;
use thiserror::Error;
use tracing::{debug, info, trace};

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Empty git hash provided")]
    EmptyGitHash,

    #[error("Failed to fetch commit data: {0}")]
    FetchError(#[from] reqwest::Error),

    #[error("Command execution failed: {0}")]
    CommandError(#[from] std::io::Error),

    #[error("Nix build failed: {0}")]
    NixBuildFailed(String),

    #[error("Result directory not found")]
    ResultDirNotFound(String),

    #[error("Failed to read directory: {0}")]
    DirReadError(String),

    #[error("Failed to read JSON file at {path}: {error}")]
    FileReadError { path: String, error: std::io::Error },

    #[error("Failed to parse JSON from {path}: {error}")]
    JsonParseError {
        path: String,
        error: serde_json::Error,
    },
}

pub type Result<T> = std::result::Result<T, BuildError>;

#[derive(Debug)]
pub struct AbiResult {
    pub command: String,
    pub data: Value,
    pub log: String,
}

impl AbiResult {
    pub fn meta(&self) -> Value {
        json!({
            "command": self.command,
            "logs": self.log,
        })
    }
}

pub async fn build_abis_with_commit_hash(commit_hash: &Vec<u8>) -> Result<AbiResult> {
    // Validate git_hash
    if commit_hash.is_empty() {
        return Err(BuildError::EmptyGitHash);
    }

    // Declare repository details
    let repo_owner = "unionlabs";
    let repo_name = "union";

    // determine nix binary location. hubble runs in a systemd service that doesn't
    // have nix on the path. it is configured in NIX_BIN, but defaults the
    // current-system (assuming we're running in nixos) when running locally.
    let nix_bin =
        std::env::var("NIX_BIN").unwrap_or_else(|_| "/run/current-system/sw/bin/nix".to_string());
    trace!("nix bin: {nix_bin}");

    let nix_build_argument = format!(
        "github:{}/{}/{}#packages.x86_64-linux.hubble-abis",
        repo_owner,
        repo_name,
        hex::encode(commit_hash)
    );
    trace!("nix build argument: {nix_build_argument}");

    let nix_command = format!("{nix_bin} build \"{nix_build_argument}\"");
    trace!("nix command: {nix_command}");

    let work_dir = tempdir()?;
    trace!("work dir: {work_dir:?}");

    let output = Command::new(nix_bin)
        .arg("build")
        .arg(nix_build_argument)
        .current_dir(&work_dir)
        .output()
        .map_err(BuildError::CommandError)?;

    info!("abi build: {} => {}", nix_command, output.status);

    let mut log = String::new();
    log.push_str(&String::from_utf8_lossy(&output.stdout));
    log.push_str(&String::from_utf8_lossy(&output.stderr));

    debug!("abi build log: \n-----------\n{}\n-----------", log);

    if !output.status.success() {
        return Err(BuildError::NixBuildFailed(log));
    }

    // Ensure result directory exists
    let result_dir = work_dir.path().join("result");
    if !result_dir.exists() || !result_dir.is_dir() {
        return Err(BuildError::ResultDirNotFound(log));
    }

    // Merge all JSON files in the result directory
    let mut merged_json = json!({});

    let entries = fs::read_dir(result_dir).map_err(|e| BuildError::DirReadError(e.to_string()))?;

    for entry_result in entries {
        let entry = entry_result.map_err(|e| BuildError::DirReadError(e.to_string()))?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let filename = path
                .file_stem()
                .and_then(|name| name.to_str())
                .ok_or_else(|| BuildError::FileReadError {
                    path: path.to_string_lossy().into_owned(),
                    error: std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid filename"),
                })?;

            let mut file = File::open(&path).map_err(|e| BuildError::FileReadError {
                path: path.to_string_lossy().into_owned(),
                error: e,
            })?;

            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|e| BuildError::FileReadError {
                    path: path.to_string_lossy().into_owned(),
                    error: e,
                })?;

            let file_json: Value =
                serde_json::from_str(&contents).map_err(|e| BuildError::JsonParseError {
                    path: path.to_string_lossy().into_owned(),
                    error: e,
                })?;

            merged_json[filename] = file_json;
        }
    }

    Ok(AbiResult {
        command: nix_command,
        data: merged_json,
        log,
    })
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_build_and_merge_abis() {
//         let git_hash = "EC9A50EC16CD055FDD8CCFD2E5D25A91F609CAD5";
//         let result = build_abis(&hex::decode(git_hash).expect("hex")).await;

//         match result {
//             Ok(abi_result) => println!(
//                 "result ({}): {}\n\n{}",
//                 hex::encode(git_hash),
//                 abi_result.data,
//                 abi_result.log
//             ),
//             Err(e) => println!("Error running build_and_merge_abis: {}", e),
//         }
//     }
// }
