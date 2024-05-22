use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::types::NearToken;
use near_gas::NearGas;
use tokio::sync::RwLock;
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

use near_jsonrpc_client::errors::{JsonRpcError, JsonRpcServerError};
use near_jsonrpc_client::methods::health::RpcStatusError;
use near_jsonrpc_client::methods::tx::RpcTransactionError;
use near_jsonrpc_client::{methods, JsonRpcClient, MethodCallResult};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::account::{AccessKey, AccessKeyPermission};
use near_primitives::errors::InvalidTxError;
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::{
    Action, AddKeyAction, CreateAccountAction, DeleteAccountAction, DeployContractAction,
    FunctionCallAction, SignedTransaction, TransferAction,
};
use near_primitives::types::{BlockReference, Finality, Gas};
use near_primitives::views::{
    AccessKeyView, BlockView, FinalExecutionOutcomeView, QueryRequest, StatusResponse,
};

#[cfg(feature = "experimental")]
use {
    near_chain_configs::{GenesisConfig, ProtocolConfigView},
    near_jsonrpc_primitives::types::{
        changes::RpcStateChangesInBlockByTypeResponse, changes::RpcStateChangesInBlockResponse,
        receipts::ReceiptReference, transactions::TransactionInfo,
    },
    near_primitives::{
        types::MaybeBlockId,
        views::{
            validator_stake_view::ValidatorStakeView, FinalExecutionOutcomeWithReceiptView,
            ReceiptView, StateChangesRequestView,
        },
    },
};

use crate::error::{Error, ErrorKind, RpcErrorCode};
use crate::operations::TransactionStatus;
use crate::result::Result;
use crate::types::{AccountId, InMemorySigner, Nonce, PublicKey};
use crate::{Network, Worker};

pub(crate) const DEFAULT_CALL_FN_GAS: NearGas = NearGas::from_tgas(10);
pub(crate) const DEFAULT_CALL_DEPOSIT: NearToken = NearToken::from_near(0);

/// A client that wraps around [`JsonRpcClient`], and provides more capabilities such
/// as retry w/ exponential backoff and utility functions for sending transactions.
pub struct Client {
    rpc_addr: String,
    rpc_client: JsonRpcClient,
    /// AccessKey nonces to reference when sending transactions.
    pub(crate) access_key_nonces: RwLock<HashMap<(AccountId, near_crypto::PublicKey), AtomicU64>>,
}

impl Client {
    pub(crate) fn new(rpc_addr: &str, api_key: Option<String>) -> Result<Self> {
        let connector = JsonRpcClient::new_client();
        let mut rpc_client = connector.connect(rpc_addr);
        if let Some(api_key) = api_key {
            let api_key = near_jsonrpc_client::auth::ApiKey::new(api_key)
                .map_err(|e| ErrorKind::DataConversion.custom(e))?;
            rpc_client = rpc_client.header(api_key);
        }

        Ok(Self {
            rpc_client,
            rpc_addr: rpc_addr.into(),
            access_key_nonces: RwLock::new(HashMap::new()),
        })
    }

    pub(crate) async fn query_broadcast_tx(
        &self,
        method: &methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest,
    ) -> MethodCallResult<FinalExecutionOutcomeView, RpcTransactionError> {
        retry(|| async {
            let result = self.rpc_client.call(method).await;
            match &result {
                Ok(response) => {
                    // When user sets logging level to INFO we only print one-liners with submitted
                    // actions and the resulting status. If the level is DEBUG or lower, we print
                    // the entire request and response structures.
                    if tracing::level_enabled!(tracing::Level::DEBUG) {
                        tracing::debug!(
                            target: "workspaces",
                            "Calling RPC method {:?} succeeded with {:?}",
                            method,
                            response
                        );
                    } else {
                        tracing::info!(
                            target: "workspaces",
                            "Submitting transaction with actions {:?} succeeded with status {:?}",
                            method.signed_transaction.transaction.actions,
                            response.status
                        );
                    }
                }
                Err(error) => {
                    tracing::error!(
                        target: "workspaces",
                        "Calling RPC method {:?} resulted in error {:?}",
                        method,
                        error
                    );
                }
            };
            result
        })
        .await
    }

    pub(crate) async fn query_nolog<M>(&self, method: M) -> MethodCallResult<M::Response, M::Error>
    where
        M: methods::RpcMethod + Send + Sync,
    {
        retry(|| async { self.rpc_client.call(&method).await }).await
    }

    pub(crate) async fn query<M>(&self, method: M) -> MethodCallResult<M::Response, M::Error>
    where
        M: methods::RpcMethod + Debug + Send + Sync,
        M::Response: Debug + Send,
        M::Error: Debug + Send,
    {
        retry(|| async {
            let result = self.rpc_client.call(&method).await;
            tracing::debug!(
                target: "workspaces",
                "Querying RPC with {:?} resulted in {:?}",
                method,
                result
            );
            result
        })
        .await
    }

