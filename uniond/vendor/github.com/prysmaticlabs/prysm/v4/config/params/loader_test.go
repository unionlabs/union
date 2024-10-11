package params_test

import (
	"bytes"
	"io"
	"os"
	"path"
	"path/filepath"
	"reflect"
	"sort"
	"strings"
	"testing"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"gopkg.in/yaml.v2"
)

// Variables defined in the placeholderFields will not be tested in `TestLoadConfigFile`.
// These are variables that we don't use in Prysm. (i.e. future hardfork, light client... etc)
// IMPORTANT: Use one field per line and sort these alphabetically to reduce conflicts.
var placeholderFields = []string{
	"EIP6110_FORK_EPOCH",
	"EIP6110_FORK_VERSION",
	"EIP7002_FORK_EPOCH",
	"EIP7002_FORK_VERSION",
	"MAX_BLOBS_PER_BLOCK",
	"REORG_HEAD_WEIGHT_THRESHOLD",
	"UPDATE_TIMEOUT",
	"WHISK_EPOCHS_PER_SHUFFLING_PHASE",
	"WHISK_FORK_EPOCH",
	"WHISK_FORK_VERSION",
	"WHISK_PROPOSER_SELECTION_GAP",
}

func TestPlaceholderFieldsDistinctSorted(t *testing.T) {
	m := make(map[string]struct{})
	for i := 0; i < len(placeholderFields)-1; i++ {
		if _, ok := m[placeholderFields[i]]; ok {
			t.Fatalf("duplicate placeholder field %s", placeholderFields[i])
		}
		m[placeholderFields[i]] = struct{}{}
	}
	if !sort.StringsAreSorted(placeholderFields) {
		t.Fatal("placeholderFields must be sorted")
	}
}

