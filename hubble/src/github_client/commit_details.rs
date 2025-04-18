use std::{fmt::Display, sync::LazyLock};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::{error, trace};

use crate::github_client::GitCommitHash;

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent("reqwest")
        .build()
        .expect("client can be built")
});

#[derive(Debug, thiserror::Error)]
pub enum CommitDetailsError {
    #[error("error requesting commit details from {0}: {1}")]
    SendRequest(String, #[source] reqwest::Error),

    #[error("error response fetching commit details from {0}: {1}")]
    ErrorResponse(String, #[source] reqwest::Error),

    #[error("error downloading commit details from {0}: {1}")]
    Download(String, #[source] reqwest::Error),

    #[error("error parsing json commit details from {0}: {1}")]
    Decode(String, #[source] serde_json::Error),

    #[error("no commit for {0}")]
    NoCommit(String),
}

#[derive(Debug, Serialize)]
pub struct CommitDetails {
    pub commit_hash: GitCommitHash,
    #[serde(serialize_with = "serialize_iso8601")]
    pub commit_timestamp: OffsetDateTime,
}

impl Display for CommitDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "commit_hash: {}, commit_timestamp: {}",
            self.commit_hash, self.commit_timestamp,
        ))
    }
}

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
    sha: GitCommitHash,
    commit: CommitInfo,
}

pub async fn fetch_commit_details_by_branch(
    repo: &str,
    path: &str,
    branch: &str,
) -> Result<CommitDetails, CommitDetailsError> {
    trace!("fetch_commit_details (branch): {repo}/{path}/{branch}");

    let commits_url = format!(
        "https://api.github.com/repos/{}/commits?path={}&branch={}&per_page=1",
        repo, path, branch
    );

    fetch_commit_details_by_url(&commits_url).await
}

async fn fetch_commit_details_by_url(url: &str) -> Result<CommitDetails, CommitDetailsError> {
    trace!("fetch_commit_details (url): {url}");

    let commits_response = CLIENT
        .get(url)
        .send()
        .await
        .map_err(|e| CommitDetailsError::SendRequest(url.into(), e))?
        .error_for_status()
        .map_err(|e| CommitDetailsError::ErrorResponse(url.into(), e))?
        .text()
        .await
        .map_err(|e| CommitDetailsError::Download(url.into(), e))?;

    let commits: Vec<GitCommit> = serde_json::from_str(&commits_response)
        .map_err(|e| CommitDetailsError::Decode(url.into(), e))?;

    let commit = commits
        .first()
        .ok_or_else(|| CommitDetailsError::NoCommit(url.into()))?;

    trace!("commit_details of {url} => {commit:?}");

    // Flatten into commit details
    Ok(CommitDetails {
        commit_hash: commit.sha.clone(),
        commit_timestamp: commit.commit.author.date,
    })
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
