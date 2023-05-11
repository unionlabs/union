use crate::{
    chain::Chain,
    context::Context,
    errors::Error,
    state::{ExecutionUpdateInfo, LightClientStore},
};
use core::time::Duration;
use ethereum_consensus::{
    beacon::{Root, Slot},
    capella::{self, LightClientUpdate},
    compute::compute_sync_committee_period_at_slot,
    context::ChainContext,
    execution::{
        BlockNumber, EXECUTION_PAYLOAD_BLOCK_NUMBER_INDEX, EXECUTION_PAYLOAD_STATE_ROOT_INDEX,
    },
    sync_protocol::SyncCommitteePeriod,
    types::{H256, U64},
};
use ethereum_light_client_verifier::{
    consensus::{CurrentNextSyncProtocolVerifier, SyncProtocolVerifier},
    context::{ConsensusVerificationContext, Fraction, LightClientContext},
    state::apply_sync_committee_update,
    updates::capella::ConsensusUpdateInfo,
};
use log::*;
type Result<T> = core::result::Result<T, Error>;

type Updates<
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
> = (
    ConsensusUpdateInfo<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    ExecutionUpdateInfo,
);

pub struct LightClient<
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
    const SYNC_COMMITTEE_SIZE: usize,
> {
    ctx: Context<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE>,
    chain: Chain,
    verifier: CurrentNextSyncProtocolVerifier<
        LightClientStore<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    >,
    genesis_time: U64,
    genesis_validators_root: Root,
    trust_level: Fraction,
}