    async fn send_tx_and_retry(
        &self,
        signer: &InMemorySigner,
        receiver_id: &AccountId,
        action: Action,
    ) -> Result<FinalExecutionOutcomeView> {
        send_batch_tx_and_retry(self, signer, receiver_id, vec![action]).await
    }

    pub(crate) async fn call(
        &self,
        signer: &InMemorySigner,
        contract_id: &AccountId,
        method_name: String,
        args: Vec<u8>,
        gas: Gas,
        deposit: NearToken,
    ) -> Result<FinalExecutionOutcomeView> {
        self.send_tx_and_retry(
            signer,
            contract_id,
            FunctionCallAction {
                args,
                method_name,
                gas,
                deposit: deposit.as_yoctonear(),
            }
            .into(),
        )
        .await
    }

    pub(crate) async fn view_block(&self, block_ref: Option<BlockReference>) -> Result<BlockView> {
        let block_reference = block_ref.unwrap_or_else(|| Finality::None.into());
        let block_view = self
            .query(&methods::block::RpcBlockRequest { block_reference })
            .await
            .map_err(|e| RpcErrorCode::QueryFailure.custom(e))?;

        Ok(block_view)
    }

    pub(crate) async fn deploy(
        &self,
        signer: &InMemorySigner,
        contract_id: &AccountId,
        wasm: Vec<u8>,
    ) -> Result<FinalExecutionOutcomeView> {
        self.send_tx_and_retry(
            signer,
            contract_id,
            DeployContractAction { code: wasm }.into(),
        )
        .await
    }

    pub(crate) async fn transfer_near(
        &self,
        signer: &InMemorySigner,
        receiver_id: &AccountId,
        amount_yocto: NearToken,
    ) -> Result<FinalExecutionOutcomeView> {
        self.send_tx_and_retry(
            signer,
            receiver_id,
            TransferAction {
                deposit: amount_yocto.as_yoctonear(),
            }
            .into(),
        )
        .await
    }

    pub(crate) async fn create_account(
        &self,
        signer: &InMemorySigner,
        new_account_id: &AccountId,
        new_account_pk: PublicKey,
        amount: NearToken,
    ) -> Result<FinalExecutionOutcomeView> {
        send_batch_tx_and_retry(
            self,
            signer,
            new_account_id,
            vec![
                CreateAccountAction {}.into(),
                AddKeyAction {
                    public_key: new_account_pk.into(),
                    access_key: AccessKey {
                        nonce: 0,
                        permission: AccessKeyPermission::FullAccess,
                    },
                }
                .into(),
                TransferAction {
                    deposit: amount.as_yoctonear(),
                }
                .into(),
            ],
        )
        .await
    }

    pub(crate) async fn create_account_and_deploy(
        &self,
        signer: &InMemorySigner,
        new_account_id: &AccountId,
        new_account_pk: PublicKey,
        amount: NearToken,
        code: Vec<u8>,
    ) -> Result<FinalExecutionOutcomeView> {
        send_batch_tx_and_retry(
            self,
            signer,
            new_account_id,
            vec![
                CreateAccountAction {}.into(),
                AddKeyAction {
                    public_key: new_account_pk.into(),
                    access_key: AccessKey {
                        nonce: 0,
                        permission: AccessKeyPermission::FullAccess,
                    },
                }
                .into(),
                TransferAction {
                    deposit: amount.as_yoctonear(),
                }
                .into(),
                DeployContractAction { code }.into(),
            ],
        )
        .await
    }

    pub(crate) async fn delete_account(
        &self,
        signer: &InMemorySigner,
        account_id: &AccountId,
        beneficiary_id: &AccountId,
    ) -> Result<FinalExecutionOutcomeView> {
        let beneficiary_id = beneficiary_id.to_owned();
        self.send_tx_and_retry(
            signer,
            account_id,
            DeleteAccountAction { beneficiary_id }.into(),
        )
        .await
    }

    pub(crate) async fn status(&self) -> Result<StatusResponse, JsonRpcError<RpcStatusError>> {
        let result = self
            .rpc_client
            .call(methods::status::RpcStatusRequest)
            .await;

        tracing::debug!(
            target: "workspaces",
            "Querying RPC with RpcStatusRequest resulted in {:?}",
            result,
        );
        result
    }

    pub(crate) async fn tx_async_status(
        &self,
        sender_id: &AccountId,
        tx_hash: CryptoHash,
    ) -> Result<FinalExecutionOutcomeView, JsonRpcError<RpcTransactionError>> {
        self.query(methods::tx::RpcTransactionStatusRequest {
            transaction_info: methods::tx::TransactionInfo::TransactionId {
                sender_account_id: sender_id.clone(),
                tx_hash,
            },
        })
        .await
    }

