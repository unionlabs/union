package server_test

import (
	"context"
	"errors"
	"fmt"
	"os"
	"path"
	"path/filepath"
	"strings"
	"testing"

	cmtcfg "github.com/cometbft/cometbft/config"
	db "github.com/cosmos/cosmos-db"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/server"
	"github.com/cosmos/cosmos-sdk/server/config"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/types/module/testutil"
	genutilcli "github.com/cosmos/cosmos-sdk/x/genutil/client/cli"
)

var errCanceledInPreRun = errors.New("canceled in prerun")

// Used in each test to run the function under test via Cobra
// but to always halt the command
func preRunETestImpl(cmd *cobra.Command, args []string) error {
	if err := server.InterceptConfigsPreRunHandler(cmd, "", nil, cmtcfg.DefaultConfig()); err != nil {
		return err
	}

	return errCanceledInPreRun
}

func TestGetAppDBBackend(t *testing.T) {
	v := viper.New()
	require.Equal(t, server.GetAppDBBackend(v), db.GoLevelDBBackend)
	v.Set("db_backend", "dbtype1") // value from CometBFT config
	require.Equal(t, server.GetAppDBBackend(v), db.BackendType("dbtype1"))
	v.Set("app-db-backend", "dbtype2") // value from app.toml
	require.Equal(t, server.GetAppDBBackend(v), db.BackendType("dbtype2"))
}

func TestInterceptConfigsPreRunHandlerCreatesConfigFilesWhenMissing(t *testing.T) {
	tempDir := t.TempDir()
	cmd := server.StartCmd[servertypes.Application](nil)
	cmd.PersistentFlags().String(flags.FlagHome, "/foobar", "")
	if err := cmd.PersistentFlags().Set(flags.FlagHome, tempDir); err != nil {
		t.Fatalf("Could not set home flag [%T] %v", err, err)
	}

	cmd.PreRunE = preRunETestImpl

	serverCtx := &server.Context{}
	ctx := context.WithValue(context.Background(), server.ServerContextKey, serverCtx)
	if err := cmd.ExecuteContext(ctx); !errors.Is(err, errCanceledInPreRun) {
		t.Fatalf("function failed with [%T] %v", err, err)
	}

	serverCtx = server.GetServerContextFromCmd(cmd)

	// Test that config.toml is created
	configTomlPath := path.Join(tempDir, "config", "config.toml")
	s, err := os.Stat(configTomlPath)
	if err != nil {
		t.Fatalf("Could not stat config.toml after run %v", err)
	}

	if !s.Mode().IsRegular() {
		t.Fatal("config.toml not created as regular file")
	}

	if s.Size() == 0 {
		t.Fatal("config.toml created as empty file")
	}

	// Test that CometBFT config is initialized
	if serverCtx.Config == nil {
		t.Fatal("CometBFT config not created")
	}

	// Test that app.toml is created
	appTomlPath := path.Join(tempDir, "config", "app.toml")
	s, err = os.Stat(appTomlPath)
	if err != nil {
		t.Fatalf("Could not stat app.toml after run %v", err)
	}

	if !s.Mode().IsRegular() {
		t.Fatal("appp.toml not created as regular file")
	}

	if s.Size() == 0 {
		t.Fatal("config.toml created as empty file")
	}

	// Test that the config for use in server/start.go is created
	if serverCtx.Viper == nil {
		t.Error("app config Viper instance not created")
	}
}

func TestInterceptConfigsPreRunHandlerReadsConfigToml(t *testing.T) {
	const testDbBackend = "awesome_test_db"
	tempDir := t.TempDir()
	err := os.Mkdir(path.Join(tempDir, "config"), os.ModePerm)
	if err != nil {
		t.Fatalf("creating config dir failed: %v", err)
	}
	configTomlPath := path.Join(tempDir, "config", "config.toml")
	writer, err := os.Create(configTomlPath)
	if err != nil {
		t.Fatalf("creating config.toml file failed: %v", err)
	}

	_, err = fmt.Fprintf(writer, "db_backend = '%s'\n", testDbBackend)
	if err != nil {
		t.Fatalf("Failed writing string to config.toml: %v", err)
	}

	if err := writer.Close(); err != nil {
		t.Fatalf("Failed closing config.toml: %v", err)
	}

	cmd := server.StartCmd[servertypes.Application](nil)
	cmd.PersistentFlags().String(flags.FlagHome, "/foobar", "")
	if err := cmd.PersistentFlags().Set(flags.FlagHome, tempDir); err != nil {
		t.Fatalf("Could not set home flag [%T] %v", err, err)
	}

	cmd.PreRunE = preRunETestImpl

	serverCtx := &server.Context{}
	ctx := context.WithValue(context.Background(), server.ServerContextKey, serverCtx)

	if err := cmd.ExecuteContext(ctx); !errors.Is(err, errCanceledInPreRun) {
		t.Fatalf("function failed with [%T] %v", err, err)
	}

	serverCtx = server.GetServerContextFromCmd(cmd)

	if testDbBackend != serverCtx.Config.DBBackend {
		t.Error("backend was not set from config.toml")
	}
}

