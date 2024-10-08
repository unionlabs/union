mod types;
mod ui;

use std::{
    io,
    net::SocketAddr,
    os::unix::fs::MetadataExt,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant, UNIX_EPOCH},
};

use async_sqlite::{rusqlite::OpenFlags, JournalMode, PoolBuilder};
use base64::{prelude::BASE64_STANDARD, Engine};
use crossterm::{cursor::Show, event, execute};
use http_body_util::{BodyExt, Full};
use httpdate::parse_http_date;
use hyper::{
    body::{Buf, Bytes},
    service::service_fn,
    Method,
};
use hyper_util::{rt::TokioIo, server::graceful::GracefulShutdown};
use mpc_shared::{phase2_contribute, signed_message, supabase::SupabaseMPCApi, CONTRIBUTION_SIZE};
use pgp::{
    cleartext::CleartextSignedMessage,
    crypto::{hash::HashAlgorithm, sym::SymmetricKeyAlgorithm},
    types::SecretKeyTrait,
    ArmorOptions, Deserializable, KeyType, SecretKeyParamsBuilder, SignedSecretKey,
};
use ratatui::{backend::CrosstermBackend, Terminal, Viewport};
use reqwest::{header::LOCATION, Body};
use serde::Deserialize;
use tokio::{
    net::TcpListener,
    sync::{
        broadcast::{self, Receiver, Sender},
        mpsc, oneshot, RwLock,
    },
};
use tokio_util::sync::CancellationToken;
use types::Status;

const CONTRIBUTE_ENDPOINT: &str = "/contribute";
const SK_ENDPOINT: &str = "/secret_key";

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Contribute {
    supabase_project: String,
    bucket: String,
    jwt: String,
    api_key: String,
    contributor_id: String,
    payload_id: String,
    user_email: String,
}

#[derive(thiserror::Error, Debug, Clone)]
enum Error {
    #[error("we are not the current contributor.")]
    NotCurrentContributor,
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
    #[error("pgp key couldn't be found")]
    PGPKeyNotFound,
}

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

type DynError = Box<dyn std::error::Error + Send + Sync>;

fn successful_file(contributor_id: &str) -> String {
    temp_file(
        contributor_id,
        &format!("{}.zkgm_successful", contributor_id),
    )
}

fn temp_dir(contributor_id: &str) -> String {
    format!("{contributor_id}.zkgm")
}

fn temp_file(contributor_id: &str, file: &str) -> String {
    let dir = temp_dir(contributor_id);
    format!("{dir}/{file}")
}

fn pgp_secret_file(email: &str) -> String {
    format!("{email}.contrib_key.sk.asc")
}

async fn is_already_successful(contributor_id: &str) -> bool {
    tokio::fs::metadata(successful_file(contributor_id))
        .await
        .is_ok()
}

