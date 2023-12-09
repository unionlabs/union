package cli

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

const (
	HomeFlag     = "home"
	TraceFlag    = "trace"
	OutputFlag   = "output"
	EncodingFlag = "encoding"
)

// Executable is the minimal interface to *corba.Command, so we can
// wrap if desired before the test
type Executable interface {
	Execute() error
}

// PrepareBaseCmd is meant for CometBFT and other servers
func PrepareBaseCmd(cmd *cobra.Command, envPrefix, defaultHome string) Executor {
	cobra.OnInitialize(func() { initEnv(envPrefix) })
	cmd.PersistentFlags().StringP(HomeFlag, "", defaultHome, "directory for config and data")
	cmd.PersistentFlags().Bool(TraceFlag, false, "print out full stack trace on errors")
	cmd.PersistentPreRunE = concatCobraCmdFuncs(bindFlagsLoadViper, cmd.PersistentPreRunE)
	return Executor{cmd, os.Exit}
}

// PrepareMainCmd is meant for client side libs that want some more flags
//
// This adds --encoding (hex, btc, base64) and --output (text, json) to
// the command.  These only really make sense in interactive commands.
func PrepareMainCmd(cmd *cobra.Command, envPrefix, defaultHome string) Executor {
	cmd.PersistentFlags().StringP(EncodingFlag, "e", "hex", "Binary encoding (hex|b64|btc)")
	cmd.PersistentFlags().StringP(OutputFlag, "o", "text", "Output format (text|json)")
	cmd.PersistentPreRunE = concatCobraCmdFuncs(validateOutput, cmd.PersistentPreRunE)
	return PrepareBaseCmd(cmd, envPrefix, defaultHome)
}

// initEnv sets to use ENV variables if set.
func initEnv(prefix string) {
	copyEnvVars(prefix)

	// env variables with TM prefix (eg. TM_ROOT)
	viper.SetEnvPrefix(prefix)
	viper.SetEnvKeyReplacer(strings.NewReplacer(".", "_", "-", "_"))
	viper.AutomaticEnv()
}

// This copies all variables like TMROOT to TM_ROOT,
// so we can support both formats for the user
func copyEnvVars(prefix string) {
	prefix = strings.ToUpper(prefix)
	ps := prefix + "_"
	for _, e := range os.Environ() {
		kv := strings.SplitN(e, "=", 2)
		if len(kv) == 2 {
			k, v := kv[0], kv[1]
			if strings.HasPrefix(k, prefix) && !strings.HasPrefix(k, ps) {
				k2 := strings.Replace(k, prefix, ps, 1)
				os.Setenv(k2, v)
			}
		}
	}
}

// Executor wraps the cobra Command with a nicer Execute method
type Executor struct {
	*cobra.Command
	Exit func(int) // this is os.Exit by default, override in tests
}

type ExitCoder interface {
	ExitCode() int
}

// execute adds all child commands to the root command sets flags appropriately.
// This is called by main.main(). It only needs to happen once to the rootCmd.
func (e Executor) Execute() error {
	e.SilenceUsage = true
	e.SilenceErrors = true
	err := e.Command.Execute()
	if err != nil {
		if viper.GetBool(TraceFlag) {
			const size = 64 << 10
			buf := make([]byte, size)
			buf = buf[:runtime.Stack(buf, false)]
			fmt.Fprintf(os.Stderr, "ERROR: %v\n%s\n", err, buf)
		} else {
			fmt.Fprintf(os.Stderr, "ERROR: %v\n", err)
		}

		// return error code 1 by default, can override it with a special error type
		exitCode := 1
		if ec, ok := err.(ExitCoder); ok {
			exitCode = ec.ExitCode()
		}
		e.Exit(exitCode)
	}
	return err
}

type cobraCmdFunc func(cmd *cobra.Command, args []string) error

// Returns a single function that calls each argument function in sequence
// RunE, PreRunE, PersistentPreRunE, etc. all have this same signature
func concatCobraCmdFuncs(fs ...cobraCmdFunc) cobraCmdFunc {
	return func(cmd *cobra.Command, args []string) error {
		for _, f := range fs {
			if f != nil {
				if err := f(cmd, args); err != nil {
					return err
				}
			}
		}
		return nil
	}
}

// Bind all flags and read the config into viper
func bindFlagsLoadViper(cmd *cobra.Command, _ []string) error {
	// cmd.Flags() includes flags from this command and all persistent flags from the parent
	if err := viper.BindPFlags(cmd.Flags()); err != nil {
		return err
	}

	homeDir := viper.GetString(HomeFlag)
	viper.Set(HomeFlag, homeDir)
	viper.SetConfigName("config")                         // name of config file (without extension)
	viper.AddConfigPath(homeDir)                          // search root directory
	viper.AddConfigPath(filepath.Join(homeDir, "config")) // search root directory /config

	// If a config file is found, read it in.
	err := viper.ReadInConfig()
	if _, ok := err.(viper.ConfigFileNotFoundError); !ok {
		// ignore not found error, return other errors
		return err
	}
	return nil
}

func validateOutput(_ *cobra.Command, _ []string) error {
	// validate output format
	output := viper.GetString(OutputFlag)
	switch output {
	case "text", "json":
	default:
		return fmt.Errorf("unsupported output format: %s", output)
	}
	return nil
}
