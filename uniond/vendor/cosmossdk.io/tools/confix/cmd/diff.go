package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
	"golang.org/x/exp/maps"

	"cosmossdk.io/tools/confix"

	"github.com/cosmos/cosmos-sdk/client"
)

func DiffCommand() *cobra.Command {
	return &cobra.Command{
		Use:   "diff [target-version] <app-toml-path>",
		Short: "Outputs all config values that are different from the app.toml defaults.",
		Args:  cobra.MinimumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var filename string
			clientCtx := client.GetClientContextFromCmd(cmd)
			switch {

			case len(args) > 1:
				filename = args[1]
			case clientCtx.HomeDir != "":
				filename = fmt.Sprintf("%s/config/app.toml", clientCtx.HomeDir)
			default:
				return fmt.Errorf("must provide a path to the app.toml file")
			}

			targetVersion := args[0]
			if _, ok := confix.Migrations[targetVersion]; !ok {
				return fmt.Errorf("unknown version %q, supported versions are: %q", targetVersion, maps.Keys(confix.Migrations))
			}

			targetVersionFile, err := confix.LoadLocalConfig(targetVersion)
			if err != nil {
				panic(fmt.Errorf("failed to load internal config: %w", err))
			}

			rawFile, err := confix.LoadConfig(filename)
			if err != nil {
				return fmt.Errorf("failed to load config: %v", err)
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
}
