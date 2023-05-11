use crate::chain::Network;
use crate::cli::Opts;
use crate::db::{FileDB, DB};
use crate::{errors::Error, state::LightClientStore};
use ethereum_consensus::capella::LightClientBootstrap;
use ethereum_consensus::config::Config;
use ethereum_consensus::context::ChainContext;
use log::*;

#[derive(Debug)]
pub struct Context<
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
    const SYNC_COMMITTEE_SIZE: usize,
> {
    pub(crate) config: Config,
    pub(crate) beacon_endpoint: String,
    pub(crate) network: Network,
    db: FileDB,
}

impl<
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
        const SYNC_COMMITTEE_SIZE: usize,
    > Context<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE>
{
    pub fn build(network: Network, opts: Opts) -> Result<Self, Error> {
        let home_dir = opts.home_dir();
        if !home_dir.exists() {
            info!("directory {:?} is created", home_dir);
            std::fs::create_dir(&home_dir)?;
        }
        Ok(Self {
            config: network.config(),
            db: FileDB::open(home_dir.join("store"))?,
            beacon_endpoint: opts.beacon_endpoint,
            network: Network::from_str(&opts.network)?,
        })
    }

    pub fn beacon_endpoint(&self) -> &str {
        &self.beacon_endpoint
    }

    pub fn network(&self) -> Network {
        self.network.clone()
    }

    /// Store accessors

    pub fn get_bootstrap(
        &self,
    ) -> Result<
        LightClientBootstrap<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
        Error,
    > {
        Ok(serde_json::from_slice(&self.db.get("bootstrap")?.ok_or(
            Error::Other {
                description: "bootstrap not found".into(),
            },
        )?)?)
    }

    pub fn store_boostrap(
        &self,
        bootstrap: &LightClientBootstrap<
            SYNC_COMMITTEE_SIZE,
            BYTES_PER_LOGS_BLOOM,
            MAX_EXTRA_DATA_BYTES,
        >,
    ) -> Result<(), Error> {
        let value = serde_json::to_string_pretty(bootstrap)?;
        debug!("store_bootstrap: {}", value);
        self.db.put("bootstrap", value)?;
        Ok(())
    }

    pub fn get_light_client_state(
        &self,
    ) -> Result<
        LightClientStore<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
        Error,
    > {
        Ok(serde_json::from_slice(&self.db.get("state")?.ok_or(
            Error::Other {
                description: "light_client_state not found".into(),
            },
        )?)?)
    }

    pub fn store_light_client_state(
        &self,
        state: &LightClientStore<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    ) -> Result<(), Error> {
        let value = serde_json::to_string_pretty(state)?;
        debug!("store_light_client_state: {}", value);
        self.db.put("state", value)?;
        Ok(())
    }
}

impl<
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
        const SYNC_COMMITTEE_SIZE: usize,
    > ChainContext for Context<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE>
{
    fn genesis_time(&self) -> ethereum_consensus::types::U64 {
        todo!()
    }

    fn fork_parameters(&self) -> &ethereum_consensus::fork::ForkParameters {
        &self.config.fork_parameters
    }

    fn seconds_per_slot(&self) -> ethereum_consensus::types::U64 {
        self.config.preset.SECONDS_PER_SLOT
    }

    fn slots_per_epoch(&self) -> ethereum_consensus::beacon::Slot {
        self.config.preset.SLOTS_PER_EPOCH
    }

    fn epochs_per_sync_committee_period(&self) -> ethereum_consensus::beacon::Epoch {
        self.config.preset.EPOCHS_PER_SYNC_COMMITTEE_PERIOD
    }
}