func assertEqualConfigs(t *testing.T, name string, fields []string, expected, actual *params.BeaconChainConfig) {
	//  Misc params.
	assert.Equal(t, expected.MaxCommitteesPerSlot, actual.MaxCommitteesPerSlot, "%s: MaxCommitteesPerSlot", name)
	assert.Equal(t, expected.TargetCommitteeSize, actual.TargetCommitteeSize, "%s: TargetCommitteeSize", name)
	assert.Equal(t, expected.MaxValidatorsPerCommittee, actual.MaxValidatorsPerCommittee, "%s: MaxValidatorsPerCommittee", name)
	assert.Equal(t, expected.MinPerEpochChurnLimit, actual.MinPerEpochChurnLimit, "%s: MinPerEpochChurnLimit", name)
	assert.Equal(t, expected.ChurnLimitQuotient, actual.ChurnLimitQuotient, "%s: ChurnLimitQuotient", name)
	assert.Equal(t, expected.ShuffleRoundCount, actual.ShuffleRoundCount, "%s: ShuffleRoundCount", name)
	assert.Equal(t, expected.MinGenesisActiveValidatorCount, actual.MinGenesisActiveValidatorCount, "%s: MinGenesisActiveValidatorCount", name)
	assert.Equal(t, expected.MinGenesisTime, actual.MinGenesisTime, "%s: MinGenesisTime", name)
	assert.Equal(t, expected.HysteresisQuotient, actual.HysteresisQuotient, "%s: HysteresisQuotient", name)
	assert.Equal(t, expected.HysteresisDownwardMultiplier, actual.HysteresisDownwardMultiplier, "%s: HysteresisDownwardMultiplier", name)
	assert.Equal(t, expected.HysteresisUpwardMultiplier, actual.HysteresisUpwardMultiplier, "%s: HysteresisUpwardMultiplier", name)

	// Fork Choice params.
	assert.Equal(t, expected.DeprecatedSafeSlotsToUpdateJustified, actual.DeprecatedSafeSlotsToUpdateJustified, "%s: SafeSlotsToUpdateJustified", name)

	// Validator params.
	assert.Equal(t, expected.Eth1FollowDistance, actual.Eth1FollowDistance, "%s: Eth1FollowDistance", name)
	assert.Equal(t, expected.TargetAggregatorsPerCommittee, actual.TargetAggregatorsPerCommittee, "%s: TargetAggregatorsPerCommittee", name)
	assert.Equal(t, expected.RandomSubnetsPerValidator, actual.RandomSubnetsPerValidator, "%s: RandomSubnetsPerValidator", name)
	assert.Equal(t, expected.EpochsPerRandomSubnetSubscription, actual.EpochsPerRandomSubnetSubscription, "%s: EpochsPerRandomSubnetSubscription", name)
	assert.Equal(t, expected.SecondsPerETH1Block, actual.SecondsPerETH1Block, "%s: SecondsPerETH1Block", name)

	// Deposit contract.
	assert.Equal(t, expected.DepositChainID, actual.DepositChainID, "%s: DepositChainID", name)
	assert.Equal(t, expected.DepositNetworkID, actual.DepositNetworkID, "%s: DepositNetworkID", name)
	assert.Equal(t, expected.DepositContractAddress, actual.DepositContractAddress, "%s: DepositContractAddress", name)

	// Gwei values.
	assert.Equal(t, expected.MinDepositAmount, actual.MinDepositAmount, "%s: MinDepositAmount", name)
	assert.Equal(t, expected.MaxEffectiveBalance, actual.MaxEffectiveBalance, "%s: MaxEffectiveBalance", name)
	assert.Equal(t, expected.EjectionBalance, actual.EjectionBalance, "%s: EjectionBalance", name)
	assert.Equal(t, expected.EffectiveBalanceIncrement, actual.EffectiveBalanceIncrement, "%s: EffectiveBalanceIncrement", name)

	// Initial values.
	assert.DeepEqual(t, expected.GenesisForkVersion, actual.GenesisForkVersion, "%s: GenesisForkVersion", name)
	assert.DeepEqual(t, expected.BLSWithdrawalPrefixByte, actual.BLSWithdrawalPrefixByte, "%s: BLSWithdrawalPrefixByte", name)
	assert.DeepEqual(t, expected.ETH1AddressWithdrawalPrefixByte, actual.ETH1AddressWithdrawalPrefixByte, "%s: ETH1AddressWithdrawalPrefixByte", name)

	// Time parameters.
	assert.Equal(t, expected.GenesisDelay, actual.GenesisDelay, "%s: GenesisDelay", name)
	assert.Equal(t, expected.SecondsPerSlot, actual.SecondsPerSlot, "%s: SecondsPerSlot", name)
	assert.Equal(t, expected.MinAttestationInclusionDelay, actual.MinAttestationInclusionDelay, "%s: MinAttestationInclusionDelay", name)
	assert.Equal(t, expected.SlotsPerEpoch, actual.SlotsPerEpoch, "%s: SlotsPerEpoch", name)
	assert.Equal(t, expected.MinSeedLookahead, actual.MinSeedLookahead, "%s: MinSeedLookahead", name)
	assert.Equal(t, expected.MaxSeedLookahead, actual.MaxSeedLookahead, "%s: MaxSeedLookahead", name)
	assert.Equal(t, expected.EpochsPerEth1VotingPeriod, actual.EpochsPerEth1VotingPeriod, "%s: EpochsPerEth1VotingPeriod", name)
	assert.Equal(t, expected.SlotsPerHistoricalRoot, actual.SlotsPerHistoricalRoot, "%s: SlotsPerHistoricalRoot", name)
	assert.Equal(t, expected.MinValidatorWithdrawabilityDelay, actual.MinValidatorWithdrawabilityDelay, "%s: MinValidatorWithdrawabilityDelay", name)
	assert.Equal(t, expected.ShardCommitteePeriod, actual.ShardCommitteePeriod, "%s: ShardCommitteePeriod", name)
	assert.Equal(t, expected.MinEpochsToInactivityPenalty, actual.MinEpochsToInactivityPenalty, "%s: MinEpochsToInactivityPenalty", name)

	// State vector lengths.
	assert.Equal(t, expected.EpochsPerHistoricalVector, actual.EpochsPerHistoricalVector, "%s: EpochsPerHistoricalVector", name)
	assert.Equal(t, expected.EpochsPerSlashingsVector, actual.EpochsPerSlashingsVector, "%s: EpochsPerSlashingsVector", name)
	assert.Equal(t, expected.HistoricalRootsLimit, actual.HistoricalRootsLimit, "%s: HistoricalRootsLimit", name)
	assert.Equal(t, expected.ValidatorRegistryLimit, actual.ValidatorRegistryLimit, "%s: ValidatorRegistryLimit", name)

	// Reward and penalty quotients.
	assert.Equal(t, expected.BaseRewardFactor, actual.BaseRewardFactor, "%s: BaseRewardFactor", name)
	assert.Equal(t, expected.WhistleBlowerRewardQuotient, actual.WhistleBlowerRewardQuotient, "%s: WhistleBlowerRewardQuotient", name)
	assert.Equal(t, expected.ProposerRewardQuotient, actual.ProposerRewardQuotient, "%s: ProposerRewardQuotient", name)
	assert.Equal(t, expected.InactivityPenaltyQuotient, actual.InactivityPenaltyQuotient, "%s: InactivityPenaltyQuotient", name)
	assert.Equal(t, expected.InactivityPenaltyQuotientAltair, actual.InactivityPenaltyQuotientAltair, "%s: InactivityPenaltyQuotientAltair", name)
	assert.Equal(t, expected.MinSlashingPenaltyQuotient, actual.MinSlashingPenaltyQuotient, "%s: MinSlashingPenaltyQuotient", name)
	assert.Equal(t, expected.MinSlashingPenaltyQuotientAltair, actual.MinSlashingPenaltyQuotientAltair, "%s: MinSlashingPenaltyQuotientAltair", name)
	assert.Equal(t, expected.ProportionalSlashingMultiplier, actual.ProportionalSlashingMultiplier, "%s: ProportionalSlashingMultiplier", name)
	assert.Equal(t, expected.ProportionalSlashingMultiplierAltair, actual.ProportionalSlashingMultiplierAltair, "%s: ProportionalSlashingMultiplierAltair", name)

	// Max operations per block.
	assert.Equal(t, expected.MaxProposerSlashings, actual.MaxProposerSlashings, "%s: MaxProposerSlashings", name)
	assert.Equal(t, expected.MaxAttesterSlashings, actual.MaxAttesterSlashings, "%s: MaxAttesterSlashings", name)
	assert.Equal(t, expected.MaxAttestations, actual.MaxAttestations, "%s: MaxAttestations", name)
	assert.Equal(t, expected.MaxDeposits, actual.MaxDeposits, "%s: MaxDeposits", name)
	assert.Equal(t, expected.MaxVoluntaryExits, actual.MaxVoluntaryExits, "%s: MaxVoluntaryExits", name)

	// Signature domains.
	assert.Equal(t, expected.DomainBeaconProposer, actual.DomainBeaconProposer, "%s: DomainBeaconProposer", name)
	assert.Equal(t, expected.DomainBeaconAttester, actual.DomainBeaconAttester, "%s: DomainBeaconAttester", name)
	assert.Equal(t, expected.DomainRandao, actual.DomainRandao, "%s: DomainRandao", name)
	assert.Equal(t, expected.DomainDeposit, actual.DomainDeposit, "%s: DomainDeposit", name)
	assert.Equal(t, expected.DomainVoluntaryExit, actual.DomainVoluntaryExit, "%s: DomainVoluntaryExit", name)
	assert.Equal(t, expected.DomainSelectionProof, actual.DomainSelectionProof, "%s: DomainSelectionProof", name)
	assert.Equal(t, expected.DomainAggregateAndProof, actual.DomainAggregateAndProof, "%s: DomainAggregateAndProof", name)
	assert.Equal(t, expected.TerminalTotalDifficulty, actual.TerminalTotalDifficulty, "%s: TerminalTotalDifficulty", name)
	assert.Equal(t, expected.AltairForkEpoch, actual.AltairForkEpoch, "%s: AltairForkEpoch", name)
	assert.Equal(t, expected.BellatrixForkEpoch, actual.BellatrixForkEpoch, "%s: BellatrixForkEpoch", name)
	assert.Equal(t, expected.CapellaForkEpoch, actual.CapellaForkEpoch, "%s: CapellaForkEpoch", name)
	assert.Equal(t, expected.DenebForkEpoch, actual.DenebForkEpoch, "%s: DenebForkEpoch", name)
	assert.Equal(t, expected.SqrRootSlotsPerEpoch, actual.SqrRootSlotsPerEpoch, "%s: SqrRootSlotsPerEpoch", name)
	assert.DeepEqual(t, expected.GenesisForkVersion, actual.GenesisForkVersion, "%s: GenesisForkVersion", name)
	assert.DeepEqual(t, expected.AltairForkVersion, actual.AltairForkVersion, "%s: AltairForkVersion", name)
	assert.DeepEqual(t, expected.BellatrixForkVersion, actual.BellatrixForkVersion, "%s: BellatrixForkVersion", name)
	assert.DeepEqual(t, expected.CapellaForkVersion, actual.CapellaForkVersion, "%s: CapellaForkVersion", name)
	assert.DeepEqual(t, expected.DenebForkVersion, actual.DenebForkVersion, "%s: DenebForkVersion", name)

	assertYamlFieldsMatch(t, name, fields, expected, actual)
}

