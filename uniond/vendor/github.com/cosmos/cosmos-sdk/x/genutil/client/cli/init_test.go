package cli_test

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"os"
	"testing"
	"time"

	abci_server "github.com/cometbft/cometbft/abci/server"
	"github.com/cometbft/cometbft/libs/cli"
	"github.com/cometbft/cometbft/libs/log"
	tmtypes "github.com/cometbft/cometbft/types"
	"github.com/spf13/viper"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/server"
	"github.com/cosmos/cosmos-sdk/server/mock"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	genutilcli "github.com/cosmos/cosmos-sdk/x/genutil/client/cli"
	genutiltest "github.com/cosmos/cosmos-sdk/x/genutil/client/testutil"
	"github.com/cosmos/cosmos-sdk/x/staking"
)

var testMbm = module.NewBasicManager(
	staking.AppModuleBasic{},
	genutil.AppModuleBasic{},
)

func TestInitCmd(t *testing.T) {
	tests := []struct {
		name      string
		flags     func(dir string) []string
		shouldErr bool
		err       error
	}{
		{
			name: "happy path",
			flags: func(dir string) []string {
				return []string{
					"appnode-test",
					"bn254",
				}
			},
			shouldErr: false,
			err:       nil,
		},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			home := t.TempDir()
			logger := log.NewNopLogger()
			cfg, err := genutiltest.CreateDefaultTendermintConfig(home)
			require.NoError(t, err)

			serverCtx := server.NewContext(viper.New(), cfg, logger)
			interfaceRegistry := types.NewInterfaceRegistry()
			marshaler := codec.NewProtoCodec(interfaceRegistry)
			clientCtx := client.Context{}.
				WithCodec(marshaler).
				WithLegacyAmino(makeCodec()).
				WithHomeDir(home)

			ctx := context.Background()
			ctx = context.WithValue(ctx, client.ClientContextKey, &clientCtx)
			ctx = context.WithValue(ctx, server.ServerContextKey, serverCtx)

			cmd := genutilcli.InitCmd(testMbm, home)
			cmd.SetArgs(
				tt.flags(home),
			)

			if tt.shouldErr {
				err := cmd.ExecuteContext(ctx)
				require.EqualError(t, err, tt.err.Error())
			} else {
				require.NoError(t, cmd.ExecuteContext(ctx))
			}
		})
	}
}

func TestInitRecover(t *testing.T) {
	home := t.TempDir()
	logger := log.NewNopLogger()
	cfg, err := genutiltest.CreateDefaultTendermintConfig(home)
	require.NoError(t, err)

	serverCtx := server.NewContext(viper.New(), cfg, logger)
	interfaceRegistry := types.NewInterfaceRegistry()
	marshaler := codec.NewProtoCodec(interfaceRegistry)
	clientCtx := client.Context{}.
		WithCodec(marshaler).
		WithLegacyAmino(makeCodec()).
		WithHomeDir(home)

	ctx := context.Background()
	ctx = context.WithValue(ctx, client.ClientContextKey, &clientCtx)
	ctx = context.WithValue(ctx, server.ServerContextKey, serverCtx)

	cmd := genutilcli.InitCmd(testMbm, home)
	mockIn := testutil.ApplyMockIODiscardOutErr(cmd)

	cmd.SetArgs([]string{
		"appnode-test",
		"bn254",
		fmt.Sprintf("--%s=true", genutilcli.FlagRecover),
	})

	// use valid mnemonic and complete recovery key generation successfully
	mockIn.Reset("decide praise business actor peasant farm drastic weather extend front hurt later song give verb rhythm worry fun pond reform school tumble august one\n")
	require.NoError(t, cmd.ExecuteContext(ctx))
}

