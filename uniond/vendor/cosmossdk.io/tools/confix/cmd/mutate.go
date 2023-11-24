package cmd

import (
	"context"
	"errors"
	"fmt"
	"strings"

	"github.com/creachadair/tomledit"
	"github.com/creachadair/tomledit/parser"
	"github.com/creachadair/tomledit/transform"
	"github.com/spf13/cobra"

	"cosmossdk.io/tools/confix"

	"github.com/cosmos/cosmos-sdk/client"
)

// SetCommand returns a CLI command to interactively update an application config value.
func SetCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "set [config] [key] [value]",
		Short: "Set an application config value",
		Long:  "Set an application config value. The [config] argument must be the path of the file when using the tool standalone, otherwise it must be the name of the config file without the .toml extension.",
		Args:  cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			filename, inputValue := args[0], args[2]
			// parse key e.g mempool.size -> [mempool, size]
			key := strings.Split(args[1], ".")

			clientCtx := client.GetClientContextFromCmd(cmd)
			if clientCtx.HomeDir != "" {
				filename = fmt.Sprintf("%s/config/%s.toml", clientCtx.HomeDir, filename)
			}

			plan := transform.Plan{
				{
					Desc: fmt.Sprintf("update %q=%q in %s", key, inputValue, filename),
					T: transform.Func(func(_ context.Context, doc *tomledit.Document) error {
						results := doc.Find(key...)
						if len(results) == 0 {
							return fmt.Errorf("key %q not found", key)
						} else if len(results) > 1 {
							return fmt.Errorf("key %q is ambiguous", key)
						}

						value, err := parser.ParseValue(inputValue)
						if err != nil {
							value = parser.MustValue(`"` + inputValue + `"`)
						}

						if ok := transform.InsertMapping(results[0].Section, &parser.KeyValue{
							Block: results[0].Block,
							Name:  results[0].Name,
							Value: value,
						}, true); !ok {
							return errors.New("failed to set value")
						}

						return nil
					}),
				},
			}

			outputPath := filename
			if FlagStdOut {
				outputPath = ""
			}

			ctx := cmd.Context()
			if FlagVerbose {
				ctx = confix.WithLogWriter(ctx, cmd.ErrOrStderr())
			}

			return confix.Upgrade(ctx, plan, filename, outputPath, FlagSkipValidate)
		},
	}

	cmd.Flags().BoolVar(&FlagStdOut, "stdout", false, "print the updated config to stdout")
	cmd.Flags().BoolVarP(&FlagVerbose, "verbose", "v", false, "log changes to stderr")
	cmd.Flags().BoolVarP(&FlagSkipValidate, "skip-validate", "s", false, "skip configuration validation (allows to mutate unknown configurations)")

	return cmd
}

// GetCommand returns a CLI command to interactively get an application config value.
func GetCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "get [config] [key]",
		Short: "Get an application config value",
		Long:  "Get an application config value. The [config] argument must be the path of the file when using the too standalone, otherwise it must be the name of the config file without the .toml extension.",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			filename, key := args[0], args[1]
			// parse key e.g mempool.size -> [mempool, size]
			keys := strings.Split(key, ".")

			clientCtx := client.GetClientContextFromCmd(cmd)
			if clientCtx.HomeDir != "" {
				filename = fmt.Sprintf("%s/config/%s.toml", clientCtx.HomeDir, filename)
			}

			doc, err := confix.LoadConfig(filename)
			if err != nil {
				return fmt.Errorf("failed to load config: %w", err)
			}

			results := doc.Find(keys...)
			if len(results) == 0 {
				return fmt.Errorf("key %q not found", key)
			} else if len(results) > 1 {
				return fmt.Errorf("key %q is ambiguous", key)
			}

			return clientCtx.PrintString(fmt.Sprintf("%s\n", results[0].Value.String()))
		},
	}

	return cmd
}