func TestModifiedE2E(t *testing.T) {
	c := params.E2ETestConfig().Copy()
	c.DepositContractAddress = "0x4242424242424242424242424242424242424242"
	c.TerminalTotalDifficulty = "0"
	c.AltairForkEpoch = 112
	c.BellatrixForkEpoch = 123
	c.CapellaForkEpoch = 235
	c.DenebForkEpoch = 358
	y := params.ConfigToYaml(c)
	cfg, err := params.UnmarshalConfig(y, nil)
	require.NoError(t, err)
	assertEqualConfigs(t, "modified-e2e", []string{}, c, cfg)
}

func TestLoadConfigFile(t *testing.T) {
	t.Run("mainnet", func(t *testing.T) {
		mn := params.MainnetConfig().Copy()
		mainnetPresetsFiles := presetsFilePath(t, "mainnet")
		var err error
		for _, fp := range mainnetPresetsFiles {
			mn, err = params.UnmarshalConfigFile(fp, mn)
			require.NoError(t, err)
		}
		// configs loaded from file get the name 'devnet' unless they specify a specific name in the yaml itself.
		// since these are partial patches for presets, they do not have the config name
		mn.ConfigName = params.MainnetName
		mainnetConfigFile := configFilePath(t, "mainnet")
		mnf, err := params.UnmarshalConfigFile(mainnetConfigFile, nil)
		require.NoError(t, err)
		fields := fieldsFromYamls(t, append(mainnetPresetsFiles, mainnetConfigFile))
		assertEqualConfigs(t, "mainnet", fields, mn, mnf)
	})

	t.Run("minimal", func(t *testing.T) {
		min := params.MinimalSpecConfig().Copy()
		minimalPresetsFiles := presetsFilePath(t, "minimal")
		var err error
		for _, fp := range minimalPresetsFiles {
			min, err = params.UnmarshalConfigFile(fp, min)
			require.NoError(t, err)
		}
		// configs loaded from file get the name 'devnet' unless they specify a specific name in the yaml itself.
		// since these are partial patches for presets, they do not have the config name
		min.ConfigName = params.MinimalName
		minimalConfigFile := configFilePath(t, "minimal")
		minf, err := params.UnmarshalConfigFile(minimalConfigFile, nil)
		require.NoError(t, err)
		fields := fieldsFromYamls(t, append(minimalPresetsFiles, minimalConfigFile))
		assertEqualConfigs(t, "minimal", fields, min, minf)
	})

	t.Run("e2e", func(t *testing.T) {
		e2e, err := params.ByName(params.EndToEndName)
		require.NoError(t, err)
		configFile := "testdata/e2e_config.yaml"
		e2ef, err := params.UnmarshalConfigFile(configFile, nil)
		require.NoError(t, err)
		fields := fieldsFromYamls(t, []string{configFile})
		assertEqualConfigs(t, "e2e", fields, e2e, e2ef)
	})
}

