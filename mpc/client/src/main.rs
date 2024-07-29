use std::{
    convert::Infallible, io::SeekFrom, net::SocketAddr, os::unix::fs::MetadataExt, path::Path,
    str::FromStr, time::UNIX_EPOCH,
};

use async_sqlite::{rusqlite::params, JournalMode, Pool, PoolBuilder};
use base64::{prelude::BASE64_STANDARD, Engine};
use clap::{Parser, Subcommand};
use http_body_util::{BodyExt, Full};
use httpdate::parse_http_date;
use hyper::{body::Buf, service::service_fn, Method};
use hyper_util::rt::TokioIo;
use mpc_shared::{
    phase2_contribute, phase2_verify,
    types::{Contribution, ContributorId, PayloadId},
    CONTRIBUTION_CHUNKS, CONTRIBUTION_CHUNK_SIZE, CONTRIBUTION_SIZE,
};
use postgrest::Postgrest;
use reqwest::{
    header::{
        HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_ENCODING, CONTENT_LENGTH,
        LOCATION, RANGE, TRANSFER_ENCODING,
    },
    StatusCode,
};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{empty, AsyncSeekExt, AsyncWriteExt},
    net::TcpListener,
};

static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOTFOUND: &[u8] = b"Not Found";

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Contribute {
    bucket: String,
    jwt: String,
    api_key: String,
    contributor_id: String,
    payload_id: String,
}

#[derive(thiserror::Error, Debug, Clone)]
enum Error {
    #[error("couldn't find expected header: {0}")]
    HeaderNotFound(String),
    #[error("current contributor not found.")]
    ContributorNotFound,
    #[error("current payload not found.")]
    PayloadNotFound,
    #[error(transparent)]
    Phase2ContributionFailed(#[from] mpc_shared::Phase2ContributionError),
    #[error(transparent)]
    Phase2VerificationFailed(#[from] mpc_shared::Phase2VerificationError),
}

async fn get_state_file(path: &str) -> Vec<u8> {
    if !tokio::fs::try_exists(path).await.unwrap() {
        tokio::fs::write(path, []).await.unwrap();
    }
    tokio::fs::read(path).await.unwrap()
}

type BoxBody = http_body_util::combinators::BoxBody<hyper::body::Bytes, hyper::Error>;

async fn handle(
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/contribute") => {
            let whole_body = req.collect().await?.aggregate();
            let Contribute {
                bucket,
                jwt,
                api_key,
                contributor_id,
                payload_id,
            } = serde_json::from_reader(whole_body.reader())?;
            const SUPABASE_PROJECT: &str = "https://bffcolwcakqrhlznyjns.supabase.co";
            const APIKEY: &str = "apikey";
            let authorization_header = format!("Bearer {}", jwt);
            let client = Postgrest::new(format!("{SUPABASE_PROJECT}/rest/v1"))
                .insert_header(APIKEY, api_key)
                .insert_header(AUTHORIZATION, authorization_header.clone());
            let current_contributor = client
                .from("current_contributor_id")
                .select("id")
                .execute()
                .await?
                .json::<Vec<ContributorId>>()
                .await?
                .first()
                .cloned()
                .ok_or(Error::ContributorNotFound)?;
            assert!(
                current_contributor.id == contributor_id,
                "not current contributor."
            );
            let current_payload = client
                .from("current_payload_id")
                .select("payload_id")
                .execute()
                .await?
                .json::<Vec<PayloadId>>()
                .await?
                .first()
                .cloned()
                .ok_or(Error::PayloadNotFound)?;
            let current_payload_download_url = format!(
                "{SUPABASE_PROJECT}/storage/v1/object/contributions/{}",
                &current_payload.id
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
            let state_path = current_payload.id;
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
            let payload = match action {
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
                            println!("eta: chunk {}.", i);
                        }
                        let written = state_file.write(&chunk).await?;
                        assert!(written == chunk.len(), "couldn't write chunk.");
                        state_file.sync_data().await?;
                        i += 1;
                    }
                    println!("download complete");
                    let final_content = tokio::fs::read(&state_path).await?;
                    final_content
                }
                StateFileAction::Done(content) => content,
            };
            let phase2_contribution = if let Ok(true) = tokio::fs::metadata(&payload_id)
                .await
                .map(|meta| meta.size() as usize == CONTRIBUTION_SIZE)
            {
                println!("loading completed contribution...");
                tokio::fs::read(&payload_id).await?
            } else {
                println!("generating contribution, may take some time...");
                let phase2_contribution = phase2_contribute(&payload)?;
                tokio::fs::write(&payload_id, &phase2_contribution).await?;
                phase2_contribution
            };
            println!("uploading contribution...");
            let pool = PoolBuilder::new()
                .path("db.sqlite3")
                .journal_mode(JournalMode::Wal)
                .open()
                .await?;

