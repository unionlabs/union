use std::{collections::VecDeque, ops::Div};

use beacon_api::types::Spec;
use chain_utils::ethereum::ETHEREUM_REVISION_NUMBER;
use enumorph::Enumorph;
use frunk::{hlist_pat, HList};
use queue_msg::{
    aggregation::{DoCallback, SubsetOf},
    call, data, promise, queue_msg, seq, Op,
};
use tracing::debug;
use unionlabs::{
    self,
    ethereum::beacon::light_client_finality_update::UnboundedLightClientFinalityUpdate,
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::{
            header::UnboundedHeader,
            light_client_update::UnboundedLightClientUpdate,
            trusted_sync_committee::{UnboundedActiveSyncCommittee, UnboundedTrustedSyncCommittee},
        },
    },
};
use voyager_message::{
    call::Call,
    callback::Callback,
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    PluginMessage, VoyagerMessage,
};

use crate::{
    call::{
        FetchAccountUpdate, FetchBeaconGenesis, FetchLightClientUpdate, FetchLightClientUpdates,
        ModuleCall,
    },
    data::{
        AccountUpdateData, BeaconGenesis, BeaconSpec, FinalityUpdate, Header, LightClientUpdate,
        LightClientUpdates, ModuleData,
    },
};

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleCallback {
    MakeCreateUpdates(MakeCreateUpdates),
    MakeCreateUpdatesFromLightClientUpdates(MakeCreateUpdatesFromLightClientUpdates),
    CreateUpdate(CreateUpdate),
    AggregateHeaders(AggregateHeaders),
}

/// This is the entrypoint into the update construction for ethereum clients. This will requeue [`MakeCreateUpdatesFromLightClientUpdates`] with [`LightClientUpdates`] in the range `update_from..<latest finality update>`. Note that the `update_to` field is currently mostly ignored.
#[queue_msg]
pub struct MakeCreateUpdates {
    pub update_from: Height,
    pub update_to: Height,
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> for MakeCreateUpdates {
    type Params = HList![PluginMessage<FinalityUpdate>, PluginMessage<BeaconSpec>];

    fn call(
        MakeCreateUpdates {
            update_from,
            update_to,
        }: Self,
        hlist_pat![
            PluginMessage {
                plugin,
                message: FinalityUpdate { finality_update },
            },
            PluginMessage {
                plugin: _,
                message: BeaconSpec { spec },
            }
        ]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        let target_period =
            sync_committee_period(finality_update.attested_header.beacon.slot, spec.period());

        let trusted_period = sync_committee_period(update_from.revision_height, spec.period());

        assert!(
            trusted_period <= target_period,
            "trusted period {trusted_period} is behind target \
            period {target_period}, something is wrong!",
        );

        // Eth chain is more than 1 signature period ahead of us. We need to do sync committee
        // updates until we reach the `target_period - 1`.
        promise(
            [call(Call::plugin(
                plugin.clone(),
                FetchLightClientUpdates {
                    trusted_period,
                    target_period,
                },
            ))],
            [Data::plugin(plugin.clone(), BeaconSpec { spec })],
            Callback::plugin(
                plugin,
                MakeCreateUpdatesFromLightClientUpdates {
                    update_from,
                    update_to,
                    finality_update,
                },
            ),
        )
    }
}

/// The second step in the update construction process.
#[queue_msg]
pub struct MakeCreateUpdatesFromLightClientUpdates {
    // this was previously duplicated as `trusted_height`
    pub update_from: Height,
    pub update_to: Height,
    pub finality_update: UnboundedLightClientFinalityUpdate,
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for MakeCreateUpdatesFromLightClientUpdates
{
    type Params = HList![PluginMessage<LightClientUpdates>, PluginMessage<BeaconSpec>];

    fn call(
        MakeCreateUpdatesFromLightClientUpdates {
            update_from,
            update_to,
            finality_update,
        }: Self,
        hlist_pat![
            PluginMessage {
                plugin,
                message: LightClientUpdates {
                    light_client_updates,
                },
            },
            PluginMessage {
                plugin: _,
                message: BeaconSpec { spec },
            }
        ]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        let target_period = sync_committee_period(finality_update.signature_slot, spec.period());

        let trusted_period = sync_committee_period(update_from.revision_height, spec.period());

        let (updates, last_update_block_number) = light_client_updates.into_iter().fold(
            (VecDeque::new(), update_from.revision_height),
            |(mut vec, mut trusted_slot), update| {
                let old_trusted_slot = trusted_slot;

                // REVIEW: Assert that this is greater (i.e. increasing)?
                trusted_slot = update.attested_header.beacon.slot;

                vec.push_back(make_create_update(
                    plugin.clone(),
                    old_trusted_slot,
                    update,
                    true,
                    &spec,
                ));

                (vec, trusted_slot)
            },
        );

        let lc_updates = if trusted_period < target_period {
            updates
        } else {
            [].into()
        };

        let does_not_have_finality_update = last_update_block_number >= update_to.revision_height;

        debug!(last_update_block_number, update_to.revision_height);

        let finality_update_msg = if does_not_have_finality_update {
            // do nothing
            None
        } else {
            // do finality update
            Some(make_create_update(
                plugin.clone(),
                last_update_block_number,
                UnboundedLightClientUpdate {
                    attested_header: finality_update.attested_header,
                    next_sync_committee: None,
                    next_sync_committee_branch: None,
                    finalized_header: finality_update.finalized_header,
                    finality_branch: finality_update.finality_branch,
                    sync_aggregate: finality_update.sync_aggregate,
                    signature_slot: finality_update.signature_slot,
                },
                false,
                &spec,
            ))
        };

        promise(
            [seq(lc_updates.into_iter().chain(finality_update_msg))],
            [],
            Callback::plugin(plugin, AggregateHeaders {}),
        )
    }
}

fn make_create_update(
    plugin: String,
    currently_trusted_slot: u64,
    light_client_update: UnboundedLightClientUpdate,
    is_next: bool,
    spec: &Spec,
) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
    // When we fetch the update at this height, the `next_sync_committee` will
    // be the current sync committee of the period that we want to update to.
    let previous_period = u64::max(
        1,
        light_client_update.attested_header.beacon.slot / spec.period(),
    ) - 1;