    pub(crate) async fn wait_for_rpc(&self) -> Result<()> {
        let timeout_secs = match std::env::var("NEAR_RPC_TIMEOUT_SECS") {
            // hard fail on not being able to parse the env var, since this isn't something
            // the user should handle with the library.
            Ok(secs) => secs.parse::<usize>().map_err(|err| {
                Error::full(
                    ErrorKind::DataConversion,
                    format!("Failed to parse provided NEAR_RPC_TIMEOUT_SECS={}", secs),
                    err,
                )
            })?,
            Err(_) => 10,
        };

        let retry_strategy =
            std::iter::repeat_with(|| Duration::from_millis(500)).take(2 * timeout_secs);
        Retry::spawn(retry_strategy, || async { self.status().await })
            .await
            .map_err(|e| {
                Error::full(
                    RpcErrorCode::ConnectionFailure.into(),
                    format!(
                        "Failed to connect to RPC service {} within {} seconds",
                        self.rpc_addr, timeout_secs
                    ),
                    e,
                )
            })?;
        Ok(())
    }
}

#[cfg(feature = "experimental")]
impl Client {
    pub(crate) async fn changes_in_block(
        &self,
        block_reference: BlockReference,
    ) -> Result<RpcStateChangesInBlockByTypeResponse> {
        let resp = self
            .rpc_client
            .call(
                methods::EXPERIMENTAL_changes_in_block::RpcStateChangesInBlockRequest {
                    block_reference,
                },
            )
            .await
            .map_err(|e| RpcErrorCode::QueryFailure.custom(e))?;

        Ok(resp)
    }

    pub(crate) async fn changes(
        &self,
        block_reference: BlockReference,
        state_changes_request: StateChangesRequestView,
    ) -> Result<RpcStateChangesInBlockResponse> {
        let resp = self
            .rpc_client
            .call(
                methods::EXPERIMENTAL_changes::RpcStateChangesInBlockByTypeRequest {
                    block_reference,
                    state_changes_request,
                },
            )
            .await
            .map_err(|e| RpcErrorCode::QueryFailure.custom(e))?;
        Ok(resp)
    }

    pub(crate) async fn genesis_config(&self) -> Result<GenesisConfig> {
        let resp = self
            .rpc_client
            .call(methods::EXPERIMENTAL_genesis_config::RpcGenesisConfigRequest)
            .await
            .map_err(|e| RpcErrorCode::QueryFailure.custom(e))?;
        Ok(resp)
    }

    pub(crate) async fn protocol_config(
        &self,
        block_reference: BlockReference,
    ) -> Result<ProtocolConfigView> {
        let resp = self
            .rpc_client
            .call(
                methods::EXPERIMENTAL_protocol_config::RpcProtocolConfigRequest { block_reference },
            )
            .await
            .map_err(|e| RpcErrorCode::QueryFailure.custom(e))?;
        Ok(resp)
    }

    pub(crate) async fn receipt(&self, receipt_reference: ReceiptReference) -> Result<ReceiptView> {
        let resp = self
            .rpc_client
            .call(methods::EXPERIMENTAL_receipt::RpcReceiptRequest { receipt_reference })
            .await
            .map_err(|e| RpcErrorCode::QueryFailure.custom(e))?;
        Ok(resp)
    }

    pub(crate) async fn tx_status(
        &self,
        transaction_info: TransactionInfo,
    ) -> Result<FinalExecutionOutcomeWithReceiptView> {
        let resp = self
            .rpc_client
            .call(methods::EXPERIMENTAL_tx_status::RpcTransactionStatusRequest { transaction_info })
            .await
            .map_err(|e| RpcErrorCode::QueryFailure.custom(e))?;
        Ok(resp)
    }

    pub(crate) async fn validators_ordered(
        &self,
        block_id: MaybeBlockId,
    ) -> Result<Vec<ValidatorStakeView>> {
        let resp = self
            .rpc_client
            .call(
                methods::EXPERIMENTAL_validators_ordered::RpcValidatorsOrderedRequest { block_id },
            )
            .await
            .map_err(|e| RpcErrorCode::QueryFailure.custom(e))?;
        Ok(resp)
    }
}

pub(crate) async fn access_key(
    client: &Client,
    account_id: AccountId,
    public_key: near_crypto::PublicKey,
) -> Result<(AccessKeyView, CryptoHash)> {
    let query_resp = client
        .query(&methods::query::RpcQueryRequest {
            block_reference: Finality::None.into(),
            request: QueryRequest::ViewAccessKey {
                account_id,
                public_key,
            },
        })
        .await
        .map_err(|e| {
            Error::full(
                RpcErrorCode::QueryFailure.into(),
                "Failed to query access key",
                e,
            )
        })?;

    match query_resp.kind {
        QueryResponseKind::AccessKey(access_key) => Ok((access_key, query_resp.block_hash)),
        _ => Err(RpcErrorCode::QueryReturnedInvalidData.message("while querying access key")),
    }
}

