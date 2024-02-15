package params

import (
	"encoding/hex"
	"fmt"
	"os"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/math"
	log "github.com/sirupsen/logrus"
	"gopkg.in/yaml.v2"
)

func isMinimal(lines []string) bool {
	for _, l := range lines {
		if strings.HasPrefix(l, "PRESET_BASE: 'minimal'") ||
			strings.HasPrefix(l, `PRESET_BASE: "minimal"`) ||
			strings.HasPrefix(l, "PRESET_BASE: minimal") ||
			strings.HasPrefix(l, "# Minimal preset") {
			return true
		}
	}
	return false
}

func UnmarshalConfig(yamlFile []byte, conf *BeaconChainConfig) (*BeaconChainConfig, error) {
	// To track if config name is defined inside config file.
	hasConfigName := false
	// Convert 0x hex inputs to fixed bytes arrays
	lines := strings.Split(string(yamlFile), "\n")
	if conf == nil {
		if isMinimal(lines) {
			conf = MinimalSpecConfig().Copy()
		} else {
			// Default to using mainnet.
			conf = MainnetConfig().Copy()
		}
	}
	for i, line := range lines {
		// No need to convert the deposit contract address to byte array (as config expects a string).
		if strings.HasPrefix(line, "DEPOSIT_CONTRACT_ADDRESS") {
			continue
		}
		if strings.HasPrefix(line, "CONFIG_NAME") {
			hasConfigName = true
		}
		if !strings.HasPrefix(line, "#") && strings.Contains(line, "0x") {
			parts := ReplaceHexStringWithYAMLFormat(line)
			lines[i] = strings.Join(parts, "\n")
		}
	}
	yamlFile = []byte(strings.Join(lines, "\n"))
	if err := yaml.UnmarshalStrict(yamlFile, conf); err != nil {
		if _, ok := err.(*yaml.TypeError); !ok {
			return nil, errors.Wrap(err, "Failed to parse chain config yaml file.")
		} else {
			log.WithError(err).Error("There were some issues parsing the config from a yaml file")
		}
	}
	if !hasConfigName {
		conf.ConfigName = DevnetName
	}
	// recompute SqrRootSlotsPerEpoch constant to handle non-standard values of SlotsPerEpoch
	conf.SqrRootSlotsPerEpoch = primitives.Slot(math.IntegerSquareRoot(uint64(conf.SlotsPerEpoch)))
	log.Debugf("Config file values: %+v", conf)
	return conf, nil
}

func UnmarshalConfigFile(path string, conf *BeaconChainConfig) (*BeaconChainConfig, error) {
	yamlFile, err := os.ReadFile(path) // #nosec G304
	if err != nil {
		return nil, errors.Wrap(err, "Failed to read chain config file.")
	}
	return UnmarshalConfig(yamlFile, conf)
}

// LoadChainConfigFile load, convert hex values into valid param yaml format,
// unmarshal , and apply beacon chain config file.
func LoadChainConfigFile(path string, conf *BeaconChainConfig) error {
	c, err := UnmarshalConfigFile(path, conf)
	if err != nil {
		return err
	}
	return SetActive(c)
}

