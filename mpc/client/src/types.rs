use serde::Serialize;

#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Idle,
    Initializing,
    DownloadStarted(String),
    Downloading(String, u8),
    DownloadEnded(String),
    ContributionStarted,
    ContributionEnded,
    UploadStarted(String),
    UploadEnded(String),
    Failed(String),
    Successful,
}