/// Fetches the transaction nonce and block hash associated to the access key. Internally
/// caches the nonce as to not need to query for it every time, and ending up having to run
/// into contention with others.
async fn fetch_tx_nonce(
    client: &Client,
    cache_key: &(AccountId, near_crypto::PublicKey),
) -> Result<(CryptoHash, Nonce)> {
    let nonces = client.access_key_nonces.read().await;
    if let Some(nonce) = nonces.get(cache_key) {
        let nonce = nonce.fetch_add(1, Ordering::SeqCst);
        drop(nonces);

        // Fetch latest block_hash since the previous one is now invalid for new transactions:
        let block = client.view_block(Some(Finality::Final.into())).await?;
        let block_hash = block.header.hash;
        Ok((block_hash, nonce + 1))
    } else {
        drop(nonces);

        let (account_id, public_key) = cache_key;
        let (access_key, block_hash) =
            access_key(client, account_id.clone(), public_key.clone()).await?;

        // case where multiple writers end up at the same lock acquisition point and tries
        // to overwrite the cached value that a previous writer already wrote.
        let nonce = client
            .access_key_nonces
            .write()
            .await
            .entry(cache_key.clone())
            .or_insert_with(|| AtomicU64::new(access_key.nonce + 1))
            .fetch_max(access_key.nonce + 1, Ordering::SeqCst)
            .max(access_key.nonce + 1);

        Ok((block_hash, nonce))
    }
}

pub(crate) async fn retry<R, E, T, F>(task: F) -> T::Output
where
    F: FnMut() -> T + Send,
    T: core::future::Future<Output = core::result::Result<R, E>> + Send,
{
    // Exponential backoff starting w/ 5ms for maximum retry of 4 times with the following delays:
    //   5, 25, 125, 625 ms
    let retry_strategy = ExponentialBackoff::from_millis(5).map(jitter).take(4);

    Retry::spawn(retry_strategy, task).await
}

pub(crate) async fn send_tx(
    client: &Client,
    cache_key: &(AccountId, near_crypto::PublicKey),
    tx: SignedTransaction,
) -> Result<FinalExecutionOutcomeView> {
    let result = client
        .query_broadcast_tx(&methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
            signed_transaction: tx,
        })
        .await;

    // InvalidNonce, cached nonce is potentially very far behind, so invalidate it.
    if let Err(JsonRpcError::ServerError(JsonRpcServerError::HandlerError(
        RpcTransactionError::InvalidTransaction {
            context: InvalidTxError::InvalidNonce { .. },
            ..
        },
    ))) = &result
    {
        let mut nonces = client.access_key_nonces.write().await;
        nonces.remove(cache_key);
    }

    result.map_err(|e| RpcErrorCode::BroadcastTxFailure.custom(e))
}

pub(crate) async fn send_batch_tx_and_retry(
    client: &Client,
    signer: &InMemorySigner,
    receiver_id: &AccountId,
    actions: Vec<Action>,
) -> Result<FinalExecutionOutcomeView> {
    let signer = signer.inner();
    let cache_key = (signer.account_id.clone(), signer.secret_key.public_key());
    retry(|| async {
        let (block_hash, nonce) = fetch_tx_nonce(client, &cache_key).await?;
        send_tx(
            client,
            &cache_key,
            SignedTransaction::from_actions(
                nonce,
                signer.account_id.clone(),
                receiver_id.clone(),
                &signer as &dyn near_crypto::Signer,
                actions.clone(),
                block_hash,
            ),
        )
        .await
    })
    .await
}

pub(crate) async fn send_batch_tx_async_and_retry(
    worker: Worker<dyn Network>,
    signer: &InMemorySigner,
    receiver_id: &AccountId,
    actions: Vec<Action>,
) -> Result<TransactionStatus> {
    let signer = &signer.inner();
    let cache_key = (signer.account_id.clone(), signer.secret_key.public_key());
    retry(|| async {
        let (block_hash, nonce) = fetch_tx_nonce(worker.client(), &cache_key).await?;
        let hash = worker
            .client()
            .query(&methods::broadcast_tx_async::RpcBroadcastTxAsyncRequest {
                signed_transaction: SignedTransaction::from_actions(
                    nonce,
                    signer.account_id.clone(),
                    receiver_id.clone(),
                    signer as &dyn near_crypto::Signer,
                    actions.clone(),
                    block_hash,
                ),
            })
            .await
            .map_err(|e| RpcErrorCode::BroadcastTxFailure.custom(e))?;

        Ok(TransactionStatus::new(
            worker.clone(),
            signer.account_id.clone(),
            hash,
        ))
    })
    .await
}