// ReplaceHexStringWithYAMLFormat will replace hex strings that the yaml parser will understand.
func ReplaceHexStringWithYAMLFormat(line string) []string {
	parts := strings.Split(line, "0x")
	decoded, err := hex.DecodeString(parts[1])
	if err != nil {
		log.WithError(err).Error("Failed to decode hex string.")
	}
	switch l := len(decoded); {
	case l == 1:
		var b byte
		b = decoded[0]
		fixedByte, err := yaml.Marshal(b)
		if err != nil {
			log.WithError(err).Error("Failed to marshal config file.")
		}
		parts[0] += string(fixedByte)
		parts = parts[:1]
	case l > 1 && l <= 4:
		var arr [4]byte
		copy(arr[:], decoded)
		fixedByte, err := yaml.Marshal(arr)
		if err != nil {
			log.WithError(err).Error("Failed to marshal config file.")
		}
		parts[1] = string(fixedByte)
	case l > 4 && l <= 8:
		var arr [8]byte
		copy(arr[:], decoded)
		fixedByte, err := yaml.Marshal(arr)
		if err != nil {
			log.WithError(err).Error("Failed to marshal config file.")
		}
		parts[1] = string(fixedByte)
	case l > 8 && l <= 16:
		var arr [16]byte
		copy(arr[:], decoded)
		fixedByte, err := yaml.Marshal(arr)
		if err != nil {
			log.WithError(err).Error("Failed to marshal config file.")
		}
		parts[1] = string(fixedByte)
	case l > 16 && l <= 20:
		var arr [20]byte
		copy(arr[:], decoded)
		fixedByte, err := yaml.Marshal(arr)
		if err != nil {
			log.WithError(err).Error("Failed to marshal config file.")
		}
		parts[1] = string(fixedByte)
	case l > 20 && l <= 32:
		var arr [32]byte
		copy(arr[:], decoded)
		fixedByte, err := yaml.Marshal(arr)
		if err != nil {
			log.WithError(err).Error("Failed to marshal config file.")
		}
		parts[1] = string(fixedByte)
	case l > 32 && l <= 48:
		var arr [48]byte
		copy(arr[:], decoded)
		fixedByte, err := yaml.Marshal(arr)
		if err != nil {
			log.WithError(err).Error("Failed to marshal config file.")
		}
		parts[1] = string(fixedByte)
	case l > 48 && l <= 64:
		var arr [64]byte
		copy(arr[:], decoded)
		fixedByte, err := yaml.Marshal(arr)
		if err != nil {
			log.WithError(err).Error("Failed to marshal config file.")
		}
		parts[1] = string(fixedByte)
	case l > 64 && l <= 96:
		var arr [96]byte
		copy(arr[:], decoded)
		fixedByte, err := yaml.Marshal(arr)
		if err != nil {
			log.WithError(err).Error("Failed to marshal config file.")
		}
		parts[1] = string(fixedByte)
	}
	return parts
}

