package keys

import (
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
)

const flagListNames = "list-names"

// ListKeysCmd lists all keys in the key store.
func ListKeysCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "list",
		Short: "List all keys",
		Long: `Return a list of all public keys stored by this key manager
along with their associated name and address.`,
		RunE: runListCmd,
	}

	cmd.Flags().BoolP(flagListNames, "n", false, "List names only")
	return cmd
}

func runListCmd(cmd *cobra.Command, _ []string) error {
	clientCtx, err := client.GetClientQueryContext(cmd)
	if err != nil {
		return err
	}

	records, err := clientCtx.Keyring.List()
	if err != nil {
		return err
	}

	if len(records) == 0 && clientCtx.OutputFormat == flags.OutputFormatText {
		cmd.Println("No records were found in keyring")
		return nil
	}

	if ok, _ := cmd.Flags().GetBool(flagListNames); !ok {
		return printKeyringRecords(clientCtx, cmd.OutOrStdout(), records, clientCtx.OutputFormat)
	}

	for _, k := range records {
		cmd.Println(k.Name)
	}

	return nil
}

// ListKeyTypesCmd lists all key types.
func ListKeyTypesCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "list-key-types",
		Short: "List all key types",
		Long:  `Return a list of all supported key types (also known as algos)`,
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			cmd.Println("Supported key types/algos:")
			keyring, _ := clientCtx.Keyring.SupportedAlgorithms()
			cmd.Printf("%+q\n", keyring)
			return nil
		},
	}
}
