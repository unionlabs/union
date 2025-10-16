use std::{
    ffi::OsString,
    io::Read,
    os::unix::ffi::{OsStrExt, OsStringExt},
};

use anyhow::Result;
use base64::{Engine, prelude::BASE64_STANDARD};
use clap::{Args, Subcommand};
use ucs03_zkgm_packet::{Ack, Root, ZkgmPacket};

use crate::Format;

#[derive(Debug, Args)]
pub struct Cmd {
    /// Input to encode. If not set, stdin will be read.
    #[arg(long, short = 'i', global = true)]
    pub input: Option<OsString>,

    /// How to parse the input into bytes.
    #[arg(long, short = 'f', global = true)]
    pub input_format: Option<Format>,

    #[command(subcommand)]
    pub ty: Type,
}

#[derive(Debug, Subcommand)]
pub enum Type {
    /// Encode an entire packet.
    Packet,
    /// Encode an instruction.
    Instruction {
        /// Output the instruction as a partially-encoded tuple, rather than fully encoded.
        #[arg(long, short = 't', global = true)]
        tuple: bool,
    },
    /// Encode an acknowledgement.
    Ack,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let input = match &self.input {
            Some(input) => input.clone(),
            None => {
                let mut buf = vec![];
                std::io::stdin().read_to_end(&mut buf)?;
                OsString::from_vec(buf)
            }
        };

        let raw_bz = match self.input_format {
            Some(Format::Base64) => {
                BASE64_STANDARD.decode(input.as_os_str().as_bytes().trim_ascii())?
            }
            Some(Format::Utf8) | None => {
                String::from_utf8(input.as_os_str().as_bytes().to_vec())?.into_bytes()
            }
            Some(Format::Raw) => input.as_os_str().as_bytes().to_vec(),
            Some(Format::Hex) => {
                let data = input.as_os_str().as_bytes();
                hex::decode(data.trim_ascii().strip_prefix(b"0x").unwrap_or(data))?
            }
        };

        match self.ty {
            Type::Packet => {
                let packet = serde_json::from_slice::<ZkgmPacket>(&raw_bz)?;

                println!("{}", packet.encode());
            }
            Type::Instruction { tuple } => {
                if tuple {
                    let instruction = serde_json::from_slice::<Root>(&raw_bz)?;

                    println!(
                        "{}",
                        serde_json::to_string(&instruction.into_instruction())?
                    );
                } else {
                    let instruction = serde_json::from_slice::<Root>(&raw_bz)?;

                    println!("{}", instruction.encode());
                }
            }
            Type::Ack => {
                let ack = serde_json::from_slice::<Ack>(&raw_bz)?;

                println!("{}", ack.encode());
            }
        };

        Ok(())
    }
}
