package params

import (
	"math"
	"time"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
)

// MainnetConfig returns the configuration to be used in the main network.
func MainnetConfig() *BeaconChainConfig {
	if mainnetBeaconConfig.ForkVersionSchedule == nil {
		mainnetBeaconConfig.InitializeForkSchedule()
	}
	return mainnetBeaconConfig
}

const (
	// Genesis Fork Epoch for the mainnet config.
	genesisForkEpoch = 0
	// Altair Fork Epoch for mainnet config.
	mainnetAltairForkEpoch = 74240 // Oct 27, 2021, 10:56:23am UTC
	// Bellatrix Fork Epoch for mainnet config.
	mainnetBellatrixForkEpoch = 144896 // Sept 6, 2022, 11:34:47am UTC
)

var mainnetNetworkConfig = &NetworkConfig{
	GossipMaxSize:                   1 << 20,      // 1 MiB
	GossipMaxSizeBellatrix:          10 * 1 << 20, // 10 MiB
	MaxChunkSize:                    1 << 20,      // 1 MiB
	MaxChunkSizeBellatrix:           10 * 1 << 20, // 10 MiB
	AttestationSubnetCount:          64,
	AttestationPropagationSlotRange: 32,
	MaxRequestBlocks:                1 << 10, // 1024
	TtfbTimeout:                     5 * time.Second,
	RespTimeout:                     10 * time.Second,
	MaximumGossipClockDisparity:     500 * time.Millisecond,
	MessageDomainInvalidSnappy:      [4]byte{00, 00, 00, 00},
	MessageDomainValidSnappy:        [4]byte{01, 00, 00, 00},
	ETH2Key:                         "eth2",
	AttSubnetKey:                    "attnets",
	SyncCommsSubnetKey:              "syncnets",
	MinimumPeersInSubnetSearch:      20,
	ContractDeploymentBlock:         11184524, // Note: contract was deployed in block 11052984 but no transactions were sent until 11184524.
	BootstrapNodes: []string{
		// Teku team's bootnode
		"enr:-KG4QOtcP9X1FbIMOe17QNMKqDxCpm14jcX5tiOE4_TyMrFqbmhPZHK_ZPG2Gxb1GE2xdtodOfx9-cgvNtxnRyHEmC0ghGV0aDKQ9aX9QgAAAAD__________4JpZIJ2NIJpcIQDE8KdiXNlY3AyNTZrMaEDhpehBDbZjM_L9ek699Y7vhUJ-eAdMyQW_Fil522Y0fODdGNwgiMog3VkcIIjKA",
		"enr:-KG4QL-eqFoHy0cI31THvtZjpYUu_Jdw_MO7skQRJxY1g5HTN1A0epPCU6vi0gLGUgrzpU-ygeMSS8ewVxDpKfYmxMMGhGV0aDKQtTA_KgAAAAD__________4JpZIJ2NIJpcIQ2_DUbiXNlY3AyNTZrMaED8GJ2vzUqgL6-KD1xalo1CsmY4X1HaDnyl6Y_WayCo9GDdGNwgiMog3VkcIIjKA",
		// Prylab team's bootnodes
		"enr:-Ku4QImhMc1z8yCiNJ1TyUxdcfNucje3BGwEHzodEZUan8PherEo4sF7pPHPSIB1NNuSg5fZy7qFsjmUKs2ea1Whi0EBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpD1pf1CAAAAAP__________gmlkgnY0gmlwhBLf22SJc2VjcDI1NmsxoQOVphkDqal4QzPMksc5wnpuC3gvSC8AfbFOnZY_On34wIN1ZHCCIyg",
		"enr:-Ku4QP2xDnEtUXIjzJ_DhlCRN9SN99RYQPJL92TMlSv7U5C1YnYLjwOQHgZIUXw6c-BvRg2Yc2QsZxxoS_pPRVe0yK8Bh2F0dG5ldHOIAAAAAAAAAACEZXRoMpD1pf1CAAAAAP__________gmlkgnY0gmlwhBLf22SJc2VjcDI1NmsxoQMeFF5GrS7UZpAH2Ly84aLK-TyvH-dRo0JM1i8yygH50YN1ZHCCJxA",
		"enr:-Ku4QPp9z1W4tAO8Ber_NQierYaOStqhDqQdOPY3bB3jDgkjcbk6YrEnVYIiCBbTxuar3CzS528d2iE7TdJsrL-dEKoBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpD1pf1CAAAAAP__________gmlkgnY0gmlwhBLf22SJc2VjcDI1NmsxoQMw5fqqkw2hHC4F5HZZDPsNmPdB1Gi8JPQK7pRc9XHh-oN1ZHCCKvg",
		// Lighthouse team's bootnodes
		"enr:-Jq4QItoFUuug_n_qbYbU0OY04-np2wT8rUCauOOXNi0H3BWbDj-zbfZb7otA7jZ6flbBpx1LNZK2TDebZ9dEKx84LYBhGV0aDKQtTA_KgEAAAD__________4JpZIJ2NIJpcISsaa0ZiXNlY3AyNTZrMaEDHAD2JKYevx89W0CcFJFiskdcEzkH_Wdv9iW42qLK79ODdWRwgiMo",
		"enr:-Jq4QN_YBsUOqQsty1OGvYv48PMaiEt1AzGD1NkYQHaxZoTyVGqMYXg0K9c0LPNWC9pkXmggApp8nygYLsQwScwAgfgBhGV0aDKQtTA_KgEAAAD__________4JpZIJ2NIJpcISLosQxiXNlY3AyNTZrMaEDBJj7_dLFACaxBfaI8KZTh_SSJUjhyAyfshimvSqo22WDdWRwgiMo",
		// EF bootnodes
		"enr:-Ku4QHqVeJ8PPICcWk1vSn_XcSkjOkNiTg6Fmii5j6vUQgvzMc9L1goFnLKgXqBJspJjIsB91LTOleFmyWWrFVATGngBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpC1MD8qAAAAAP__________gmlkgnY0gmlwhAMRHkWJc2VjcDI1NmsxoQKLVXFOhp2uX6jeT0DvvDpPcU8FWMjQdR4wMuORMhpX24N1ZHCCIyg",
		"enr:-Ku4QG-2_Md3sZIAUebGYT6g0SMskIml77l6yR-M_JXc-UdNHCmHQeOiMLbylPejyJsdAPsTHJyjJB2sYGDLe0dn8uYBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpC1MD8qAAAAAP__________gmlkgnY0gmlwhBLY-NyJc2VjcDI1NmsxoQORcM6e19T1T9gi7jxEZjk_sjVLGFscUNqAY9obgZaxbIN1ZHCCIyg",
		"enr:-Ku4QPn5eVhcoF1opaFEvg1b6JNFD2rqVkHQ8HApOKK61OIcIXD127bKWgAtbwI7pnxx6cDyk_nI88TrZKQaGMZj0q0Bh2F0dG5ldHOIAAAAAAAAAACEZXRoMpC1MD8qAAAAAP__________gmlkgnY0gmlwhDayLMaJc2VjcDI1NmsxoQK2sBOLGcUb4AwuYzFuAVCaNHA-dy24UuEKkeFNgCVCsIN1ZHCCIyg",
		"enr:-Ku4QEWzdnVtXc2Q0ZVigfCGggOVB2Vc1ZCPEc6j21NIFLODSJbvNaef1g4PxhPwl_3kax86YPheFUSLXPRs98vvYsoBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpC1MD8qAAAAAP__________gmlkgnY0gmlwhDZBrP2Jc2VjcDI1NmsxoQM6jr8Rb1ktLEsVcKAPa08wCsKUmvoQ8khiOl_SLozf9IN1ZHCCIyg",
		// Nimbus bootnodes
		"enr:-LK4QA8FfhaAjlb_BXsXxSfiysR7R52Nhi9JBt4F8SPssu8hdE1BXQQEtVDC3qStCW60LSO7hEsVHv5zm8_6Vnjhcn0Bh2F0dG5ldHOIAAAAAAAAAACEZXRoMpC1MD8qAAAAAP__________gmlkgnY0gmlwhAN4aBKJc2VjcDI1NmsxoQJerDhsJ-KxZ8sHySMOCmTO6sHM3iCFQ6VMvLTe948MyYN0Y3CCI4yDdWRwgiOM",
		"enr:-LK4QKWrXTpV9T78hNG6s8AM6IO4XH9kFT91uZtFg1GcsJ6dKovDOr1jtAAFPnS2lvNltkOGA9k29BUN7lFh_sjuc9QBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpC1MD8qAAAAAP__________gmlkgnY0gmlwhANAdd-Jc2VjcDI1NmsxoQLQa6ai7y9PMN5hpLe5HmiJSlYzMuzP7ZhwRiwHvqNXdoN0Y3CCI4yDdWRwgiOM",
	},
}