async fn wait_successful(latest_status: Arc<RwLock<Status>>) {
    loop {
        if *latest_status.read().await == Status::Successful {
            tokio::time::sleep(tokio::time::Duration::from_millis(10000)).await;
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
}

async fn create_temp_dir(contributor_id: &str) -> Result<(), DynError> {
    if let Err(_) = tokio::fs::metadata(temp_dir(contributor_id)).await {
        tokio::fs::create_dir(temp_dir(contributor_id)).await?;
    }
    Ok(())
}

async fn remove_temp_dir(contributor_id: &str) -> Result<(), DynError> {
    tokio::fs::remove_dir_all(temp_dir(contributor_id)).await?;
    Ok(())
}

fn generate_pgp_key(email: String) -> SignedSecretKey {
    let mut key_params = SecretKeyParamsBuilder::default();
    key_params
        .key_type(KeyType::EdDSA)
        .can_certify(false)
        .can_sign(true)
        .can_encrypt(false)
        .primary_user_id(email)
        .preferred_symmetric_algorithms(
            [SymmetricKeyAlgorithm::AES256].to_vec().try_into().unwrap(),
        )
        .preferred_hash_algorithms([HashAlgorithm::None].to_vec().try_into().unwrap());
    let secret_key_params = key_params.build().expect("impossible");
    let secret_key = secret_key_params.generate().expect("impossible");
    let passwd_fn = || String::new();
    let signed_secret_key = secret_key.sign(passwd_fn).expect("impossible");
    signed_secret_key
}

async fn contribute(
    tx_status: Sender<Status>,
    Contribute {
        supabase_project,
        bucket,
        jwt,
        api_key,
        contributor_id,
        payload_id,
        user_email,
        ..
    }: Contribute,
) -> Result<(), DynError> {
    create_temp_dir(&contributor_id).await?;
    if is_already_successful(&contributor_id).await {
        remove_temp_dir(&contributor_id).await?;
        tx_status.send(Status::Successful).expect("impossible");
        return Ok(());
    }
    let pgp_secret_file = pgp_secret_file(&user_email);
    let mut secret_key = if let Ok(_) = tokio::fs::metadata(&pgp_secret_file).await {
        SignedSecretKey::from_armor_single::<&[u8]>(
            tokio::fs::read(&pgp_secret_file).await?.as_ref(),
        )
        .expect("impossible")
        .0
    } else {
        return Err(Error::PGPKeyNotFound.into());
    };
    let client = SupabaseMPCApi::new(supabase_project.clone(), api_key, jwt);
    let current_contributor = client
        .current_contributor()
        .await?
        .ok_or(Error::ContributorNotFound)?;
    if current_contributor.id != contributor_id {
        return Err(Error::NotCurrentContributor.into());
    }
    let current_payload = client
        .current_payload()
        .await?
        .ok_or(Error::PayloadNotFound)?;
    tx_status
        .send(Status::DownloadStarted(current_payload.id.clone()))
        .expect("impossible");
    let payload = client
        .download_payload(
            &current_payload.id,
            &temp_file(&contributor_id, &current_payload.id),
            |percent| {
                let tx_status = tx_status.clone();
                let current_payload_clone = current_payload.id.clone();
                async move {
                    tx_status
                        .send(Status::Downloading(current_payload_clone, percent as u8))
                        .expect("impossible");
                }
            },
        )
        .await?;
    tx_status
        .send(Status::DownloadEnded(current_payload.id.clone()))
        .expect("impossible");
    let phase2_contribution = if let Ok(true) =
        tokio::fs::metadata(temp_file(&contributor_id, &payload_id))
            .await
            .map(|meta| meta.size() as usize == CONTRIBUTION_SIZE)
    {
        tokio::fs::read(temp_file(&contributor_id, &payload_id)).await?
    } else {
        tx_status
            .send(Status::ContributionStarted)
            .expect("impossible");
        let (tx_contrib, rx_contrib) = oneshot::channel();
        let handle = tokio::task::spawn_blocking(move || {
            tx_contrib
                .send(phase2_contribute(&payload))
                .expect("impossible");
        });
        let phase2_contribution = rx_contrib.await??;
        handle.await?;
        tx_status
            .send(Status::ContributionEnded)
            .expect("impossible");
        tokio::fs::write(
            temp_file(&contributor_id, &payload_id),
            &phase2_contribution,
        )
        .await?;
        phase2_contribution
    };

    // ------------------------
    // Sign and submits the sig
    // Gnark phase2 contribution appends the sha256 hash at the end
    let phase2_contribution_hash = &phase2_contribution[phase2_contribution.len() - 32..];
    let signature = CleartextSignedMessage::sign(
        &signed_message(
            &current_payload.id,
            &payload_id,
            &hex::encode(phase2_contribution_hash),
        ),
        &mut secret_key,
        || String::new(),
    )
    .expect("impossible");
    let public_key = secret_key
        .public_key()
        .sign(&secret_key, || String::new())
        .expect("impossible")
        .to_armored_bytes(ArmorOptions::default())
        .expect("impossible");
    client
        .insert_contribution_signature(
            current_contributor.id,
            public_key,
            signature
                .to_armored_bytes(ArmorOptions::default())
                .expect("impossible"),
        )
        .await?;
    let pool = PoolBuilder::new()
        .path(temp_file(&contributor_id, "state.sqlite3"))
        .flags(
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX
                | OpenFlags::SQLITE_OPEN_URI,
        )
        .journal_mode(JournalMode::Wal)
        .num_conns(1)
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
            let mut stmt = conn.prepare(
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
            upload_location = None;
        }
    }
    let upload_location = match upload_location {
        Some(location) => location,
        None => {
            // =====================================================
            // https://tus.io/protocols/resumable-upload#creation ==
            // =====================================================
            let response = upload_client
                .post(format!("{supabase_project}/storage/v1/upload/resumable"))
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
                .await?
                .error_for_status()?;
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
                let mut stmt =
                    conn.prepare("INSERT INTO resumable_upload (location, expire) VALUES (?, ?)")?;
                let r = stmt.execute((location_clone, expire_timestamp))?;
                assert!(r == 1);
                Ok(())
            })
            .await?;
            location
        }
    };
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
    if upload_offset < upload_length {
        tx_status
            .send(Status::UploadStarted(payload_id.clone()))
            .expect("impossible");
        // ==================================================
        // https://tus.io/protocols/resumable-upload#patch ==
        // ==================================================
        let chunks = phase2_contribution
            .into_iter()
            .skip(upload_offset)
            .collect::<Vec<_>>()
            // 4mb
            .chunks(4 * 1024 * 1024)
            .map(|x| Ok::<_, std::io::Error>(x.to_vec()))
            .collect::<Vec<_>>();
        upload_client
            .patch(&upload_location)
            .header("Tus-Resumable", "1.0.0")
            .header("Content-Type", "application/offset+octet-stream")
            .header("Upload-Offset", upload_offset.to_string())
            .body(Body::wrap_stream(futures_util::stream::iter(chunks)))
            .send()
            .await?
            .error_for_status()?;
        tx_status
            .send(Status::UploadEnded(payload_id.clone()))
            .expect("impossible");
    }
    pool.close().await?;
    tokio::fs::write(successful_file(&contributor_id), &[0xBE, 0xEF]).await?;
    Ok(())
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

