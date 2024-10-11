package node

import (
	"flag"
	"fmt"
	"os"
	"strconv"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	logTest "github.com/sirupsen/logrus/hooks/test"
	"github.com/urfave/cli/v2"
)

func TestConfigureHistoricalSlasher(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	hook := logTest.NewGlobal()

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.Bool(flags.HistoricalSlasherNode.Name, true, "")
	cliCtx := cli.NewContext(&app, set, nil)

	require.NoError(t, configureHistoricalSlasher(cliCtx))

	assert.Equal(t, params.BeaconConfig().SlotsPerEpoch*4, params.BeaconConfig().SlotsPerArchivedPoint)
	assert.LogsContain(t, hook,
		fmt.Sprintf(
			"Setting %d slots per archive point and %d max RPC page size for historical slasher usage",
			params.BeaconConfig().SlotsPerArchivedPoint,
			int(params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().MaxAttestations))),
	)
}

func TestConfigureSlotsPerArchivedPoint(t *testing.T) {
	params.SetupTestConfigCleanup(t)

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.Int(flags.SlotsPerArchivedPoint.Name, 0, "")
	require.NoError(t, set.Set(flags.SlotsPerArchivedPoint.Name, strconv.Itoa(100)))
	cliCtx := cli.NewContext(&app, set, nil)

	require.NoError(t, configureSlotsPerArchivedPoint(cliCtx))

	assert.Equal(t, primitives.Slot(100), params.BeaconConfig().SlotsPerArchivedPoint)
}

func TestConfigureProofOfWork(t *testing.T) {
	params.SetupTestConfigCleanup(t)

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.Uint64(flags.ChainID.Name, 0, "")
	set.Uint64(flags.NetworkID.Name, 0, "")
	set.String(flags.DepositContractFlag.Name, "", "")
	require.NoError(t, set.Set(flags.ChainID.Name, strconv.Itoa(100)))
	require.NoError(t, set.Set(flags.NetworkID.Name, strconv.Itoa(200)))
	require.NoError(t, set.Set(flags.DepositContractFlag.Name, "deposit-contract"))
	cliCtx := cli.NewContext(&app, set, nil)

	require.NoError(t, configureEth1Config(cliCtx))

	assert.Equal(t, uint64(100), params.BeaconConfig().DepositChainID)
	assert.Equal(t, uint64(200), params.BeaconConfig().DepositNetworkID)
	assert.Equal(t, "deposit-contract", params.BeaconConfig().DepositContractAddress)
}