            pool.conn(|conn| {
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS resumable_upload (
                         location TEXT PRIMARY KEY NOT NULL,
                         create_at TIMESTAMPTZ NOT NULL DEFAULT(unixepoch()),
                         expire TIMSTAMPTZ NOT NULL
                     )",
                    (), // empty list of parameters.
                )?;
                Ok(())
            })
            .await?;
            let mut upload_location = pool
                .conn(move |conn| {
                    let mut stmt = conn.prepare_cached(
                        "SELECT location FROM resumable_upload WHERE expire > unixepoch() LIMIT 1",
                    )?;
                    let mut rows = stmt.query(())?;
                    if let Some(row) = rows.next()? {
                        Ok(Some(row.get::<_, String>(0)?))
                    } else {
                        Ok(None)
                    }
                })
                .await?;
            if let Some(ref location) = upload_location {
                if client
                    .head(location)
                    .header("Tus-Resumable", "1.0.0")
                    .send()
                    .await?
                    .error_for_status()
                    .is_err()
                {
                    upload_location = None;
                }
            }
            let upload_location = match upload_location {
                Some(location) => {
                    println!("location already stored in db.");
                    location
                }
                None => {
                    println!("location not found, generating a new one...");
                    // =====================================================
                    // https://tus.io/protocols/resumable-upload#creation ==
                    // =====================================================
                    let response = client
                        .post(format!("{SUPABASE_PROJECT}/storage/v1/upload/resumable"))
                        .header("Tus-Resumable", "1.0.0")
                        .header("Upload-Length", CONTRIBUTION_SIZE.to_string())
                        .header(
                            "Upload-Metadata",
                            format!(
                                "bucketName {},objectName {}",
                                BASE64_STANDARD.encode(&bucket),
                                BASE64_STANDARD.encode(&payload_id)
                            ),
                        )
                        .send()
                        .await?;
                    let location = response
                        .headers()
                        .get(LOCATION)
                        .ok_or(Error::HeaderNotFound(LOCATION.as_str().into()))?
                        .to_str()?
                        .to_string();
                    let expire = response
                        .headers()
                        .get("Upload-Expires")
                        .ok_or(Error::HeaderNotFound("Upload-Expires".into()))?
                        .to_str()?
                        .into();
                    let expire = parse_http_date(expire)?;
                    let expire_timestamp = expire.duration_since(UNIX_EPOCH)?.as_secs();
                    let location_clone = location.clone();
                    pool.conn(move |conn| {
                        let mut stmt = conn.prepare_cached(
                            "INSERT INTO resumable_upload (location, expire) VALUES (?, ?)",
                        )?;
                        let r = stmt.execute((location_clone, expire_timestamp))?;
                        assert!(r == 1);
                        Ok(())
                    })
                    .await?;
                    location
                }
            };

            println!("upload location: {upload_location}");

            // =================================================
            // https://tus.io/protocols/resumable-upload#head ==
            // =================================================
            let response = client
                .head(&upload_location)
                .header("Tus-Resumable", "1.0.0")
                .send()
                .await?
                .error_for_status()?;
            let upload_length = usize::from_str(
                response
                    .headers()
                    .get("Upload-Length")
                    .ok_or(Error::HeaderNotFound("Upload-Length".into()))?
                    .to_str()?,
            )?;
            let upload_offset = usize::from_str(
                response
                    .headers()
                    .get("Upload-Offset")
                    .ok_or(Error::HeaderNotFound("Upload-Offset".into()))?
                    .to_str()?,
            )?;
            assert!(upload_length == CONTRIBUTION_SIZE, "invalid upload-length.");
            println!("upload-offset: {}", upload_offset);
            if upload_offset < upload_length {
                println!("uploading contribution...");
                // ==================================================
                // https://tus.io/protocols/resumable-upload#patch ==
                // ==================================================
                client
                    .patch(&upload_location)
                    .header("Tus-Resumable", "1.0.0")
                    .header("Content-Type", "application/offset+octet-stream")
                    .header("Upload-Offset", upload_offset.to_string())
                    .body(
                        phase2_contribution
                            .into_iter()
                            .skip(upload_offset)
                            .collect::<Vec<_>>(),
                    )
                    .send()
                    .await?
                    .error_for_status()?;
            }
            println!("upload complete.");
            Ok(hyper::Response::builder()
                .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(hyper::header::CONTENT_TYPE, "application/json")
                .status(hyper::StatusCode::OK)
                .body(BoxBody::default())
                .unwrap())
        }
        // Preflight options request from the browser.
        (&Method::OPTIONS, "/contribute") => Ok(hyper::Response::builder()
            .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(
                hyper::header::ACCESS_CONTROL_ALLOW_HEADERS,
                hyper::header::CONTENT_TYPE,
            )
            .header(hyper::header::ACCESS_CONTROL_ALLOW_METHODS, "POST, OPTIONS")
            .status(hyper::StatusCode::OK)
            .body(BoxBody::default())
            .unwrap()),
        _ => Ok(hyper::Response::builder()
            .status(hyper::StatusCode::NOT_FOUND)
            .body(full(NOTFOUND))
            .unwrap()),
    }
}

fn full<T: Into<hyper::body::Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0x1337));
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service_fn(handle))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