func TestInterceptConfigsPreRunHandlerReadsAppToml(t *testing.T) {
	const testHaltTime = 1337
	tempDir := t.TempDir()
	err := os.Mkdir(path.Join(tempDir, "config"), os.ModePerm)
	if err != nil {
		t.Fatalf("creating config dir failed: %v", err)
	}
	appTomlPath := path.Join(tempDir, "config", "app.toml")
	writer, err := os.Create(appTomlPath)
	if err != nil {
		t.Fatalf("creating app.toml file failed: %v", err)
	}

	_, err = fmt.Fprintf(writer, "halt-time = %d\n", testHaltTime)
	if err != nil {
		t.Fatalf("Failed writing string to app.toml: %v", err)
	}

	if err := writer.Close(); err != nil {
		t.Fatalf("Failed closing app.toml: %v", err)
	}
	cmd := server.StartCmd[servertypes.Application](nil)
	cmd.PersistentFlags().String(flags.FlagHome, tempDir, "")

	cmd.PreRunE = preRunETestImpl

	serverCtx := &server.Context{}
	ctx := context.WithValue(context.Background(), server.ServerContextKey, serverCtx)

	if err := cmd.ExecuteContext(ctx); !errors.Is(err, errCanceledInPreRun) {
		t.Fatalf("function failed with [%T] %v", err, err)
	}

	serverCtx = server.GetServerContextFromCmd(cmd)

	if testHaltTime != serverCtx.Viper.GetInt("halt-time") {
		t.Error("Halt time was not set from app.toml")
	}
}

func TestInterceptConfigsPreRunHandlerReadsFlags(t *testing.T) {
	const testAddr = "tcp://127.1.2.3:12345"
	tempDir := t.TempDir()
	cmd := server.StartCmd[servertypes.Application](nil)
	cmd.PersistentFlags().String(flags.FlagHome, "/foobar", "")
	if err := cmd.PersistentFlags().Set(flags.FlagHome, tempDir); err != nil {
		t.Fatalf("Could not set home flag [%T] %v", err, err)
	}

	// This flag is added by tendermint
	if err := cmd.Flags().Set("rpc.laddr", testAddr); err != nil {
		t.Fatalf("Could not set address flag [%T] %v", err, err)
	}

	cmd.PreRunE = preRunETestImpl

	serverCtx := &server.Context{}
	ctx := context.WithValue(context.Background(), server.ServerContextKey, serverCtx)

	if err := cmd.ExecuteContext(ctx); !errors.Is(err, errCanceledInPreRun) {
		t.Fatalf("function failed with [%T] %v", err, err)
	}

	serverCtx = server.GetServerContextFromCmd(cmd)

	if testAddr != serverCtx.Config.RPC.ListenAddress {
		t.Error("RPCListenAddress was not set from command flags")
	}
}

func TestInterceptConfigsPreRunHandlerReadsEnvVars(t *testing.T) {
	const testAddr = "tcp://127.1.2.3:12345"
	tempDir := t.TempDir()
	cmd := server.StartCmd[servertypes.Application](nil)
	cmd.PersistentFlags().String(flags.FlagHome, "/foobar", "")
	if err := cmd.PersistentFlags().Set(flags.FlagHome, tempDir); err != nil {
		t.Fatalf("Could not set home flag [%T] %v", err, err)
	}

	executableName, err := os.Executable()
	if err != nil {
		t.Fatalf("Could not get executable name: %v", err)
	}
	basename := path.Base(executableName)
	basename = strings.ReplaceAll(basename, ".", "_")
	// This is added by tendermint
	envVarName := fmt.Sprintf("%s_RPC_LADDR", strings.ToUpper(basename))
	require.NoError(t, os.Setenv(envVarName, testAddr))
	t.Cleanup(func() {
		require.NoError(t, os.Unsetenv(envVarName))
	})

	cmd.PreRunE = preRunETestImpl

	serverCtx := &server.Context{}
	ctx := context.WithValue(context.Background(), server.ServerContextKey, serverCtx)

	if err := cmd.ExecuteContext(ctx); !errors.Is(err, errCanceledInPreRun) {
		t.Fatalf("function failed with [%T] %v", err, err)
	}

	serverCtx = server.GetServerContextFromCmd(cmd)

	if testAddr != serverCtx.Config.RPC.ListenAddress {
		t.Errorf("RPCListenAddress was not set from env. var. %q", envVarName)
	}
}