func TestLoadConfigFile_OverwriteCorrectly(t *testing.T) {
	f, err := os.CreateTemp("", "")
	require.NoError(t, err)
	// Set current config to minimal config
	cfg := params.MinimalSpecConfig().Copy()
	params.FillTestVersions(cfg, 128)
	_, err = io.Copy(f, bytes.NewBuffer(params.ConfigToYaml(cfg)))
	require.NoError(t, err)

	// set active config to mainnet, so that we can confirm LoadChainConfigFile overrides it
	mainnet, err := params.ByName(params.MainnetName)
	require.NoError(t, err)
	undo, err := params.SetActiveWithUndo(mainnet)
	require.NoError(t, err)
	defer func() {
		err := undo()
		require.NoError(t, err)
	}()

	// load empty config file, so that it defaults to mainnet values
	require.NoError(t, params.LoadChainConfigFile(f.Name(), nil))
	if params.BeaconConfig().MinGenesisTime != cfg.MinGenesisTime {
		t.Errorf("Expected MinGenesisTime to be set to value written to config: %d found: %d",
			cfg.MinGenesisTime,
			params.BeaconConfig().MinGenesisTime)
	}
	if params.BeaconConfig().SlotsPerEpoch != cfg.SlotsPerEpoch {
		t.Errorf("Expected SlotsPerEpoch to be set to value written to config: %d found: %d",
			cfg.SlotsPerEpoch,
			params.BeaconConfig().SlotsPerEpoch)
	}
	require.Equal(t, params.MinimalName, params.BeaconConfig().ConfigName)
}

