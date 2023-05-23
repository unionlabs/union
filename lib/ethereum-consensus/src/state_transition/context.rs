use crate::altair;
use crate::bellatrix;
use crate::clock::{self, Clock, SystemTimeProvider};
use crate::configs::{self, Config};
use crate::phase0;
use crate::primitives::{Epoch, ExecutionAddress, Gwei, Hash32, Slot, Version, U256};
use crate::state_transition::Error;

#[derive(Debug)]
pub enum Forks {
    Phase0,
    Altair,
    Bellatrix,
    Capella,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct Context {
    // phase0 preset
    pub max_committees_per_slot: u64,
    pub target_committee_size: u64,
    pub max_validators_per_committee: usize,
    pub shuffle_round_count: u64,
    pub hysteresis_quotient: u64,
    pub hysteresis_downward_multiplier: u64,
    pub hysteresis_upward_multiplier: u64,
    pub min_deposit_amount: Gwei,
    pub max_effective_balance: Gwei,
    pub effective_balance_increment: Gwei,
    pub min_attestation_inclusion_delay: Slot,
    pub slots_per_epoch: Slot,
    pub min_seed_lookahead: Epoch,
    pub max_seed_lookahead: Epoch,
    pub min_epochs_to_inactivity_penalty: Epoch,
    pub epochs_per_eth1_voting_period: Epoch,
    pub slots_per_historical_root: Slot,
    pub epochs_per_historical_vector: Epoch,
    pub epochs_per_slashings_vector: Epoch,
    pub historical_roots_limit: usize,
    pub validator_registry_limit: usize,
    pub base_reward_factor: u64,
    pub whistleblower_reward_quotient: u64,
    pub proposer_reward_quotient: u64,
    pub inactivity_penalty_quotient: u64,
    pub min_slashing_penalty_quotient: u64,
    pub proportional_slashing_multiplier: u64,
    pub max_proposer_slashings: usize,
    pub max_attester_slashings: usize,
    pub max_attestations: usize,
    pub max_deposits: usize,
    pub max_voluntary_exits: usize,

    // altair preset
    pub inactivity_penalty_quotient_altair: u64,
    pub min_slashing_penalty_quotient_altair: u64,
    pub proportional_slashing_multiplier_altair: u64,
    pub sync_committee_size: usize,
    pub epochs_per_sync_committee_period: Epoch,
    pub min_sync_committee_participants: usize,
    pub update_timeout: usize,

    // bellatrix preset
    pub inactivity_penalty_quotient_bellatrix: u64,
    pub min_slashing_penalty_quotient_bellatrix: u64,
    pub proportional_slashing_multiplier_bellatrix: u64,
    pub max_bytes_per_transaction: usize,
    pub max_transactions_per_payload: usize,
    pub bytes_per_logs_bloom: usize,
    pub max_extra_data_bytes: usize,

    // config
    pub name: String,

    pub terminal_total_difficulty: U256,
    pub terminal_block_hash: Hash32,
    pub terminal_block_hash_activation_epoch: Epoch,

    pub min_genesis_active_validator_count: usize,
    pub min_genesis_time: u64,
    pub genesis_fork_version: Version,
    pub genesis_delay: u64,

    pub altair_fork_version: Version,
    pub altair_fork_epoch: Epoch,
    pub bellatrix_fork_version: Version,
    pub bellatrix_fork_epoch: Epoch,
    pub capella_fork_version: Version,
    pub capella_fork_epoch: Epoch,
    pub eip4844_fork_version: Version,
    pub eip4844_fork_epoch: Epoch,

    pub seconds_per_slot: u64,
    pub seconds_per_eth1_block: u64,
    pub min_validator_withdrawability_delay: Epoch,
    pub shard_committee_period: Epoch,
    pub eth1_follow_distance: u64,

    pub inactivity_score_bias: u64,
    pub inactivity_score_recovery_rate: u64,
    pub ejection_balance: Gwei,
    pub min_per_epoch_churn_limit: u64,
    pub churn_limit_quotient: u64,

    pub proposer_score_boost: u64,

    pub deposit_chain_id: usize,
    pub deposit_network_id: usize,
    pub deposit_contract_address: ExecutionAddress,
}

impl Context {
    #[cfg(feature = "serde")]
    pub fn try_from_file<P: AsRef<std::path::Path>>(config_file: P) -> Result<Self, Error> {
        let mut file = std::fs::File::open(config_file)?;
        let config: Config = serde_yaml::from_reader(&mut file)?;
        let context = match config.preset_base.as_ref() {
            "mainnet" => {
                let phase0_preset = &phase0::mainnet::PRESET;
                let altair_preset = &altair::mainnet::PRESET;
                let bellatrix_preset = &bellatrix::mainnet::PRESET;
                Self::from(phase0_preset, altair_preset, bellatrix_preset, &config)
            }
            "minimal" => {
                let phase0_preset = &phase0::minimal::PRESET;
                let altair_preset = &altair::minimal::PRESET;
                let bellatrix_preset = &bellatrix::minimal::PRESET;
                Self::from(phase0_preset, altair_preset, bellatrix_preset, &config)
            }
            other => return Err(Error::UnknownPreset(other.to_string())),
        };
        Ok(context)
    }

