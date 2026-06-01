use std::{ffi::OsString, os::unix::ffi::OsStrExt};

use anyhow::{Context, Result, bail};
use base64::{Engine, prelude::BASE64_STANDARD};
use clap::{Args, Subcommand};
use gno_rpc::{
    rpc_types::{Msg, Tx, TxFee, TxSignature},
    types::PublicKey,
};
use serde_json::json;
use sha2::Digest;
use unionlabs::{
    bounded::BoundedI64,
    primitives::{H256, encoding::Base64},
    prost::Message,
};

use crate::{Format, print_json};

#[derive(Debug, Args)]
pub struct Cmd {
    #[arg(global = true, short = 'r', default_value = "http://localhost:26657")]
    pub rpc_url: String,
    #[command(subcommand)]
    pub method: Method,
}

#[derive(Debug, Subcommand)]
pub enum Method {
    /// /abci_info?
    AbciInfo,
    /// /abci_query?path=_&data=_&height=_&prove=_
    AbciQuery {
        path: String,
        data: OsString,
        #[arg(long, short = 'f', default_value = "utf8")]
        data_format: Format,
        #[arg(long, short = 'H')]
        height: Option<BoundedI64<1>>,
        #[arg(long, short = 'p', default_value_t = false)]
        prove: bool,
    },
    /// /block?height=_
    Block {
        height: BoundedI64<1>,
        /// Decode the transactions contained within this block. This will return only the decoded transactions, and will discard the other response fields.
        #[arg(long, short = 'd', default_value_t = false)]
        decode_txs: bool,
    },
    /// /block_results?height=_
    BlockResults { height: BoundedI64<0> },
    /// /blockchain?minHeight=_&maxHeight=_
    Blockchain,
    /// /broadcast_evidence?evidence=_
    BroadcastTxAsync,
    /// /broadcast_tx_commit?tx=_
    BroadcastTxCommit,
    /// /broadcast_tx_sync?tx=_
    BroadcastTxSync,
    /// /commit?height=_
    Commit { height: BoundedI64<0> },
    /// /consensus_params?height=_
    ConsensusParams,
    /// /consensus_state?
    ConsensusState,
    /// /dump_consensus_state?
    DumpConsensusState,
    /// /genesis?
    Genesis,
    /// /health?
    Health,
    /// /net_info?
    NetInfo,
    /// /num_unconfirmed_txs?
    NumUnconfirmedTxs,
    /// /status?heightGte=
    Status { height_gte: Option<BoundedI64<0>> },
    /// /tx?hash=_&prove=_
    Tx {
        hash: H256<Base64>,
        /// Decode the transaction. This will return only the decoded transaction, and will discard the other response fields.
        #[arg(long, short = 'd', default_value_t = false)]
        decode_tx: bool,
    },
    /// /unconfirmed_txs?limit=_
    UnconfirmedTxs,
    /// /validators?height=
    Validators { height: BoundedI64<0> },
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let client = gno_rpc::Client::new(self.rpc_url).await?;

        match self.method {
            Method::AbciInfo => print_json(&client.abci_info().await?),
            Method::AbciQuery {
                path,
                data,
                data_format,
                height,
                prove,
            } => {
                let data = match data_format {
                    Format::Base64 => {
                        BASE64_STANDARD.decode(data.as_os_str().as_bytes().trim_ascii())?
                    }
                    Format::Utf8 => {
                        String::from_utf8(data.as_os_str().as_bytes().to_vec())?.into_bytes()
                    }
                    Format::Raw => data.as_os_str().as_bytes().to_vec(),
                    Format::Hex => {
                        let data = data.as_os_str().as_bytes();
                        hex::decode(data.trim_ascii().strip_prefix(b"0x").unwrap_or(data))?
                    }
                };

                print_json(&client.abci_query(path, data, height, prove).await?)
            }
            Method::Commit { height } => print_json(&client.commit(height).await?),
            Method::Block { height, decode_txs } => {
                let block_response = client.block(height).await?;

                if decode_txs {
                    let txs = block_response
                        .block
                        .data
                        .txs
                        .into_iter()
                        .flatten()
                        .map(|tx_bytes| {
                            proto_decode_tx(&tx_bytes).map(|tx| {
                                json!({
                                    "hash": <H256<Base64>>::from(sha2::Sha256::digest(&tx_bytes)),
                                    "tx": tx,
                                })
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    print_json(&txs);
                } else {
                    print_json(&block_response)
                }
            }
            Method::BlockResults { height } => print_json(&client.block_results(height).await?),
            Method::Status { height_gte } => print_json(&client.status(height_gte).await?),
            Method::Validators { height } => print_json(&client.validators(height).await?),
            Method::Tx { hash, decode_tx } => {
                let tx_response = client.tx(hash).await?;

                if decode_tx {
                    let tx = proto_decode_tx(&tx_response.tx)?;

                    print_json(&tx)
                } else {
                    print_json(&tx_response)
                }
            }
            _ => bail!("not yet implemented"),
        }

        Ok(())
    }
}

fn proto_decode_tx(tx: &[u8]) -> Result<Tx> {
    let tx_raw = protos::tm2::tx::Tx::decode(tx)?;
    let tx_fee = tx_raw.fee.unwrap_or_default();
    let tx = Tx {
        messages: tx_raw
            .messages
            .into_iter()
            .map(|any| Msg::try_from_any(&any))
            .collect::<Result<Vec<_>, _>>()?,
        fee: TxFee {
            gas_wanted: tx_fee.gas_wanted,
            gas_fee: tx_fee.gas_fee,
        },
        signatures: tx_raw
            .signatures
            .into_iter()
            .map(|s| {
                let any = s.pub_key.context("missing pubkey in signature")?;
                anyhow::Ok(TxSignature {
                    pub_key: pub_key_from_any(any)?,
                    signature: s.signature.into(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        memo: tx_raw.memo,
    };
    Ok(tx)
}

fn pub_key_from_any(any: protos::google::protobuf::Any) -> Result<PublicKey> {
    Ok(match &*any.type_url {
        "/tm.PubKeySecp256k1" => PublicKey::Secp256k1(
            protos::tm2::tx::PubKeySecp256k1::decode(&*any.value)?
                .key
                .into(),
        ),
        "/tm.PubKeyMultisig" => {
            let raw = protos::tm::PubKeyMultisig::decode(&*any.value)?;
            PublicKey::Multisig {
                k: raw.k,
                pub_keys: raw
                    .pub_keys
                    .into_iter()
                    .map(pub_key_from_any)
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        ty => bail!("unknown public key type: {ty}"),
    })
}
