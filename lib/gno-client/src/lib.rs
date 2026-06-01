#![cfg_attr(not(test), warn(clippy::unwrap_used))]
#![allow(async_fn_in_trait)]

use gno_rpc::{
    JsonRpcError,
    rpc_types::{AbciAccount, BroadcastTxCommitResult, Msg, Tx, TxFee, TxSignature},
    types::PublicKey,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};
use sha2::Digest;
use tracing::{info, instrument};
use unionlabs::{
    TypeUrl,
    encoding::{Decode, Proto},
    google::protobuf::any::TryFromAnyError,
    primitives::H256,
    prost::{self, Message},
};

use crate::{gas::GasFillerT, rpc::RpcT, wallet::WalletT};

pub mod gas;
pub mod rpc;
pub mod wallet;

pub struct TxClient<W, Q, G> {
    wallet: W,
    rpc: Q,
    gas: G,
}

impl<W, Q, G> TxClient<W, Q, G> {
    pub fn new(wallet: W, rpc: Q, gas: G) -> Self {
        Self { wallet, rpc, gas }
    }

    pub fn wallet(&self) -> &W {
        &self.wallet
    }

    pub fn rpc(&self) -> &Q {
        &self.rpc
    }

    pub fn gas(&self) -> &G {
        &self.gas
    }
}

impl<W: WalletT, Q: RpcT, G: GasFillerT> TxClient<W, Q, G> {
    // #[instrument(
    //     skip_all,
    //     fields(
    //         signer = %self.wallet().address(),
    //         memo = %memo.as_ref()
    //     )
    // )]
    // #[allow(clippy::type_complexity)] // coward
    // pub async fn tx<M: Msg>(
    //     &self,
    //     msg: M,
    //     // TODO: Extract these out into an Options struct?
    //     memo: impl AsRef<str>,
    //     simulate: bool,
    // ) -> Result<(H256<HexUnprefixed>, M::Response), TxError<M::Response>> {
    //     let result = self.broadcast_tx_commit([Any(msg)], memo, simulate).await?;

    //     let mut response =
    //         <TxMsgData as Message>::decode(&*result.tx_result.data.unwrap_or_default())
    //             .map_err(TxError::TxMsgDataDecode)?;

    //     Ok((
    //         result.hash,
    //         <Any<M::Response>>::try_from(
    //             response
    //                 .msg_responses
    //                 .pop()
    //                 .map(|any| protos::google::protobuf::Any {
    //                     type_url: any.type_url,
    //                     value: any.value,
    //                 })
    //                 .expect("must contain at least one msg response"),
    //         )?
    //         .0,
    //     ))
    // }

    /// - simulate tx
    /// - submit tx
    /// - wait for inclusion
    /// - return (tx_hash, gas_used)
    #[instrument(skip_all, fields(memo = %memo.as_ref()))]
    pub async fn broadcast_tx_commit(
        &self,
        messages: impl IntoIterator<Item = Msg> + Clone,
        memo: impl AsRef<str>,
        fee: TxFee,
    ) -> Result<BroadcastTxCommitResult, BroadcastTxCommitError> {
        let account = self
            .rpc()
            .client()
            .account_info(&self.wallet.address())
            .await?
            .unwrap_or_default();

        let (tx, _) = build_tx_and_sign(
            &self.wallet,
            fee,
            self.rpc().chain_id().to_owned(),
            messages,
            memo,
            account,
        );

        let tx_bytes = proto_encode(tx);

        let response = self.rpc().client().broadcast_tx_commit(&tx_bytes).await?;

        let tx_hash: H256 = sha2::Sha256::digest(&tx_bytes).into();

        info!(
            %tx_hash,
            check_tx_log = %response.check_tx.response_base.log,
            check_tx_error = ?response.check_tx.response_base.error,
            deliver_tx_log = %response.deliver_tx.response_base.log,
            deliver_tx_error = ?response.deliver_tx.response_base.error,
            "submitted tx",
        );

        if let Some(error) = response.deliver_tx.response_base.error {
            return Err(BroadcastTxCommitError::TxFailed {
                error,
                log: response.deliver_tx.response_base.log,
            });
        };

        if let Some(error) = response.check_tx.response_base.error {
            return Err(BroadcastTxCommitError::TxFailed {
                error,
                log: response.check_tx.response_base.log,
            });
        };

        Ok(response)
    }

    // pub async fn simulate_tx(
    //     &self,
    //     messages: impl IntoIterator<Item: Into<RawAny>> + Clone,
    //     memo: impl AsRef<str>,
    // ) -> Result<(TxBody, AuthInfo, GasInfo), BroadcastTxCommitError> {
    //     use protos::cosmos::tx;

    //     let account = self
    //         .account_info(self.wallet.address())
    //         .await?
    //         .unwrap_or_default();

    //     let (tx_body, auth_info) = self.tx_info(messages, memo, &account).await;