    promise(
        [
            call(Call::plugin(
                &plugin,
                FetchLightClientUpdate {
                    period: previous_period,
                },
            )),
            call(Call::plugin(
                &plugin,
                FetchAccountUpdate {
                    slot: light_client_update.attested_header.beacon.slot,
                },
            )),
            call(Call::plugin(&plugin, FetchBeaconGenesis {})),
        ],
        [Data::plugin(&plugin, BeaconSpec { spec: spec.clone() })],
        Callback::plugin(
            &plugin,
            CreateUpdate {
                // chain_id,
                // counterparty_chain_id,
                currently_trusted_slot,
                light_client_update,
                is_next,
            },
        ),
    )
}

#[queue_msg]
pub struct CreateUpdate {
    // pub chain_id: String,
    // pub counterparty_chain_id: String,
    pub currently_trusted_slot: u64,
    pub light_client_update: UnboundedLightClientUpdate,
    pub is_next: bool,
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> for CreateUpdate {
    type Params = HList![
        PluginMessage<LightClientUpdate>,
        PluginMessage<AccountUpdateData>,
        PluginMessage<BeaconGenesis>,
        PluginMessage<BeaconSpec>,
    ];

    fn call(
        CreateUpdate {
            // chain_id,
            // counterparty_chain_id,
            currently_trusted_slot,
            light_client_update,
            is_next,
        }: Self,
        hlist_pat![
            PluginMessage {
                plugin,
                message: LightClientUpdate {
                    update: previous_period_light_client_update
                },
            },
            PluginMessage {
                plugin: _,
                message: AccountUpdateData {
                    slot: _,
                    update: account_update,
                },
            },
            PluginMessage {
                plugin: _,
                message: BeaconGenesis { genesis },
            },
            PluginMessage {
                plugin: _,
                message: BeaconSpec { spec },
            },
        ]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        // seq([
        // REVIEW: Why did we add this?
        // void(wait(WaitForTimestamp {
        //     chain_id: counterparty_chain_id.clone(),
        //     timestamp: (genesis.genesis_time
        //         + (light_client_update.signature_slot * spec.seconds_per_slot))
        //         .try_into()
        //         .unwrap(),
        // })),
        data(Data::plugin(
            plugin,
            Header {
                header: UnboundedHeader {
                    consensus_update: light_client_update,
                    trusted_sync_committee: UnboundedTrustedSyncCommittee {
                        trusted_height: Height {
                            revision_number: ETHEREUM_REVISION_NUMBER,
                            revision_height: currently_trusted_slot,
                        },
                        sync_committee: if is_next {
                            UnboundedActiveSyncCommittee::Next(
                                previous_period_light_client_update
                                    .next_sync_committee
                                    .unwrap(),
                            )
                        } else {
                            UnboundedActiveSyncCommittee::Current(
                                previous_period_light_client_update
                                    .next_sync_committee
                                    .unwrap(),
                            )
                        },
                    },
                    account_update,
                },
            },
        ))
        // ])
    }
}

// REVIEW: Does this function exist anywhere else?
fn sync_committee_period(height: u64, period: u64) -> u64 {
    height.div(period)
}

/// Aggregates all [`Header`] datas into an [`OrderedHeaders`] data.
#[queue_msg]
pub struct AggregateHeaders {}

impl AggregateHeaders {
    pub fn aggregate(self, data: VecDeque<Data<ModuleData>>) -> OrderedHeaders {
        let mut headers = data
            .into_iter()
            .map(PluginMessage::<Header>::try_from_super)
            .map(|d| d.expect("invalid type?").message.header)
            .collect::<Vec<_>>();

        headers.sort_by_key(|header| header.consensus_update.attested_header.beacon.slot);

        OrderedHeaders {
            headers: headers
                .into_iter()
                .map(|header| {
                    (
                        DecodedHeaderMeta {
                            height: Height {
                                revision_number: ETHEREUM_REVISION_NUMBER,
                                revision_height: header
                                    .consensus_update
                                    .attested_header
                                    .beacon
                                    .slot,
                            },
                        },
                        serde_json::to_value(header).unwrap(),
                    )
                })
                .collect(),
        }
    }
}