var mainnetBeaconConfig = &BeaconChainConfig{
	// Constants (Non-configurable)
	FarFutureEpoch:           math.MaxUint64,
	FarFutureSlot:            math.MaxUint64,
	BaseRewardsPerEpoch:      4,
	DepositContractTreeDepth: 32,
	GenesisDelay:             604800, // 1 week.

	// Misc constant.
	TargetCommitteeSize:            128,
	MaxValidatorsPerCommittee:      2048,
	MaxCommitteesPerSlot:           64,
	MinPerEpochChurnLimit:          4,
	ChurnLimitQuotient:             1 << 16,
	ShuffleRoundCount:              90,
	MinGenesisActiveValidatorCount: 16384,
	MinGenesisTime:                 1606824000, // Dec 1, 2020, 12pm UTC.
	TargetAggregatorsPerCommittee:  16,
	HysteresisQuotient:             4,
	HysteresisDownwardMultiplier:   1,
	HysteresisUpwardMultiplier:     5,

	// Gwei value constants.
	MinDepositAmount:          1 * 1e9,
	MaxEffectiveBalance:       32 * 1e9,
	EjectionBalance:           16 * 1e9,
	EffectiveBalanceIncrement: 1 * 1e9,

	// Initial value constants.
	BLSWithdrawalPrefixByte:         byte(0),
	ETH1AddressWithdrawalPrefixByte: byte(1),
	ZeroHash:                        [32]byte{},

	// Time parameter constants.
	MinAttestationInclusionDelay:     1,
	SecondsPerSlot:                   12,
	SlotsPerEpoch:                    32,
	SqrRootSlotsPerEpoch:             5,
	MinSeedLookahead:                 1,
	MaxSeedLookahead:                 4,
	EpochsPerEth1VotingPeriod:        64,
	SlotsPerHistoricalRoot:           8192,
	MinValidatorWithdrawabilityDelay: 256,
	ShardCommitteePeriod:             256,
	MinEpochsToInactivityPenalty:     4,
	Eth1FollowDistance:               2048,

	// Fork choice algorithm constants.
	ProposerScoreBoost:              40,
	ReorgWeightThreshold:            20,
	ReorgParentWeightThreshold:      160,
	ReorgMaxEpochsSinceFinalization: 2,
	IntervalsPerSlot:                3,

	// Ethereum PoW parameters.
	DepositChainID:         1, // Chain ID of eth1 mainnet.
	DepositNetworkID:       1, // Network ID of eth1 mainnet.
	DepositContractAddress: "0x00000000219ab540356cBB839Cbe05303d7705Fa",

	// Validator params.
	RandomSubnetsPerValidator:         1 << 0,
	EpochsPerRandomSubnetSubscription: 1 << 8,

	// While eth1 mainnet block times are closer to 13s, we must conform with other clients in
	// order to vote on the correct eth1 blocks.
	//
	// Additional context: https://github.com/ethereum/consensus-specs/issues/2132
	// Bug prompting this change: https://github.com/prysmaticlabs/prysm/issues/7856
	// Future optimization: https://github.com/prysmaticlabs/prysm/issues/7739
	SecondsPerETH1Block: 14,

	// State list length constants.
	EpochsPerHistoricalVector: 65536,
	EpochsPerSlashingsVector:  8192,
	HistoricalRootsLimit:      16777216,
	ValidatorRegistryLimit:    1099511627776,

	// Reward and penalty quotients constants.
	BaseRewardFactor:               64,
	WhistleBlowerRewardQuotient:    512,
	ProposerRewardQuotient:         8,
	InactivityPenaltyQuotient:      67108864,
	MinSlashingPenaltyQuotient:     128,
	ProportionalSlashingMultiplier: 1,

	// Max operations per block constants.
	MaxProposerSlashings:             16,
	MaxAttesterSlashings:             2,
	MaxAttestations:                  128,
	MaxDeposits:                      16,
	MaxVoluntaryExits:                16,
	MaxWithdrawalsPerPayload:         16,
	MaxBlsToExecutionChanges:         16,
	MaxValidatorsPerWithdrawalsSweep: 16384,

	// BLS domain values.
	DomainBeaconProposer:              bytesutil.Uint32ToBytes4(0x00000000),
	DomainBeaconAttester:              bytesutil.Uint32ToBytes4(0x01000000),
	DomainRandao:                      bytesutil.Uint32ToBytes4(0x02000000),
	DomainDeposit:                     bytesutil.Uint32ToBytes4(0x03000000),
	DomainVoluntaryExit:               bytesutil.Uint32ToBytes4(0x04000000),
	DomainSelectionProof:              bytesutil.Uint32ToBytes4(0x05000000),
	DomainAggregateAndProof:           bytesutil.Uint32ToBytes4(0x06000000),
	DomainSyncCommittee:               bytesutil.Uint32ToBytes4(0x07000000),
	DomainSyncCommitteeSelectionProof: bytesutil.Uint32ToBytes4(0x08000000),
	DomainContributionAndProof:        bytesutil.Uint32ToBytes4(0x09000000),
	DomainApplicationMask:             bytesutil.Uint32ToBytes4(0x00000001),
	DomainApplicationBuilder:          bytesutil.Uint32ToBytes4(0x00000001),
	DomainBLSToExecutionChange:        bytesutil.Uint32ToBytes4(0x0A000000),

	// Prysm constants.
	GweiPerEth:                     1000000000,
	BLSSecretKeyLength:             32,
	BLSPubkeyLength:                48,
	DefaultBufferSize:              10000,
	WithdrawalPrivkeyFileName:      "/shardwithdrawalkey",
	ValidatorPrivkeyFileName:       "/validatorprivatekey",
	RPCSyncCheck:                   1,
	EmptySignature:                 [96]byte{},
	DefaultPageSize:                250,
	MaxPeersToSync:                 15,
	SlotsPerArchivedPoint:          2048,
	GenesisCountdownInterval:       time.Minute,
	ConfigName:                     MainnetName,
	PresetBase:                     "mainnet",
	BeaconStateFieldCount:          21,
	BeaconStateAltairFieldCount:    24,
	BeaconStateBellatrixFieldCount: 25,
	BeaconStateCapellaFieldCount:   28,

	// Slasher related values.
	WeakSubjectivityPeriod:          54000,
	PruneSlasherStoragePeriod:       10,
	SlashingProtectionPruningEpochs: 512,

	// Weak subjectivity values.
	SafetyDecay: 10,

	// Fork related values.
	GenesisEpoch:         genesisForkEpoch,
	GenesisForkVersion:   []byte{0, 0, 0, 0},
	AltairForkVersion:    []byte{1, 0, 0, 0},
	AltairForkEpoch:      mainnetAltairForkEpoch,
	BellatrixForkVersion: []byte{2, 0, 0, 0},
	BellatrixForkEpoch:   mainnetBellatrixForkEpoch,
	CapellaForkVersion:   []byte{3, 0, 0, 0},
	CapellaForkEpoch:     194048,

	// New values introduced in Altair hard fork 1.
	// Participation flag indices.
	TimelySourceFlagIndex: 0,
	TimelyTargetFlagIndex: 1,
	TimelyHeadFlagIndex:   2,

	// Incentivization weight values.
	TimelySourceWeight: 14,
	TimelyTargetWeight: 26,
	TimelyHeadWeight:   14,
	SyncRewardWeight:   2,
	ProposerWeight:     8,
	WeightDenominator:  64,

	// Validator related values.
	TargetAggregatorsPerSyncSubcommittee: 16,
	SyncCommitteeSubnetCount:             4,

	// Misc values.
	SyncCommitteeSize:            512,
	InactivityScoreBias:          4,
	InactivityScoreRecoveryRate:  16,
	EpochsPerSyncCommitteePeriod: 256,

	// Updated penalty values.
	InactivityPenaltyQuotientAltair:         3 * 1 << 24, //50331648
	MinSlashingPenaltyQuotientAltair:        64,
	ProportionalSlashingMultiplierAltair:    2,
	MinSlashingPenaltyQuotientBellatrix:     32,
	ProportionalSlashingMultiplierBellatrix: 3,
	InactivityPenaltyQuotientBellatrix:      1 << 24,

	// Light client
	MinSyncCommitteeParticipants: 1,

	// Bellatrix
	TerminalBlockHashActivationEpoch: 18446744073709551615,
	TerminalBlockHash:                [32]byte{},
	TerminalTotalDifficulty:          "58750000000000000000000", // Estimated: Sept 15, 2022
	EthBurnAddressHex:                "0x0000000000000000000000000000000000000000",
	DefaultBuilderGasLimit:           uint64(30000000),

	// Mevboost circuit breaker
	MaxBuilderConsecutiveMissedSlots: 3,
	MaxBuilderEpochMissedSlots:       5,
	// Execution engine timeout value
	ExecutionEngineTimeoutValue: 8, // 8 seconds default based on: https://github.com/ethereum/execution-apis/blob/main/src/engine/specification.md#core
}

