// Package params defines important constants that are essential to Prysm services.
package params

import (
	"time"

	"github.com/ethereum/go-ethereum/common"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
)

// BeaconChainConfig contains constant configs for node to participate in beacon chain.
type BeaconChainConfig struct {
	// Constants (non-configurable)
	GenesisSlot              primitives.Slot  `yaml:"GENESIS_SLOT"`                // GenesisSlot represents the first canonical slot number of the beacon chain.
	GenesisEpoch             primitives.Epoch `yaml:"GENESIS_EPOCH"`               // GenesisEpoch represents the first canonical epoch number of the beacon chain.
	FarFutureEpoch           primitives.Epoch `yaml:"FAR_FUTURE_EPOCH"`            // FarFutureEpoch represents a epoch extremely far away in the future used as the default penalization epoch for validators.
	FarFutureSlot            primitives.Slot  `yaml:"FAR_FUTURE_SLOT"`             // FarFutureSlot represents a slot extremely far away in the future.
	BaseRewardsPerEpoch      uint64           `yaml:"BASE_REWARDS_PER_EPOCH"`      // BaseRewardsPerEpoch is used to calculate the per epoch rewards.
	DepositContractTreeDepth uint64           `yaml:"DEPOSIT_CONTRACT_TREE_DEPTH"` // DepositContractTreeDepth depth of the Merkle trie of deposits in the validator deposit contract on the PoW chain.
	JustificationBitsLength  uint64           `yaml:"JUSTIFICATION_BITS_LENGTH"`   // JustificationBitsLength defines number of epochs to track when implementing k-finality in Casper FFG.

	// Misc constants.
	PresetBase                     string `yaml:"PRESET_BASE" spec:"true"`                        // PresetBase represents the underlying spec preset this config is based on.
	ConfigName                     string `yaml:"CONFIG_NAME" spec:"true"`                        // ConfigName for allowing an easy human-readable way of knowing what chain is being used.
	TargetCommitteeSize            uint64 `yaml:"TARGET_COMMITTEE_SIZE" spec:"true"`              // TargetCommitteeSize is the number of validators in a committee when the chain is healthy.
	MaxValidatorsPerCommittee      uint64 `yaml:"MAX_VALIDATORS_PER_COMMITTEE" spec:"true"`       // MaxValidatorsPerCommittee defines the upper bound of the size of a committee.
	MaxCommitteesPerSlot           uint64 `yaml:"MAX_COMMITTEES_PER_SLOT" spec:"true"`            // MaxCommitteesPerSlot defines the max amount of committee in a single slot.
	MinPerEpochChurnLimit          uint64 `yaml:"MIN_PER_EPOCH_CHURN_LIMIT" spec:"true"`          // MinPerEpochChurnLimit is the minimum amount of churn allotted for validator rotations.
	ChurnLimitQuotient             uint64 `yaml:"CHURN_LIMIT_QUOTIENT" spec:"true"`               // ChurnLimitQuotient is used to determine the limit of how many validators can rotate per epoch.
	ShuffleRoundCount              uint64 `yaml:"SHUFFLE_ROUND_COUNT" spec:"true"`                // ShuffleRoundCount is used for retrieving the permuted index.
	MinGenesisActiveValidatorCount uint64 `yaml:"MIN_GENESIS_ACTIVE_VALIDATOR_COUNT" spec:"true"` // MinGenesisActiveValidatorCount defines how many validator deposits needed to kick off beacon chain.
	MinGenesisTime                 uint64 `yaml:"MIN_GENESIS_TIME" spec:"true"`                   // MinGenesisTime is the time that needed to pass before kicking off beacon chain.
	TargetAggregatorsPerCommittee  uint64 `yaml:"TARGET_AGGREGATORS_PER_COMMITTEE" spec:"true"`   // TargetAggregatorsPerCommittee defines the number of aggregators inside one committee.
	HysteresisQuotient             uint64 `yaml:"HYSTERESIS_QUOTIENT" spec:"true"`                // HysteresisQuotient defines the hysteresis quotient for effective balance calculations.
	HysteresisDownwardMultiplier   uint64 `yaml:"HYSTERESIS_DOWNWARD_MULTIPLIER" spec:"true"`     // HysteresisDownwardMultiplier defines the hysteresis downward multiplier for effective balance calculations.
	HysteresisUpwardMultiplier     uint64 `yaml:"HYSTERESIS_UPWARD_MULTIPLIER" spec:"true"`       // HysteresisUpwardMultiplier defines the hysteresis upward multiplier for effective balance calculations.

	// Gwei value constants.
	MinDepositAmount          uint64 `yaml:"MIN_DEPOSIT_AMOUNT" spec:"true"`          // MinDepositAmount is the minimum amount of Gwei a validator can send to the deposit contract at once (lower amounts will be reverted).
	MaxEffectiveBalance       uint64 `yaml:"MAX_EFFECTIVE_BALANCE" spec:"true"`       // MaxEffectiveBalance is the maximal amount of Gwei that is effective for staking.
	EjectionBalance           uint64 `yaml:"EJECTION_BALANCE" spec:"true"`            // EjectionBalance is the minimal GWei a validator needs to have before ejected.
	EffectiveBalanceIncrement uint64 `yaml:"EFFECTIVE_BALANCE_INCREMENT" spec:"true"` // EffectiveBalanceIncrement is used for converting the high balance into the low balance for validators.

	// Initial value constants.
	BLSWithdrawalPrefixByte         byte     `yaml:"BLS_WITHDRAWAL_PREFIX" spec:"true"`          // BLSWithdrawalPrefixByte is used for BLS withdrawal and it's the first byte.
	ETH1AddressWithdrawalPrefixByte byte     `yaml:"ETH1_ADDRESS_WITHDRAWAL_PREFIX" spec:"true"` // ETH1AddressWithdrawalPrefixByte is used for withdrawals and it's the first byte.
	ZeroHash                        [32]byte // ZeroHash is used to represent a zeroed out 32 byte array.

	// Time parameters constants.
	GenesisDelay                              uint64           `yaml:"GENESIS_DELAY" spec:"true"`                   // GenesisDelay is the minimum number of seconds to delay starting the Ethereum Beacon Chain genesis. Must be at least 1 second.
	MinAttestationInclusionDelay              primitives.Slot  `yaml:"MIN_ATTESTATION_INCLUSION_DELAY" spec:"true"` // MinAttestationInclusionDelay defines how many slots validator has to wait to include attestation for beacon block.
	SecondsPerSlot                            uint64           `yaml:"SECONDS_PER_SLOT" spec:"true"`                // SecondsPerSlot is how many seconds are in a single slot.
	SlotsPerEpoch                             primitives.Slot  `yaml:"SLOTS_PER_EPOCH" spec:"true"`                 // SlotsPerEpoch is the number of slots in an epoch.
	SqrRootSlotsPerEpoch                      primitives.Slot  // SqrRootSlotsPerEpoch is a hard coded value where we take the square root of `SlotsPerEpoch` and round down.
	MinSeedLookahead                          primitives.Epoch `yaml:"MIN_SEED_LOOKAHEAD" spec:"true"`                  // MinSeedLookahead is the duration of randao look ahead seed.
	MaxSeedLookahead                          primitives.Epoch `yaml:"MAX_SEED_LOOKAHEAD" spec:"true"`                  // MaxSeedLookahead is the duration a validator has to wait for entry and exit in epoch.
	EpochsPerEth1VotingPeriod                 primitives.Epoch `yaml:"EPOCHS_PER_ETH1_VOTING_PERIOD" spec:"true"`       // EpochsPerEth1VotingPeriod defines how often the merkle root of deposit receipts get updated in beacon node on per epoch basis.
	SlotsPerHistoricalRoot                    primitives.Slot  `yaml:"SLOTS_PER_HISTORICAL_ROOT" spec:"true"`           // SlotsPerHistoricalRoot defines how often the historical root is saved.
	MinValidatorWithdrawabilityDelay          primitives.Epoch `yaml:"MIN_VALIDATOR_WITHDRAWABILITY_DELAY" spec:"true"` // MinValidatorWithdrawabilityDelay is the shortest amount of time a validator has to wait to withdraw.
	ShardCommitteePeriod                      primitives.Epoch `yaml:"SHARD_COMMITTEE_PERIOD" spec:"true"`              // ShardCommitteePeriod is the minimum amount of epochs a validator must participate before exiting.
	MinEpochsToInactivityPenalty              primitives.Epoch `yaml:"MIN_EPOCHS_TO_INACTIVITY_PENALTY" spec:"true"`    // MinEpochsToInactivityPenalty defines the minimum amount of epochs since finality to begin penalizing inactivity.
	Eth1FollowDistance                        uint64           `yaml:"ETH1_FOLLOW_DISTANCE" spec:"true"`                // Eth1FollowDistance is the number of eth1.0 blocks to wait before considering a new deposit for voting. This only applies after the chain as been started.
	DeprecatedSafeSlotsToUpdateJustified      primitives.Slot  `yaml:"SAFE_SLOTS_TO_UPDATE_JUSTIFIED" spec:"true"`      // DeprecateSafeSlotsToUpdateJustified is the minimal slots needed to update justified check point.
	DeprecatedSafeSlotsToImportOptimistically primitives.Slot  `yaml:"SAFE_SLOTS_TO_IMPORT_OPTIMISTICALLY" spec:"true"` // SafeSlotsToImportOptimistically is the minimal number of slots to wait before importing optimistically a pre-merge block
	SecondsPerETH1Block                       uint64           `yaml:"SECONDS_PER_ETH1_BLOCK" spec:"true"`              // SecondsPerETH1Block is the approximate time for a single eth1 block to be produced.

	// Fork choice algorithm constants.
	ProposerScoreBoost              uint64           `yaml:"PROPOSER_SCORE_BOOST" spec:"true"`                // ProposerScoreBoost defines a value that is a % of the committee weight for fork-choice boosting.
	ReorgWeightThreshold            uint64           `yaml:"REORG_WEIGHT_THRESHOLD" spec:"true"`              // ReorgWeightThreshold defines a value that is a % of the committee weight to consider a block weak and subject to being orphaned.
	ReorgParentWeightThreshold      uint64           `yaml:"REORG_PARENT_WEIGHT_THRESHOLD" spec:"true"`       // ReorgParentWeightThreshold defines a value that is a % of the committee weight to consider a parent block strong and subject its child to being orphaned.
	ReorgMaxEpochsSinceFinalization primitives.Epoch `yaml:"REORG_MAX_EPOCHS_SINCE_FINALIZATION" spec:"true"` // This defines a limit to consider safe to orphan a block if the network is finalizing
	IntervalsPerSlot                uint64           `yaml:"INTERVALS_PER_SLOT" spec:"true"`                  // IntervalsPerSlot defines the number of fork choice intervals in a slot defined in the fork choice spec.

	// Ethereum PoW parameters.
	DepositChainID         uint64 `yaml:"DEPOSIT_CHAIN_ID" spec:"true"`         // DepositChainID of the eth1 network. This used for replay protection.
	DepositNetworkID       uint64 `yaml:"DEPOSIT_NETWORK_ID" spec:"true"`       // DepositNetworkID of the eth1 network. This used for replay protection.
	DepositContractAddress string `yaml:"DEPOSIT_CONTRACT_ADDRESS" spec:"true"` // DepositContractAddress is the address of the deposit contract.

	// Validator parameters.
	RandomSubnetsPerValidator         uint64 `yaml:"RANDOM_SUBNETS_PER_VALIDATOR" spec:"true"`          // RandomSubnetsPerValidator specifies the amount of subnets a validator has to be subscribed to at one time.
	EpochsPerRandomSubnetSubscription uint64 `yaml:"EPOCHS_PER_RANDOM_SUBNET_SUBSCRIPTION" spec:"true"` // EpochsPerRandomSubnetSubscription specifies the minimum duration a validator is connected to their subnet.

	// State list lengths
	EpochsPerHistoricalVector primitives.Epoch `yaml:"EPOCHS_PER_HISTORICAL_VECTOR" spec:"true"` // EpochsPerHistoricalVector defines max length in epoch to store old historical stats in beacon state.
	EpochsPerSlashingsVector  primitives.Epoch `yaml:"EPOCHS_PER_SLASHINGS_VECTOR" spec:"true"`  // EpochsPerSlashingsVector defines max length in epoch to store old stats to recompute slashing witness.
	HistoricalRootsLimit      uint64           `yaml:"HISTORICAL_ROOTS_LIMIT" spec:"true"`       // HistoricalRootsLimit defines max historical roots that can be saved in state before roll over.
	ValidatorRegistryLimit    uint64           `yaml:"VALIDATOR_REGISTRY_LIMIT" spec:"true"`     // ValidatorRegistryLimit defines the upper bound of validators can participate in eth2.

	// Reward and penalty quotients constants.
	BaseRewardFactor               uint64 `yaml:"BASE_REWARD_FACTOR" spec:"true"`               // BaseRewardFactor is used to calculate validator per-slot interest rate.
	WhistleBlowerRewardQuotient    uint64 `yaml:"WHISTLEBLOWER_REWARD_QUOTIENT" spec:"true"`    // WhistleBlowerRewardQuotient is used to calculate whistle blower reward.
	ProposerRewardQuotient         uint64 `yaml:"PROPOSER_REWARD_QUOTIENT" spec:"true"`         // ProposerRewardQuotient is used to calculate the reward for proposers.
	InactivityPenaltyQuotient      uint64 `yaml:"INACTIVITY_PENALTY_QUOTIENT" spec:"true"`      // InactivityPenaltyQuotient is used to calculate the penalty for a validator that is offline.
	MinSlashingPenaltyQuotient     uint64 `yaml:"MIN_SLASHING_PENALTY_QUOTIENT" spec:"true"`    // MinSlashingPenaltyQuotient is used to calculate the minimum penalty to prevent DoS attacks.
	ProportionalSlashingMultiplier uint64 `yaml:"PROPORTIONAL_SLASHING_MULTIPLIER" spec:"true"` // ProportionalSlashingMultiplier is used as a multiplier on slashed penalties.

	// Max operations per block constants.
	MaxProposerSlashings             uint64 `yaml:"MAX_PROPOSER_SLASHINGS" spec:"true"`               // MaxProposerSlashings defines the maximum number of slashings of proposers possible in a block.
	MaxAttesterSlashings             uint64 `yaml:"MAX_ATTESTER_SLASHINGS" spec:"true"`               // MaxAttesterSlashings defines the maximum number of casper FFG slashings possible in a block.
	MaxAttestations                  uint64 `yaml:"MAX_ATTESTATIONS" spec:"true"`                     // MaxAttestations defines the maximum allowed attestations in a beacon block.
	MaxDeposits                      uint64 `yaml:"MAX_DEPOSITS" spec:"true"`                         // MaxDeposits defines the maximum number of validator deposits in a block.
	MaxVoluntaryExits                uint64 `yaml:"MAX_VOLUNTARY_EXITS" spec:"true"`                  // MaxVoluntaryExits defines the maximum number of validator exits in a block.
	MaxWithdrawalsPerPayload         uint64 `yaml:"MAX_WITHDRAWALS_PER_PAYLOAD" spec:"true"`          // MaxWithdrawalsPerPayload defines the maximum number of withdrawals in a block.
	MaxBlsToExecutionChanges         uint64 `yaml:"MAX_BLS_TO_EXECUTION_CHANGES" spec:"true"`         // MaxBlsToExecutionChanges defines the maximum number of BLS-to-execution-change objects in a block.
	MaxValidatorsPerWithdrawalsSweep uint64 `yaml:"MAX_VALIDATORS_PER_WITHDRAWALS_SWEEP" spec:"true"` //MaxValidatorsPerWithdrawalsSweep bounds the size of the sweep searching for withdrawals per slot.

	// BLS domain values.
	DomainBeaconProposer              [4]byte `yaml:"DOMAIN_BEACON_PROPOSER" spec:"true"`                // DomainBeaconProposer defines the BLS signature domain for beacon proposal verification.
	DomainRandao                      [4]byte `yaml:"DOMAIN_RANDAO" spec:"true"`                         // DomainRandao defines the BLS signature domain for randao verification.
	DomainBeaconAttester              [4]byte `yaml:"DOMAIN_BEACON_ATTESTER" spec:"true"`                // DomainBeaconAttester defines the BLS signature domain for attestation verification.
	DomainDeposit                     [4]byte `yaml:"DOMAIN_DEPOSIT" spec:"true"`                        // DomainDeposit defines the BLS signature domain for deposit verification.
	DomainVoluntaryExit               [4]byte `yaml:"DOMAIN_VOLUNTARY_EXIT" spec:"true"`                 // DomainVoluntaryExit defines the BLS signature domain for exit verification.
	DomainSelectionProof              [4]byte `yaml:"DOMAIN_SELECTION_PROOF" spec:"true"`                // DomainSelectionProof defines the BLS signature domain for selection proof.
	DomainAggregateAndProof           [4]byte `yaml:"DOMAIN_AGGREGATE_AND_PROOF" spec:"true"`            // DomainAggregateAndProof defines the BLS signature domain for aggregate and proof.
	DomainSyncCommittee               [4]byte `yaml:"DOMAIN_SYNC_COMMITTEE" spec:"true"`                 // DomainVoluntaryExit defines the BLS signature domain for sync committee.
	DomainSyncCommitteeSelectionProof [4]byte `yaml:"DOMAIN_SYNC_COMMITTEE_SELECTION_PROOF" spec:"true"` // DomainSelectionProof defines the BLS signature domain for sync committee selection proof.
	DomainContributionAndProof        [4]byte `yaml:"DOMAIN_CONTRIBUTION_AND_PROOF" spec:"true"`         // DomainAggregateAndProof defines the BLS signature domain for contribution and proof.
	DomainApplicationMask             [4]byte `yaml:"DOMAIN_APPLICATION_MASK" spec:"true"`               // DomainApplicationMask defines the BLS signature domain for application mask.
	DomainApplicationBuilder          [4]byte `yaml:"DOMAIN_APPLICATION_BUILDER" spec:"true"`            // DomainApplicationBuilder defines the BLS signature domain for application builder.
	DomainBLSToExecutionChange        [4]byte `yaml:"DOMAIN_BLS_TO_EXECUTION_CHANGE" spec:"true"`        // DomainBLSToExecutionChange defines the BLS signature domain to change withdrawal addresses to ETH1 prefix

	// Prysm constants.
	GweiPerEth                     uint64          // GweiPerEth is the amount of gwei corresponding to 1 eth.
	BLSSecretKeyLength             int             // BLSSecretKeyLength defines the expected length of BLS secret keys in bytes.
	BLSPubkeyLength                int             // BLSPubkeyLength defines the expected length of BLS public keys in bytes.
	DefaultBufferSize              int             // DefaultBufferSize for channels across the Prysm repository.
	ValidatorPrivkeyFileName       string          // ValidatorPrivKeyFileName specifies the string name of a validator private key file.
	WithdrawalPrivkeyFileName      string          // WithdrawalPrivKeyFileName specifies the string name of a withdrawal private key file.
	RPCSyncCheck                   time.Duration   // Number of seconds to query the sync service, to find out if the node is synced or not.
	EmptySignature                 [96]byte        // EmptySignature is used to represent a zeroed out BLS Signature.
	DefaultPageSize                int             // DefaultPageSize defines the default page size for RPC server request.
	MaxPeersToSync                 int             // MaxPeersToSync describes the limit for number of peers in round robin sync.
	SlotsPerArchivedPoint          primitives.Slot // SlotsPerArchivedPoint defines the number of slots per one archived point.
	GenesisCountdownInterval       time.Duration   // How often to log the countdown until the genesis time is reached.
	BeaconStateFieldCount          int             // BeaconStateFieldCount defines how many fields are in the Phase0 beacon state.
	BeaconStateAltairFieldCount    int             // BeaconStateAltairFieldCount defines how many fields are in the beacon state post upgrade to Altair.
	BeaconStateBellatrixFieldCount int             // BeaconStateBellatrixFieldCount defines how many fields are in beacon state post upgrade to Bellatrix.
	BeaconStateCapellaFieldCount   int             // BeaconStateCapellaFieldCount defines how many fields are in beacon state post upgrade to Capella.

	// Slasher constants.
	WeakSubjectivityPeriod    primitives.Epoch // WeakSubjectivityPeriod defines the time period expressed in number of epochs were proof of stake network should validate block headers and attestations for slashable events.
	PruneSlasherStoragePeriod primitives.Epoch // PruneSlasherStoragePeriod defines the time period expressed in number of epochs were proof of stake network should prune attestation and block header store.

	// Slashing protection constants.
	SlashingProtectionPruningEpochs primitives.Epoch // SlashingProtectionPruningEpochs defines a period after which all prior epochs are pruned in the validator database.

	// Fork-related values.
	GenesisForkVersion   []byte           `yaml:"GENESIS_FORK_VERSION" spec:"true"`   // GenesisForkVersion is used to track fork version between state transitions.
	AltairForkVersion    []byte           `yaml:"ALTAIR_FORK_VERSION" spec:"true"`    // AltairForkVersion is used to represent the fork version for altair.
	AltairForkEpoch      primitives.Epoch `yaml:"ALTAIR_FORK_EPOCH" spec:"true"`      // AltairForkEpoch is used to represent the assigned fork epoch for altair.
	BellatrixForkVersion []byte           `yaml:"BELLATRIX_FORK_VERSION" spec:"true"` // BellatrixForkVersion is used to represent the fork version for bellatrix.
	BellatrixForkEpoch   primitives.Epoch `yaml:"BELLATRIX_FORK_EPOCH" spec:"true"`   // BellatrixForkEpoch is used to represent the assigned fork epoch for bellatrix.
	CapellaForkVersion   []byte           `yaml:"CAPELLA_FORK_VERSION" spec:"true"`   // CapellaForkVersion is used to represent the fork version for capella.
	CapellaForkEpoch     primitives.Epoch `yaml:"CAPELLA_FORK_EPOCH" spec:"true"`     // CapellaForkEpoch is used to represent the assigned fork epoch for capella.

	ForkVersionSchedule map[[fieldparams.VersionLength]byte]primitives.Epoch // Schedule of fork epochs by version.
	ForkVersionNames    map[[fieldparams.VersionLength]byte]string           // Human-readable names of fork versions.

	// Weak subjectivity values.
	SafetyDecay uint64 // SafetyDecay is defined as the loss in the 1/3 consensus safety margin of the casper FFG mechanism.

	// New values introduced in Altair hard fork 1.
	// Participation flag indices.
	TimelySourceFlagIndex uint8 `yaml:"TIMELY_SOURCE_FLAG_INDEX" spec:"true"` // TimelySourceFlagIndex is the source flag position of the participation bits.
	TimelyTargetFlagIndex uint8 `yaml:"TIMELY_TARGET_FLAG_INDEX" spec:"true"` // TimelyTargetFlagIndex is the target flag position of the participation bits.
	TimelyHeadFlagIndex   uint8 `yaml:"TIMELY_HEAD_FLAG_INDEX" spec:"true"`   // TimelyHeadFlagIndex is the head flag position of the participation bits.

	// Incentivization weights.
	TimelySourceWeight uint64 `yaml:"TIMELY_SOURCE_WEIGHT" spec:"true"` // TimelySourceWeight is the factor of how much source rewards receives.
	TimelyTargetWeight uint64 `yaml:"TIMELY_TARGET_WEIGHT" spec:"true"` // TimelyTargetWeight is the factor of how much target rewards receives.
	TimelyHeadWeight   uint64 `yaml:"TIMELY_HEAD_WEIGHT" spec:"true"`   // TimelyHeadWeight is the factor of how much head rewards receives.
	SyncRewardWeight   uint64 `yaml:"SYNC_REWARD_WEIGHT" spec:"true"`   // SyncRewardWeight is the factor of how much sync committee rewards receives.
	WeightDenominator  uint64 `yaml:"WEIGHT_DENOMINATOR" spec:"true"`   // WeightDenominator accounts for total rewards denomination.
	ProposerWeight     uint64 `yaml:"PROPOSER_WEIGHT" spec:"true"`      // ProposerWeight is the factor of how much proposer rewards receives.

	// Validator related.
	TargetAggregatorsPerSyncSubcommittee uint64 `yaml:"TARGET_AGGREGATORS_PER_SYNC_SUBCOMMITTEE" spec:"true"` // TargetAggregatorsPerSyncSubcommittee for aggregating in sync committee.
	SyncCommitteeSubnetCount             uint64 `yaml:"SYNC_COMMITTEE_SUBNET_COUNT" spec:"true"`              // SyncCommitteeSubnetCount for sync committee subnet count.

	// Misc.
	SyncCommitteeSize            uint64           `yaml:"SYNC_COMMITTEE_SIZE" spec:"true"`              // SyncCommitteeSize for light client sync committee size.
	InactivityScoreBias          uint64           `yaml:"INACTIVITY_SCORE_BIAS" spec:"true"`            // InactivityScoreBias for calculating score bias penalties during inactivity
	InactivityScoreRecoveryRate  uint64           `yaml:"INACTIVITY_SCORE_RECOVERY_RATE" spec:"true"`   // InactivityScoreRecoveryRate for recovering score bias penalties during inactivity.
	EpochsPerSyncCommitteePeriod primitives.Epoch `yaml:"EPOCHS_PER_SYNC_COMMITTEE_PERIOD" spec:"true"` // EpochsPerSyncCommitteePeriod defines how many epochs per sync committee period.

	// Updated penalty values. This moves penalty parameters toward their final, maximum security values.
	// Note: We do not override previous configuration values but instead creates new values and replaces usage throughout.
	InactivityPenaltyQuotientAltair         uint64 `yaml:"INACTIVITY_PENALTY_QUOTIENT_ALTAIR" spec:"true"`         // InactivityPenaltyQuotientAltair for penalties during inactivity post Altair hard fork.
	MinSlashingPenaltyQuotientAltair        uint64 `yaml:"MIN_SLASHING_PENALTY_QUOTIENT_ALTAIR" spec:"true"`       // MinSlashingPenaltyQuotientAltair for slashing penalties post Altair hard fork.
	ProportionalSlashingMultiplierAltair    uint64 `yaml:"PROPORTIONAL_SLASHING_MULTIPLIER_ALTAIR" spec:"true"`    // ProportionalSlashingMultiplierAltair for slashing penalties' multiplier post Alair hard fork.
	MinSlashingPenaltyQuotientBellatrix     uint64 `yaml:"MIN_SLASHING_PENALTY_QUOTIENT_BELLATRIX" spec:"true"`    // MinSlashingPenaltyQuotientBellatrix for slashing penalties post Bellatrix hard fork.
	ProportionalSlashingMultiplierBellatrix uint64 `yaml:"PROPORTIONAL_SLASHING_MULTIPLIER_BELLATRIX" spec:"true"` // ProportionalSlashingMultiplierBellatrix for slashing penalties' multiplier post Bellatrix hard fork.
	InactivityPenaltyQuotientBellatrix      uint64 `yaml:"INACTIVITY_PENALTY_QUOTIENT_BELLATRIX" spec:"true"`      // InactivityPenaltyQuotientBellatrix for penalties during inactivity post Bellatrix hard fork.

	// Light client
	MinSyncCommitteeParticipants uint64 `yaml:"MIN_SYNC_COMMITTEE_PARTICIPANTS" spec:"true"` // MinSyncCommitteeParticipants defines the minimum amount of sync committee participants for which the light client acknowledges the signature.

	// Bellatrix
	TerminalBlockHash                common.Hash      `yaml:"TERMINAL_BLOCK_HASH" spec:"true"`                  // TerminalBlockHash of beacon chain.
	TerminalBlockHashActivationEpoch primitives.Epoch `yaml:"TERMINAL_BLOCK_HASH_ACTIVATION_EPOCH" spec:"true"` // TerminalBlockHashActivationEpoch of beacon chain.
	TerminalTotalDifficulty          string           `yaml:"TERMINAL_TOTAL_DIFFICULTY" spec:"true"`            // TerminalTotalDifficulty is part of the experimental Bellatrix spec. This value is type is currently TBD.
	DefaultFeeRecipient              common.Address   // DefaultFeeRecipient where the transaction fee goes to.
	EthBurnAddressHex                string           // EthBurnAddressHex is the constant eth address written in hex format to burn fees in that network. the default is 0x0
	DefaultBuilderGasLimit           uint64           // DefaultBuilderGasLimit is the default used to set the gaslimit for the Builder APIs, typically at around 30M wei.

	// Mev-boost circuit breaker
	MaxBuilderConsecutiveMissedSlots primitives.Slot // MaxBuilderConsecutiveMissedSlots defines the number of consecutive skip slot to fallback from using relay/builder to local execution engine for block construction.
	MaxBuilderEpochMissedSlots       primitives.Slot // MaxBuilderEpochMissedSlots is defines the number of total skip slot (per epoch rolling windows) to fallback from using relay/builder to local execution engine for block construction.
	LocalBlockValueBoost             uint64          // LocalBlockValueBoost is the value boost for local block construction. This is used to prioritize local block construction over relay/builder block construction.

	// Execution engine timeout value
	ExecutionEngineTimeoutValue uint64 // ExecutionEngineTimeoutValue defines the seconds to wait before timing out engine endpoints with execution payload execution semantics (newPayload, forkchoiceUpdated).
}

