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

use async_sqlite::{JournalMode, PoolBuilder};
use base64::{prelude::BASE64_STANDARD, Engine};
use crossterm::event;
use http_body_util::{BodyExt, Full};
use httpdate::parse_http_date;
use hyper::{
    body::{Buf, Bytes},
    service::service_fn,
    Method,
};
use hyper_util::{rt::TokioIo, server::graceful::GracefulShutdown};
use mpc_shared::{phase2_contribute, supabase::SupabaseMPCApi, CONTRIBUTION_SIZE};
use ratatui::{backend::CrosstermBackend, Terminal, Viewport};
use reqwest::header::LOCATION;
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

const SUPABASE_PROJECT: &str = "https://wwqpylbrcpriyaqugzsi.supabase.co";
const ENDPOINT: &str = "/contribute";

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
}

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

type DynError = Box<dyn std::error::Error + Send + Sync>;

async fn contribute(
    tx_status: Sender<Status>,
    Contribute {
        bucket,
        jwt,
        api_key,
        contributor_id,
        payload_id,
    }: Contribute,
) -> Result<(), DynError> {
    let client = SupabaseMPCApi::new(SUPABASE_PROJECT.into(), api_key, jwt);
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
        .download_payload(&current_payload.id, &current_payload.id, |percent| {
            let tx_status = tx_status.clone();
            let current_payload_clone = current_payload.id.clone();
            async move {
                tx_status
                    .send(Status::Downloading(current_payload_clone, percent as u8))
                    .expect("impossible");
            }
        })
        .await?;
    tx_status
        .send(Status::DownloadEnded(current_payload.id.clone()))
        .expect("impossible");
    let phase2_contribution = if let Ok(true) = tokio::fs::metadata(&payload_id)
        .await
        .map(|meta| meta.size() as usize == CONTRIBUTION_SIZE)
    {
        tokio::fs::read(&payload_id).await?
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
        tokio::fs::write(&payload_id, &phase2_contribution).await?;
        phase2_contribution
    };
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
        tx_status
            .send(Status::UploadEnded(payload_id.clone()))
            .expect("impossible");
    }
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
    let response_empty = |status| response(status, BoxBody::default());
    match (req.method(), req.uri().path()) {
        (&Method::POST, ENDPOINT)
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
                        lock.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                            .expect("impossible");
                        tx_status.send(Status::Successful).expect("impossible")
                    }
                    Err(e) => tx_status
                        .send(Status::Failed(format!("{:?}", e)))
                        .expect("impossible"),
                }
            });
            response_empty(hyper::StatusCode::ACCEPTED)
        }
        // FE must poll GET and dispatch accordingly.
        (&Method::POST, ENDPOINT) => response_empty(hyper::StatusCode::SERVICE_UNAVAILABLE),
        (&Method::GET, ENDPOINT) => match latest_status.read().await.clone() {
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
        (&Method::OPTIONS, ENDPOINT) => Ok(hyper::Response::builder()
            .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(
                hyper::header::ACCESS_CONTROL_ALLOW_HEADERS,
                hyper::header::CONTENT_TYPE,
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
    let handle = tokio::spawn(async move {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0x1337));
        let listener = TcpListener::bind(addr).await.unwrap();
        loop {
            tokio::select! {
                Ok((stream, _)) = listener.accept() => {
                    let io = TokioIo::new(stream);
                    let status_clone = status_clone.clone();
                    let tx_status_clone = tx_status.clone();
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
                        if let Err(err) = fut.await {
                            eprintln!("error serving connection: {:?}", err);
                        }
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
    input_and_status_handling(status, rx_status, tx_ui).await;
    ui::run_ui(&mut terminal, rx_ui).await?;
    crossterm::terminal::disable_raw_mode()?;
    terminal.clear()?;
    token.cancel();
    handle.await.expect("impossible");
    std::process::exit(0);
}
