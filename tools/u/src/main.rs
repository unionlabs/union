use std::{
    ffi::OsString,
    io::{Read, Write},
    os::unix::ffi::{OsStrExt, OsStringExt},
};

use anyhow::Result;
use clap::{
    Parser, Subcommand, ValueEnum,
    builder::styling::{AnsiColor, Effects, Styles},
};
use serde::Serialize;
use tracing_subscriber::EnvFilter;
use unionlabs::primitives::{
    Bytes,
    encoding::{HexPrefixed, HexUnprefixed},
};

pub mod arbitrum;
pub mod codec;
pub mod cometbft;
pub mod deployments;
pub mod packet;
pub mod parlia;
pub mod path;
pub mod vanity;
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
    #[command(visible_alias = "arb", subcommand)]
    Arbitrum(arbitrum::Cmd),
    #[command(subcommand)]
    Parlia(parlia::Cmd),
    #[command(visible_alias = "c", subcommand)]
    Codec(codec::Cmd),
    #[command(visible_aliases(["comet", "cmt"]), subcommand)]
    Cometbft(cometbft::Cmd),
    #[command(visible_aliases(["z", "ucs03"]), subcommand)]
    Zkgm(zkgm::Cmd),
    #[command(visible_alias = "d", subcommand)]
    Deployments(deployments::Cmd),
    Path(path::Cmd),
    #[command(subcommand)]
    Packet(packet::Cmd),
    #[command(visible_alias = "v", subcommand)]
    Vanity(vanity::Cmd),
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
    Utf8 {
        /// Use lossy decoding.
        #[arg(long, short = 'L')]
        lossy: bool,

        /// Input to decode. If not set, stdin will be read.
        input: Option<OsString>,
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
        Cmd::Arbitrum(cmd) => cmd.run().await,
        Cmd::Parlia(cmd) => cmd.run().await,
        Cmd::Codec(cmd) => cmd.run(),
        Cmd::Cometbft(cmd) => cmd.run().await,
        Cmd::Zkgm(cmd) => cmd.run().await,
        Cmd::Deployments(cmd) => cmd.run(),
        Cmd::Path(cmd) => cmd.run(),
        Cmd::Packet(cmd) => cmd.run(),
        Cmd::Vanity(cmd) => cmd.run().await,
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
        Cmd::Utf8 { lossy, input } => {
            let input = match input {
                Some(input) => input.as_bytes().to_vec(),
                None => {
                    let mut buf = vec![];
                    std::io::stdin().read_to_end(&mut buf)?;
                    OsString::from_vec(buf).as_bytes().to_vec()
                }
            };

            if lossy {
                let s = String::from_utf8_lossy(&input);
                std::io::stdout().write_all(s.as_bytes())?;
            } else {
                let s = String::from_utf8(input)?;
                std::io::stdout().write_all(s.as_bytes())?;
            }

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

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum Format {
    #[default]
    Hex,
    #[value(alias = "b64")]
    Base64,
    #[value(alias = "string")]
    Utf8,
    #[value(alias = "bytes", alias = "bz")]
    Raw,
}
