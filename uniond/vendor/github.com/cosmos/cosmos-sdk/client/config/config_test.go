package config_test

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"testing"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/config"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	"github.com/cosmos/cosmos-sdk/x/staking/client/cli"
	"github.com/stretchr/testify/require"
)

const (
	nodeEnv   = "NODE"
	testNode1 = "http://localhost:1"
	testNode2 = "http://localhost:2"
)

// initClientContext initiates client Context for tests
func initClientContext(t *testing.T, envVar string) (client.Context, func()) {
	home := t.TempDir()
	chainId := "test-chain"
	clientCtx := client.Context{}.
		WithHomeDir(home).
		WithViper("").
		WithCodec(codec.NewProtoCodec(codectypes.NewInterfaceRegistry())).
		WithChainID(chainId)

	require.NoError(t, clientCtx.Viper.BindEnv(nodeEnv))
	if envVar != "" {
		require.NoError(t, os.Setenv(nodeEnv, envVar))
	}

	clientCtx, err := config.ReadFromClientConfig(clientCtx)
	require.NoError(t, err)
	require.Equal(t, clientCtx.ChainID, chainId)

	return clientCtx, func() { _ = os.RemoveAll(home) }
}

func TestConfigCmd(t *testing.T) {
	clientCtx, cleanup := initClientContext(t, testNode1)
	defer func() {
		_ = os.Unsetenv(nodeEnv)
		cleanup()
	}()

	// NODE=http://localhost:1 ./build/simd config node http://localhost:2
	cmd := config.Cmd()
	args := []string{"node", testNode2}
	_, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, args)
	require.NoError(t, err)

	//./build/simd config node //http://localhost:1
	b := bytes.NewBufferString("")
	cmd.SetOut(b)
	cmd.SetArgs([]string{"node"})
	require.NoError(t, cmd.Execute())
	out, err := io.ReadAll(b)
	require.NoError(t, err)
	require.Equal(t, string(out), testNode1+"\n")
}

func TestConfigCmdEnvFlag(t *testing.T) {
	const (
		defaultNode = "http://localhost:26657"
	)

	tt := []struct {
		name    string
		envVar  string
		args    []string
		expNode string
	}{
		{"env var is set with no flag", testNode1, []string{"validators"}, testNode1},
		{"env var is set with a flag", testNode1, []string{"validators", fmt.Sprintf("--%s=%s", flags.FlagNode, testNode2)}, testNode2},
		{"env var is not set with no flag", "", []string{"validators"}, defaultNode},
		{"env var is not set with a flag", "", []string{"validators", fmt.Sprintf("--%s=%s", flags.FlagNode, testNode2)}, testNode2},
	}

	for _, tc := range tt {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			clientCtx, cleanup := initClientContext(t, tc.envVar)
			defer func() {
				if tc.envVar != "" {
					_ = os.Unsetenv(nodeEnv)
				}
				cleanup()
			}()
			/*
				env var is set with a flag

				NODE=http://localhost:1 ./build/simd q staking validators --node http://localhost:2
				Error: post failed: Post "http://localhost:2": dial tcp 127.0.0.1:2: connect: connection refused

				We dial http://localhost:2 cause a flag has the higher priority than env variable.
			*/
			cmd := cli.GetQueryCmd()
			_, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)
			require.Error(t, err)
			require.Contains(t, err.Error(), tc.expNode, "Output does not contain expected Node")
		})
	}
}
