use fastcrypto::{hash::HashFunction, traits::Signer};
use ibc_union_spec::datagram::{MsgPacketAcknowledgement, MsgPacketRecv, MsgPacketTimeout};
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};
use shared_crypto::intent::{Intent, IntentMessage};
use sui_sdk::{
    SuiClient,
    rpc_types::{SuiTransactionBlockResponse, SuiTransactionBlockResponseOptions},
    types::{
        base_types::{SequenceNumber, SuiAddress},
        crypto::{DefaultHash, SignatureScheme, SuiKeyPair, SuiSignature},
        signature::GenericSignature,
        transaction::{ProgrammableTransaction, Transaction, TransactionData},
    },
};
use tracing::{debug, info};
use voyager_sdk::rpc::{RpcError, RpcResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub original_address: SuiAddress,
    pub latest_address: SuiAddress,
    pub stores: Vec<(SuiAddress, SequenceNumber)>,
}

#[rpc(client, server)]
trait TransactionPlugin {
    #[method(name = "onRecvPacket")]
    async fn on_recv_packet(
        &self,
        pk: SuiKeyPair,
        module_info: ModuleInfo,
        fee_recipient: SuiAddress,
        data: MsgPacketRecv,
    ) -> RpcResult<ProgrammableTransaction>;

    #[method(name = "onAcknowledgePacket")]
    async fn on_acknowledge_packet(
        &self,
        pk: SuiKeyPair,
        module_info: ModuleInfo,
        fee_recipient: SuiAddress,
        data: MsgPacketAcknowledgement,
    ) -> RpcResult<ProgrammableTransaction>;

    #[method(name = "onTimeoutPacket")]
    async fn on_timeout_packet(
        &self,
        pk: SuiKeyPair,
        module_info: ModuleInfo,
        data: MsgPacketTimeout,
    ) -> RpcResult<ProgrammableTransaction>;
}

pub async fn send_transactions(
    sui_client: &SuiClient,
    pk: &SuiKeyPair,
    ptb: ProgrammableTransaction,
) -> RpcResult<SuiTransactionBlockResponse> {
    let sender = SuiAddress::from(&pk.public());
    let gas_coin = sui_client
        .coin_read_api()
        .get_coins(sender, None, None, None)
        .await
        .expect("sender is broke")
        .data
        .into_iter()
        .next()
        .expect("sender has a gas token");

    let gas_budget = 180_000_000; //TODO: change it later
    let gas_price = sui_client
        .read_api()
        .get_reference_gas_price()
        .await
        .map_err(RpcError::retryable(
            "error fetching the reference gas price",
        ))?;

    debug!(?ptb, "ptb");

    let tx_data = TransactionData::new_programmable(
        sender,
        vec![gas_coin.object_ref()],
        ptb,
        gas_budget,
        gas_price,
    );

    let intent_msg = IntentMessage::new(Intent::sui_transaction(), tx_data);
    let raw_tx = bcs::to_bytes(&intent_msg).expect("bcs should not fail");
    let mut hasher = DefaultHash::default();
    hasher.update(raw_tx.clone());
    let digest = hasher.finalize().digest;

    // use SuiKeyPair to sign the digest.
    let sui_sig = pk.sign(&digest);

    sui_sig
        .verify_secure(&intent_msg, sender, SignatureScheme::ED25519)
        .expect("sender has a valid signature");

    info!("submitting sui tx");

    let transaction_response = sui_client
        .quorum_driver_api()
        .execute_transaction_block(
            Transaction::from_generic_sig_data(
                intent_msg.value,
                vec![GenericSignature::Signature(sui_sig)],
            ),
            SuiTransactionBlockResponseOptions::default(),
            None,
        )
        .await;

    info!("{transaction_response:?}");

    transaction_response.map_err(RpcError::retryable(
        "error executing RpcError::retryablea tx",
    ))
}