func Test_replaceHexStringWithYAMLFormat(t *testing.T) {

	testLines := []struct {
		line   string
		wanted string
	}{
		{
			line:   "ONE_BYTE: 0x41",
			wanted: "ONE_BYTE: 65\n",
		},
		{
			line:   "FOUR_BYTES: 0x41414141",
			wanted: "FOUR_BYTES: \n- 65\n- 65\n- 65\n- 65\n",
		},
		{
			line:   "THREE_BYTES: 0x414141",
			wanted: "THREE_BYTES: \n- 65\n- 65\n- 65\n- 0\n",
		},
		{
			line:   "EIGHT_BYTES: 0x4141414141414141",
			wanted: "EIGHT_BYTES: \n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n",
		},
		{
			line: "SIXTEEN_BYTES: 0x41414141414141414141414141414141",
			wanted: "SIXTEEN_BYTES: \n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n" +
				"- 65\n- 65\n- 65\n- 65\n",
		},
		{
			line: "TWENTY_BYTES: 0x4141414141414141414141414141414141414141",
			wanted: "TWENTY_BYTES: \n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n" +
				"- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n",
		},
		{
			line: "THIRTY_TWO_BYTES: 0x4141414141414141414141414141414141414141414141414141414141414141",
			wanted: "THIRTY_TWO_BYTES: \n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n" +
				"- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n" +
				"- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n",
		},
		{
			line: "FORTY_EIGHT_BYTES: 0x41414141414141414141414141414141414141414141414141414141414141414141" +
				"4141414141414141414141414141",
			wanted: "FORTY_EIGHT_BYTES: \n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n" +
				"- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n" +
				"- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n" +
				"- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n",
		},
		{
			line: "NINETY_SIX_BYTES: 0x414141414141414141414141414141414141414141414141414141414141414141414141" +
				"4141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141" +
				"41414141414141414141414141",
			wanted: "NINETY_SIX_BYTES: \n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n" +
				"- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n" +
				"- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n" +
				"- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n" +
				"- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n" +
				"- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n" +
				"- 65\n- 65\n- 65\n- 65\n- 65\n- 65\n",
		},
	}
	for _, line := range testLines {
		parts := params.ReplaceHexStringWithYAMLFormat(line.line)
		res := strings.Join(parts, "\n")

		if res != line.wanted {
			t.Errorf("expected conversion to be: %v got: %v", line.wanted, res)
		}
	}
}

