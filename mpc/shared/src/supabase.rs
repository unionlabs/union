use std::{future::Future, io::SeekFrom, str::FromStr};

use postgrest::Postgrest;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_LENGTH, RANGE},
    ClientBuilder, StatusCode,
};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use crate::{
    types::{Contribution, ContributionSignature, ContributorId, PayloadId},
    CONTRIBUTION_SIZE,
};

const API_KEY: &str = "apikey";

pub type DynError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateFileAction {
    Download(usize),
    Done(Vec<u8>),
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("couldn't find expected header: {0}")]
    HeaderNotFound(String),
    #[error("current contributor not found.")]
    ContributorNotFound,
    #[error("current payload not found.")]
    CurrentPayloadNotFound,
    #[error("next payload not found.")]
    NextPayloadNotFound,
}

pub struct SupabaseMPCApi {
    project_url: String,
    jwt: String,
    client: Postgrest,
}

impl SupabaseMPCApi {
    pub fn new(project_url: String, api_key: String, jwt: String) -> Self {
        let client = Postgrest::new(format!("{project_url}/rest/v1"))
            .insert_header(API_KEY, &api_key)
            .insert_header(AUTHORIZATION, format!("Bearer {}", &jwt));
        Self {
            project_url,
            jwt,
            client,
        }
    }

    pub fn new_reqwest_builder(&self) -> Result<ClientBuilder, DynError> {
        Ok(ClientBuilder::new().default_headers(HeaderMap::from_iter([(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", &self.jwt))?,
        )])))
    }

    pub async fn current_contributor(&self) -> Result<Option<ContributorId>, DynError> {
        Ok(self
            .client
            .from("current_contributor_id")
            .select("id")
            .execute()
            .await?
            .json::<Vec<ContributorId>>()
            .await?
            .first()
            .cloned())
    }

    pub async fn current_payload(&self) -> Result<Option<PayloadId>, DynError> {
        Ok(self
            .client
            .from("current_payload_id")
            .select("payload_id")
            .execute()
            .await?
            .json::<Vec<PayloadId>>()
            .await?
            .first()
            .cloned())
    }

    pub async fn contribution_submitted(&self, contributor_id: &str) -> Result<bool, DynError> {
        Ok(self
            .client
            .from("contribution_submitted")
            .eq("id", &contributor_id)
            .select("id")
            .execute()
            .await?
            .json::<Vec<ContributorId>>()
            .await?
            .len()
            == 1)
    }

    pub async fn contributor_payload(
        &self,
        contributor_id: &str,
    ) -> Result<Option<PayloadId>, DynError> {
        Ok(self
            .client
            .from("queue")
            .eq("id", &contributor_id)
            .select("payload_id")
            .execute()
            .await?
            .json::<Vec<PayloadId>>()
            .await?
            .first()
            .cloned())
    }

    pub async fn insert_contribution(
        &self,
        contributor_id: String,
        success: bool,
    ) -> Result<(), DynError> {
        self.client
            .from("contribution")
            .insert(serde_json::to_string(&Contribution {
                id: contributor_id,
                success,
            })?)
            .execute()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn insert_contribution_signature(
        &self,
        contributor_id: String,
        public_key: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<(), DynError> {
        if let Err(e) = self
            .client
            .from("contribution_signature")
            .insert(serde_json::to_string(&ContributionSignature {
                id: contributor_id,
                public_key: hex::encode(&public_key),
                signature: hex::encode(&signature),
            })?)
            .execute()
            .await?
            .error_for_status()
        {
            // Conflict means we already have an entry.
            // If network drops or something we must allow this to happen.
            if e.status() == Some(StatusCode::CONFLICT) {
                return Ok(());
            } else {
                return Err(e.into());
            }
        }
        Ok(())
    }

    pub async fn contributor_signature(
        &self,
        contributor_id: &str,
    ) -> Result<Option<ContributionSignature>, DynError> {
        Ok(self
            .client
            .from("contribution_signature")
            .eq("id", &contributor_id)
            .select("*")
            .execute()
            .await?
            .json::<Vec<ContributionSignature>>()
            .await?
            .first()
            .cloned())
    }

    pub async fn download_payload<F>(
        &self,
        payload_id: &str,
        payload_output: &str,
        mut progress: impl FnMut(f64) -> F,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Future<Output = ()>,
    {
        let current_payload_download_url = format!(
            "{}/storage/v1/object/contributions/{}",
            &self.project_url, &payload_id
        );
        let client = ClientBuilder::new()
            .default_headers(HeaderMap::from_iter([(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", &self.jwt))?,
            )]))
            .build()?;
        let state_path = payload_output;
        let action = match get_state_file(&state_path).await {
            content if content.len() < CONTRIBUTION_SIZE => {
                StateFileAction::Download(content.len())
            }
            content if content.len() == CONTRIBUTION_SIZE => StateFileAction::Done(content),
            _ => StateFileAction::Download(0),
        };
        match action {
            StateFileAction::Download(start_position) => {
                let mut response = client
                    .get(current_payload_download_url)
                    .header(RANGE, format!("bytes={}-", start_position))
                    .send()
                    .await?
                    .error_for_status()?;
                let headers = response.headers();
                let total_length = start_position
                    + u64::from_str(
                        headers
                            .get(CONTENT_LENGTH)
                            .ok_or(Error::HeaderNotFound(CONTENT_LENGTH.as_str().into()))?
                            .to_str()?,
                    )? as usize;
                assert!(
                    total_length == CONTRIBUTION_SIZE,
                    "contribution length mismatch."
                );
                let mut state_file = tokio::fs::OpenOptions::new()
                    .write(true)
                    .create(false)
                    .open(&state_path)
                    .await?;
                state_file.set_len(start_position as u64).await?;
                state_file
                    .seek(SeekFrom::Start(start_position as u64))
                    .await?;
                let mut i = start_position;
                let mut freq = 0;
                while let Some(chunk) = response.chunk().await? {
                    let k = (i as f64 / CONTRIBUTION_SIZE as f64) * 100.;
                    if freq % 200 == 0 {
                        progress(k).await;
                    }
                    let written = state_file.write(&chunk).await?;
                    assert!(written == chunk.len(), "couldn't write chunk.");
                    i += written;
                    freq += 1;
                }
                state_file.sync_data().await?;
                let final_content = tokio::fs::read(&state_path).await?;
                Ok(final_content)
            }
            StateFileAction::Done(content) => Ok(content),
        }
    }
}

async fn get_state_file(path: &str) -> Vec<u8> {
    if !tokio::fs::try_exists(path).await.unwrap() {
        tokio::fs::write(path, []).await.unwrap();
    }
    tokio::fs::read(path).await.unwrap()
}
