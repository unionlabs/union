use std::{
    error::Error,
    ffi::OsString,
    io::{Read, Write},
    marker::PhantomData,
    os::unix::ffi::{OsStrExt, OsStringExt},
};

use anyhow::{Result, bail};
use base64::{Engine, prelude::BASE64_STANDARD};
use clap::{Args, Subcommand, ValueEnum};
use unionlabs::encoding::{
    Bcs, Bincode, Decode, DecodeAs, Encode, EncodeAs, Encoding, EthAbi, Json, Proto, Ssz,
};

use crate::Format;

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(visible_alias = "arb")]
    Arbitrum(LightClientCodecCmdArgs),
    Base(LightClientCodecCmdArgs),
    #[command(visible_alias = "bera")]
    Berachain(LightClientCodecCmdArgs),
    Bob(LightClientCodecCmdArgs),
    Cometbls(LightClientCodecCmdArgs),
    #[command(visible_alias = "eth")]
    Ethereum(LightClientCodecCmdArgs),
    Ethermint(LightClientCodecCmdArgs),
    Movement(LightClientCodecCmdArgs),
    Parlia(LightClientCodecCmdArgs),
    #[command(visible_alias = "ics23ics23")]
    StateLensIcs23Ics23(LightClientCodecCmdArgs),
    #[command(visible_alias = "ics23mpt")]
    StateLensIcs23Mpt(LightClientCodecCmdArgs),
    #[command(visible_alias = "ics23smt")]
    StateLensIcs23Smt(LightClientCodecCmdArgs),
    #[command(visible_alias = "tm")]
    Tendermint(LightClientCodecCmdArgs),
    TrustedEvm(LightClientCodecCmdArgs),
}

#[derive(Debug, Args)]
pub struct LightClientCodecCmdArgs {
    #[arg(value_enum)]
    pub state_type: StateType,
    #[arg(value_enum)]
    pub from_encoding: AllEncodings,
    #[arg(value_enum)]
    pub to_encoding: AllEncodings,

    /// Input to decode. If not set, stdin will be read.
    pub input: Option<OsString>,

    /// How to parse the input.
    ///
    /// If <FROM_ENCODING> is json, this will default to utf8, otherwise the default is hex.
    #[arg(long, short = 'i', value_enum)]
    pub input_format: Option<Format>,

    /// How to format the output.
    ///
    /// If <TO_ENCODING> is json, this will default to utf8, otherwise the default is hex.
    #[arg(long, short = 'o', value_enum)]
    pub output_format: Option<Format>,
}

impl LightClientCodecCmdArgs {
    pub(crate) fn input_bytes(&self) -> Result<Vec<u8>> {
        let input = match &self.input {
            Some(input) => input.clone(),
            None => {
                let mut buf = vec![];
                std::io::stdin().read_to_end(&mut buf)?;
                OsString::from_vec(buf)
            }
        };

        Ok(match (self.input_format, self.from_encoding) {
            (Some(Format::Base64), _) => {
                BASE64_STANDARD.decode(input.as_os_str().as_bytes().trim_ascii())?
            }
            (Some(Format::Utf8), _) | (None, AllEncodings::Json) => {
                String::from_utf8(input.as_os_str().as_bytes().to_vec())?.into_bytes()
            }
            (Some(Format::Raw), _) => input.as_os_str().as_bytes().to_vec(),
            (Some(Format::Hex) | None, _) => {
                let data = input.as_os_str().as_bytes();
                hex::decode(data.trim_ascii().strip_prefix(b"0x").unwrap_or(data))?
            }
        })
    }