/*
 The following tests are here to check the precedence of each
 of the configuration sources. A common setup functionality is used
 to avoid duplication of code between tests.
*/

var (
	TestAddrExpected    = "tcp://127.126.125.124:12345" // expected to be used in test
	TestAddrNotExpected = "tcp://127.127.127.127:11111" // not expected to be used in test
)

type precedenceCommon struct {
	envVarName     string
	flagName       string
	configTomlPath string

	cmd *cobra.Command
}

func newPrecedenceCommon(t *testing.T) precedenceCommon {
	t.Helper()
	retval := precedenceCommon{}

	// Determine the env. var. name based off the executable name
	executableName, err := os.Executable()
	if err != nil {
		t.Fatalf("Could not get executable name: %v", err)
	}
	basename := path.Base(executableName)
	basename = strings.ReplaceAll(basename, ".", "_")
	basename = strings.ReplaceAll(basename, "-", "_")
	// Store the name of the env. var.
	retval.envVarName = fmt.Sprintf("%s_RPC_LADDR", strings.ToUpper(basename))

	// Store the flag name. This flag is added by tendermint
	retval.flagName = "rpc.laddr"

	// Create a tempdir and create './config' under that
	tempDir := t.TempDir()
	err = os.Mkdir(path.Join(tempDir, "config"), os.ModePerm)
	if err != nil {
		t.Fatalf("creating config dir failed: %v", err)
	}
	// Store the path for config.toml
	retval.configTomlPath = path.Join(tempDir, "config", "config.toml")

	// always remove the env. var. after each test execution
	t.Cleanup(func() {
		// This should not fail but if it does just panic
		if err := os.Unsetenv(retval.envVarName); err != nil {
			panic("Could not clear configuration env. var. used in test")
		}
	})

	// Set up the command object that is used in this test
	retval.cmd = server.StartCmd[servertypes.Application](nil)
	retval.cmd.PersistentFlags().String(flags.FlagHome, tempDir, "")
	retval.cmd.PreRunE = preRunETestImpl

	return retval
}

func (v precedenceCommon) setAll(t *testing.T, setFlag, setEnvVar, setConfigFile *string) {
	t.Helper()
	if setFlag != nil {
		if err := v.cmd.Flags().Set(v.flagName, *setFlag); err != nil {
			t.Fatalf("Failed setting flag %q", v.flagName)
		}
	}

	if setEnvVar != nil {
		require.NoError(t, os.Setenv(v.envVarName, *setEnvVar))
	}

	if setConfigFile != nil {
		writer, err := os.Create(v.configTomlPath)
		if err != nil {
			t.Fatalf("creating config.toml file failed: %v", err)
		}

		_, err = fmt.Fprintf(writer, "[rpc]\nladdr = \"%s\"\n", *setConfigFile)
		if err != nil {
			t.Fatalf("Failed writing string to config.toml: %v", err)
		}

		if err := writer.Close(); err != nil {
			t.Fatalf("Failed closing config.toml: %v", err)
		}
	}
}

func TestInterceptConfigsPreRunHandlerPrecedenceFlag(t *testing.T) {
	testCommon := newPrecedenceCommon(t)
	testCommon.setAll(t, &TestAddrExpected, &TestAddrNotExpected, &TestAddrNotExpected)

	serverCtx := &server.Context{}
	ctx := context.WithValue(context.Background(), server.ServerContextKey, serverCtx)

	if err := testCommon.cmd.ExecuteContext(ctx); !errors.Is(err, errCanceledInPreRun) {
		t.Fatalf("function failed with [%T] %v", err, err)
	}

	serverCtx = server.GetServerContextFromCmd(testCommon.cmd)

	if TestAddrExpected != serverCtx.Config.RPC.ListenAddress {
		t.Fatalf("RPCListenAddress was not set from flag %q", testCommon.flagName)
	}
}

func TestInterceptConfigsPreRunHandlerPrecedenceEnvVar(t *testing.T) {
	testCommon := newPrecedenceCommon(t)
	testCommon.setAll(t, nil, &TestAddrExpected, &TestAddrNotExpected)

	serverCtx := &server.Context{}
	ctx := context.WithValue(context.Background(), server.ServerContextKey, serverCtx)

	if err := testCommon.cmd.ExecuteContext(ctx); !errors.Is(err, errCanceledInPreRun) {
		t.Fatalf("function failed with [%T] %v", err, err)
	}

	serverCtx = server.GetServerContextFromCmd(testCommon.cmd)

	if TestAddrExpected != serverCtx.Config.RPC.ListenAddress {
		t.Errorf("RPCListenAddress was not set from env. var. %q", testCommon.envVarName)
	}
}