// ConfigToYaml takes a provided config and outputs its contents
// in yaml. This allows prysm's custom configs to be read by other clients.
func ConfigToYaml(cfg *BeaconChainConfig) []byte {
	lines := []string{
		fmt.Sprintf("PRESET_BASE: '%s'", cfg.PresetBase),
		fmt.Sprintf("CONFIG_NAME: '%s'", cfg.ConfigName),
		fmt.Sprintf("MIN_GENESIS_ACTIVE_VALIDATOR_COUNT: %d", cfg.MinGenesisActiveValidatorCount),
		fmt.Sprintf("GENESIS_DELAY: %d", cfg.GenesisDelay),
		fmt.Sprintf("MIN_GENESIS_TIME: %d", cfg.MinGenesisTime),
		fmt.Sprintf("GENESIS_FORK_VERSION: %#x", cfg.GenesisForkVersion),
		fmt.Sprintf("CHURN_LIMIT_QUOTIENT: %d", cfg.ChurnLimitQuotient),
		fmt.Sprintf("SECONDS_PER_SLOT: %d", cfg.SecondsPerSlot),
		fmt.Sprintf("SLOTS_PER_EPOCH: %d", cfg.SlotsPerEpoch),
		fmt.Sprintf("SECONDS_PER_ETH1_BLOCK: %d", cfg.SecondsPerETH1Block),
		fmt.Sprintf("ETH1_FOLLOW_DISTANCE: %d", cfg.Eth1FollowDistance),
		fmt.Sprintf("EPOCHS_PER_ETH1_VOTING_PERIOD: %d", cfg.EpochsPerEth1VotingPeriod),
		fmt.Sprintf("SHARD_COMMITTEE_PERIOD: %d", cfg.ShardCommitteePeriod),
		fmt.Sprintf("MIN_VALIDATOR_WITHDRAWABILITY_DELAY: %d", cfg.MinValidatorWithdrawabilityDelay),
		fmt.Sprintf("MAX_VALIDATORS_PER_WITHDRAWALS_SWEEP: %d", cfg.MaxValidatorsPerWithdrawalsSweep),
		fmt.Sprintf("MAX_SEED_LOOKAHEAD: %d", cfg.MaxSeedLookahead),
		fmt.Sprintf("EJECTION_BALANCE: %d", cfg.EjectionBalance),
		fmt.Sprintf("MIN_PER_EPOCH_CHURN_LIMIT: %d", cfg.MinPerEpochChurnLimit),
		fmt.Sprintf("DEPOSIT_CHAIN_ID: %d", cfg.DepositChainID),
		fmt.Sprintf("DEPOSIT_NETWORK_ID: %d", cfg.DepositNetworkID),
		fmt.Sprintf("ALTAIR_FORK_EPOCH: %d", cfg.AltairForkEpoch),
		fmt.Sprintf("ALTAIR_FORK_VERSION: %#x", cfg.AltairForkVersion),
		fmt.Sprintf("BELLATRIX_FORK_EPOCH: %d", cfg.BellatrixForkEpoch),
		fmt.Sprintf("BELLATRIX_FORK_VERSION: %#x", cfg.BellatrixForkVersion),
		fmt.Sprintf("CAPELLA_FORK_EPOCH: %d", cfg.CapellaForkEpoch),
		fmt.Sprintf("CAPELLA_FORK_VERSION: %#x", cfg.CapellaForkVersion),
		fmt.Sprintf("INACTIVITY_SCORE_BIAS: %d", cfg.InactivityScoreBias),
		fmt.Sprintf("INACTIVITY_SCORE_RECOVERY_RATE: %d", cfg.InactivityScoreRecoveryRate),
		fmt.Sprintf("TERMINAL_TOTAL_DIFFICULTY: %s", cfg.TerminalTotalDifficulty),
		fmt.Sprintf("TERMINAL_BLOCK_HASH: %#x", cfg.TerminalBlockHash),
		fmt.Sprintf("TERMINAL_BLOCK_HASH_ACTIVATION_EPOCH: %d", cfg.TerminalBlockHashActivationEpoch),
		fmt.Sprintf("DEPOSIT_CONTRACT_ADDRESS: %s", cfg.DepositContractAddress),
		fmt.Sprintf("MAX_PER_EPOCH_ACTIVATION_CHURN_LIMIT: %d", cfg.MaxPerEpochActivationChurnLimit),
		fmt.Sprintf("MIN_EPOCHS_FOR_BLOB_SIDECARS_REQUESTS: %d", cfg.MinEpochsForBlobsSidecarsRequest),
		fmt.Sprintf("MAX_REQUEST_BLOCKS_DENEB: %d", cfg.MaxRequestBlocksDeneb),
		fmt.Sprintf("MAX_REQUEST_BLOB_SIDECARS: %d", cfg.MaxRequestBlobSidecars),
		fmt.Sprintf("BLOB_SIDECAR_SUBNET_COUNT: %d", cfg.BlobsidecarSubnetCount),
		fmt.Sprintf("DENEB_FORK_EPOCH: %d", cfg.DenebForkEpoch),
		fmt.Sprintf("DENEB_FORK_VERSION: %#x", cfg.DenebForkVersion),
		fmt.Sprintf("EPOCHS_PER_SUBNET_SUBSCRIPTION: %d", cfg.EpochsPerSubnetSubscription),
		fmt.Sprintf("ATTESTATION_SUBNET_EXTRA_BITS: %d", cfg.AttestationSubnetExtraBits),
		fmt.Sprintf("ATTESTATION_SUBNET_PREFIX_BITS: %d", cfg.AttestationSubnetPrefixBits),
		fmt.Sprintf("SUBNETS_PER_NODE: %d", cfg.SubnetsPerNode),
		fmt.Sprintf("NODE_ID_BITS: %d", cfg.NodeIdBits),
		fmt.Sprintf("GOSSIP_MAX_SIZE: %d", cfg.GossipMaxSize),
		fmt.Sprintf("MAX_CHUNK_SIZE: %d", cfg.MaxChunkSize),
		fmt.Sprintf("ATTESTATION_SUBNET_COUNT: %d", cfg.AttestationSubnetCount),
		fmt.Sprintf("ATTESTATION_PROPAGATION_SLOT_RANGE: %d", cfg.AttestationPropagationSlotRange),
		fmt.Sprintf("MAX_REQUEST_BLOCKS: %d", cfg.MaxRequestBlocks),
		fmt.Sprintf("TTFB_TIMEOUT: %d", int(cfg.TtfbTimeout)),
		fmt.Sprintf("RESP_TIMEOUT: %d", int(cfg.RespTimeout)),
		fmt.Sprintf("MAXIMUM_GOSSIP_CLOCK_DISPARITY: %d", int(cfg.MaximumGossipClockDisparity)),
		fmt.Sprintf("MESSAGE_DOMAIN_INVALID_SNAPPY:  %#x", cfg.MessageDomainInvalidSnappy),
		fmt.Sprintf("MESSAGE_DOMAIN_VALID_SNAPPY: %#x", cfg.MessageDomainValidSnappy),
		fmt.Sprintf("MIN_EPOCHS_FOR_BLOCK_REQUESTS: %d", int(cfg.MinEpochsForBlockRequests)),
	}

	yamlFile := []byte(strings.Join(lines, "\n"))
	return yamlFile
}
