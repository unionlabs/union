package cmd

import (
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

// DiffCommand creates a new command for comparing configuration files
func DiffCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "diff [target-version] <config-path>",
		Short: "Outputs all config values that are different from the default.",
		Long:  "This command compares the specified configuration file (app.toml or client.toml) with the defaults and outputs any differences.",
		Args:  cobra.MinimumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var configPath string
			clientCtx := client.GetClientContextFromCmd(cmd)
			switch {
			case len(args) > 1:
				configPath = args[1]
			case clientCtx.HomeDir != "":
				configPath = filepath.Join(clientCtx.HomeDir, "config", "app.toml")
			default:
				return errors.New("must provide a path to the app.toml or client.toml")
			}

			configType := confix.AppConfigType
			if ok, _ := cmd.Flags().GetBool(confix.ClientConfigType); ok {
				configPath = strings.ReplaceAll(configPath, "app.toml", "client.toml") // for the case we are using the home dir of client ctx
				configType = confix.ClientConfigType
			} else if strings.HasSuffix(configPath, "client.toml") {
				return errors.New("app.toml file expected, got client.toml, use --client flag to diff client.toml")
			}

			targetVersion := args[0]
			if _, ok := confix.Migrations[targetVersion]; !ok {
				return fmt.Errorf("unknown version %q, supported versions are: %q", targetVersion, slices.Collect(maps.Keys(confix.Migrations)))
			}

			targetVersionFile, err := confix.LoadLocalConfig(targetVersion, configType)
			if err != nil {
				return fmt.Errorf("failed to load internal config: %w", err)
			}

			rawFile, err := confix.LoadConfig(configPath)
			if err != nil {
				return fmt.Errorf("failed to load config: %w", err)
			}

			diff := confix.DiffValues(rawFile, targetVersionFile)
			if len(diff) == 0 {
				return clientCtx.PrintString("All config values are the same as the defaults.\n")
			}

			if err := clientCtx.PrintString("The following config values are different from the defaults:\n"); err != nil {
				return err
			}

			confix.PrintDiff(cmd.OutOrStdout(), diff)
			return nil
		},
	}

	cmd.Flags().Bool(confix.ClientConfigType, false, "diff client.toml instead of app.toml")

	return cmd
}
