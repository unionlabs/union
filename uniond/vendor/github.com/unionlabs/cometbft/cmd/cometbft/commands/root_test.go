package commands

import (
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"testing"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	cfg "github.com/cometbft/cometbft/config"
	cmtos "github.com/cometbft/cometbft/internal/os"
	"github.com/cometbft/cometbft/libs/cli"
)

// clearConfig clears env vars, the given root dir, and resets viper.
func clearConfig(t *testing.T, dir string) {
	t.Helper()
	os.Clearenv()
	err := os.RemoveAll(dir)
	require.NoError(t, err)

	viper.Reset()
	config = cfg.DefaultConfig()
}

// prepare new rootCmd.
func testRootCmd() *cobra.Command {
	rootCmd := &cobra.Command{
		Use:               RootCmd.Use,
		PersistentPreRunE: RootCmd.PersistentPreRunE,
		Run:               func(_ *cobra.Command, _ []string) {},
	}
	registerFlagsRootCmd(rootCmd)
	var l string
	rootCmd.PersistentFlags().String("log", l, "Log")
	return rootCmd
}

func testSetup(t *testing.T, root string, args []string, env map[string]string) error {
	t.Helper()
	clearConfig(t, root)

	rootCmd := testRootCmd()
	cmd := cli.PrepareBaseCmd(rootCmd, "CMT", root)

	// run with the args and env
	args = append([]string{rootCmd.Use}, args...)
	return cli.RunWithArgs(cmd, args, env)
}

func TestRootHome(t *testing.T) {
	tmpDir := os.TempDir()
	root := filepath.Join(tmpDir, "adir")
	newRoot := filepath.Join(tmpDir, "something-else")
	defer clearConfig(t, root)
	defer clearConfig(t, newRoot)

	cases := []struct {
		args []string
		env  map[string]string
		root string
	}{
		{nil, nil, root},
		{[]string{"--home", newRoot}, nil, newRoot},
		{nil, map[string]string{"TMHOME": newRoot}, newRoot}, // XXX: Deprecated.
		{nil, map[string]string{"CMTHOME": newRoot}, newRoot},
	}

	for i, tc := range cases {
		idxString := "idx: " + strconv.Itoa(i)

		err := testSetup(t, root, tc.args, tc.env)
		require.NoError(t, err, idxString)

		assert.Equal(t, tc.root, config.RootDir, idxString)
		assert.Equal(t, tc.root, config.P2P.RootDir, idxString)
		assert.Equal(t, tc.root, config.Consensus.RootDir, idxString)
		assert.Equal(t, tc.root, config.Mempool.RootDir, idxString)
	}
}

func TestRootFlagsEnv(t *testing.T) {
	// defaults
	defaults := cfg.DefaultConfig()
	defaultLogLvl := defaults.LogLevel

	cases := []struct {
		args     []string
		env      map[string]string
		logLevel string
	}{
		{[]string{"--log", "debug"}, nil, defaultLogLvl},                  // wrong flag
		{[]string{"--log_level", "debug"}, nil, "debug"},                  // right flag
		{nil, map[string]string{"TM_LOW": "debug"}, defaultLogLvl},        // wrong env flag
		{nil, map[string]string{"MT_LOG_LEVEL": "debug"}, defaultLogLvl},  // wrong env prefix
		{nil, map[string]string{"TM_LOG_LEVEL": "debug"}, defaultLogLvl},  // right, but deprecated env
		{nil, map[string]string{"CMT_LOW": "debug"}, defaultLogLvl},       // wrong env flag
		{nil, map[string]string{"TMC_LOG_LEVEL": "debug"}, defaultLogLvl}, // wrong env prefix
		{nil, map[string]string{"CMT_LOG_LEVEL": "debug"}, "debug"},       // right env
	}

	for i, tc := range cases {
		idxString := strconv.Itoa(i)
		root := filepath.Join(os.TempDir(), "adir2_"+idxString)
		idxString = "idx: " + idxString
		defer clearConfig(t, root)
		err := testSetup(t, root, tc.args, tc.env)
		require.NoError(t, err, idxString)

		assert.Equal(t, tc.logLevel, config.LogLevel, idxString)
	}
}

func TestRootConfig(t *testing.T) {
	// write non-default config
	nonDefaultLogLvl := "abc:debug"
	cvals := map[string]string{
		"log_level": nonDefaultLogLvl,
	}

	cases := []struct {
		args []string
		env  map[string]string

		logLvl string
	}{
		{nil, nil, nonDefaultLogLvl},                                           // should load config
		{[]string{"--log_level=abc:info"}, nil, "abc:info"},                    // flag over rides
		{nil, map[string]string{"TM_LOG_LEVEL": "abc:info"}, nonDefaultLogLvl}, // env over rides //XXX: Deprecated
		{nil, map[string]string{"CMT_LOG_LEVEL": "abc:info"}, "abc:info"},      // env over rides
	}

	for i, tc := range cases {
		idxString := strconv.Itoa(i)
		root := filepath.Join(os.TempDir(), "adir3_"+idxString)
		idxString = "idx: " + idxString
		defer clearConfig(t, root)
		// XXX: path must match cfg.defaultConfigPath
		configFilePath := filepath.Join(root, "config")
		err := cmtos.EnsureDir(configFilePath, 0o700)
		require.NoError(t, err)

		// write the non-defaults to a different path
		// TODO: support writing sub configs so we can test that too
		err = WriteConfigVals(configFilePath, cvals)
		require.NoError(t, err)

		rootCmd := testRootCmd()
		cmd := cli.PrepareBaseCmd(rootCmd, "CMT", root)

		// run with the args and env
		tc.args = append([]string{rootCmd.Use}, tc.args...)
		err = cli.RunWithArgs(cmd, tc.args, tc.env)
		require.NoError(t, err, idxString)

		assert.Equal(t, tc.logLvl, config.LogLevel, idxString)
	}
}

// WriteConfigVals writes a toml file with the given values.
// It returns an error if writing was impossible.
func WriteConfigVals(dir string, vals map[string]string) error {
	data := ""
	for k, v := range vals {
		data += fmt.Sprintf("%s = \"%s\"\n", k, v)
	}
	cfile := filepath.Join(dir, "config.toml")
	return os.WriteFile(cfile, []byte(data), 0o600)
}