// InitializeForkSchedule initializes the schedules forks baked into the config.
func (b *BeaconChainConfig) InitializeForkSchedule() {
	// Reset Fork Version Schedule.
	b.ForkVersionSchedule = configForkSchedule(b)
	b.ForkVersionNames = configForkNames(b)
}

func configForkSchedule(b *BeaconChainConfig) map[[fieldparams.VersionLength]byte]primitives.Epoch {
	fvs := map[[fieldparams.VersionLength]byte]primitives.Epoch{}
	fvs[bytesutil.ToBytes4(b.GenesisForkVersion)] = b.GenesisEpoch
	fvs[bytesutil.ToBytes4(b.AltairForkVersion)] = b.AltairForkEpoch
	fvs[bytesutil.ToBytes4(b.BellatrixForkVersion)] = b.BellatrixForkEpoch
	fvs[bytesutil.ToBytes4(b.CapellaForkVersion)] = b.CapellaForkEpoch
	return fvs
}

func configForkNames(b *BeaconChainConfig) map[[fieldparams.VersionLength]byte]string {
	fvn := map[[fieldparams.VersionLength]byte]string{}
	fvn[bytesutil.ToBytes4(b.GenesisForkVersion)] = "phase0"
	fvn[bytesutil.ToBytes4(b.AltairForkVersion)] = "altair"
	fvn[bytesutil.ToBytes4(b.BellatrixForkVersion)] = "bellatrix"
	fvn[bytesutil.ToBytes4(b.CapellaForkVersion)] = "capella"
	return fvn
}

// Eth1DataVotesLength returns the maximum length of the votes on the Eth1 data,
// computed from the parameters in BeaconChainConfig.
func (b *BeaconChainConfig) Eth1DataVotesLength() uint64 {
	return uint64(b.EpochsPerEth1VotingPeriod.Mul(uint64(b.SlotsPerEpoch)))
}

// PreviousEpochAttestationsLength returns the maximum length of the pending
// attestation list for the previous epoch, computed from the parameters in
// BeaconChainConfig.
func (b *BeaconChainConfig) PreviousEpochAttestationsLength() uint64 {
	return uint64(b.SlotsPerEpoch.Mul(b.MaxAttestations))
}

// CurrentEpochAttestationsLength returns the maximum length of the pending
// attestation list for the current epoch, computed from the parameters in
// BeaconChainConfig.
func (b *BeaconChainConfig) CurrentEpochAttestationsLength() uint64 {
	return uint64(b.SlotsPerEpoch.Mul(b.MaxAttestations))
}
