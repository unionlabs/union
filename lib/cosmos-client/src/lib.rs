#![warn(clippy::unwrap_used)]
#![allow(async_fn_in_trait)]

use std::num::NonZeroU32;

use cometbft_rpc::{
    rpc_types::{GrpcAbciQueryError, TxResponse},
    types::code::Code,
    JsonRpcError,
};
use protos::cosmos::base::abci;
use sha2::Digest;
use tracing::{debug, info, instrument};
use unionlabs::{
    bech32::Bech32,
    cosmos::{
        auth::base_account::BaseAccount,
        base::abci::gas_info::GasInfo,
        crypto::{secp256k1, AnyPubKey},
        tx::{
            auth_info::AuthInfo, mode_info::ModeInfo, sign_doc::SignDoc, signer_info::SignerInfo,
            signing::sign_info::SignMode, tx::Tx, tx_body::TxBody, tx_raw::TxRaw,
        },
    },
    encoding::{EncodeAs, Proto},
    google::protobuf::any::{Any, TryFromAnyError},
    primitives::H256,
    prost::{self, Message, Name},
    ErrorReporter,
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
    #[instrument(
        skip_all,
        fields(
            signer = %self.wallet().address(),
            memo = %memo.as_ref()
        )
    )]
    pub async fn tx<M: Message + Name, R: Message + Default + Name>(
        &self,
        msg: M,
        memo: impl AsRef<str>,
    ) -> Result<(H256, R), TxError> {
        let (tx_hash, result) = self
            .broadcast_tx_commit(
                [protos::google::protobuf::Any {
                    type_url: M::type_url(),
                    value: msg.encode_to_vec().into(),
                }],
                memo,
            )
            .await?;

        let response = <abci::v1beta1::TxMsgData as Message>::decode(
            &*result.tx_result.data.unwrap_or_default(),
        )
        .map_err(TxError::TxMsgDataDecode)?;

        if *response.msg_responses[0].type_url != R::type_url() {
            return Err(TxError::IncorrectResponseTypeUrl {
                expected: R::type_url(),
                found: response.msg_responses[0].clone().type_url,
            });
        }

        let response =
            R::decode(&*response.msg_responses[0].value).map_err(TxError::TxMsgDataDecode)?;

        Ok((tx_hash, response))
    }

    /// - simulate tx
    /// - submit tx
    /// - wait for inclusion
    /// - return (tx_hash, gas_used)
    pub async fn broadcast_tx_commit(
        &self,
        messages: impl IntoIterator<Item = protos::google::protobuf::Any> + Clone,
        memo: impl AsRef<str>,
    ) -> Result<(H256, TxResponse), BroadcastTxCommitError> {
        let account = self
            .account_info(self.wallet.address())
            .await?
            .unwrap_or_default();

        let (tx_body, mut auth_info, simulation_gas_info) =
            self.simulate_tx(messages, memo).await?;

        info!(
            gas_used = %simulation_gas_info.gas_used,
            gas_wanted = %simulation_gas_info.gas_wanted,
            "tx simulation successful"
        );

        auth_info.fee = self.gas.mk_fee(simulation_gas_info.gas_used).await;

        info!(
            fee = %auth_info.fee.amount[0].amount,
            "submitting transaction with gas"
        );

        // re-sign the new auth info with the simulated gas
        let signature = self.wallet.sign(
            &SignDoc {
                body_bytes: tx_body.clone().encode_as::<Proto>(),
                auth_info_bytes: auth_info.clone().encode_as::<Proto>(),
                chain_id: self.rpc.chain_id().to_string(),
                account_number: account.account_number,
            }
            .encode_as::<Proto>(),
        );

        let tx_raw_bytes = TxRaw {
            body_bytes: tx_body.clone().encode_as::<Proto>(),
            auth_info_bytes: auth_info.clone().encode_as::<Proto>(),
            signatures: [signature.into()].to_vec(),
        }
        .encode_as::<Proto>();

        let tx_hash: H256 = sha2::Sha256::new()
            .chain_update(&tx_raw_bytes)
            .finalize()
            .into();

        if let Ok(tx) = self.rpc.client().tx(tx_hash, false).await {
            debug!(%tx_hash, "tx already included");
            return Ok((tx_hash, tx));
        }

        let response = self.rpc.client().broadcast_tx_sync(&tx_raw_bytes).await?;

        assert_eq!(tx_hash, response.hash, "tx hash calculated incorrectly");

        info!(
            %tx_hash,
            check_tx_code = %response.code,
            check_tx_log = %response.log,
            codespace = %response.codespace,
        );

        if let Code::Err(error_code) = response.code {
            return Err(BroadcastTxCommitError::TxFailed {
                codespace: response.codespace,
                error_code,
                log: response.log,
            });
        };

        let mut target_height = self.rpc.client().block(None).await?.block.header.height;

        let mut i = 0;
        loop {
            let reached_height = 'l: loop {
                let current_height = self.rpc.client().block(None).await?.block.header.height;

                if current_height >= target_height {
                    break 'l current_height;
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            };

            let tx_inclusion = self.rpc.client().tx(tx_hash, false).await;

            match tx_inclusion {
                Ok(tx) => match tx.tx_result.code {
                    Code::Ok => break Ok((tx_hash, tx)),
                    Code::Err(error_code) => {
                        return Err(BroadcastTxCommitError::TxFailed {
                            codespace: response.codespace,
                            error_code,
                            log: response.log,
                        })
                    }
                },
                Err(source) if i > 5 => {
                    return Err(BroadcastTxCommitError::Inclusion {
                        attempts: i,
                        tx_hash,
                        error: source,
                    });
                }
                Err(err) => {
                    debug!(err = %ErrorReporter(err), "unable to retrieve tx inclusion, trying again");
                    target_height = reached_height.add(&1);
                    i += 1;
                    continue;
                }
            }
        }
    }

    pub async fn simulate_tx(
        &self,
        messages: impl IntoIterator<Item = protos::google::protobuf::Any> + Clone,
        memo: impl AsRef<str>,
    ) -> Result<(TxBody, AuthInfo, GasInfo), SimulateTxError> {
        use protos::cosmos::tx;

        let account = self
            .account_info(self.wallet.address())
            .await?
            .unwrap_or_default();

        let tx_body = TxBody {
            // TODO: Use RawAny here
            messages: messages.clone().into_iter().map(Into::into).collect(),
            memo: memo.as_ref().to_owned(),
            timeout_height: 0,
            extension_options: vec![],
            non_critical_extension_options: vec![],
            unordered: false,
            timeout_timestamp: None,
        };

        let auth_info = AuthInfo {
            signer_infos: [SignerInfo {
                public_key: Some(AnyPubKey::Secp256k1(secp256k1::PubKey {
                    key: self.wallet.public_key().into_encoding(),
                })),
                mode_info: ModeInfo::Single {
                    mode: SignMode::Direct,
                },
                sequence: account.sequence,
            }]
            .to_vec(),
            fee: self.gas.mk_fee(self.gas.max_gas().await).await,
        };

        let simulation_signature = self.wallet.sign(
            &SignDoc {
                body_bytes: tx_body.clone().encode_as::<Proto>(),
                auth_info_bytes: auth_info.clone().encode_as::<Proto>(),
                chain_id: self.rpc.chain_id().to_string(),
                account_number: account.account_number,
            }
            .encode_as::<Proto>(),
        );

        let simulate_response = self
            .rpc
            .client()
            .grpc_abci_query::<_, tx::v1beta1::SimulateResponse>(
                "/cosmos.tx.v1beta1.Service/Simulate",
                &tx::v1beta1::SimulateRequest {
                    tx_bytes: Tx {
                        body: tx_body.clone(),
                        auth_info: auth_info.clone(),
                        signatures: [simulation_signature.into()].to_vec(),
                    }
                    .encode_as::<Proto>(),
                    ..Default::default()
                },
                None,
                false,
            )
            .await?
            .into_result()?
            .ok_or(SimulateTxError::NoResponse)?;

        Ok((
            tx_body,
            auth_info,
            simulate_response.gas_info.unwrap_or_default().into(),
        ))
    }

    pub async fn account_info<T: Clone + AsRef<[u8]>>(
        &self,
        account: Bech32<T>,
    ) -> Result<Option<BaseAccount>, FetchAccountInfoError> {
        debug!(%account, "fetching account");

        let account = self
            .rpc
            .client()
            .grpc_abci_query::<_, protos::cosmos::auth::v1beta1::QueryAccountResponse>(
                "/cosmos.auth.v1beta1.Query/Account",
                &protos::cosmos::auth::v1beta1::QueryAccountRequest {
                    address: account.to_string(),
                },
                None,
                false,
            )
            .await?
            .into_result()
            .map_err(|e| match (e.error_code.get(), e.codespace.as_str()) {
                (22, "sdk") => Ok(None),
                _ => Err(FetchAccountInfoError::Query(e)),
            })
            .or_else(|e| e)?
            .map(|response| {
                response
                    .account
                    .map(<Any<BaseAccount>>::try_from)
                    .map(|res| res.map(|a| a.0))
                    .transpose()
                    .map_err(Into::into)
            })
            .transpose()
            .map(Option::flatten);

        debug!(?account, "fetched account");

        account
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BroadcastTxCommitError {
    #[error("error fetching account info")]
    FetchAccountInfo(#[from] FetchAccountInfoError),
    #[error("error simulating tx")]
    SimulateTx(#[from] SimulateTxError),
    #[error("jsonrpc error")]
    JsonRpc(#[from] JsonRpcError),
    #[error("tx failed: code={error_code}, codespace={codespace}, log={log}")]
    TxFailed {
        codespace: String,
        error_code: NonZeroU32,
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
            BroadcastTxCommitError::FetchAccountInfo(FetchAccountInfoError::JsonRpc(error))
            | BroadcastTxCommitError::SimulateTx(SimulateTxError::JsonRpc(error))
            | BroadcastTxCommitError::JsonRpc(error)
            | BroadcastTxCommitError::Inclusion { error, .. } => Some(error),
            _ => None,
        }
    }

    pub fn as_grpc_abci_query_error(&self) -> Option<&GrpcAbciQueryError> {
        match self {
            BroadcastTxCommitError::FetchAccountInfo(FetchAccountInfoError::Query(error))
            | BroadcastTxCommitError::SimulateTx(SimulateTxError::Query(error)) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SimulateTxError {
    #[error("error fetching account info")]
    FetchAccountInfo(#[from] FetchAccountInfoError),
    #[error("grpc abci query error")]
    Query(#[from] GrpcAbciQueryError),
    #[error("jsonrpc error")]
    JsonRpc(#[from] JsonRpcError),
    #[error("tx simulation returned an empty response")]
    NoResponse,
}

#[derive(Debug, thiserror::Error)]
pub enum FetchAccountInfoError {
    // #[error("account {0} not found")]
    // AccountNotFound(Bech32<Bytes>),
    #[error("grpc abci query error")]
    Query(#[from] GrpcAbciQueryError),
    #[error("error decoding account")]
    Decode(#[from] TryFromAnyError<BaseAccount>),
    #[error("jsonrpc error")]
    JsonRpc(#[from] JsonRpcError),
}

#[derive(Debug, thiserror::Error)]
pub enum TxError {
    #[error("error broadcasting transaction")]
    BroadcastTxCommit(#[from] BroadcastTxCommitError),
    #[error("incorrect type url for msg response, expected {expected} but found {found}")]
    IncorrectResponseTypeUrl { expected: String, found: String },
    #[error("unable to decode msg response")]
    ResponseDecode(#[source] prost::DecodeError),
    #[error("unable to tx response")]
    TxMsgDataDecode(#[source] prost::DecodeError),
}
