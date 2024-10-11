package keys

import (
	"bufio"
	"errors"
	"fmt"
	"strings"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/input"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
)

// RenameKeyCommand renames a key from the key store.
func RenameKeyCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "rename <old_name> <new_name>",
		Short: "Rename an existing key",
		Long: `Rename a key from the Keybase backend.

Note that renaming offline or ledger keys will rename
only the public key references stored locally, i.e.
private keys stored in a ledger device cannot be renamed with the CLI.
`,
		Args: cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			buf := bufio.NewReader(cmd.InOrStdin())
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			oldName, newName := args[0], args[1]
			if strings.TrimSpace(newName) == "" {
				return errors.New("the new name cannot be empty or consist solely of whitespace")
			}

			k, err := clientCtx.Keyring.Key(oldName)
			if err != nil {
				return err
			}

			// confirm rename, unless -y is passed
			if skip, _ := cmd.Flags().GetBool(flagYes); !skip {
				prompt := fmt.Sprintf("Key reference will be renamed from %s to %s. Continue?", args[0], args[1])
				if yes, err := input.GetConfirmation(prompt, buf, cmd.ErrOrStderr()); err != nil {
					return err
				} else if !yes {
					return nil
				}
			}

			if err := clientCtx.Keyring.Rename(oldName, newName); err != nil {
				return err
			}

			if k.GetType() == keyring.TypeLedger || k.GetType() == keyring.TypeOffline {
				cmd.PrintErrln("Public key reference renamed")
				return nil
			}

			cmd.PrintErrln(fmt.Sprintf("Key was successfully renamed from %s to %s", oldName, newName))

			return nil
		},
	}

	cmd.Flags().BoolP(flagYes, "y", false, "Skip confirmation prompt when renaming offline or ledger key references")

	return cmd
}