    //     let simulation_signature = self.wallet.sign(
    //         &SignDoc {
    //             body_bytes: tx_body.clone().encode_as::<Proto>(),
    //             auth_info_bytes: auth_info.clone().encode_as::<Proto>(),
    //             chain_id: self.rpc.chain_id().to_string(),
    //             account_number: account.account_number,
    //         }
    //         .encode_as::<Proto>(),
    //     );

    //     let simulate_response = self
    //         .rpc
    //         .client()
    //         .grpc_abci_query::<_, tx::v1beta1::SimulateResponse>(
    //             "/cosmos.tx.v1beta1.Service/Simulate",
    //             &tx::v1beta1::SimulateRequest {
    //                 tx_bytes: Tx {
    //                     body: tx_body.clone(),
    //                     auth_info: auth_info.clone(),
    //                     signatures: [simulation_signature.into()].to_vec(),
    //                 }
    //                 .encode_as::<Proto>(),
    //                 ..Default::default()
    //             },
    //             None,
    //             false,
    //         )
    //         .await?
    //         .into_result()?
    //         .ok_or(BroadcastTxCommitError::NoResponse)?;

    //     Ok((
    //         tx_body,
    //         auth_info,
    //         simulate_response.gas_info.unwrap_or_default().into(),
    //     ))
    // }

    // async fn tx_info(
    //     &self,
    //     messages: impl IntoIterator<Item: Into<RawAny>> + Clone,
    //     memo: impl AsRef<str>,
    //     account: &BaseAccount,
    // ) -> (TxBody, AuthInfo) {
    //     let tx_body = TxBody {
    //         // TODO: Use RawAny here
    //         messages: messages.clone().into_iter().map(Into::into).collect(),
    //         memo: memo.as_ref().to_owned(),
    //         timeout_height: 0,
    //         extension_options: vec![],
    //         non_critical_extension_options: vec![],
    //         // unordered: false,
    //         // timeout_timestamp: None,
    //     };

    //     let auth_info = AuthInfo {
    //         signer_infos: [SignerInfo {
    //             public_key: Some(AnyPubKey::Secp256k1(Any(secp256k1::PubKey {
    //                 key: self.wallet.public_key().into_encoding(),
    //             }))),
    //             mode_info: ModeInfo::Single {
    //                 mode: SignMode::Direct,
    //             },
    //             sequence: account.sequence,
    //         }]
    //         .to_vec(),
    //         fee: self.gas.mk_fee(self.gas.max_gas().await).await,
    //     };

    //     (tx_body, auth_info)
    // }
}

fn build_tx_and_sign(
    wallet: &impl WalletT,
    fee: TxFee,
    chain_id: String,
    messages: impl IntoIterator<Item = Msg> + Clone,
    memo: impl AsRef<str>,
    account: AbciAccount,
) -> (Tx, String) {
    let msgs = messages.into_iter().collect::<Vec<_>>();

    let sign_doc = SignDoc {
        chain_id,
        account_number: account.base_account.account_number,
        sequence: account.base_account.sequence,
        fee: fee.clone(),
        msgs: msgs.clone(),
        memo: memo.as_ref().to_owned(),
    };

    let sign_payload =
        sort_json(to_value(sign_doc).expect("serialization is infallible; qed;")).to_string();

    let tx_signature = TxSignature {
        pub_key: PublicKey::Secp256k1(wallet.public_key().into()),
        signature: wallet.sign(sign_payload.as_bytes()).into(),
    };

    let tx = Tx {
        messages: msgs.clone(),
        fee: TxFee {
            gas_wanted: fee.gas_wanted,
            gas_fee: fee.gas_fee,
        },
        signatures: vec![tx_signature],
        memo: memo.as_ref().to_owned(),
    };

    (tx, sign_payload)
}

fn proto_encode(tx: Tx) -> Vec<u8> {
    protos::tm2::tx::Tx {
        messages: tx.messages.into_iter().map(Msg::into_any).collect(),
        fee: Some(protos::tm2::tx::TxFee {
            gas_wanted: tx.fee.gas_wanted,
            gas_fee: tx.fee.gas_fee,
        }),
        signatures: tx
            .signatures
            .into_iter()
            .map(|tx_signature| protos::tm2::tx::TxSignature {
                pub_key: Some(match tx_signature.pub_key {
                    PublicKey::Ed25519(_key) => todo!(),
                    PublicKey::Secp256k1(key) => protos::google::protobuf::Any {
                        type_url: "/tm.PubKeySecp256k1".to_owned(),
                        value: protos::tm2::tx::PubKeySecp256k1 { key: key.into() }.encode_to_vec(),
                    },
                    PublicKey::Multisig { .. } => todo!(),
                }),
                signature: tx_signature.signature.into(),
            })
            .collect(),
        memo: tx.memo,
    }
    .encode_to_vec()
}