    pub fn from(
        phase0_preset: &phase0::Preset,
        altair_preset: &altair::Preset,
        bellatrix_preset: &bellatrix::Preset,
        config: &Config,
    ) -> Self {
        Self {
            // phase0
            max_committees_per_slot: phase0_preset.max_committees_per_slot,
            target_committee_size: phase0_preset.target_committee_size,
            max_validators_per_committee: phase0_preset.max_validators_per_committee,
            shuffle_round_count: phase0_preset.shuffle_round_count,
            hysteresis_quotient: phase0_preset.hysteresis_quotient,
            hysteresis_downward_multiplier: phase0_preset.hysteresis_downward_multiplier,
            hysteresis_upward_multiplier: phase0_preset.hysteresis_upward_multiplier,
            min_deposit_amount: phase0_preset.min_deposit_amount,
            max_effective_balance: phase0_preset.max_effective_balance,
            effective_balance_increment: phase0_preset.effective_balance_increment,
            min_attestation_inclusion_delay: phase0_preset.min_attestation_inclusion_delay,
            slots_per_epoch: phase0_preset.slots_per_epoch,
            min_seed_lookahead: phase0_preset.min_seed_lookahead,
            max_seed_lookahead: phase0_preset.max_seed_lookahead,
            min_epochs_to_inactivity_penalty: phase0_preset.min_epochs_to_inactivity_penalty,
            epochs_per_eth1_voting_period: phase0_preset.epochs_per_eth1_voting_period,
            slots_per_historical_root: phase0_preset.slots_per_historical_root,
            epochs_per_historical_vector: phase0_preset.epochs_per_historical_vector,
            epochs_per_slashings_vector: phase0_preset.epochs_per_slashings_vector,
            historical_roots_limit: phase0_preset.historical_roots_limit,
            validator_registry_limit: phase0_preset.validator_registry_limit,
            base_reward_factor: phase0_preset.base_reward_factor,
            whistleblower_reward_quotient: phase0_preset.whistleblower_reward_quotient,
            proposer_reward_quotient: phase0_preset.proposer_reward_quotient,
            inactivity_penalty_quotient: phase0_preset.inactivity_penalty_quotient,
            min_slashing_penalty_quotient: phase0_preset.min_slashing_penalty_quotient,
            proportional_slashing_multiplier: phase0_preset.proportional_slashing_multiplier,
            max_proposer_slashings: phase0_preset.max_proposer_slashings,
            max_attester_slashings: phase0_preset.max_attester_slashings,
            max_attestations: phase0_preset.max_attestations,
            max_deposits: phase0_preset.max_deposits,
            max_voluntary_exits: phase0_preset.max_voluntary_exits,
            // altair
            inactivity_penalty_quotient_altair: altair_preset.inactivity_penalty_quotient_altair,
            min_slashing_penalty_quotient_altair: altair_preset
                .min_slashing_penalty_quotient_altair,
            proportional_slashing_multiplier_altair: altair_preset
                .proportional_slashing_multiplier_altair,
            sync_committee_size: altair_preset.sync_committee_size,
            epochs_per_sync_committee_period: altair_preset.epochs_per_sync_committee_period,
            min_sync_committee_participants: altair_preset.min_sync_committee_participants,
            update_timeout: altair_preset.update_timeout,
            // bellatrix
            inactivity_penalty_quotient_bellatrix: bellatrix_preset
                .inactivity_penalty_quotient_bellatrix,
            min_slashing_penalty_quotient_bellatrix: bellatrix_preset
                .min_slashing_penalty_quotient_bellatrix,
            proportional_slashing_multiplier_bellatrix: bellatrix_preset
                .proportional_slashing_multiplier_bellatrix,
            max_bytes_per_transaction: bellatrix_preset.max_bytes_per_transaction,
            max_transactions_per_payload: bellatrix_preset.max_transactions_per_payload,
            bytes_per_logs_bloom: bellatrix_preset.bytes_per_logs_bloom,
            max_extra_data_bytes: bellatrix_preset.max_extra_data_bytes,

            // config
            name: config.name.to_string(),
            terminal_total_difficulty: config.terminal_total_difficulty.clone(),
            terminal_block_hash: config.terminal_block_hash.clone(),
            terminal_block_hash_activation_epoch: config.terminal_block_hash_activation_epoch,
            min_genesis_active_validator_count: config.min_genesis_active_validator_count,
            min_genesis_time: config.min_genesis_time,
            genesis_fork_version: config.genesis_fork_version,
            genesis_delay: config.genesis_delay,
            altair_fork_version: config.altair_fork_version,
            altair_fork_epoch: config.altair_fork_epoch,
            bellatrix_fork_version: config.bellatrix_fork_version,
            bellatrix_fork_epoch: config.bellatrix_fork_epoch,
            capella_fork_version: config.capella_fork_version,
            capella_fork_epoch: config.capella_fork_epoch,
            eip4844_fork_version: config.eip4844_fork_version,
            eip4844_fork_epoch: config.eip4844_fork_epoch,
            seconds_per_slot: config.seconds_per_slot,
            seconds_per_eth1_block: config.seconds_per_eth1_block,
            min_validator_withdrawability_delay: config.min_validator_withdrawability_delay,
            shard_committee_period: config.shard_committee_period,
            eth1_follow_distance: config.eth1_follow_distance,
            inactivity_score_bias: config.inactivity_score_bias,
            inactivity_score_recovery_rate: config.inactivity_score_recovery_rate,
            ejection_balance: config.ejection_balance,
            min_per_epoch_churn_limit: config.min_per_epoch_churn_limit,
            churn_limit_quotient: config.churn_limit_quotient,
            proposer_score_boost: config.proposer_score_boost,
            deposit_chain_id: config.deposit_chain_id,
            deposit_network_id: config.deposit_network_id,
            deposit_contract_address: config.deposit_contract_address.clone(),
        }
    }

