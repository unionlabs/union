use std::fmt::Display;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::{error, trace};

use crate::github_fetcher::Subscription;

#[derive(Debug, thiserror::Error)]
pub enum FileDownloadError {
    #[error("error requesting commit details from {0}: {1}")]
    CommitDetailsRequest(String, reqwest::Error),

    #[error("error response fetching commit details from {0}: {1}")]
    CommitDetailsErrorResponse(String, reqwest::Error),

    #[error("error downloading commit details from {0}: {1}")]
    CommitDetailsDownload(String, reqwest::Error),

    #[error("error parsing json commit details from {0}: {1}")]
    CommitDetailsDecode(String, serde_json::Error),

    #[error("no commit for {0}")]
    CommitDetailsNoCommit(String),

    #[error("error requesting file contents from {0}: {1}")]
    FileContentsRequest(String, reqwest::Error),

    #[error("error response fetching file contents from {0}: {1}")]
    FileContentsErrorResponse(String, reqwest::Error),

    #[error("error downloading file content from {0}: {1}")]
    FileContentsDownload(String, reqwest::Error),
}

#[derive(Debug, Serialize)]
pub struct CommitDetails {
    #[serde(serialize_with = "as_hex")]
    commit_hash: Vec<u8>,
    #[serde(serialize_with = "serialize_iso8601")]
    commit_timestamp: OffsetDateTime,
}

impl Display for CommitDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "commit_hash: {}, commit_timestamp: {}",
            hex::encode(&self.commit_hash),
            self.commit_timestamp
        ))
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FileContents(pub Vec<u8>);

// Define structs for deserializing the JSON response
#[derive(Debug, Deserialize)]
struct Author {
    #[serde(with = "time::serde::rfc3339")]
    date: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
struct CommitInfo {
    author: Author,
}

#[derive(Debug, Deserialize)]
struct GitCommit {
    #[serde(deserialize_with = "deserialize_sha")]
    sha: Vec<u8>,
    commit: CommitInfo,
}

pub async fn fetch_commit_details(
    subscription: &Subscription,
) -> Result<CommitDetails, FileDownloadError> {
    trace!("fetch_commit_details: {subscription}");

    let client = Client::new();
    let commits_url = format!(
        "https://api.github.com/repos/{}/commits?path={}&sha={}&per_page=1",
        subscription.repo, subscription.path, subscription.branch
    );

    let commits_response = client
        .get(&commits_url)
        .header("User-Agent", "reqwest")
        .send()
        .await
        .map_err(|e| FileDownloadError::CommitDetailsRequest(commits_url.clone(), e))?
        .error_for_status()
        .map_err(|e| FileDownloadError::CommitDetailsErrorResponse(commits_url.clone(), e))?
        .text()
        .await
        .map_err(|e| FileDownloadError::CommitDetailsDownload(commits_url.clone(), e))?;

    let commits: Vec<GitCommit> = serde_json::from_str(&commits_response)
        .map_err(|e| FileDownloadError::CommitDetailsDecode(commits_url.clone(), e))?;

    let commit = commits
        .first()
        .ok_or_else(|| FileDownloadError::CommitDetailsNoCommit(commits_url.clone()))?;

    trace!("commit_details of {subscription}: {commit:?}");

    // Flatten into commit details
    Ok(CommitDetails {
        commit_hash: commit.sha.clone(),
        commit_timestamp: commit.commit.author.date,
    })
}

pub async fn fetch_file_contents(
    subscription: &Subscription,
    commit_details: &CommitDetails,
) -> Result<FileContents, FileDownloadError> {
    trace!("fetch_file_contents: {subscription}");

    let raw_url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}",
        subscription.repo,
        hex::encode(&commit_details.commit_hash),
        subscription.path
    );

    let client = Client::new();

    let result = client
        .get(&raw_url)
        .header("User-Agent", "reqwest")
        .send()
        .await
        .map_err(|e| FileDownloadError::FileContentsRequest(raw_url.clone(), e))?
        .error_for_status()
        .map_err(|e| FileDownloadError::FileContentsErrorResponse(raw_url.clone(), e))?
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| FileDownloadError::FileContentsDownload(raw_url.clone(), e))?;

    Ok(FileContents(result))
}

// Custom deserializer for SHA hex string to Vec<u8>
fn deserialize_sha<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let hex_string = String::deserialize(deserializer)?;

    // Use the hex crate to decode the SHA
    hex::decode(&hex_string).map_err(D::Error::custom)
}

fn as_hex<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let hex_string = format!("0x{}", hex::encode(bytes));
    serializer.serialize_str(&hex_string)
}

fn serialize_iso8601<S>(dt: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = dt
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&s)
}
