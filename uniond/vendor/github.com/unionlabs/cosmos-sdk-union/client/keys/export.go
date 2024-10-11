package keys

import (
	"bufio"
	"encoding/hex"
	"fmt"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/input"
	"github.com/cosmos/cosmos-sdk/crypto/types"
)

const (
	flagUnarmoredHex = "unarmored-hex"
	flagUnsafe       = "unsafe"
)

// ExportKeyCommand exports private keys from the key store.
func ExportKeyCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "export <name>",
		Short: "Export private keys",
		Long: `Export a private key from the local keyring in ASCII-armored encrypted format.

When both the --unarmored-hex and --unsafe flags are selected, cryptographic
private key material is exported in an INSECURE fashion that is designed to
allow users to import their keys in hot wallets. This feature is for advanced
users only that are confident about how to handle private keys work and are
FULLY AWARE OF THE RISKS. If you are unsure, you may want to do some research
and export your keys in ASCII-armored encrypted format.`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			buf := bufio.NewReader(clientCtx.Input)
			unarmored, _ := cmd.Flags().GetBool(flagUnarmoredHex)
			unsafe, _ := cmd.Flags().GetBool(flagUnsafe)

			if unarmored && unsafe {
				return exportUnsafeUnarmored(clientCtx, cmd, args[0], buf)
			} else if unarmored || unsafe {
				return fmt.Errorf("the flags %s and %s must be used together", flagUnsafe, flagUnarmoredHex)
			}

			encryptPassword, err := input.GetPassword("Enter passphrase to encrypt the exported key:", buf)
			if err != nil {
				return err
			}

			armored, err := clientCtx.Keyring.ExportPrivKeyArmor(args[0], encryptPassword)
			if err != nil {
				return err
			}

			cmd.Println(armored)

			return nil
		},
	}

	cmd.Flags().Bool(flagUnarmoredHex, false, "Export unarmored hex privkey. Requires --unsafe.")
	cmd.Flags().Bool(flagUnsafe, false, "Enable unsafe operations. This flag must be switched on along with all unsafe operation-specific options.")
	cmd.Flags().Bool(flagIndiscreet, false, "Print unarmored hex privkey directly on current terminal (only valid when --unarmored-hex is true)")
	cmd.Flags().BoolP(flagYes, "y", false, "Skip confirmation prompt when export unarmored hex privkey")

	return cmd
}

func exportUnsafeUnarmored(ctx client.Context, cmd *cobra.Command, uid string, buf *bufio.Reader) error {
	// confirm export unarmored hex privkey, unless -y is passed
	if skip, _ := cmd.Flags().GetBool(flagYes); !skip {
		if yes, err := input.GetConfirmation("WARNING: The private key will be exported as an unarmored hexadecimal string. USE AT YOUR OWN RISK. Continue?", buf, cmd.ErrOrStderr()); err != nil {
			return err
		} else if !yes {
			return nil
		}
	}

	hexPrivKey, err := unsafeExportPrivKeyHex(ctx.Keyring.(unsafeExporter), uid)
	if err != nil {
		return err
	}
	indiscreet, _ := cmd.Flags().GetBool(flagIndiscreet)
	if indiscreet {
		cmd.Println(hexPrivKey)
		return nil
	}
	if err = printDiscreetly(ctx, cmd.ErrOrStderr(), "**Important** Do not share this private key.", hexPrivKey); err != nil {
		return fmt.Errorf("failed to print private key: %w", err)
	}
	cmd.Println("Export private key successfully")
	return nil
}

// unsafeExporter is implemented by key stores that support unsafe export
// of private keys' material.
type unsafeExporter interface {
	// ExportPrivateKeyObject returns a private key in unarmored format.
	ExportPrivateKeyObject(uid string) (types.PrivKey, error)
}

// unsafeExportPrivKeyHex exports private keys in unarmored hexadecimal format.
func unsafeExportPrivKeyHex(ks unsafeExporter, uid string) (privkey string, err error) {
	priv, err := ks.ExportPrivateKeyObject(uid)
	if err != nil {
		return "", err
	}

	return hex.EncodeToString(priv.Bytes()), nil
}
