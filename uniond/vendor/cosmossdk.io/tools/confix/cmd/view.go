package cmd

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"github.com/pelletier/go-toml/v2"
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
)

func ViewCommand() *cobra.Command {
	flagOutputFormat := "output-format"

	cmd := &cobra.Command{
		Use:   "view <config>",
		Short: "View the config file",
		Long:  "View the config file. The [config] argument must be the path of the file when using the `confix` tool standalone, otherwise it must be the name of the config file without the .toml extension.",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			filename := args[0]

			clientCtx := client.GetClientContextFromCmd(cmd)
			if clientCtx.HomeDir != "" {
				filename = filepath.Join(clientCtx.HomeDir, "config", filename+tomlSuffix)
			}

			file, err := os.ReadFile(filename)
			if err != nil {
				return err
			}

			if format, _ := cmd.Flags().GetString(flagOutputFormat); format == "toml" {
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
	cmd.Flags().String(flagOutputFormat, "toml", "Output format (json|toml)")

	return cmd
}