func TestInitDefaultBondDenom(t *testing.T) {
	home := t.TempDir()
	logger := log.NewNopLogger()
	cfg, err := genutiltest.CreateDefaultTendermintConfig(home)
	require.NoError(t, err)

	serverCtx := server.NewContext(viper.New(), cfg, logger)
	interfaceRegistry := types.NewInterfaceRegistry()
	marshaler := codec.NewProtoCodec(interfaceRegistry)
	clientCtx := client.Context{}.
		WithCodec(marshaler).
		WithLegacyAmino(makeCodec()).
		WithHomeDir(home)

	ctx := context.Background()
	ctx = context.WithValue(ctx, client.ClientContextKey, &clientCtx)
	ctx = context.WithValue(ctx, server.ServerContextKey, serverCtx)

	cmd := genutilcli.InitCmd(testMbm, home)

	cmd.SetArgs([]string{
		"appnode-test",
		"bn254",
		fmt.Sprintf("--%s=%s", cli.HomeFlag, home),
		fmt.Sprintf("--%s=testtoken", genutilcli.FlagDefaultBondDenom),
	})
	require.NoError(t, cmd.ExecuteContext(ctx))
}

func TestEmptyState(t *testing.T) {
	home := t.TempDir()
	logger := log.NewNopLogger()
	cfg, err := genutiltest.CreateDefaultTendermintConfig(home)
	require.NoError(t, err)

	serverCtx := server.NewContext(viper.New(), cfg, logger)
	interfaceRegistry := types.NewInterfaceRegistry()
	marshaler := codec.NewProtoCodec(interfaceRegistry)
	clientCtx := client.Context{}.
		WithCodec(marshaler).
		WithLegacyAmino(makeCodec()).
		WithHomeDir(home)

	ctx := context.Background()
	ctx = context.WithValue(ctx, client.ClientContextKey, &clientCtx)
	ctx = context.WithValue(ctx, server.ServerContextKey, serverCtx)

	cmd := genutilcli.InitCmd(testMbm, home)
	cmd.SetArgs([]string{"appnode-test", "bn254", fmt.Sprintf("--%s=%s", cli.HomeFlag, home)})

	require.NoError(t, cmd.ExecuteContext(ctx))

	old := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	cmd = server.ExportCmd(nil, home)
	cmd.SetArgs([]string{fmt.Sprintf("--%s=%s", cli.HomeFlag, home)})
	require.NoError(t, cmd.ExecuteContext(ctx))

	outC := make(chan string)
	go func() {
		var buf bytes.Buffer
		io.Copy(&buf, r)
		outC <- buf.String()
	}()

	w.Close()
	os.Stdout = old
	out := <-outC

	require.Contains(t, out, "genesis_time")
	require.Contains(t, out, "chain_id")
	require.Contains(t, out, "consensus_params")
	require.Contains(t, out, "app_hash")
	require.Contains(t, out, "app_state")
}

func TestStartStandAlone(t *testing.T) {
	home := t.TempDir()
	logger := log.NewNopLogger()
	interfaceRegistry := types.NewInterfaceRegistry()
	marshaler := codec.NewProtoCodec(interfaceRegistry)
	err := genutiltest.ExecInitCmd(testMbm, home, marshaler)
	require.NoError(t, err)

	app, err := mock.NewApp(home, logger)
	require.NoError(t, err)

	svrAddr, _, err := server.FreeTCPAddr()
	require.NoError(t, err)

	svr, err := abci_server.NewServer(svrAddr, "socket", app)
	require.NoError(t, err, "error creating listener")

	svr.SetLogger(logger.With("module", "abci-server"))
	err = svr.Start()
	require.NoError(t, err)

	timer := time.NewTimer(time.Duration(2) * time.Second)
	for range timer.C {
		err = svr.Stop()
		require.NoError(t, err)
		break
	}
}

func TestInitNodeValidatorFiles(t *testing.T) {
	home := t.TempDir()
	cfg, err := genutiltest.CreateDefaultTendermintConfig(home)
	require.NoError(t, err)

	nodeID, valPubKey, err := genutil.InitializeNodeValidatorFiles(cfg)
	require.NoError(t, err)

	require.NotEqual(t, "", nodeID)
	require.NotEqual(t, 0, len(valPubKey.Bytes()))
}

