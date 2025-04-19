use std::sync::LazyLock;

use reqwest::Client;
use serde::Serialize;
use tracing::{error, trace};

use crate::github_client::GitCommitHash;

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent("reqwest")
        .build()
        .expect("client can be built")
});

#[derive(Debug, thiserror::Error)]
pub enum FileDownloadError {
    #[error("error requesting file contents from {0}")]
    SendRequest(String, #[source] reqwest::Error),

    #[error("error response fetching file contents from {0}")]
    ErrorResponse(String, #[source] reqwest::Error),

    #[error("error downloading file content from {0}")]
    Download(String, #[source] reqwest::Error),
}

#[derive(Clone, Debug, Serialize)]
pub struct FileContents(pub Vec<u8>);

pub async fn download(
    repo: &str,
    commit_hash: &GitCommitHash,
    path: &str,
) -> Result<FileContents, FileDownloadError> {
    trace!("fetch_file_contents: {repo}/{path}/{commit_hash}");

    let raw_url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}",
        repo, commit_hash, path
    );

    let result = CLIENT
        .get(&raw_url)
        .send()
        .await
        .map_err(|e| FileDownloadError::SendRequest(raw_url.clone(), e))?
        .error_for_status()
        .map_err(|e| FileDownloadError::ErrorResponse(raw_url.clone(), e))?
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| FileDownloadError::Download(raw_url.clone(), e))?;

    Ok(FileContents(result))
}
