package config_test

import (
	"fmt"
	"os"
	"testing"

	"github.com/spf13/cobra"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/config"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
)

const (
	chainID   = "test-chain"
	nodeEnv   = "CONFIG_TEST_NODE"
	testNode1 = "http://localhost:1"
	testNode2 = "http://localhost:2"
)

// initClientContext initiates client.Context for tests
func initClientContext(t *testing.T, envVar string) (client.Context, func()) {
	t.Helper()

	clientCtx, cleanup, err := initClientContextWithTemplate(t, envVar, "", nil)
	require.NoError(t, err)
	require.Equal(t, chainID, clientCtx.ChainID)

	return clientCtx, cleanup
}

// initClientContextWithTemplate initiates client.Context with custom config and template for tests
func initClientContextWithTemplate(t *testing.T, envVar, customTemplate string, customConfig interface{}) (client.Context, func(), error) {
	t.Helper()
	home := t.TempDir()
	clientCtx := client.Context{}.
		WithHomeDir(home).
		WithViper("").
		WithCodec(codec.NewProtoCodec(codectypes.NewInterfaceRegistry())).
		WithChainID(chainID)

	if envVar != "" {
		require.NoError(t, os.Setenv(nodeEnv, envVar))
	}

	clientCtx, err := config.CreateClientConfig(clientCtx, customTemplate, customConfig)
	return clientCtx, func() {
		_ = os.RemoveAll(home)
		_ = os.Unsetenv(nodeEnv)
	}, err
}

func TestCustomTemplateAndConfig(t *testing.T) {
	type GasConfig struct {
		GasAdjustment float64 `mapstructure:"gas-adjustment"`
	}

	type CustomClientConfig struct {
		config.Config `mapstructure:",squash"`

		GasConfig GasConfig `mapstructure:"gas"`

		Note string `mapstructure:"note"`
	}

	clientCfg := config.DefaultConfig()
	// Overwrite the default keyring backend.
	clientCfg.KeyringBackend = "test"

	customClientConfig := CustomClientConfig{
		Config: *clientCfg,
		GasConfig: GasConfig{
			GasAdjustment: 1.5,
		},
		Note: "Sent from the CLI.",
	}

	customClientConfigTemplate := config.DefaultClientConfigTemplate + `
# This is the gas adjustment factor used by the tx commands.
# Sets the default and can be overwritten by the --gas-adjustment flag in tx commands.
gas-adjustment = {{ .GasConfig.GasAdjustment }}
# Memo to include in all transactions.
note = "{{ .Note }}"
`
	t.Run("custom template and config provided", func(t *testing.T) {
		clientCtx, cleanup, err := initClientContextWithTemplate(t, "", customClientConfigTemplate, customClientConfig)
		defer func() {
			cleanup()
		}()

		require.NoError(t, err)
		require.Equal(t, customClientConfig.KeyringBackend, clientCtx.Viper.Get(flags.FlagKeyringBackend))
		require.Equal(t, customClientConfig.GasConfig.GasAdjustment, clientCtx.Viper.GetFloat64(flags.FlagGasAdjustment))
		require.Equal(t, customClientConfig.Note, clientCtx.Viper.GetString(flags.FlagNote))
	})

	t.Run("no template and custom config provided", func(t *testing.T) {
		_, cleanup, err := initClientContextWithTemplate(t, "", "", customClientConfig)
		defer func() {
			cleanup()
		}()

		require.Error(t, err)
	})

	t.Run("default template and custom config provided", func(t *testing.T) {
		clientCtx, cleanup, err := initClientContextWithTemplate(t, "", config.DefaultClientConfigTemplate, customClientConfig)
		defer func() {
			cleanup()
		}()

		require.NoError(t, err)
		require.Equal(t, customClientConfig.KeyringBackend, clientCtx.Viper.Get(flags.FlagKeyringBackend))
		require.Nil(t, clientCtx.Viper.Get(flags.FlagGasAdjustment)) // nil because we do not read the flags
	})

	t.Run("no template and no config provided", func(t *testing.T) {
		clientCtx, cleanup, err := initClientContextWithTemplate(t, "", "", nil)
		defer func() {
			cleanup()
		}()

		require.NoError(t, err)
		require.Equal(t, config.DefaultConfig().KeyringBackend, clientCtx.Viper.Get(flags.FlagKeyringBackend))
		require.Nil(t, clientCtx.Viper.Get(flags.FlagGasAdjustment)) // nil because we do not read the flags
	})
}

func TestConfigCmdEnvFlag(t *testing.T) {
	tt := []struct {
		name    string
		envVar  string
		args    []string
		expNode string
	}{
		{"env var is set with no flag", testNode1, []string{}, testNode1},
		{"env var is set with a flag", testNode1, []string{fmt.Sprintf("--%s=%s", flags.FlagNode, testNode2)}, testNode2},
		{"env var is not set with no flag", "", []string{}, "tcp://localhost:26657"},
		{"env var is not set with a flag", "", []string{fmt.Sprintf("--%s=%s", flags.FlagNode, testNode2)}, testNode2},
	}

	for _, tc := range tt {
		t.Run(tc.name, func(t *testing.T) {
			testCmd := &cobra.Command{
				Use: "test",
				RunE: func(cmd *cobra.Command, args []string) error {
					clientCtx, err := client.GetClientQueryContext(cmd)
					if err != nil {
						return err
					}

					return fmt.Errorf("%s", clientCtx.NodeURI)
				},
			}
			flags.AddQueryFlagsToCmd(testCmd)

			clientCtx, cleanup := initClientContext(t, tc.envVar)
			defer func() {
				cleanup()
			}()

			/*
				env var is set with a flag

				NODE=http://localhost:1 test-cmd --node http://localhost:2
				Prints "http://localhost:2"

				It prints http://localhost:2 cause a flag has the higher priority than env variable.
			*/

			_, err := clitestutil.ExecTestCLICmd(clientCtx, testCmd, tc.args)
			require.Error(t, err)
			require.Contains(t, err.Error(), tc.expNode)
		})
	}
}

func TestGRPCConfig(t *testing.T) {
	expectedGRPCConfig := config.GRPCConfig{
		Address:  "localhost:7070",
		Insecure: true,
	}

	clientCfg := config.DefaultConfig()
	clientCfg.GRPC = expectedGRPCConfig

	t.Run("custom template with gRPC config", func(t *testing.T) {
		clientCtx, cleanup, err := initClientContextWithTemplate(t, "", config.DefaultClientConfigTemplate, clientCfg)
		defer cleanup()

		require.NoError(t, err)

		require.Equal(t, expectedGRPCConfig.Address, clientCtx.Viper.GetString("grpc-address"))
		require.Equal(t, expectedGRPCConfig.Insecure, clientCtx.Viper.GetBool("grpc-insecure"))
	})
}
