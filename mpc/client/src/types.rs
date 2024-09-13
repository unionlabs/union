use serde::Serialize;

#[derive(PartialEq, Eq, Debug, Clone)]
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

impl Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Status::Idle => "idle",
            Status::Initializing => "initializing",
            Status::DownloadStarted(_) => "downloadStarted",
            Status::Downloading(_, _) => "downloading",
            Status::DownloadEnded(_) => "downloadEnded",
            Status::ContributionStarted => "contributionStarted",
            Status::ContributionEnded => "contributionEnded",
            Status::UploadStarted(_) => "uploadStarted",
            Status::UploadEnded(_) => "uploadEnded",
            Status::Failed(_) => "failed",
            Status::Successful => "successful",
        })
    }
}