// MainnetTestConfig provides a version of the mainnet config that has a different name
// and a different fork choice schedule. This can be used in cases where we want to use config values
// that are consistent with mainnet, but won't conflict or cause the hard-coded genesis to be loaded.
func MainnetTestConfig() *BeaconChainConfig {
	mn := MainnetConfig().Copy()
	mn.ConfigName = MainnetTestName
	FillTestVersions(mn, 128)
	return mn
}

// FillTestVersions replaces the fork schedule in the given BeaconChainConfig with test values, using the given
// byte argument as the high byte (common across forks).
func FillTestVersions(c *BeaconChainConfig, b byte) {
	c.GenesisForkVersion = make([]byte, fieldparams.VersionLength)
	c.AltairForkVersion = make([]byte, fieldparams.VersionLength)
	c.BellatrixForkVersion = make([]byte, fieldparams.VersionLength)
	c.CapellaForkVersion = make([]byte, fieldparams.VersionLength)

	c.GenesisForkVersion[fieldparams.VersionLength-1] = b
	c.AltairForkVersion[fieldparams.VersionLength-1] = b
	c.BellatrixForkVersion[fieldparams.VersionLength-1] = b
	c.CapellaForkVersion[fieldparams.VersionLength-1] = b

	c.GenesisForkVersion[0] = 0
	c.AltairForkVersion[0] = 1
	c.BellatrixForkVersion[0] = 2
	c.CapellaForkVersion[0] = 3
}