func TestConfigParityYaml(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	testDir := bazel.TestTmpDir()
	yamlDir := filepath.Join(testDir, "config.yaml")

	testCfg := params.E2ETestConfig()
	yamlObj := params.ConfigToYaml(testCfg)
	assert.NoError(t, file.WriteFile(yamlDir, yamlObj))

	require.NoError(t, params.LoadChainConfigFile(yamlDir, params.E2ETestConfig().Copy()))
	assert.DeepEqual(t, params.BeaconConfig(), testCfg)
}

// configFilePath sets the proper config and returns the relevant
// config file path from eth2-spec-tests directory.
func configFilePath(t *testing.T, config string) string {
	fPath, err := bazel.Runfile("external/consensus_spec")
	require.NoError(t, err)
	configFilePath := path.Join(fPath, "configs", config+".yaml")
	return configFilePath
}

// presetsFilePath returns the relevant preset file paths from eth2-spec-tests
// directory. This method returns a preset file path for each hard fork or
// major network upgrade, in order.
func presetsFilePath(t *testing.T, config string) []string {
	fPath, err := bazel.Runfile("external/consensus_spec")
	require.NoError(t, err)
	return []string{
		path.Join(fPath, "presets", config, "phase0.yaml"),
		path.Join(fPath, "presets", config, "altair.yaml"),
	}
}

func fieldsFromYamls(t *testing.T, fps []string) []string {
	var keys []string
	for _, fp := range fps {
		yamlFile, err := os.ReadFile(fp)
		require.NoError(t, err)
		m := make(map[string]interface{})
		require.NoError(t, yaml.Unmarshal(yamlFile, &m))

		for k := range m {
			if k == "SHARDING_FORK_VERSION" || k == "SHARDING_FORK_EPOCH" {
				continue
			}
			keys = append(keys, k)
		}

		if len(keys) == 0 {
			t.Errorf("No fields loaded from yaml file %s", fp)
		}
	}

	return keys
}

func assertYamlFieldsMatch(t *testing.T, name string, fields []string, c1, c2 *params.BeaconChainConfig) {
	// Ensure all fields from the yaml file exist, were set, and correctly match the expected value.
	ft1 := reflect.TypeOf(*c1)
	for _, field := range fields {
		var found bool
		for i := 0; i < ft1.NumField(); i++ {
			v, ok := ft1.Field(i).Tag.Lookup("yaml")
			if ok && v == field {
				if isPlaceholderField(v) {
					// If you see this error, remove the field from placeholderFields.
					t.Errorf("beacon config has a placeholder field defined, remove %s from the placeholder fields variable", v)
					continue
				}
				found = true
				v1 := reflect.ValueOf(*c1).Field(i).Interface()
				v2 := reflect.ValueOf(*c2).Field(i).Interface()
				if reflect.ValueOf(v1).Kind() == reflect.Slice {
					assert.DeepEqual(t, v1, v2, "%s: %s", name, field)
				} else {
					assert.Equal(t, v1, v2, "%s: %s", name, field)
				}
				break
			}
		}
		if !found && !isPlaceholderField(field) { // Ignore placeholder fields
			t.Errorf("No struct tag found `yaml:%s`", field)
		}
	}
}

func isPlaceholderField(field string) bool {
	for _, f := range placeholderFields {
		if f == field {
			return true
		}
	}
	return false
}
