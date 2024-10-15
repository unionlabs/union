package cmd

import (
	"context"
	"errors"
	"fmt"
	"maps"
	"path/filepath"
	"slices"
	"strings"

	"github.com/spf13/cobra"

	"cosmossdk.io/tools/confix"

	"github.com/cosmos/cosmos-sdk/client"
)

var (
	FlagStdOut       bool
	FlagVerbose      bool
	FlagSkipValidate bool
)

func MigrateCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "migrate [target-version] <config-path>",
		Short: "Migrate Cosmos SDK configuration file to the specified version",
		Long: `Migrate the contents of the Cosmos SDK configuration (app.toml or client.toml) to the specified version. Configuration type is app by default.
The output is written in-place unless --stdout is provided.
In case of any error in updating the file, no output is written.`,
		Args: cobra.MinimumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var configPath string
			clientCtx := client.GetClientContextFromCmd(cmd)

			configType := confix.AppConfigType
			isClient, _ := cmd.Flags().GetBool(confix.ClientConfigType)

			if isClient {
				configType = confix.ClientConfigType
			}

			switch {
			case len(args) > 1:
				configPath = args[1]
			case clientCtx.HomeDir != "":
				suffix := "app.toml"
				if isClient {
					suffix = "client.toml"
				}
				configPath = filepath.Join(clientCtx.HomeDir, "config", suffix)
			default:
				return errors.New("must provide a path to the app.toml or client.toml")
			}

			if strings.HasSuffix(configPath, "client.toml") && !isClient {
				return errors.New("app.toml file expected, got client.toml, use --client flag to migrate client.toml")
			}

			targetVersion := args[0]
			plan, ok := confix.Migrations[targetVersion]
			if !ok {
				return fmt.Errorf("unknown version %q, supported versions are: %q", targetVersion, slices.Collect(maps.Keys(confix.Migrations)))
			}

			rawFile, err := confix.LoadConfig(configPath)
			if err != nil {
				return fmt.Errorf("failed to load config: %w", err)
			}

			ctx := context.Background()
			if FlagVerbose {
				ctx = confix.WithLogWriter(ctx, cmd.ErrOrStderr())
			}

			outputPath := configPath
			if FlagStdOut {
				outputPath = ""
			}

			// get transformation steps and formatDoc in which plan need to be applied
			steps, formatDoc := plan(rawFile, targetVersion, configType)

			if err := confix.Upgrade(ctx, steps, formatDoc, configPath, outputPath, FlagSkipValidate); err != nil {
				return fmt.Errorf("failed to migrate config: %w", err)
			}

			return nil
		},
	}

	cmd.Flags().BoolVar(&FlagStdOut, "stdout", false, "print the updated config to stdout")
	cmd.Flags().BoolVar(&FlagVerbose, "verbose", false, "log changes to stderr")
	cmd.Flags().BoolVar(&FlagSkipValidate, "skip-validate", false, "skip configuration validation (allows to migrate unknown configurations)")
	cmd.Flags().Bool(confix.ClientConfigType, false, "migrate client.toml instead of app.toml")

	return cmd
}
