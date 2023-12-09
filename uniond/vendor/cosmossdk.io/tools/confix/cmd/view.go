package cmd

import (
	"encoding/json"
	"fmt"
	"os"

	"github.com/pelletier/go-toml/v2"
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
)

func ViewCommand() *cobra.Command {
	flagOutputFomat := "output-format"

	cmd := &cobra.Command{
		Use:   "view [config]",
		Short: "View the config file",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			filename := args[0]

			clientCtx := client.GetClientContextFromCmd(cmd)
			if clientCtx.HomeDir != "" {
				filename = fmt.Sprintf("%s/config/%s.toml", clientCtx.HomeDir, filename)
			}

			file, err := os.ReadFile(filename)
			if err != nil {
				return err
			}

			if format, _ := cmd.Flags().GetString(flagOutputFomat); format == "toml" {
				cmd.Println(string(file))
				return nil
			}

			var v interface{}
			if err := toml.Unmarshal(file, &v); err != nil {
				return fmt.Errorf("failed to decode config file: %w", err)
			}

			e := json.NewEncoder(cmd.OutOrStdout())
			e.SetIndent("", "  ")
			return e.Encode(v)
		},
	}

	// output flag
	cmd.Flags().String(flagOutputFomat, "toml", "Output format (json|toml)")

	return cmd
}
