use std::{io::SeekFrom, path::Path, str::FromStr};

use clap::{Parser, Subcommand};
use mpc_shared::{
    phase2_contribute, phase2_verify, types::Contribution, CONTRIBUTION_CHUNKS,
    CONTRIBUTION_CHUNK_SIZE, CONTRIBUTION_SIZE,
};
use reqwest::{
    header::{
        HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_ENCODING, CONTENT_LENGTH, RANGE,
        TRANSFER_ENCODING,
    },
    StatusCode,
};
use sha2::Digest;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    Contribute {
        #[arg(short, long)]
        token: String,
    },
    Verify {
        #[arg(short, long)]
        phase2_payload_path: String,
        #[arg(short, long)]
        phase2_contrib_path: String,
    },
}

#[derive(thiserror::Error, Debug, Copy, Clone)]
enum Error {
    #[error(transparent)]
    Phase2ContributionFailed(#[from] mpc_shared::Phase2ContributionError),
    #[error(transparent)]
    Phase2VerificationFailed(#[from] mpc_shared::Phase2VerificationError),
    #[error("Failed to read current state file.")]
    FailedToReadPhase2Payload,
    #[error("Failed to read contrib state file.")]
    FailedToReadPhase2Contrib,
    #[error("Failed to write the contribution file.")]
    FailedToWriteContribution,
}

async fn get_state_file(path: &Path) -> (Vec<u8>, [u8; 32]) {
    if !tokio::fs::try_exists(path).await.unwrap() {
        tokio::fs::write(path, []).await.unwrap();
    }
    let content = tokio::fs::read(path).await.unwrap();
    let hash = sha2::Sha256::new().chain_update(&content).finalize();
    (content, hash.into())
}

#[derive(Debug)]
pub enum CompressionType {
    Gzip,
    Deflate,
    Brotli,
    Zstd,
}

impl FromStr for CompressionType {
    type Err = ();
    fn from_str(value: &str) -> Result<CompressionType, Self::Err> {
        match value {
            "gzip" => Ok(CompressionType::Gzip),
            "deflate" => Ok(CompressionType::Deflate),
            "br" => Ok(CompressionType::Brotli),
            "zstd" => Ok(CompressionType::Zstd),
            _ => Err(()),
        }
    }
}

fn get_compression_type(headers: &HeaderMap) -> Option<CompressionType> {
    let mut compression_type = headers
        .get_all(CONTENT_ENCODING)
        .iter()
        .find_map(|value| value.to_str().ok().and_then(|value| value.parse().ok()));

    if compression_type.is_none() {
        compression_type = headers
            .get_all(TRANSFER_ENCODING)
            .iter()
            .find_map(|value| value.to_str().ok().and_then(|value| value.parse().ok()));
    }

    if compression_type.is_some() {
        if let Some(content_length) = headers.get(CONTENT_LENGTH) {
            if content_length == "0" {
                return None;
            }
        }
    }

    compression_type
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    match args.command {
        Command::Contribute { token } => {
            let url = |path: String| format!("http://localhost:8000/{}", path);
            println!("client token {}", token);
            let client = reqwest::ClientBuilder::new()
                .default_headers(HeaderMap::from_iter([(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
                )]))
                .build()
                .unwrap();
            println!("joining the queue");
            client
                .post(url("join".into()))
                .send()
                .await
                .unwrap()
                .error_for_status()
                .unwrap();
            println!("waiting our turn...");
            let expected_state_file_hash = loop {
                let response = client.get(url("me".into())).send().await.unwrap();
                if response.status().is_success() {
                    println!("finally our turn!");
                    let expected_hash = response.bytes().await.unwrap();
                    break expected_hash;
                }
                let response = client.get(url("contribution".into())).send().await.unwrap();
                match response.status() {
                    StatusCode::OK => {
                        println!(
                            "contribution done: {:?}",
                            response.json::<Contribution>().await.unwrap()
                        );
                        return Ok(());
                    }
                    _ => {}
                }
                println!("checking contribution status...");
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            };
            println!("verifying state file integrity...");
            enum StateFileAction {
                Download(usize),
                Done(Vec<u8>),
            }
            let state_path = Path::new("./state.ph2");
            let action = match get_state_file(&state_path).await {
                (content, _) if content.len() < CONTRIBUTION_SIZE => {
                    println!("partial download, continuing from {}...", content.len());
                    StateFileAction::Download(content.len())
                }
                (content, hash)
                    if content.len() == CONTRIBUTION_SIZE && hash == *expected_state_file_hash =>
                {
                    println!("integrity verified, download complete.");
                    StateFileAction::Done(content)
                }
                _ => {
                    println!("invalid size or invalid hash detected, redownloading...");
                    StateFileAction::Download(0)
                }
            };
            let payload = match action {
                StateFileAction::Download(start_position) => {
                    let mut response = client
                        .get(url("state".into()))
                        .header(RANGE, format!("bytes={}-", start_position))
                        .send()
                        .await
                        .unwrap()
                        .error_for_status()
                        .unwrap();
                    let headers = response.headers();
                    assert!(
                        get_compression_type(headers).is_none(),
                        "compression not supported."
                    );
                    let total_length = start_position
                        + u64::from_str(headers.get(CONTENT_LENGTH).unwrap().to_str().unwrap())
                            .unwrap() as usize;
                    println!("state file length: {}", total_length);
                    assert!(
                        total_length == CONTRIBUTION_SIZE,
                        "contribution length mismatch."
                    );
                    let mut state_file = tokio::fs::OpenOptions::new()
                        .write(true)
                        .create(false)
                        .open(state_path)
                        .await
                        .unwrap();
                    state_file.set_len(start_position as u64).await.unwrap();
                    state_file
                        .seek(SeekFrom::Start(start_position as u64))
                        .await
                        .unwrap();
                    let mut i = 0;
                    while let Some(chunk) = response.chunk().await.unwrap() {
                        if i % 10 == 0 {
                            println!("eta: chunk {}.", i);
                        }
                        let written = state_file.write(&chunk).await.unwrap();
                        assert!(written == chunk.len(), "couldn't write chunk.");
                        state_file.sync_data().await.unwrap();
                        i += 1;
                    }
                    println!("download complete");
                    println!("verifying integrity...");
                    let final_content = tokio::fs::read(state_path).await.unwrap();
                    let final_content_hash = sha2::Sha256::new()
                        .chain_update(&final_content)
                        .finalize()
                        .to_vec();
                    assert!(
                        &final_content_hash == expected_state_file_hash.as_ref(),
                        "invalid file hash after download."
                    );
                    println!("integrity verified");
                    final_content
                }
                StateFileAction::Done(content) => content,
            };
            println!("generating contribution, may take some time...");
            let phase2_contribution = phase2_contribute(&payload).unwrap();
            println!("uploading contribution...");
            for i in 0..=CONTRIBUTION_CHUNKS {
                println!("eta: chunk {}.", i);
                let chunk = &phase2_contribution[i * CONTRIBUTION_CHUNK_SIZE
                    ..std::cmp::min((i + 1) * CONTRIBUTION_CHUNK_SIZE, CONTRIBUTION_SIZE)];
                client
                    .post(url(format!("contribute/{}", i)))
                    .body(chunk.to_vec())
                    .send()
                    .await
                    .unwrap()
                    .error_for_status()
                    .unwrap();
            }
            Ok(())
        }
        Command::Verify {
            phase2_payload_path,
            phase2_contrib_path,
        } => {
            // let phase2_payload =
            //     std::fs::read(phase2_payload_path).map_err(|_| Error::FailedToReadPhase2Payload)?;
            // let phase2_contrib =
            //     std::fs::read(phase2_contrib_path).map_err(|_| Error::FailedToReadPhase2Contrib)?;
            // phase2_verify(&phase2_payload, &phase2_contrib)?;
            Ok(())
        }
    }
}
