package cmd

import (
	"context"
	"strings"

	"github.com/rs/zerolog"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/server"
)

// Execute executes the root command of an application. It handles creating a
// server context object with the appropriate server and client objects injected
// into the underlying stdlib Context. It also handles adding core CLI flags,
// specifically the logging flags. It returns an error upon execution failure.
func Execute(rootCmd *cobra.Command, envPrefix, defaultHome string) error {
	// Create and set a client.Context on the command's Context. During the pre-run
	// of the root command, a default initialized client.Context is provided to
	// seed child command execution with values such as AccountRetriever, Keyring,
	// and a CometBFT RPC. This requires the use of a pointer reference when
	// getting and setting the client.Context. Ideally, we utilize
	// https://github.com/spf13/cobra/pull/1118.
	ctx := CreateExecuteContext(context.Background())

	rootCmd.PersistentFlags().String(flags.FlagLogLevel, zerolog.InfoLevel.String(), "The logging level (trace|debug|info|warn|error|fatal|panic|disabled or '*:<level>,<key>:<level>')")
	rootCmd.PersistentFlags().String(flags.FlagLogFormat, "plain", "The logging format (json|plain)")
	rootCmd.PersistentFlags().Bool(flags.FlagLogNoColor, false, "Disable colored logs")
	rootCmd.PersistentFlags().StringP(flags.FlagHome, "", defaultHome, "directory for config and data")
	rootCmd.PersistentFlags().Bool(server.FlagTrace, false, "print out full stack trace on errors")

	// update the global viper with the root command's configuration
	viper.SetEnvPrefix(envPrefix)
	viper.SetEnvKeyReplacer(strings.NewReplacer(".", "_", "-", "_"))
	viper.AutomaticEnv()

	return rootCmd.ExecuteContext(ctx)
}

// CreateExecuteContext returns a base Context with server and client context
// values initialized.
func CreateExecuteContext(ctx context.Context) context.Context {
	srvCtx := server.NewDefaultContext()
	ctx = context.WithValue(ctx, client.ClientContextKey, &client.Context{})
	ctx = context.WithValue(ctx, server.ServerContextKey, srvCtx)

	return ctx
}
