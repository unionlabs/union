use std::{
    ffi::OsString,
    io::Read,
    os::unix::ffi::{OsStrExt, OsStringExt},
};

use anyhow::{Result, bail};
use base64::{Engine, prelude::BASE64_STANDARD};
use clap::{Args, Subcommand};
use ucs03_zkgm_packet::{Ack, Root, ZkgmPacket, root::RootShape};

use crate::Format;

#[derive(Debug, Args)]
pub struct Cmd {
    /// Input to decode. If not set, stdin will be read.
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
    /// Decode an entire packet.
    Packet {
        /// Only decode the shape of the instruction.
        #[arg(long, short = 's', global = true)]
        shape: bool,
    },
    /// Decode an instruction.
    Instruction {
        /// Only decode the shape of the instruction.
        #[arg(long, short = 's', global = true)]
        shape: bool,
    },
    /// Decode an acknowledgement.
    Ack {
        #[arg(long, short = 's', global = true)]
        /// The shape of the packet. If not provided, stdin will be read.
        shape: Option<OsString>,
    },
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

        if let (None, Type::Ack { shape: None }) = (&self.input, &self.ty) {
            bail!("only one of input and shape can be read from stdin")
        }

        let raw_bz = match self.input_format {
            Some(Format::Base64) => {
                BASE64_STANDARD.decode(input.as_os_str().as_bytes().trim_ascii())?
            }
            Some(Format::Utf8) => {
                String::from_utf8(input.as_os_str().as_bytes().to_vec())?.into_bytes()
            }
            Some(Format::Raw) => input.as_os_str().as_bytes().to_vec(),
            Some(Format::Hex) | None => {
                let data = input.as_os_str().as_bytes();
                hex::decode(data.trim_ascii().strip_prefix(b"0x").unwrap_or(data))?
            }
        };

        match self.ty {
            Type::Packet { shape } => {
                let packet = ZkgmPacket::decode(raw_bz).map_err(anyhow::Error::from_boxed)?;
                if shape {
                    println!("{}", serde_json::to_string(&packet.instruction.shape())?);
                } else {
                    println!("{}", serde_json::to_string(&packet)?);
                }
            }
            Type::Instruction { shape } => {
                let instruction = Root::decode(&raw_bz).map_err(anyhow::Error::from_boxed)?;
                if shape {
                    println!("{}", serde_json::to_string(&instruction.shape())?);
                } else {
                    println!("{}", serde_json::to_string(&instruction)?);
                }
            }
            Type::Ack { shape } => {
                let shape = match shape {
                    Some(shape) => shape.clone(),
                    None => {
                        let mut buf = vec![];
                        std::io::stdin().read_to_end(&mut buf)?;
                        OsString::from_vec(buf)
                    }
                };

                let shape = serde_json::from_slice::<RootShape>(shape.as_bytes())?;

                let ack = Ack::decode(shape, raw_bz).map_err(anyhow::Error::from_boxed)?;

                println!("{}", serde_json::to_string(&ack)?);
            }
        };

        Ok(())
    }
}
