use std::fmt::Display;

use serde::de::Error;

pub mod commit_details;
pub mod download;

#[derive(Clone, Debug)]
pub struct GitCommitHash(pub [u8; 20]);

impl GitCommitHash {
    pub fn from_slice(commit: &[u8]) -> Result<Self, String> {
        commit.try_into().map(Self).map_err(|_| {
            format!(
                "Commit hash must be exactly 20 bytes ({})",
                hex::encode(commit)
            )
        })
    }

    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl serde::Serialize for GitCommitHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> serde::Deserialize<'de> for GitCommitHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut bytes = [0u8; 20];
        hex::decode_to_slice(&s, &mut bytes).map_err(D::Error::custom)?;
        Ok(Self(bytes))
    }
}

impl Display for GitCommitHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", hex::encode(self.0)))
    }
}
