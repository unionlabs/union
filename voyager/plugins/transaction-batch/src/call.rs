use enumorph::Enumorph;
use ibc_classic_spec::IbcClassic;
use ibc_union_spec::IbcUnion;
use jsonrpsee::{core::RpcResult, types::ErrorObject};
use macros::model;
use serde_json::json;
use tracing::info;
use unionlabs::ibc::core::client::height::Height;
use voyager_message::{
    call::FetchUpdateHeaders,
    primitives::{ChainId, QueryHeight},
    PluginMessage, RawClientId, VoyagerClient, VoyagerMessage, MISSING_STATE_ERROR_CODE,
};
use voyager_vm::{now, promise, Op};

use crate::{
    call,
    callback::{make_msgs, MakeBatchTransaction, MakeIbcMessagesFromUpdate, ModuleCallback},
    data::BatchableEvent,
    IbcSpecExt, Module,
};

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    MakeTransactionBatchesWithUpdateV1(MakeTransactionBatchesWithUpdate<IbcClassic>),
    MakeTransactionBatchesWithUpdateUnion(MakeTransactionBatchesWithUpdate<IbcUnion>),

    MakeMsgV1(MakeMsg<IbcClassic>),
    MakeMsgUnion(MakeMsg<IbcUnion>),
}

/// Constructs multiple batch transactions, where all of the batches are provable at the new consensus height.
#[model]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MakeTransactionBatchesWithUpdate<V: IbcSpecExt> {
    pub client_id: V::ClientId,
    pub batches: Vec<Vec<BatchableEvent<V>>>,
}

impl<V: IbcSpecExt> MakeTransactionBatchesWithUpdate<V>
where
    ModuleCall: From<MakeMsg<V>>,
    ModuleCallback: From<MakeBatchTransaction<V>> + From<MakeIbcMessagesFromUpdate<V>>,
{
    pub async fn call(
        self,
        module: &Module,
        voyager_client: &VoyagerClient,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let client_state_meta = voyager_client
            .client_state_meta::<V>(
                module.chain_id.clone(),
                QueryHeight::Latest,
                self.client_id.clone(),
            )
            .await?;

        let client_info = voyager_client
            .client_info::<V>(module.chain_id.clone(), self.client_id.clone())
            .await?;

        let latest_height = voyager_client
            .query_latest_height(client_state_meta.counterparty_chain_id.clone(), true)
            .await?;

        let target_height = self
            .batches
            .iter()
            .flatten()
            .map(|e| e.provable_height)
            .max()
            .expect("batch has at least one event; qed;");

        // at this point we assume that a valid update exists - we only ever enqueue this message behind the relevant WaitForHeight on the counterparty chain. to prevent explosions, we do a sanity check here.
        if latest_height < target_height {
            return Err(ErrorObject::owned(
                // we treat this as a missing state error, since this message assumes the state exists.
                MISSING_STATE_ERROR_CODE,
                format!(
                    "the latest height of the counterparty chain ({counterparty_chain_id}) \
                    is {latest_height} and the latest trusted height on the client tracking \
                    it ({client_id}) on this chain ({self_chain_id}) is {trusted_height}. \
                    in order to create an update for this client, we need to wait for the \
                    counterparty chain to progress to the next consensus checkpoint greater \
                    than the required target height {target_height}",
                    counterparty_chain_id = client_state_meta.counterparty_chain_id,
                    trusted_height = client_state_meta.counterparty_height,
                    client_id = self.client_id,
                    self_chain_id = module.chain_id,
                ),
                Some(json!({
                    "current_timestamp": now(),
                })),
            ));
        }

        if client_state_meta.counterparty_height >= target_height {
            info!(
                "client {client_id} has already been updated to a height \
                >= the desired target height ({} >= {target_height})",
                client_state_meta.counterparty_height,
                client_id = self.client_id,
            );

            make_msgs(
                module,
                self.client_id,
                self.batches,
                None,
                client_state_meta.clone(),
                client_state_meta.counterparty_height,
            )
        } else {
            Ok(promise(
                [call(FetchUpdateHeaders {
                    client_type: client_info.client_type,
                    counterparty_chain_id: module.chain_id.clone(),
                    chain_id: client_state_meta.counterparty_chain_id,
                    client_id: RawClientId::new(self.client_id.clone()),
                    update_from: client_state_meta.counterparty_height,
                    update_to: latest_height,
                })],
                [],
                PluginMessage::new(
                    module.plugin_name(),
                    ModuleCallback::from(MakeIbcMessagesFromUpdate::<V> {
                        client_id: self.client_id.clone(),
                        batches: self.batches,
                    }),
                ),
            ))
        }
    }
}

#[model]
pub struct MakeMsg<V: IbcSpecExt> {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId,
    /// The original event that was emitted on the origin chain.
    pub event: V::BatchableEvent,
}
