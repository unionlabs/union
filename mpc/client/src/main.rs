use std::{
    net::SocketAddr,
    os::unix::fs::MetadataExt,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::UNIX_EPOCH,
};

use async_sqlite::{JournalMode, PoolBuilder};
use base64::{prelude::BASE64_STANDARD, Engine};
use http_body_util::BodyExt;
use httpdate::parse_http_date;
use hyper::{body::Buf, service::service_fn, Method};
use hyper_util::rt::TokioIo;
use mpc_shared::{phase2_contribute, supabase::SupabaseMPCApi, CONTRIBUTION_SIZE};
use reqwest::header::LOCATION;
use serde::Deserialize;
use tokio::net::TcpListener;

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

type BoxBody = http_body_util::combinators::BoxBody<hyper::body::Bytes, hyper::Error>;

type DynError = Box<dyn std::error::Error + Send + Sync>;

async fn contribute(
    Contribute {
        bucket,
        jwt,
        api_key,
        contributor_id,
        payload_id,
    }: Contribute,
) -> Result<(), DynError> {
    const SUPABASE_PROJECT: &str = "https://bffcolwcakqrhlznyjns.supabase.co";
    let client = SupabaseMPCApi::new(SUPABASE_PROJECT.into(), api_key, jwt);
    let current_contributor = client
        .current_contributor()
        .await?
        .ok_or(Error::ContributorNotFound)?;
    assert!(
        current_contributor.id == contributor_id,
        "not current contributor."
    );
    let current_payload = client
        .current_payload()
        .await?
        .ok_or(Error::PayloadNotFound)?;
    let payload = client
        .download_payload(&current_payload.id, &current_payload.id)
        .await?;
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
    let upload_client = client.new_reqwest_builder()?.build()?;
    if let Some(ref location) = upload_location {
        if upload_client
            .head(location)
            .header("Tus-Resumable", "1.0.0")
            .send()
            .await?
            .error_for_status()
            .is_err()
        {
            println!("upload location expired, removing it...");
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
            let response = upload_client
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
    let response = upload_client
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
        upload_client
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
    Ok(())
}

async fn handle(
    handling: Arc<AtomicBool>,
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<BoxBody>, DynError> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/contribute")
            if handling
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok() =>
        {
            let result = (|| async {
                let whole_body = req.collect().await?.aggregate();
                contribute(serde_json::from_reader(whole_body.reader())?).await?;
                Ok::<_, DynError>(())
            })()
            .await;
            handling
                .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                .expect("impossible");
            result?;
            Ok(hyper::Response::builder()
                .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(hyper::header::CONTENT_TYPE, "application/json")
                .status(hyper::StatusCode::OK)
                .body(BoxBody::default())
                .unwrap())
        }
        // Busy building
        (&Method::POST, "/contribute") => Ok(hyper::Response::builder()
            .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .status(hyper::StatusCode::TOO_MANY_REQUESTS)
            .body(BoxBody::default())
            .unwrap()),
        // CORS preflight request.
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
            .body(BoxBody::default())
            .unwrap()),
    }
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0x1337));
    let listener = TcpListener::bind(addr).await?;
    let handling = Arc::new(AtomicBool::new(false));
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        // TODO: can't we avoid the clone tower?
        let handling_clone = handling.clone();
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = hyper::server::conn::http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| handle(handling_clone.clone(), req)),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