func TestConfigureExecutionSetting(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	hook := logTest.NewGlobal()

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(flags.SuggestedFeeRecipient.Name, "", "")
	set.Uint64(flags.TerminalTotalDifficultyOverride.Name, 0, "")
	set.String(flags.TerminalBlockHashOverride.Name, "", "")
	set.Uint64(flags.TerminalBlockHashActivationEpochOverride.Name, 0, "")

	require.NoError(t, set.Set(flags.TerminalTotalDifficultyOverride.Name, strconv.Itoa(100)))
	require.NoError(t, set.Set(flags.TerminalBlockHashOverride.Name, "0xA"))
	require.NoError(t, set.Set(flags.TerminalBlockHashActivationEpochOverride.Name, strconv.Itoa(200)))
	require.NoError(t, set.Set(flags.SuggestedFeeRecipient.Name, "0xB"))
	cliCtx := cli.NewContext(&app, set, nil)
	err := configureExecutionSetting(cliCtx)
	assert.LogsContain(t, hook, "0xB is not a valid fee recipient address")
	require.NoError(t, err)

	require.NoError(t, set.Set(flags.SuggestedFeeRecipient.Name, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"))
	cliCtx = cli.NewContext(&app, set, nil)
	err = configureExecutionSetting(cliCtx)
	require.NoError(t, err)
	assert.Equal(t, common.HexToAddress("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"), params.BeaconConfig().DefaultFeeRecipient)

	assert.LogsContain(t, hook,
		"is not a checksum Ethereum address",
	)
	require.NoError(t, set.Set(flags.SuggestedFeeRecipient.Name, "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa"))
	cliCtx = cli.NewContext(&app, set, nil)
	err = configureExecutionSetting(cliCtx)
	require.NoError(t, err)
	assert.Equal(t, common.HexToAddress("0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa"), params.BeaconConfig().DefaultFeeRecipient)

	assert.Equal(t, "100", params.BeaconConfig().TerminalTotalDifficulty)
	assert.Equal(t, common.HexToHash("0xA"), params.BeaconConfig().TerminalBlockHash)
	assert.Equal(t, primitives.Epoch(200), params.BeaconConfig().TerminalBlockHashActivationEpoch)

}

func TestConfigureNetwork(t *testing.T) {
	params.SetupTestConfigCleanup(t)

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	bootstrapNodes := cli.StringSlice{}
	set.Var(&bootstrapNodes, cmd.BootstrapNode.Name, "")
	set.Int(flags.ContractDeploymentBlock.Name, 0, "")
	require.NoError(t, set.Set(cmd.BootstrapNode.Name, "node1"))
	require.NoError(t, set.Set(cmd.BootstrapNode.Name, "node2"))
	require.NoError(t, set.Set(flags.ContractDeploymentBlock.Name, strconv.Itoa(100)))
	cliCtx := cli.NewContext(&app, set, nil)

	configureNetwork(cliCtx)

	assert.DeepEqual(t, []string{"node1", "node2"}, params.BeaconNetworkConfig().BootstrapNodes)
	assert.Equal(t, uint64(100), params.BeaconNetworkConfig().ContractDeploymentBlock)
}

func TestConfigureNetwork_ConfigFile(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	context := cli.NewContext(&app, set, nil)

	require.NoError(t, os.WriteFile("flags_test.yaml", []byte(fmt.Sprintf("%s:\n - %s\n - %s\n", cmd.BootstrapNode.Name,
		"node1",
		"node2")), 0666))

	require.NoError(t, set.Parse([]string{"test-command", "--" + cmd.ConfigFileFlag.Name, "flags_test.yaml"}))
	comFlags := cmd.WrapFlags([]cli.Flag{
		&cli.StringFlag{
			Name: cmd.ConfigFileFlag.Name,
		},
		&cli.StringSliceFlag{
			Name: cmd.BootstrapNode.Name,
		},
	})
	command := &cli.Command{
		Name:  "test-command",
		Flags: comFlags,
		Before: func(cliCtx *cli.Context) error {
			return cmd.LoadFlagsFromConfig(cliCtx, comFlags)
		},
		Action: func(cliCtx *cli.Context) error {
			require.Equal(t, true, cliCtx.IsSet(cmd.BootstrapNode.Name))

			require.Equal(t, strings.Join([]string{"node1", "node2"}, ","),
				strings.Join(cliCtx.StringSlice(cmd.BootstrapNode.Name), ","))
			return nil
		},
	}
	require.NoError(t, command.Run(context, context.Args().Slice()...))
	require.NoError(t, os.Remove("flags_test.yaml"))
}

func TestConfigureInterop(t *testing.T) {
	params.SetupTestConfigCleanup(t)

	tests := []struct {
		name       string
		flagSetter func() *cli.Context
		configName string
	}{
		{
			"nothing set",
			func() *cli.Context {
				app := cli.App{}
				set := flag.NewFlagSet("test", 0)
				return cli.NewContext(&app, set, nil)
			},
			"mainnet",
		},
		{
			"mock votes set",
			func() *cli.Context {
				app := cli.App{}
				set := flag.NewFlagSet("test", 0)
				set.Bool(flags.InteropMockEth1DataVotesFlag.Name, false, "")
				assert.NoError(t, set.Set(flags.InteropMockEth1DataVotesFlag.Name, "true"))
				return cli.NewContext(&app, set, nil)
			},
			"interop",
		},
		{
			"validators set",
			func() *cli.Context {
				app := cli.App{}
				set := flag.NewFlagSet("test", 0)
				set.Uint64(flags.InteropNumValidatorsFlag.Name, 0, "")
				assert.NoError(t, set.Set(flags.InteropNumValidatorsFlag.Name, "20"))
				return cli.NewContext(&app, set, nil)
			},
			"interop",
		},
		{
			"genesis time set",
			func() *cli.Context {
				app := cli.App{}
				set := flag.NewFlagSet("test", 0)
				set.Uint64(flags.InteropGenesisTimeFlag.Name, 0, "")
				assert.NoError(t, set.Set(flags.InteropGenesisTimeFlag.Name, "200"))
				return cli.NewContext(&app, set, nil)
			},
			"interop",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			require.NoError(t, configureInteropConfig(tt.flagSetter()))
			assert.DeepEqual(t, tt.configName, params.BeaconConfig().ConfigName)
		})
	}
}
