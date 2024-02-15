package params

const (
	AltairE2EForkEpoch    = 6
	BellatrixE2EForkEpoch = 8
	CapellaE2EForkEpoch   = 10
	DenebE2EForkEpoch     = 12
)

// E2ETestConfig retrieves the configurations made specifically for E2E testing.
//
// WARNING: This config is only for testing, it is not meant for use outside of E2E.
func E2ETestConfig() *BeaconChainConfig {
	e2eConfig := MinimalSpecConfig()
	e2eConfig.DepositContractAddress = "0x4242424242424242424242424242424242424242"
	e2eConfig.Eth1FollowDistance = 8

	// Misc.
	e2eConfig.MinGenesisActiveValidatorCount = 256
	e2eConfig.GenesisDelay = 10 // 10 seconds so E2E has enough time to process deposits and get started.
	e2eConfig.ChurnLimitQuotient = 65536
	e2eConfig.MaxValidatorsPerWithdrawalsSweep = 128

	// Time parameters.
	e2eConfig.SecondsPerSlot = 10
	e2eConfig.SlotsPerEpoch = 6
	e2eConfig.SqrRootSlotsPerEpoch = 2
	e2eConfig.SecondsPerETH1Block = 2
	e2eConfig.EpochsPerEth1VotingPeriod = 2
	e2eConfig.ShardCommitteePeriod = 4
	e2eConfig.MaxSeedLookahead = 1
	e2eConfig.MinValidatorWithdrawabilityDelay = 1

	// PoW parameters.
	e2eConfig.DepositChainID = 1337   // Chain ID of eth1 dev net.
	e2eConfig.DepositNetworkID = 1337 // Network ID of eth1 dev net.

	// Fork Parameters.
	e2eConfig.AltairForkEpoch = AltairE2EForkEpoch
	e2eConfig.BellatrixForkEpoch = BellatrixE2EForkEpoch
	e2eConfig.CapellaForkEpoch = CapellaE2EForkEpoch
	e2eConfig.DenebForkEpoch = DenebE2EForkEpoch

	// Terminal Total Difficulty.
	e2eConfig.TerminalTotalDifficulty = "480"

	// Prysm constants.
	e2eConfig.ConfigName = EndToEndName
	e2eConfig.GenesisForkVersion = []byte{0, 0, 0, 253}
	e2eConfig.AltairForkVersion = []byte{1, 0, 0, 253}
	e2eConfig.BellatrixForkVersion = []byte{2, 0, 0, 253}
	e2eConfig.CapellaForkVersion = []byte{3, 0, 0, 253}
	e2eConfig.DenebForkVersion = []byte{4, 0, 0, 253}

	e2eConfig.InitializeForkSchedule()
	return e2eConfig
}

func E2EMainnetTestConfig() *BeaconChainConfig {
	e2eConfig := MainnetConfig().Copy()
	e2eConfig.DepositContractAddress = "0x4242424242424242424242424242424242424242"
	e2eConfig.Eth1FollowDistance = 8

	// Misc.
	e2eConfig.MinGenesisActiveValidatorCount = 256
	e2eConfig.GenesisDelay = 25 // 25 seconds so E2E has enough time to process deposits and get started.
	e2eConfig.ChurnLimitQuotient = 65536

	// Time parameters.
	e2eConfig.SecondsPerSlot = 6
	e2eConfig.SqrRootSlotsPerEpoch = 5
	e2eConfig.SecondsPerETH1Block = 2
	e2eConfig.ShardCommitteePeriod = 4
	e2eConfig.MinValidatorWithdrawabilityDelay = 1

	// PoW parameters.
	e2eConfig.DepositChainID = 1337   // Chain ID of eth1 dev net.
	e2eConfig.DepositNetworkID = 1337 // Network ID of eth1 dev net.

	// Altair Fork Parameters.
	e2eConfig.AltairForkEpoch = AltairE2EForkEpoch
	e2eConfig.BellatrixForkEpoch = BellatrixE2EForkEpoch
	e2eConfig.CapellaForkEpoch = CapellaE2EForkEpoch
	e2eConfig.DenebForkEpoch = DenebE2EForkEpoch

	// Terminal Total Difficulty.
	e2eConfig.TerminalTotalDifficulty = "480"

	// Prysm constants.
	e2eConfig.ConfigName = EndToEndMainnetName
	e2eConfig.GenesisForkVersion = []byte{0, 0, 0, 254}
	e2eConfig.AltairForkVersion = []byte{1, 0, 0, 254}
	e2eConfig.BellatrixForkVersion = []byte{2, 0, 0, 254}
	e2eConfig.CapellaForkVersion = []byte{3, 0, 0, 254}
	e2eConfig.DenebForkVersion = []byte{4, 0, 0, 254}

	// Deneb changes.
	e2eConfig.MinPerEpochChurnLimit = 2

	e2eConfig.InitializeForkSchedule()
	return e2eConfig
}
