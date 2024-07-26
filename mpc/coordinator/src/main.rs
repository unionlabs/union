pub mod fsm;

use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
    marker::PhantomData,
    sync::{
        mpsc::{channel, Sender, TryRecvError},
        Arc, RwLock,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{Parser, Subcommand};
use mpc_shared::{
    phase2_verify, types::Contribution, CONTRIBUTION_CHUNKS, CONTRIBUTION_CHUNK_SIZE,
    CONTRIBUTION_CHUNK_SIZE_FINAL,
};
use priority_queue::PriorityQueue;
use rocket::{
    data::{Limits, ToByteUnit},
    fs::TempFile,
    get,
    http::{
        hyper::{header::AUTHORIZATION, HeaderValue},
        ContentType, Cookie, CookieJar, Header, Status,
    },
    post,
    request::{FromParam, FromRequest, Outcome, Request},
    response::status::{self, Forbidden, NotFound, Unauthorized},
    routes,
    serde::{
        json::{json, Json, Value},
        Deserialize, Serialize,
    },
    Response, State,
};
use rocket_authorization::{AuthError, Authorization, Credential};
use rocket_seek_stream::SeekStream;
use sha2::Digest;

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
        phase2_payload_path: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(crate = "rocket::serde")]
struct Contributor {
    token: String,
}

#[derive(Debug)]
struct AppState {
    queue: PriorityQueue<Contributor, usize>,
    processed: HashMap<Contributor, Contribution>,
    payload: Vec<u8>,
    payload_hash: Vec<u8>,
    contributor: Contributor,
    upload_index: Index,
    contribution_payload: Vec<u8>,
    machine: fsm::union_mpc::StateMachine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AppMessage {
    Join {
        token: String,
    },
    ContributePartial {
        token: String,
        index: Index,
        payload_fraction: Vec<u8>,
    },
}

struct Current;
struct Any;
struct AuthContributor<T>(String, PhantomData<T>);
#[rocket::async_trait]
impl Authorization for AuthContributor<Current> {
    const KIND: &'static str = "Bearer";
    async fn parse(_: &str, token: &str, request: &Request) -> Result<Self, AuthError> {
        let app_state = request
            .rocket()
            .state::<Arc<RwLock<AppState>>>()
            .unwrap()
            .read()
            .unwrap();
        if app_state.machine.state() == &fsm::union_mpc::State::AwaitContribution
            && app_state.contributor.token == token
        {
            Ok(Self(token.to_string(), PhantomData))
        } else {
            Err(AuthError::Unauthorized)
        }
    }
}
#[rocket::async_trait]
impl Authorization for AuthContributor<Any> {
    const KIND: &'static str = "Bearer";
    async fn parse(_: &str, token: &str, request: &Request) -> Result<Self, AuthError> {
        Ok(Self(token.to_string(), PhantomData))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(crate = "rocket::serde")]
struct Index(u8);
impl<'a> FromParam<'a> for Index {
    type Error = ();
    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        let result = u8::from_param(param).map_err(|_| ())?;
        if result as usize <= CONTRIBUTION_CHUNKS {
            Ok(Index(result))
        } else {
            Err(())
        }
    }
}

#[get("/contribution")]
async fn contribution(
    c: Credential<AuthContributor<Any>>,
    app_state: &State<Arc<RwLock<AppState>>>,
) -> Result<Json<Contribution>, NotFound<()>> {
    match app_state
        .read()
        .unwrap()
        .processed
        .get(&Contributor { token: c.0 .0 })
    {
        Some(contribution) => Ok(Json(contribution.clone())),
        None => Err(NotFound(())),
    }
}

#[post("/contribute/<index>", data = "<payload>")]
async fn contribute(
    c: Credential<AuthContributor<Current>>,
    index: Index,
    payload: Vec<u8>,
    app_state: &State<Arc<RwLock<AppState>>>,
    tx: &State<Sender<AppMessage>>,
) -> Result<(), Forbidden<()>> {
    tx.send(AppMessage::ContributePartial {
        token: c.0 .0,
        index,
        payload_fraction: payload,
    })
    .unwrap();
    Ok(())
}

#[get("/me")]
async fn me(
    c: Credential<AuthContributor<Current>>,
    app_state: &State<Arc<RwLock<AppState>>>,
) -> Result<Vec<u8>, Forbidden<()>> {
    let hash = { app_state.read().unwrap().payload_hash.clone() };
    Ok(hash)
}

#[get("/state")]
async fn state<'a>(
    c: Credential<AuthContributor<Current>>,
    app_state: &State<Arc<RwLock<AppState>>>,
) -> Result<SeekStream<'a>, Forbidden<()>> {
    let bytes = { app_state.read().unwrap().payload.clone() };
    let len = bytes.len() as u64;
    Ok(SeekStream::with_opts(
        Cursor::new(bytes),
        len,
        "application/octet-stream",
    ))
}

#[post("/join")]
async fn join(c: Credential<AuthContributor<Any>>, tx: &State<Sender<AppMessage>>) -> Status {
    // TODO: verify token
    tx.send(AppMessage::Join { token: c.0 .0 }).unwrap();
    Status::Ok
}

#[get("/")]
fn index() -> &'static str {
    "
      Bruh u ain't mess with the kraken
    "
}