    pub fn for_mainnet() -> Self {
        let config = &configs::mainnet::config();
        let phase0_preset = &phase0::mainnet::PRESET;
        let altair_preset = &altair::mainnet::PRESET;
        let bellatrix_preset = &bellatrix::mainnet::PRESET;
        Self::from(phase0_preset, altair_preset, bellatrix_preset, config)
    }

    pub fn for_minimal() -> Self {
        let config = &configs::minimal::config();
        let phase0_preset = &phase0::minimal::PRESET;
        let altair_preset = &altair::minimal::PRESET;
        let bellatrix_preset = &bellatrix::minimal::PRESET;
        Self::from(phase0_preset, altair_preset, bellatrix_preset, config)
    }

    pub fn for_goerli() -> Self {
        let config = &configs::goerli::config();
        let phase0_preset = &phase0::mainnet::PRESET;
        let altair_preset = &altair::mainnet::PRESET;
        let bellatrix_preset = &bellatrix::mainnet::PRESET;
        Self::from(phase0_preset, altair_preset, bellatrix_preset, config)
    }

    pub fn for_sepolia() -> Self {
        let config = &configs::sepolia::config();
        let phase0_preset = &phase0::mainnet::PRESET;
        let altair_preset = &altair::mainnet::PRESET;
        let bellatrix_preset = &bellatrix::mainnet::PRESET;
        Self::from(phase0_preset, altair_preset, bellatrix_preset, config)
    }

    pub fn fork_for(&self, slot: Slot) -> Forks {
        let epoch = slot / self.slots_per_epoch;
        if epoch >= self.capella_fork_epoch {
            Forks::Capella
        } else if epoch >= self.bellatrix_fork_epoch {
            Forks::Bellatrix
        } else if epoch >= self.altair_fork_epoch {
            Forks::Altair
        } else {
            Forks::Phase0
        }
    }

    pub fn genesis_time(&self) -> Result<u64, Error> {
        match self.name.as_ref() {
            "mainnet" => Ok(crate::clock::MAINNET_GENESIS_TIME),
            "sepolia" => Ok(crate::clock::SEPOLIA_GENESIS_TIME),
            "goerli" => Ok(crate::clock::GOERLI_GENESIS_TIME),
            name => Err(Error::UnknownGenesisTime(name.to_string())),
        }
    }

    pub fn clock(&self, genesis_time: Option<u64>) -> Clock<SystemTimeProvider> {
        match self.name.as_ref() {
            "mainnet" => clock::for_mainnet(),
            "sepolia" => clock::for_sepolia(),
            "goerli" => clock::for_goerli(),
            _ => {
                // NOTE: the default genesis time here is usually seen on testnets
                // where we have control over the genesis details
                let genesis_time =
                    genesis_time.unwrap_or(self.min_genesis_time + self.genesis_delay);
                clock::from_system_time(genesis_time, self.seconds_per_slot, self.slots_per_epoch)
            }
        }
    }
}
