use std::{
    ffi::OsString,
    io::{Read, Write},
    os::unix::ffi::{OsStrExt, OsStringExt},
};

use anyhow::Result;
use clap::{
    builder::{
        styling::{AnsiColor, Effects, Styles},
        ArgPredicate,
    },
    Parser, Subcommand,
};
use ibc_union_spec::{ChannelId, Packet, Timestamp};
use serde::Serialize;
use tracing_subscriber::EnvFilter;
use unionlabs::{
    encoding::{DecodeAs, Json},
    primitives::{
        encoding::{HexPrefixed, HexUnprefixed},
        Bytes,
    },
};

pub mod codec;
pub mod deployments;
pub mod path;
pub mod zkgm;

pub const STYLE: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default())
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD));

#[derive(Debug, Parser)]
#[command(version, styles = STYLE, arg_required_else_help = true)]
pub struct App {
    #[command(subcommand)]
    pub cmd: Cmd,

    #[arg(
        long,
        short = 'l',
        global = true,
        default_value_t = LogFormat::default(),
        help_heading = "Global options"
    )]
    pub log_format: LogFormat,
}

#[derive(Debug, Clone, PartialEq, Default, clap::ValueEnum, derive_more::Display)]
pub enum LogFormat {
    #[default]
    #[display(fmt = "text")]
    Text,
    #[display(fmt = "json")]
    Json,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(visible_alias = "c", subcommand)]
    Codec(codec::Cmd),
    #[command(visible_aliases(["z", "ucs03"]), subcommand)]
    Zkgm(zkgm::Cmd),
    #[command(visible_alias = "d", subcommand)]
    Deployments(deployments::Cmd),
    Path(path::Cmd),
    #[command(visible_alias = "h")]
    Hex {
        /// Decode data instead of encoding it.
        #[arg(long, short = 'd')]
        decode: bool,
        /// If encoding data, don't prepend the 0x prefix to the encoded data.
        #[arg(long, short = 'p')]
        no_prefix: Option<bool>,

        /// Input to decode. If not set, stdin will be read.
        input: Option<OsString>,
    },
    #[command(visible_alias = "ph")]
    // defaults on the individual field args are arbitrary, they need to have *a* value for if --json is used (and making them optional messes with the cli by making them not required)
    PacketHash {
        #[arg(
            long,
            short = 's',
            visible_alias = "src",
            default_value_if("json", ArgPredicate::IsPresent, "1")
        )]
        source_channel_id: ChannelId,
        #[arg(
            long,
            short = 'd',
            visible_alias = "dst",
            default_value_if("json", ArgPredicate::IsPresent, "1")
        )]
        destination_channel_id: ChannelId,
        #[arg(
            long,
            short = 'D',
            default_value_if("json", ArgPredicate::IsPresent, "0x")
        )]
        data: Bytes,
        #[arg(
            long,
            short = 't',
            default_value_if("json", ArgPredicate::IsPresent, "0")
        )]
        timeout_timestamp: Timestamp,
        /// Provide the full packet as json instead of each field individually.
        #[arg(long, short = 'j', exclusive = true)]
        json: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::parse();

    match app.log_format {
        LogFormat::Text => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .init();
        }
        LogFormat::Json => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .json()
                .init();
        }
    }

    match app.cmd {
        Cmd::Codec(cmd) => cmd.run(),
        Cmd::Zkgm(cmd) => cmd.run().await,
        Cmd::Deployments(cmd) => cmd.run(),
        Cmd::Path(cmd) => cmd.run(),
        Cmd::Hex {
            decode,
            no_prefix,
            input,
        } => {
            let input = match input {
                Some(input) => input.as_bytes().to_vec(),
                None => {
                    let mut buf = vec![];
                    std::io::stdin().read_to_end(&mut buf)?;
                    OsString::from_vec(buf).as_bytes().to_vec()
                }
            };

            if decode {
                let s = String::from_utf8_lossy(&input);
                let s = s.trim();

                let bz = s
                    .strip_prefix("0x")
                    .unwrap_or(s)
                    .parse::<Bytes<HexUnprefixed>>()?;

                std::io::stdout().write_all(&bz)?;
            } else if no_prefix.unwrap_or_default() {
                println!("{}", <Bytes<HexUnprefixed>>::from(input));
            } else {
                println!("{}", <Bytes<HexPrefixed>>::from(input));
            }

            Ok(())
        }
        Cmd::PacketHash {
            source_channel_id,
            destination_channel_id,
            data,
            timeout_timestamp,
            json,
        } => {
            let hash = match json {
                Some(json) => Packet::decode_as::<Json>(json.as_bytes())?.hash(),
                None => Packet {
                    source_channel_id,
                    destination_channel_id,
                    data,
                    // deprecated
                    timeout_height: 0,
                    timeout_timestamp,
                }
                .hash(),
            };

            println!("{hash}");

            Ok(())
        }
    }
}

fn print_json<T: Serialize>(t: &T) {
    println!(
        "{}",
        serde_json::to_string(&t).expect("serialization is infallible; qed;")
    );
}