impl<
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
        const SYNC_COMMITTEE_SIZE: usize,
    > LightClient<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE>
{
    pub fn new(
        ctx: Context<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE>,
        chain: Chain,
        genesis_time: U64,
        genesis_validators_root: Root,
        trust_level: Option<Fraction>,
    ) -> Self {
        Self {
            ctx,
            chain,
            verifier: Default::default(),
            genesis_time,
            genesis_validators_root,
            trust_level: trust_level.unwrap_or(Fraction::new(2, 3)),
        }
    }

    pub async fn init_with_bootstrap(&self, trusted_block_root: Option<H256>) -> Result<()> {
        let bootstrap = self.chain.get_bootstrap(trusted_block_root.clone()).await?;

        self.verifier
            .validate_boostrap(&bootstrap, trusted_block_root)?;

        let state = LightClientStore::from_bootstrap(
            bootstrap.clone().0,
            bootstrap.header.execution.clone(),
        );
        self.ctx.store_boostrap(&bootstrap)?;
        self.ctx.store_light_client_state(&state)?;
        Ok(())
    }

    pub async fn update_until_target(&self, target: Target, interval: Duration) -> Result<bool> {
        loop {
            println!("Trying to update the light client..");
            if let Some((slot, bn)) = self.update_sync_committee().await? {
                println!("[ + ] Updated the sync committee");
                if target <= Updated(slot, bn) {
                    break Ok(true);
                }
            } else if let Some((slot, bn)) = self.update_slot_on_current_period().await? {
                println!("[ + ] Updated slot on current period");
                if target <= Updated(slot, bn) {
                    break Ok(true);
                }
            } else if target == Target::None {
                println!("None");
                break Ok(false);
            }
            println!("Going for next phase..");
            tokio::time::sleep(interval).await;
        }
    }

    async fn update_sync_committee(&self) -> Result<Option<(Slot, U64)>> {
        println!("Update sync committee");
        let state = self.ctx.get_light_client_state()?;
        println!("[ + ] Got the light client state.");

        let period =
            compute_sync_committee_period_at_slot(&self.ctx, state.latest_finalized_header.slot);
        info!(
            "latest finalized header: period={} slot={}",
            period, state.latest_finalized_header.slot
        );

        println!("Going through the light client updates..");
        let mut updates = self
            .chain
            .rpc_client
            .get_light_client_updates(period, 2)
            .await?
            .0
            .into_iter()
            .map(|u| u.data.into());
        println!("[ + ] Got the light client updates.");

        // if next_sync_committee is known, first update is skipped
        if state.next_sync_committee.is_some() {
            updates.next();
        }

        let vctx = self.build_verification_context();
        println!("[ + ] Built the verification context.");
        let new_state = match [updates.next(), updates.next()] {
            [None, None] => return Ok(None), // do nothing here
            [Some(update), None] => {
                println!("Processing light client update with Some, None");
                self.process_light_client_update(&vctx, update, &state)
                    .await?
            }
            [Some(update_first), Some(update_second)] => {
                println!("Processing light client update with Some, Some first");
                let state = if let Some(new_state) = self
                    .process_light_client_update(&vctx, update_first, &state)
                    .await?
                {
                    new_state
                } else {
                    state
                };
                println!("Processing light client update with Some, Some second");
                self.process_light_client_update(&vctx, update_second, &state)
                    .await?
            }
            _ => unreachable!(),
        };
        println!("Checking if we have a new state..");
        if let Some(new_state) = new_state {
            println!("[ + ] Yes we have.");
            info!(
                "post finalized header: period={} slot={}",
                compute_sync_committee_period_at_slot(
                    &self.ctx,
                    new_state.latest_finalized_header.slot
                ),
                new_state.latest_finalized_header.slot
            );
            Ok(Some((
                new_state.latest_finalized_header.slot,
                new_state.latest_execution_payload_header.block_number,
            )))
        } else {
            Ok(None)
        }
    }

    async fn update_slot_on_current_period(&self) -> Result<Option<(Slot, BlockNumber)>> {
        let state = self.ctx.get_light_client_state()?;
        let store_period =
            compute_sync_committee_period_at_slot(&self.ctx, state.latest_finalized_header.slot);

        let update = self
            .chain
            .rpc_client
            .get_finality_update::<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>(
            )
            .await?
            .data;
        let finality_update_period =
            compute_sync_committee_period_at_slot(&self.ctx, update.finalized_header.beacon.slot);

        if store_period != finality_update_period
            || state.latest_finalized_header.slot >= update.finalized_header.beacon.slot
        {
            debug!("this finality update cannot apply to the store: store_period={} store_slot={} update_slot={}", store_period, state.latest_finalized_header.slot, update.finalized_header.beacon.slot);
            return Ok(None);
        }

        let vctx = self.build_verification_context();
        if let Some(new_state) = self
            .process_light_client_update(&vctx, update.into(), &state)
            .await?
        {
            info!(
                "post finalized header: period={} slot={}",
                compute_sync_committee_period_at_slot(
                    &self.ctx,
                    new_state.latest_finalized_header.slot
                ),
                new_state.latest_finalized_header.slot
            );
            Ok(Some((
                new_state.latest_finalized_header.slot,
                new_state.latest_execution_payload_header.block_number,
            )))
        } else {
            Ok(None)
        }
    }

    async fn build_updates(
        &self,
        update: LightClientUpdate<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    ) -> Result<Updates<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>> {
        if update.finalized_header.beacon == Default::default() {
            return Err(Error::FinalizedHeaderNotFound);
        }

        let execution_update = {
            let execution_payload_header = update.finalized_header.execution.clone();
            let (_, state_root_branch) = capella::gen_execution_payload_fields_proof(
                &execution_payload_header,
                &[EXECUTION_PAYLOAD_STATE_ROOT_INDEX],
            )?;
            let (_, block_number_branch) = capella::gen_execution_payload_fields_proof(
                &execution_payload_header,
                &[EXECUTION_PAYLOAD_BLOCK_NUMBER_INDEX],
            )?;
            ExecutionUpdateInfo {
                state_root: execution_payload_header.state_root,
                state_root_branch,
                block_number: execution_payload_header.block_number,
                block_number_branch,
            }
        };
        Ok((ConsensusUpdateInfo(update), execution_update))
    }

    async fn process_light_client_update(
        &self,
        vctx: &(impl ChainContext + ConsensusVerificationContext),
        update: LightClientUpdate<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
        state: &LightClientStore<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    ) -> Result<
        Option<LightClientStore<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>>,
    > {
        let updates = match self.build_updates(update).await {
            Ok(updates) => updates,
            Err(Error::FinalizedHeaderNotFound) => {
                info!("updates: finalized header not found");
                return Ok(None);
            }
            Err(e) => return Err(e.into()),
        };

        info!(
            "updates: finalized_beacon_header_slot={} execution_block_number={}",
            updates.0.finalized_header.beacon.slot, updates.1.block_number
        );

        self.verifier
            .validate_updates(vctx, state, &updates.0, &updates.1)?;
        self.verifier
            .ensure_relevant_update(vctx, state, &updates.0)?;

        let mut updated = false;
        let mut new_state = state.clone();
        if apply_sync_committee_update(&self.ctx, &mut new_state, &updates.0)? {
            updated = true;
        }
        if updates.0.finalized_header.execution.block_number
            > state.latest_execution_payload_header.block_number
        {
            new_state.latest_execution_payload_header =
                updates.0.finalized_header.execution.clone();
            updated = true;
        }

        if updated {
            self.ctx.store_light_client_state(&new_state)?;
            Ok(Some(new_state))
        } else {
            Ok(None)
        }
    }

    fn build_verification_context(&self) -> impl ChainContext + ConsensusVerificationContext {
        LightClientContext::new_with_config(
            self.ctx.config.clone(),
            self.genesis_validators_root.clone(),
            self.genesis_time,
            self.trust_level.clone(),
            || U64(1000000000000),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    None,
    Infinity,
    Slot(Slot),
    BlockNumber(U64),
}

impl Target {
    pub fn from_string<CC: ChainContext>(
        ctx: &CC,
        value: &str,
    ) -> core::result::Result<Self, anyhow::Error> {
        let value = value.trim().to_lowercase();
        if value == "none" {
            Ok(Target::None)
        } else if value == "infinity" {
            Ok(Target::Infinity)
        } else if let Some(period) = value.strip_suffix("period") {
            let period: u64 = period.parse().unwrap();
            if period == 0 {
                Ok(Target::Slot(0u64.into()))
            } else {
                Ok(Target::Slot(compute_last_slot_at_period(
                    ctx,
                    (period - 1).into(),
                )))
            }
        } else if let Some(slot) = value.strip_suffix("slot") {
            let slot: u64 = slot.parse().unwrap();
            Ok(Target::Slot(slot.into()))
        } else if let Some(bn) = value.strip_suffix("bn") {
            let bn: u64 = bn.parse().unwrap();
            Ok(Target::BlockNumber(bn.into()))
        } else {
            anyhow::bail!("unsupported format: {}", value)
        }
    }
}

struct Updated(pub Slot, pub BlockNumber);

impl PartialEq<Updated> for Target {
    fn eq(&self, other: &Updated) -> bool {
        match self {
            Target::Slot(v) => other.0.eq(v),
            Target::BlockNumber(v) => other.1.eq(v),
            Target::None => false,
            Target::Infinity => false,
        }
    }
}

impl PartialOrd<Updated> for Target {
    fn partial_cmp(&self, other: &Updated) -> Option<core::cmp::Ordering> {
        match self {
            Target::Slot(v) => v.partial_cmp(&other.0.into()),
            Target::BlockNumber(v) => v.partial_cmp(&other.1.into()),
            Target::None => Some(core::cmp::Ordering::Less),
            Target::Infinity => Some(core::cmp::Ordering::Greater),
        }
    }
}

fn compute_last_slot_at_period<CC: ChainContext>(ctx: &CC, period: SyncCommitteePeriod) -> Slot {
    (period + 1) * ctx.epochs_per_sync_committee_period() * ctx.slots_per_epoch() - 1
}