#[derive(Debug, thiserror::Error)]
pub enum BroadcastTxCommitError {
    #[error("tx simulation returned an empty response")]
    NoResponse,
    #[error("jsonrpc error")]
    JsonRpc(#[from] JsonRpcError),
    #[error("error decoding json")]
    JsonDecode(#[from] serde_json::Error),
    #[error("tx failed: log={log}, error={error:?}")]
    TxFailed {
        error: gno_rpc::types::response_base::Error,
        log: String,
    },
    #[error("tx inclusion couldn't be retrieved after {attempts} attempt(s) (tx hash: {tx_hash})")]
    Inclusion {
        attempts: usize,
        tx_hash: H256,
        #[source]
        error: JsonRpcError,
    },
}

impl BroadcastTxCommitError {
    pub fn as_json_rpc_error(&self) -> Option<&JsonRpcError> {
        match self {
            BroadcastTxCommitError::JsonRpc(error)
            | BroadcastTxCommitError::Inclusion { error, .. } => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TxError<T: Decode<Proto, Error: core::error::Error> + TypeUrl> {
    #[error("error broadcasting transaction")]
    BroadcastTxCommit(#[from] BroadcastTxCommitError),
    #[error("unable to tx response")]
    TxMsgDataDecode(#[from] prost::DecodeError),
    #[error("unable to msg response")]
    MsgResponseDecode(#[from] TryFromAnyError<T>),
}

fn sort_json(mut json: Value) -> Value {
    json.sort_all_objects();
    json
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignDoc {
    pub chain_id: String,
    #[serde(with = "::serde_utils::string")]
    pub account_number: u64,
    #[serde(with = "::serde_utils::string")]
    pub sequence: u64,
    pub fee: TxFee,
    pub msgs: Vec<Msg>,
    pub memo: String,
}

#[cfg(test)]
mod tests {
    use gno_rpc::rpc_types::{BaseAccount, MemFile, MemPackage, MsgRun};
    use hex_literal::hex;

    use super::*;
    use crate::wallet::LocalSigner;

    #[test]
    fn amino_smoke_test() {
        let msg = Msg::Run(
            MsgRun {
                caller: "g14sarpj4p7l68eze5shfx4xtxr7vl92gejxfdw4".to_owned(),
                send: "".to_owned(),
                max_deposit: "".to_owned(),
                package: MemPackage {
                    name: "main".to_owned(),
                    path: "".to_owned(),
                    files: vec![MemFile {
                        name: "main.gno".to_owned(),
                        body: "package main\n\nimport core \"gno.land/r/aib/ibc/core\"\n\nfunc main() {\n\tclientID := core.CreateClient(core.MsgCreateClient{\n\t\tClientType: \"gno\",\n\t\tClientState: []byte{1, 2, 3},\n\t\tConsensusState: []byte{4, 5, 6},\n\t})\n\tprintln(clientID)\n}\n".to_owned(),
                    }],
                    r#type: None,
                    info: None,
                },
            }
        );

        let wallet = LocalSigner::new_from_private_key(
            hex!("4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77").into(),
            "g",
        );

        let account = AbciAccount {
            base_account: BaseAccount {
                address: "g14sarpj4p7l68eze5shfx4xtxr7vl92gejxfdw4".to_owned(),
                coins: "".to_owned(),
                public_key: Some(PublicKey::Secp256k1(
                    hex!("024c53721dcd3b246a74dd892ca0fb9d747bddb3e82abe3384ccd6b41b7540de4d")
                        .into(),
                )),
                account_number: 7,
                sequence: 3,
            },
            attributes: 0,
        };

        let (tx, sign_bytes) = build_tx_and_sign(
            &wallet,
            TxFee {
                gas_wanted: 5000000,
                gas_fee: "1000000ugnot".to_owned(),
            },
            "dev".to_owned(),
            [msg],
            "gno amino fixture",
            account,
        );

        let expected_sign_bytes = r#"{"account_number":"7","chain_id":"dev","fee":{"gas_fee":"1000000ugnot","gas_wanted":"5000000"},"memo":"gno amino fixture","msgs":[{"@type":"/vm.m_run","caller":"g14sarpj4p7l68eze5shfx4xtxr7vl92gejxfdw4","max_deposit":"","package":{"files":[{"body":"package main\n\nimport core \"gno.land/r/aib/ibc/core\"\n\nfunc main() {\n\tclientID := core.CreateClient(core.MsgCreateClient{\n\t\tClientType: \"gno\",\n\t\tClientState: []byte{1, 2, 3},\n\t\tConsensusState: []byte{4, 5, 6},\n\t})\n\tprintln(clientID)\n}\n","name":"main.gno"}],"name":"main","path":""},"send":""}],"sequence":"3"}"#;

        assert_eq!(sign_bytes, expected_sign_bytes);

        let tx_bytes = proto_encode(tx);

        assert_eq!(
            <H256>::new(hex!(
                "ad20a4c862fd49842a496530d76e5a2aafcf6d0819430a710a55a5e333f37a1c"
            )),
            <H256>::from(sha2::Sha256::digest(tx_bytes)),
        );
    }
}
