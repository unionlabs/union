package offchain

import (
	"os"
	"path/filepath"

	"github.com/spf13/cobra"

	v2flags "cosmossdk.io/client/v2/internal/flags"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
)

const (
	flagNotEmitUnpopulated = "notEmitUnpopulated"
	flagIndent             = "indent"
	flagEncoding           = "encoding"
	flagFileFormat         = "file-format"
)

// OffChain off-chain utilities.
func OffChain() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "off-chain",
		Short: "Off-chain utilities.",
		Long:  `Utilities for off-chain data.`,
	}

	cmd.AddCommand(
		SignFile(),
		VerifyFile(),
	)

	flags.AddKeyringFlags(cmd.PersistentFlags())
	return cmd
}

// SignFile signs a file with a key.
func SignFile() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "sign-file <keyName> <fileName>",
		Short: "Sign a file.",
		Long:  "Sign a file using a given key.",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx := client.GetClientContextFromCmd(cmd)

			bz, err := os.ReadFile(args[1])
			if err != nil {
				return err
			}

			notEmitUnpopulated, _ := cmd.Flags().GetBool(flagNotEmitUnpopulated)
			indent, _ := cmd.Flags().GetString(flagIndent)
			encoding, _ := cmd.Flags().GetString(flagEncoding)
			outputFormat, _ := cmd.Flags().GetString(v2flags.FlagOutput)
			outputFile, _ := cmd.Flags().GetString(flags.FlagOutputDocument)

			signedTx, err := Sign(clientCtx, bz, args[0], indent, encoding, outputFormat, !notEmitUnpopulated)
			if err != nil {
				return err
			}

			if outputFile != "" {
				fp, err := os.OpenFile(filepath.Clean(outputFile), os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0o600)
				if err != nil {
					return err
				}
				cmd.SetOut(fp)
			}

			cmd.Println(signedTx)
			return nil
		},
	}

	cmd.Flags().String(flagIndent, "  ", "Choose an indent for the tx")
	cmd.Flags().String(v2flags.FlagOutput, "json", "Choose an output format for the tx (json|text")
	cmd.Flags().Bool(flagNotEmitUnpopulated, false, "Don't show unpopulated fields in the tx")
	cmd.Flags().String(flagEncoding, "no-encoding", "Choose an encoding method for the file content to be added as msg data (no-encoding|base64|hex)")
	cmd.Flags().String(flags.FlagOutputDocument, "", "The document will be written to the given file instead of STDOUT")
	return cmd
}

// VerifyFile verifies given file with given key.
func VerifyFile() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "verify-file <keyName> <fileName>",
		Short: "Verify a file.",
		Long:  "Verify a previously signed file with the given key.",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			bz, err := os.ReadFile(args[1])
			if err != nil {
				return err
			}

			fileFormat, _ := cmd.Flags().GetString(flagFileFormat)

			err = Verify(clientCtx, bz, fileFormat)
			if err == nil {
				cmd.Println("Verification OK!")
			}
			return err
		},
	}

	cmd.Flags().String(flagFileFormat, "json", "Choose what's the file format to be verified (json|text)")
	return cmd
}