func TestInitConfig(t *testing.T) {
	home := t.TempDir()
	logger := log.NewNopLogger()
	cfg, err := genutiltest.CreateDefaultTendermintConfig(home)
	require.NoError(t, err)

	serverCtx := server.NewContext(viper.New(), cfg, logger)
	interfaceRegistry := types.NewInterfaceRegistry()
	marshaler := codec.NewProtoCodec(interfaceRegistry)
	clientCtx := client.Context{}.
		WithCodec(marshaler).
		WithLegacyAmino(makeCodec()).
		WithChainID("foo"). // add chain-id to clientCtx
		WithHomeDir(home)

	ctx := context.Background()
	ctx = context.WithValue(ctx, client.ClientContextKey, &clientCtx)
	ctx = context.WithValue(ctx, server.ServerContextKey, serverCtx)

	cmd := genutilcli.InitCmd(testMbm, home)
	cmd.SetArgs([]string{"testnode", "bn254"})

	require.NoError(t, cmd.ExecuteContext(ctx))

	old := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	cmd = server.ExportCmd(nil, home)
	require.NoError(t, cmd.ExecuteContext(ctx))

	outC := make(chan string)
	go func() {
		var buf bytes.Buffer
		io.Copy(&buf, r)
		outC <- buf.String()
	}()

	w.Close()
	os.Stdout = old
	out := <-outC

	require.Contains(t, out, "\"chain_id\": \"foo\"")
}

func TestInitWithHeight(t *testing.T) {
	home := t.TempDir()
	logger := log.NewNopLogger()
	cfg, err := genutiltest.CreateDefaultTendermintConfig(home)
	require.NoError(t, err)

	serverCtx := server.NewContext(viper.New(), cfg, logger)
	interfaceRegistry := types.NewInterfaceRegistry()
	marshaler := codec.NewProtoCodec(interfaceRegistry)
	clientCtx := client.Context{}.
		WithCodec(marshaler).
		WithLegacyAmino(makeCodec()).
		WithChainID("foo"). // add chain-id to clientCtx
		WithHomeDir(home)

	ctx := context.Background()
	ctx = context.WithValue(ctx, client.ClientContextKey, &clientCtx)
	ctx = context.WithValue(ctx, server.ServerContextKey, serverCtx)

	testInitialHeight := int64(333)

	cmd := genutilcli.InitCmd(testMbm, home)
	cmd.SetArgs([]string{"init-height-test", "bn254", fmt.Sprintf("--%s=%d", flags.FlagInitHeight, testInitialHeight)})

	require.NoError(t, cmd.ExecuteContext(ctx))

	appGenesis, importErr := tmtypes.GenesisDocFromFile(cfg.GenesisFile())
	require.NoError(t, importErr)

	require.Equal(t, testInitialHeight, appGenesis.InitialHeight)
}

func TestInitWithNegativeHeight(t *testing.T) {
	home := t.TempDir()
	logger := log.NewNopLogger()
	cfg, err := genutiltest.CreateDefaultTendermintConfig(home)
	require.NoError(t, err)

	serverCtx := server.NewContext(viper.New(), cfg, logger)
	interfaceRegistry := types.NewInterfaceRegistry()
	marshaler := codec.NewProtoCodec(interfaceRegistry)
	clientCtx := client.Context{}.
		WithCodec(marshaler).
		WithLegacyAmino(makeCodec()).
		WithChainID("foo"). // add chain-id to clientCtx
		WithHomeDir(home)

	ctx := context.Background()
	ctx = context.WithValue(ctx, client.ClientContextKey, &clientCtx)
	ctx = context.WithValue(ctx, server.ServerContextKey, serverCtx)

	testInitialHeight := int64(-333)

	cmd := genutilcli.InitCmd(testMbm, home)
	cmd.SetArgs([]string{"init-height-test", "bn254", fmt.Sprintf("--%s=%d", flags.FlagInitHeight, testInitialHeight)})

	require.NoError(t, cmd.ExecuteContext(ctx))

	appGenesis, importErr := tmtypes.GenesisDocFromFile(cfg.GenesisFile())
	require.NoError(t, importErr)

	require.Equal(t, int64(1), appGenesis.InitialHeight)
}

// custom tx codec
func makeCodec() *codec.LegacyAmino {
	cdc := codec.NewLegacyAmino()
	sdk.RegisterLegacyAminoCodec(cdc)
	cryptocodec.RegisterCrypto(cdc)
	return cdc
}