#[rocket::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Start {
            phase2_payload_path,
        } => {
            let payload = std::fs::read(phase2_payload_path).unwrap();
            let payload_hash = sha2::Sha256::new()
                .chain_update(&payload)
                .finalize()
                .to_vec();
            let (tx, rx) = channel::<AppMessage>();
            let initial_contributor = Contributor {
                token: "union".into(),
            };
            let app_state = Arc::new(RwLock::new(AppState {
                queue: PriorityQueue::with_capacity(4096),
                processed: HashMap::with_capacity(4096),
                contributor: initial_contributor,
                payload,
                payload_hash,
                upload_index: Index(0),
                contribution_payload: Vec::new(),
                machine: fsm::union_mpc::StateMachine::new(),
            }));
            let app_state_clone = app_state.clone();
            rocket::tokio::spawn(async move {
                let figment = rocket::Config::figment()
                    .merge(("limits", Limits::new().limit("bytes", 12.mebibytes())));
                rocket::custom(figment)
                    .manage(app_state_clone)
                    .manage(tx)
                    .mount(
                        "/",
                        routes![index, join, me, state, contribute, contribution],
                    )
                    .launch()
                    .await
                    .unwrap();
            });
            let (tx_verify, rx_verify) = channel::<(Vec<u8>, Vec<u8>)>();
            let (tx_verify_result, rx_verify_result) = channel::<bool>();
            rocket::tokio::spawn(async move {
                loop {
                    let (payload, contribution_payload) = rx_verify.recv().unwrap();
                    println!("verifying contribution payloads");
                    if phase2_verify(&payload, &contribution_payload).is_ok() {
                        println!("valid");
                        tx_verify_result.send(true).unwrap();
                    } else {
                        println!("invalid");
                        tx_verify_result.send(false).unwrap();
                    }
                }
            });

            loop {
                {
                    let mut app_state = app_state.write().unwrap();
                    let machine_state = *app_state.machine.state();
                    let mut join = |token: String| {
                        let new_contributor = Contributor { token };
                        if app_state.queue.get(&new_contributor).is_none()
                            && app_state.processed.get(&new_contributor).is_none()
                            && app_state.contributor != new_contributor
                        {
                            let queue_len = app_state.queue.len();
                            const BASE_PRORITY: usize = 1000000;
                            let priority = BASE_PRORITY - queue_len;
                            println!(
                                "contributor joined: {} with priority {}",
                                new_contributor.token, priority
                            );
                            app_state.queue.push(new_contributor, priority);
                            app_state
                                .machine
                                .consume(&fsm::union_mpc::Input::Join)
                                .unwrap();
                        }
                    };
                    match machine_state {
                        fsm::union_mpc::State::InitContributor => {
                            let input = match app_state.queue.pop() {
                                Some((contributor, _)) => {
                                    println!("new contributor slot");
                                    app_state.contributor = contributor;
                                    app_state.upload_index = Index(0);
                                    app_state.contribution_payload.clear();
                                    fsm::union_mpc::Input::ContributorSet
                                }
                                None => {
                                    println!("no contributor");
                                    fsm::union_mpc::Input::NoContributor
                                }
                            };
                            app_state.machine.consume(&input).unwrap();
                        }
                        fsm::union_mpc::State::AwaitContributor => match rx.try_recv() {
                            Ok(message) => match message {
                                AppMessage::Join { token } => {
                                    join(token);
                                }
                                _ => {}
                            },
                            Err(TryRecvError::Empty) => {}
                            Err(e) => {
                                println!("error in awaiting contributor {:?}", e);
                                break;
                            }
                        },
                        fsm::union_mpc::State::AwaitContribution => match rx.try_recv() {
                            Ok(message) => match message {
                                AppMessage::Join { token } => {
                                    join(token);
                                }
                                AppMessage::ContributePartial {
                                    token,
                                    index: Index(index),
                                    mut payload_fraction,
                                } => {
                                    if app_state.contributor.token == token
                                        && app_state.upload_index == Index(index)
                                    {
                                        let expected_len = if (index as usize) < CONTRIBUTION_CHUNKS
                                        {
                                            CONTRIBUTION_CHUNK_SIZE
                                        } else {
                                            CONTRIBUTION_CHUNK_SIZE_FINAL
                                        };
                                        if payload_fraction.len() == expected_len {
                                            println!("partial contribution chunk {}", index);
                                            app_state.upload_index = Index(index + 1);
                                            app_state
                                                .contribution_payload
                                                .append(&mut payload_fraction);
                                            if index as usize == CONTRIBUTION_CHUNKS {
                                                println!("contribution complete");
                                                tx_verify
                                                    .send((
                                                        app_state.payload.clone(),
                                                        app_state.contribution_payload.clone(),
                                                    ))
                                                    .unwrap();
                                                app_state.payload =
                                                    app_state.contribution_payload.clone();
                                                app_state
                                                    .machine
                                                    .consume(&fsm::union_mpc::Input::Contribute)
                                                    .unwrap();
                                            }
                                        } else {
                                            println!("invalid chunk length {}", index);
                                        }
                                    }
                                }
                            },
                            Err(TryRecvError::Empty) => {}
                            Err(e) => {
                                println!("error in await {:?}", e);
                                break;
                            }
                        },
                        fsm::union_mpc::State::Verify => match rx_verify_result.try_recv() {
                            Ok(result) => {
                                let timestamp = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs();
                                let contribution = Contribution {
                                    success: result,
                                    timestamp,
                                };
                                let contributor = app_state.contributor.clone();
                                app_state.processed.insert(contributor, contribution);
                                if result {
                                    println!("verification succeeded.");
                                    app_state
                                        .machine
                                        .consume(&fsm::union_mpc::Input::Valid)
                                        .unwrap();
                                } else {
                                    println!("verification failed.");
                                    app_state
                                        .machine
                                        .consume(&fsm::union_mpc::Input::Invalid)
                                        .unwrap();
                                }
                            }
                            Err(TryRecvError::Empty) => {}
                            Err(e) => {
                                println!("error in verify {:?}", e);
                                break;
                            }
                        },
                    };
                }
                rocket::tokio::time::sleep(std::time::Duration::from_millis(1000 / 30)).await;
            }
            Ok(())
        }
    }
}