async fn handle(
    lock: Arc<AtomicBool>,
    tx_status: Sender<Status>,
    latest_status: Arc<RwLock<Status>>,
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<BoxBody>, DynError> {
    let response = |status, body| {
        Ok(hyper::Response::builder()
            .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .status(status)
            .body(body)
            .unwrap())
    };
    let file_response = |status, body, name| {
        Ok(hyper::Response::builder()
            .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(hyper::header::CONTENT_TYPE, "application/octet-stream")
            .header(
                hyper::header::CONTENT_DISPOSITION,
                format!("attachment; filename={name}"),
            )
            .status(status)
            .body(body)
            .unwrap())
    };
    let response_empty = |status| response(status, BoxBody::default());
    let path = req.uri().path();
    match (req.method(), path) {
        (&Method::POST, SK_ENDPOINT) => {
            let whole_body = req.collect().await?.aggregate();
            let email = serde_json::from_reader::<_, String>(whole_body.reader())?;
            let pgp_secret_file = pgp_secret_file(&email);
            let guard = latest_status.write().await;
            let result = {
                if let Err(_) = tokio::fs::metadata(&pgp_secret_file).await {
                    let secret_key = generate_pgp_key(email);
                    let secret_key_serialized = secret_key
                        .to_armored_bytes(ArmorOptions::default())
                        .expect("impossible");
                    tokio::fs::write(pgp_secret_file, &secret_key_serialized).await?;
                    response_empty(hyper::StatusCode::CREATED)
                } else {
                    response_empty(hyper::StatusCode::OK)
                }
            };
            drop(guard);
            result
        }
        (&Method::GET, _) if path.starts_with(SK_ENDPOINT) => {
            if let Some(email) = path
                .strip_prefix(SK_ENDPOINT)
                .and_then(|x| x.strip_prefix("/"))
            {
                let pgp_secret_file = pgp_secret_file(email);
                if let Err(_) = tokio::fs::metadata(&pgp_secret_file).await {
                    response_empty(hyper::StatusCode::NOT_FOUND)
                } else {
                    let content = tokio::fs::read(&pgp_secret_file).await?;
                    file_response(hyper::StatusCode::OK, full(content), pgp_secret_file)
                }
            } else {
                response_empty(hyper::StatusCode::NOT_FOUND)
            }
        }
        (&Method::POST, CONTRIBUTE_ENDPOINT)
            if lock
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok() =>
        {
            tx_status.send(Status::Initializing).expect("impossible");
            tokio::spawn(async move {
                let result = (|| async {
                    let whole_body = req.collect().await?.aggregate();
                    contribute(
                        tx_status.clone(),
                        serde_json::from_reader(whole_body.reader())?,
                    )
                    .await?;
                    Ok::<_, DynError>(())
                })()
                .await;
                match result {
                    Ok(_) => {
                        tx_status.send(Status::Successful).expect("impossible");
                    }
                    Err(e) => {
                        tx_status
                            .send(Status::Failed(format!("{:?}", e)))
                            .expect("impossible");
                    }
                }
            });
            response_empty(hyper::StatusCode::ACCEPTED)
        }
        // FE must poll GET and dispatch accordingly.
        (&Method::POST, CONTRIBUTE_ENDPOINT) => {
            response_empty(hyper::StatusCode::SERVICE_UNAVAILABLE)
        }
        (&Method::GET, CONTRIBUTE_ENDPOINT) => match latest_status.read().await.clone() {
            Status::Failed(e) => {
                lock.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                    .expect("impossible");
                // Only idle if the FE poll after a failure.
                tx_status.send(Status::Idle).expect("impossible");
                response(
                    hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    full(serde_json::to_vec(&format!("{:#?}", e)).expect("impossible")),
                )
            }
            x => response(
                hyper::StatusCode::OK,
                full(serde_json::to_vec(&x).expect("impossible")),
            ),
        },
        // CORS preflight request.
        (&Method::OPTIONS, CONTRIBUTE_ENDPOINT | SK_ENDPOINT) => Ok(hyper::Response::builder()
            .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(
                hyper::header::ACCESS_CONTROL_ALLOW_HEADERS,
                format!(
                    "{}, {}",
                    hyper::header::CONTENT_TYPE,
                    hyper::header::CONTENT_DISPOSITION
                ),
            )
            .header(
                hyper::header::ACCESS_CONTROL_ALLOW_METHODS,
                format!(
                    "{}, {}, {}",
                    Method::OPTIONS.as_str(),
                    Method::GET.as_str(),
                    Method::POST.as_str()
                ),
            )
            .status(hyper::StatusCode::OK)
            .body(BoxBody::default())
            .unwrap()),
        _ => response_empty(hyper::StatusCode::NOT_FOUND),
    }
}

async fn input_and_status_handling(
    latest_status: Arc<RwLock<Status>>,
    mut rx_status: Receiver<Status>,
    tx_ui: mpsc::UnboundedSender<ui::Event>,
) {
    let tx_ui_clone = tx_ui.clone();
    tokio::spawn(async move {
        while let Ok(status) = rx_status.recv().await {
            *latest_status.write().await = status.clone();
            if let Err(_) = tx_ui_clone.send(ui::Event::NewStatus(status)) {
                break;
            }
        }
    });
    tokio::spawn(async move {
        let tick_rate = Duration::from_millis(1000 / 60);
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout).unwrap() {
                match event::read().unwrap() {
                    event::Event::Key(key) => tx_ui.send(ui::Event::Input(key)).unwrap(),
                    event::Event::Resize(_, _) => tx_ui.send(ui::Event::Resize).unwrap(),
                    _ => {}
                };
            }
            if last_tick.elapsed() >= tick_rate {
                if let Err(_) = tx_ui.send(ui::Event::Tick) {
                    break;
                }
                last_tick = Instant::now();
            }
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let status = Arc::new(RwLock::new(Status::Idle));
    let lock = Arc::new(AtomicBool::new(false));
    let (tx_status, rx_status) = broadcast::channel(64);
    let graceful = GracefulShutdown::new();
    let status_clone = status.clone();
    let token = CancellationToken::new();
    let token_clone = token.clone();
    let tx_status_clone = tx_status.clone();
    let handle = tokio::spawn(async move {
        let addr = SocketAddr::from(([0, 0, 0, 0], 0x1337));
        let listener = TcpListener::bind(addr).await.unwrap();
        loop {
            tokio::select! {
                Ok((stream, _)) = listener.accept() => {
                    let io = TokioIo::new(stream);
                    let status_clone = status_clone.clone();
                    let tx_status_clone = tx_status_clone.clone();
                    let lock_clone = lock.clone();
                    let conn = hyper::server::conn::http1::Builder::new().serve_connection(
                        io,
                        service_fn(move |req| {
                            handle(
                                lock_clone.clone(),
                                tx_status_clone.clone(),
                                status_clone.clone(),
                                req,
                            )
                        }),
                    );
                    let fut = graceful.watch(conn);
                    tokio::task::spawn(async move {
                        let _ = fut.await;
                    });
                }
                _ = token_clone.cancelled() => {
                    break;
                }
            }
        }
        graceful.shutdown().await;
    });
    // Dispatch terminal
    let (tx_ui, rx_ui) = mpsc::unbounded_channel();
    crossterm::terminal::enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::with_options(
        backend,
        ratatui::TerminalOptions {
            viewport: Viewport::Inline(8),
        },
    )?;
    input_and_status_handling(status.clone(), rx_status, tx_ui).await;
    tokio::select! {
        _ = ui::run_ui(&mut terminal, rx_ui) => {}
        _ = wait_successful(status) => {}
    }
    terminal.clear()?;
    crossterm::terminal::disable_raw_mode()?;
    let _ = execute!(io::stdout(), Show);
    token.cancel();
    handle.await.expect("impossible");
    std::process::exit(0);
}