    pub(crate) fn output_bytes(&self, bz: Vec<u8>) -> Result<Vec<u8>> {
        Ok(match (self.output_format, self.to_encoding) {
            (Some(Format::Base64), _) => BASE64_STANDARD.encode(bz).into_bytes(),
            (Some(Format::Utf8), _) | (None, AllEncodings::Json) => {
                String::from_utf8(bz)?.into_bytes()
            }
            (Some(Format::Raw), _) => bz,
            (Some(Format::Hex) | None, _) => hex::encode(bz).into_bytes(),
        })
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AllEncodings {
    Json,
    Bincode,
    Proto,
    Bcs,
    #[value(name = "ethabi", alias = "eth")]
    EthAbi,
    Ssz,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum StateType {
    #[value(alias = "cl")]
    ClientState,
    #[value(alias = "co")]
    ConsensusState,
    #[value(alias = "h")]
    Header,
}

pub struct Tag<E, T>(PhantomData<fn() -> (E, T)>);

impl<E, T> Tag<E, T> {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

pub trait SupportedCodec<E: Encoding, T>: Sized {
    fn encode(&self, t: T) -> Result<Vec<u8>>;
    fn decode(&self, bz: &[u8]) -> Result<T>;
}

pub trait UnsupportedCodec<E: Encoding, T>: Sized {
    fn encode(&self, t: T) -> Result<Vec<u8>>;
    fn decode(&self, bz: &[u8]) -> Result<T>;
}

impl<E, T> SupportedCodec<E, T> for &Tag<E, T>
where
    E: Encoding,
    T: Encode<E> + Decode<E, Error: Error + 'static>,
{
    fn encode(&self, t: T) -> Result<Vec<u8>> {
        Ok(t.encode_as::<E>())
    }

    fn decode(&self, bz: &[u8]) -> Result<T> {
        Ok(T::decode_as::<E>(bz)?)
    }
}

impl<E: Encoding, T> UnsupportedCodec<E, T> for &&Tag<E, T> {
    fn encode(&self, _: T) -> Result<Vec<u8>> {
        bail!("this codec is not supported")
    }

    fn decode(&self, _: &[u8]) -> Result<T> {
        bail!("this codec is not supported")
    }
}

macro_rules! convert {
    ($path:tt, $c:expr) => {{
        let input = $c.input_bytes()?;

        let output = match $c.state_type {
            StateType::Header => {
                let t = convert!(@INNER: decode, $path::Header, &input, $c.from_encoding);
                convert!(@INNER: encode, $path::Header, t, $c.to_encoding)
            },
            StateType::ClientState => {
                let t = convert!(@INNER: decode, $path::ClientState, &input, $c.from_encoding);
                convert!(@INNER: encode, $path::ClientState, t, $c.to_encoding)
            },
            StateType::ConsensusState => {
                let t = convert!(@INNER: decode, $path::ConsensusState, &input, $c.from_encoding);
                convert!(@INNER: encode, $path::ConsensusState, t, $c.to_encoding)
            },
        };

        let output = $c.output_bytes(output)?;

        std::io::stdout().write_all(&output)?;

        Ok(())
    }};
    (@INNER: $method:ident, $path:ty, $input:expr, $encoding:expr) => {
        match $encoding {
            AllEncodings::Json => (&&<Tag<Json, $path>>::new()).$method($input)?,
            AllEncodings::Bincode => (&&<Tag<Bincode, $path>>::new()).$method($input)?,
            AllEncodings::Proto => (&&<Tag<Proto, $path>>::new()).$method($input)?,
            AllEncodings::Bcs => (&&<Tag<Bcs, $path>>::new()).$method($input)?,
            AllEncodings::EthAbi => (&&<Tag<EthAbi, $path>>::new()).$method($input)?,
            AllEncodings::Ssz => (&&<Tag<Ssz, $path>>::new()).$method($input)?,
        }
    };
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        match self {
            Cmd::Arbitrum(c) => {
                convert!(arbitrum_light_client_types, c)
            }
            Cmd::Base(c) => {
                convert!(base_light_client_types, c)
            }
            Cmd::Berachain(c) => {
                convert!(berachain_light_client_types, c)
            }
            Cmd::Bob(c) => {
                convert!(bob_light_client_types, c)
            }
            Cmd::Cometbls(c) => {
                convert!(cometbls_light_client_types, c)
            }
            Cmd::Ethereum(c) => {
                convert!(ethereum_light_client_types, c)
            }
            Cmd::Ethermint(c) => {
                mod m {
                    pub use ethermint_light_client_types::*;
                    pub enum ConsensusState {}
                    pub enum Header {}
                }
                convert!(m, c)
            }
            Cmd::Movement(c) => {
                convert!(movement_light_client_types, c)
            }
            Cmd::Parlia(c) => {
                convert!(parlia_light_client_types, c)
            }
            Cmd::StateLensIcs23Ics23(c) => {
                convert!(state_lens_ics23_ics23_light_client_types, c)
            }
            Cmd::StateLensIcs23Mpt(c) => {
                mod m {
                    pub use state_lens_ics23_mpt_light_client_types::*;
                    pub enum Header {}
                }
                convert!(m, c)
            }
            Cmd::StateLensIcs23Smt(c) => {
                mod m {
                    pub use state_lens_ics23_smt_light_client_types::*;
                    pub enum Header {}
                }
                convert!(m, c)
            }
            Cmd::Tendermint(c) => {
                convert!(tendermint_light_client_types, c)
            }
            Cmd::TrustedEvm(c) => {
                convert!(trusted_mpt_light_client_types, c)
            }
        }
    }
}