func TestInterceptConfigsPreRunHandlerPrecedenceConfigFile(t *testing.T) {
	testCommon := newPrecedenceCommon(t)
	testCommon.setAll(t, nil, nil, &TestAddrExpected)

	serverCtx := &server.Context{}
	ctx := context.WithValue(context.Background(), server.ServerContextKey, serverCtx)

	if err := testCommon.cmd.ExecuteContext(ctx); !errors.Is(err, errCanceledInPreRun) {
		t.Fatalf("function failed with [%T] %v", err, err)
	}

	serverCtx = server.GetServerContextFromCmd(testCommon.cmd)

	if TestAddrExpected != serverCtx.Config.RPC.ListenAddress {
		t.Errorf("RPCListenAddress was not read from file %q", testCommon.configTomlPath)
	}
}

func TestInterceptConfigsPreRunHandlerPrecedenceConfigDefault(t *testing.T) {
	testCommon := newPrecedenceCommon(t)
	// Do not set anything

	serverCtx := &server.Context{}
	ctx := context.WithValue(context.Background(), server.ServerContextKey, serverCtx)

	if err := testCommon.cmd.ExecuteContext(ctx); !errors.Is(err, errCanceledInPreRun) {
		t.Fatalf("function failed with [%T] %v", err, err)
	}

	serverCtx = server.GetServerContextFromCmd(testCommon.cmd)

	if serverCtx.Config.RPC.ListenAddress != "tcp://127.0.0.1:26657" {
		t.Error("RPCListenAddress is not using default")
	}
}

// Ensure that if interceptConfigs encounters any error other than non-existen errors
// that we correctly return the offending error, for example a permission error.
// See https://github.com/cosmos/cosmos-sdk/issues/7578
func TestInterceptConfigsWithBadPermissions(t *testing.T) {
	tempDir := t.TempDir()
	subDir := filepath.Join(tempDir, "nonPerms")
	if err := os.Mkdir(subDir, 0o600); err != nil {
		t.Fatalf("Failed to create sub directory: %v", err)
	}
	cmd := server.StartCmd[servertypes.Application](nil)
	cmd.PersistentFlags().String(flags.FlagHome, "/foobar", "")
	if err := cmd.PersistentFlags().Set(flags.FlagHome, subDir); err != nil {
		t.Fatalf("Could not set home flag [%T] %v", err, err)
	}

	cmd.PreRunE = preRunETestImpl

	serverCtx := &server.Context{}
	ctx := context.WithValue(context.Background(), server.ServerContextKey, serverCtx)
	if err := cmd.ExecuteContext(ctx); !os.IsPermission(err) {
		t.Fatalf("Failed to catch permissions error, got: [%T] %v", err, err)
	}
}

func TestEmptyMinGasPrices(t *testing.T) {
	tempDir := t.TempDir()
	err := os.Mkdir(filepath.Join(tempDir, "config"), os.ModePerm)
	require.NoError(t, err)
	encCfg := testutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})

	// Run InitCmd to create necessary config files.
	clientCtx := client.Context{}.WithHomeDir(tempDir).WithCodec(encCfg.Codec)
	serverCtx := server.NewDefaultContext()
	serverCtx.Config.SetRoot(tempDir)
	ctx := context.WithValue(context.Background(), server.ServerContextKey, serverCtx)
	ctx = context.WithValue(ctx, client.ClientContextKey, &clientCtx)
	cmd := genutilcli.InitCmd(module.NewManager())
	cmd.SetArgs([]string{"appnode-test"})
	err = cmd.ExecuteContext(ctx)
	require.NoError(t, err)

	// Modify app.toml.
	appCfgTempFilePath := filepath.Join(tempDir, "config", "app.toml")
	appConf := config.DefaultConfig()
	appConf.BaseConfig.MinGasPrices = ""
	err = config.WriteConfigFile(appCfgTempFilePath, appConf)
	require.NoError(t, err)

	// Run StartCmd.
	cmd = server.StartCmd[servertypes.Application](nil)
	cmd.PersistentFlags().String(flags.FlagHome, tempDir, "")
	cmd.PreRunE = func(cmd *cobra.Command, _ []string) error {
		ctx, err := server.InterceptConfigsAndCreateContext(cmd, "", nil, cmtcfg.DefaultConfig())
		if err != nil {
			return err
		}

		return server.SetCmdServerContext(cmd, ctx)
	}
	err = cmd.ExecuteContext(ctx)
	require.Errorf(t, err, sdkerrors.ErrAppConfig.Error())
}

type mapGetter map[string]interface{}

func (m mapGetter) Get(key string) interface{} {
	return m[key]
}

func (m mapGetter) GetString(key string) string {
	str, ok := m[key]
	if !ok {
		return ""
	}

	return str.(string)
}

var _ servertypes.AppOptions = mapGetter{}
