package params

import (
	"math"

	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
)

// MinimalSpecConfig retrieves the minimal config used in spec tests.
func MinimalSpecConfig() *BeaconChainConfig {
	minimalConfig := mainnetBeaconConfig.Copy()
	// Misc
	minimalConfig.MaxCommitteesPerSlot = 4
	minimalConfig.TargetCommitteeSize = 4
	minimalConfig.MaxValidatorsPerCommittee = 2048
	minimalConfig.MinPerEpochChurnLimit = 2           // Changed in EIP7514
	minimalConfig.MaxPerEpochActivationChurnLimit = 4 // New in EIP7514
	minimalConfig.ChurnLimitQuotient = 32
	minimalConfig.ShuffleRoundCount = 10
	minimalConfig.MinGenesisActiveValidatorCount = 64
	minimalConfig.MinGenesisTime = 1578009600
	minimalConfig.GenesisDelay = 300 // 5 minutes
	minimalConfig.TargetAggregatorsPerCommittee = 16

	// Gwei values
	minimalConfig.MinDepositAmount = 1e9
	minimalConfig.MaxEffectiveBalance = 32e9
	minimalConfig.EjectionBalance = 16e9
	minimalConfig.EffectiveBalanceIncrement = 1e9

	// Initial values
	minimalConfig.BLSWithdrawalPrefixByte = byte(0)
	minimalConfig.ETH1AddressWithdrawalPrefixByte = byte(1)

	// Time parameters
	minimalConfig.SecondsPerSlot = 6
	minimalConfig.MinAttestationInclusionDelay = 1
	minimalConfig.SlotsPerEpoch = 8
	minimalConfig.SqrRootSlotsPerEpoch = 2
	minimalConfig.MinSeedLookahead = 1
	minimalConfig.MaxSeedLookahead = 4
	minimalConfig.EpochsPerEth1VotingPeriod = 4
	minimalConfig.SlotsPerHistoricalRoot = 64
	minimalConfig.MinValidatorWithdrawabilityDelay = 256
	minimalConfig.ShardCommitteePeriod = 64
	minimalConfig.MinEpochsToInactivityPenalty = 4
	minimalConfig.Eth1FollowDistance = 16
	minimalConfig.SecondsPerETH1Block = 14

	// State vector lengths
	minimalConfig.EpochsPerHistoricalVector = 64
	minimalConfig.EpochsPerSlashingsVector = 64
	minimalConfig.HistoricalRootsLimit = 16777216
	minimalConfig.ValidatorRegistryLimit = 1099511627776

	// Reward and penalty quotients
	minimalConfig.BaseRewardFactor = 64
	minimalConfig.WhistleBlowerRewardQuotient = 512
	minimalConfig.ProposerRewardQuotient = 8
	minimalConfig.InactivityPenaltyQuotient = 33554432
	minimalConfig.MinSlashingPenaltyQuotient = 64
	minimalConfig.ProportionalSlashingMultiplier = 2

	// Max operations per block
	minimalConfig.MaxProposerSlashings = 16
	minimalConfig.MaxAttesterSlashings = 2
	minimalConfig.MaxAttestations = 128
	minimalConfig.MaxDeposits = 16
	minimalConfig.MaxVoluntaryExits = 16
	minimalConfig.MaxWithdrawalsPerPayload = 4
	minimalConfig.MaxValidatorsPerWithdrawalsSweep = 16

	// Signature domains
	minimalConfig.DomainBeaconProposer = bytesutil.ToBytes4(bytesutil.Bytes4(0))
	minimalConfig.DomainBeaconAttester = bytesutil.ToBytes4(bytesutil.Bytes4(1))
	minimalConfig.DomainRandao = bytesutil.ToBytes4(bytesutil.Bytes4(2))
	minimalConfig.DomainDeposit = bytesutil.ToBytes4(bytesutil.Bytes4(3))
	minimalConfig.DomainVoluntaryExit = bytesutil.ToBytes4(bytesutil.Bytes4(4))
	minimalConfig.GenesisForkVersion = []byte{0, 0, 0, 1}

	minimalConfig.DepositContractTreeDepth = 32
	minimalConfig.FarFutureEpoch = math.MaxUint64
	minimalConfig.FarFutureSlot = math.MaxUint64

	// New Altair params
	minimalConfig.AltairForkVersion = []byte{1, 0, 0, 1} // Highest byte set to 0x01 to avoid collisions with mainnet versioning
	minimalConfig.AltairForkEpoch = math.MaxUint64
	minimalConfig.BellatrixForkVersion = []byte{2, 0, 0, 1}
	minimalConfig.BellatrixForkEpoch = math.MaxUint64
	minimalConfig.CapellaForkVersion = []byte{3, 0, 0, 1}
	minimalConfig.CapellaForkEpoch = math.MaxUint64
	minimalConfig.DenebForkVersion = []byte{4, 0, 0, 1}

	minimalConfig.SyncCommitteeSize = 32
	minimalConfig.InactivityScoreBias = 4
	minimalConfig.EpochsPerSyncCommitteePeriod = 8
	minimalConfig.MinEpochsForBlockRequests = 272

	// Ethereum PoW parameters.
	minimalConfig.DepositChainID = 5   // Chain ID of eth1 goerli.
	minimalConfig.DepositNetworkID = 5 // Network ID of eth1 goerli.
	minimalConfig.DepositContractAddress = "0x1234567890123456789012345678901234567890"
	// 2**256-2**10 for fake minimal network
	minimalConfig.TerminalTotalDifficulty = "115792089237316195423570985008687907853269984665640564039457584007913129638912"

	minimalConfig.ConfigName = MinimalName
	minimalConfig.PresetBase = "minimal"

	minimalConfig.InitializeForkSchedule()
	return minimalConfig
}
