use std::{io::SeekFrom, str::FromStr};

use clap::{Parser, Subcommand};
use mpc_shared::{
    phase2_verify,
    types::{Contribution, ContributorId, PayloadId},
    CONTRIBUTION_SIZE,
};
use postgrest::Postgrest;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_LENGTH, RANGE};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

const SUPABASE_PROJECT: &str = "https://bffcolwcakqrhlznyjns.supabase.co";
const APIKEY: &str = "apikey";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    Start {
        #[arg(short, long)]
        jwt: String,
        #[arg(short, long)]
        api_key: String,
    },
}

#[derive(thiserror::Error, Debug, Clone)]
enum Error {
    #[error("couldn't find expected header: {0}")]
    HeaderNotFound(String),
    #[error("current contributor not found.")]
    ContributorNotFound,
    #[error("current payload not found.")]
    CurrentPayloadNotFound,
    #[error("next payload not found.")]
    NextPayloadNotFound,
}

async fn get_state_file(path: &str) -> Vec<u8> {
    if !tokio::fs::try_exists(path).await.unwrap() {
        tokio::fs::write(path, []).await.unwrap();
    }
    tokio::fs::read(path).await.unwrap()
}

async fn download_payload(
    authorization_header: String,
    payload_id: &str,
    payload_output: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let current_payload_download_url = format!(
        "{SUPABASE_PROJECT}/storage/v1/object/contributions/{}",
        &payload_id
    );
    let client = reqwest::ClientBuilder::new()
        .default_headers(HeaderMap::from_iter([(
            AUTHORIZATION,
            HeaderValue::from_str(&authorization_header)?,
        )]))
        .build()?;
    println!("checking payload file...");
    enum StateFileAction {
        Download(usize),
        Done(Vec<u8>),
    }
    let state_path = payload_output;
    let action = match get_state_file(&state_path).await {
        content if content.len() < CONTRIBUTION_SIZE => {
            println!("partial download, continuing from {}...", content.len());
            StateFileAction::Download(content.len())
        }
        content if content.len() == CONTRIBUTION_SIZE => {
            println!("download complete.");
            StateFileAction::Done(content)
        }
        _ => {
            println!("invalid size detected, redownloading...");
            StateFileAction::Download(0)
        }
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
            println!("state file length: {}", total_length);
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
            let mut i = 0;
            while let Some(chunk) = response.chunk().await? {
                if i % 10 == 0 {
                    println!("Eta: chunk {}.", i);
                }
                let written = state_file.write(&chunk).await?;
                assert!(written == chunk.len(), "couldn't write chunk.");
                state_file.sync_data().await?;
                i += 1;
            }
            println!("download complete");
            let final_content = tokio::fs::read(&state_path).await?;
            Ok(final_content)
        }
        StateFileAction::Done(content) => Ok(content),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    match args.command {
        Command::Start { jwt, api_key } => {
            let authorization_header = format!("Bearer {}", jwt);
            let client = Postgrest::new(format!("{SUPABASE_PROJECT}/rest/v1"))
                .insert_header(APIKEY, api_key)
                .insert_header(AUTHORIZATION, authorization_header.clone());
            loop {
                let current_contributor = {
                    let contributor = client
                        .from("current_contributor_id")
                        .select("id")
                        .execute()
                        .await?
                        .json::<Vec<ContributorId>>()
                        .await?
                        .first()
                        .cloned()
                        .ok_or(Error::ContributorNotFound);
                    match contributor {
                        Ok(contributor) => contributor,
                        Err(_) => {
                            println!("no more contributor to process.");
                            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                            continue;
                        }
                    }
                };
                let current_payload = client
                    .from("current_payload_id")
                    .select("payload_id")
                    .execute()
                    .await?
                    .json::<Vec<PayloadId>>()
                    .await?
                    .first()
                    .cloned()
                    .ok_or(Error::CurrentPayloadNotFound)?;
                let payload_current = download_payload(
                    authorization_header.clone(),
                    &current_payload.id,
                    &current_payload.id,
                )
                .await?;
                println!("awaiting contribution of {}...", &current_contributor.id);
                loop {
                    if client
                        .from("contribution_submitted")
                        .eq("id", &current_contributor.id)
                        .select("id")
                        .execute()
                        .await?
                        .json::<Vec<ContributorId>>()
                        .await?
                        .len()
                        == 1
                    {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                }
                println!("contribution submitted!");
                let next_payload = client
                    .from("queue")
                    .eq("id", &current_contributor.id)
                    .select("payload_id")
                    .execute()
                    .await?
                    .json::<Vec<PayloadId>>()
                    .await?
                    .first()
                    .cloned()
                    .ok_or(Error::NextPayloadNotFound)?;
                let payload_next = download_payload(
                    authorization_header.clone(),
                    &next_payload.id,
                    &next_payload.id,
                )
                .await?;
                if phase2_verify(&payload_current, &payload_next).is_ok() {
                    println!("verification succeeded.");
                    client
                        .from("contribution")
                        .insert(serde_json::to_string(&Contribution {
                            id: current_contributor.id.clone(),
                            success: true,
                        })?)
                        .execute()
                        .await?
                        .error_for_status()?;
                    tokio::fs::remove_file(&current_payload.id).await?;
                } else {
                    println!("verification failed.");
                    client
                        .from("contribution")
                        .insert(serde_json::to_string(&Contribution {
                            id: current_contributor.id.clone(),
                            success: false,
                        })?)
                        .execute()
                        .await?
                        .error_for_status()?;
                }
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        }
    }
}
