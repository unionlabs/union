package cli

import (
	"errors"
	"fmt"
	"os"
	"strings"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"google.golang.org/protobuf/types/known/anypb"

	errorsmod "cosmossdk.io/errors"
	txsigning "cosmossdk.io/x/tx/signing"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	kmultisig "github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	"github.com/cosmos/cosmos-sdk/crypto/types/multisig"
	sdk "github.com/cosmos/cosmos-sdk/types"
	signingtypes "github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/version"
	authclient "github.com/cosmos/cosmos-sdk/x/auth/client"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
)

// GetMultiSignCommand returns the multi-sign command
func GetMultiSignCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "multi-sign <file> <name> [<signature>...]",
		Aliases: []string{"multisign"},
		Short:   "Generate multisig signatures for transactions generated offline",
		Long: strings.TrimSpace(
			fmt.Sprintf(`Sign transactions created with the --generate-only flag that require multisig signatures.

Read one or more signatures from one or more [signature] file, generate a multisig signature compliant to the
multisig key [name], and attach the key name to the transaction read from [file].

Example:
$ %s tx multisign transaction.json k1k2k3 k1sig.json k2sig.json k3sig.json

If --signature-only flag is on, output a JSON representation
of only the generated signature.

If the --offline flag is on, the client will not reach out to an external node.
Account number or sequence number lookups are not performed so you must
set these parameters manually.

If the --skip-signature-verification flag is on, the command will not verify the
signatures in the provided signature files. This is useful when the multisig
account is a signer in a nested multisig scenario.

The current multisig implementation defaults to amino-json sign mode.
The SIGN_MODE_DIRECT sign mode is not supported.'
`,
				version.AppName,
			),
		),
		RunE: makeMultiSignCmd(),
		Args: cobra.MinimumNArgs(3),
	}

	cmd.Flags().Bool(flagSkipSignatureVerification, false, "Skip signature verification")
	cmd.Flags().Bool(flagSigOnly, false, "Print only the generated signature, then exit")
	cmd.Flags().String(flags.FlagOutputDocument, "", "The document is written to the given file instead of STDOUT")
	flags.AddTxFlagsToCmd(cmd)
	_ = cmd.Flags().MarkHidden(flags.FlagOutput)

	return cmd
}

func makeMultiSignCmd() func(cmd *cobra.Command, args []string) (err error) {
	return func(cmd *cobra.Command, args []string) (err error) {
		file := args[0]
		name := args[1]
		sigsRaw := args[2:]
		_ = cmd.Flags().Set(flags.FlagFrom, args[1])

		clientCtx, err := client.GetClientTxContext(cmd)
		if err != nil {
			return err
		}
		parsedTx, err := authclient.ReadTxFromFile(clientCtx, file)
		if err != nil {
			return err
		}

		txFactory, err := tx.NewFactoryCLI(clientCtx, cmd.Flags())
		if err != nil {
			return err
		}
		if txFactory.SignMode() == signingtypes.SignMode_SIGN_MODE_UNSPECIFIED {
			txFactory = txFactory.WithSignMode(signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
		}

		txCfg := clientCtx.TxConfig
		txBuilder, err := txCfg.WrapTxBuilder(parsedTx)
		if err != nil {
			return err
		}

		k, err := clientCtx.Keyring.Key(name)
		if err != nil {
			return errorsmod.Wrap(err, "error getting keybase multisig account")
		}
		pubKey, err := k.GetPubKey()
		if err != nil {
			return err
		}

		addr, err := k.GetAddress()
		if err != nil {
			return err
		}

		// avoid signature verification if the sender of the tx is different than
		// the multisig key (useful for nested multisigs).
		skipSigVerify, _ := cmd.Flags().GetBool(flagSkipSignatureVerification)

		multisigPub := pubKey.(*kmultisig.LegacyAminoPubKey)
		multisigSig := multisig.NewMultisig(len(multisigPub.PubKeys))
		if !clientCtx.Offline {
			accnum, seq, err := clientCtx.AccountRetriever.GetAccountNumberSequence(clientCtx, addr)
			if err != nil {
				return err
			}

			txFactory = txFactory.WithAccountNumber(accnum).WithSequence(seq)
		}

		// read each signature and add it to the multisig if valid
		for i := 0; i < len(sigsRaw); i++ {
			sigs, err := unmarshalSignatureJSON(clientCtx, sigsRaw[i])
			if err != nil {
				return err
			}

			if txFactory.ChainID() == "" {
				return errors.New("set the chain id with either the --chain-id flag or config file")
			}

			for _, sig := range sigs {
				anyPk, err := codectypes.NewAnyWithValue(sig.PubKey)
				if err != nil {
					return err
				}
				txSignerData := txsigning.SignerData{
					ChainID:       txFactory.ChainID(),
					AccountNumber: txFactory.AccountNumber(),
					Sequence:      txFactory.Sequence(),
					Address:       sdk.AccAddress(sig.PubKey.Address()).String(),
					PubKey: &anypb.Any{
						TypeUrl: anyPk.TypeUrl,
						Value:   anyPk.Value,
					},
				}
				builtTx := txBuilder.GetTx()
				adaptableTx, ok := builtTx.(signing.V2AdaptableTx)
				if !ok {
					return fmt.Errorf("expected Tx to be signing.V2AdaptableTx, got %T", builtTx)
				}
				txData := adaptableTx.GetSigningTxData()

				if !skipSigVerify {
					err = signing.VerifySignature(cmd.Context(), sig.PubKey, txSignerData, sig.Data,
						txCfg.SignModeHandler(), txData)
					if err != nil {
						addr, _ := sdk.AccAddressFromHexUnsafe(sig.PubKey.Address().String())
						return fmt.Errorf("couldn't verify signature for address %s %w", addr, err)
					}
				}

				if err := multisig.AddSignatureV2(multisigSig, sig, multisigPub.GetPubKeys()); err != nil {
					return err
				}
			}
		}

		sigV2 := signingtypes.SignatureV2{
			PubKey:   multisigPub,
			Data:     multisigSig,
			Sequence: txFactory.Sequence(),
		}

		err = txBuilder.SetSignatures(sigV2)
		if err != nil {
			return err
		}

		sigOnly, _ := cmd.Flags().GetBool(flagSigOnly)

		var json []byte
		json, err = marshalSignatureJSON(txCfg, txBuilder.GetTx(), sigOnly)
		if err != nil {
			return err
		}

		closeFunc, err := setOutputFile(cmd)
		if err != nil {
			return err
		}

		defer closeFunc()

		cmd.Printf("%s\n", json)
		return nil
	}
}

func GetMultiSignBatchCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "multisign-batch <file> <name> <[signature-file>...]",
		Aliases: []string{"multi-sign-batch"},
		Short:   "Assemble multisig transactions in batch from batch signatures",
		Long: strings.TrimSpace(
			fmt.Sprintf(`Assemble a batch of multisig transactions generated by batch sign command.

Read one or more signatures from one or more [signature] file, generate a multisig signature compliant to the
multisig key [name], and attach the key name to the transaction read from [file].

Example:
$ %s tx multisign-batch transactions.json multisigk1k2k3 k1sigs.json k2sigs.json k3sig.json

The current multisig implementation defaults to amino-json sign mode.
The SIGN_MODE_DIRECT sign mode is not supported.'
`, version.AppName,
			),
		),
		PreRun: preSignCmd,
		RunE:   makeBatchMultisignCmd(),
		Args:   cobra.MinimumNArgs(3),
	}

	cmd.Flags().Bool(flagNoAutoIncrement, false, "disable sequence auto increment")
	cmd.Flags().String(
		flagMultisig, "",
		"Address of the multisig account that the transaction signs on behalf of",
	)
	cmd.Flags().String(flags.FlagOutputDocument, "", "The document is written to the given file instead of STDOUT")
	flags.AddTxFlagsToCmd(cmd)
	_ = cmd.Flags().MarkHidden(flags.FlagOutput) // signing makes sense to output only json

	return cmd
}

func makeBatchMultisignCmd() func(cmd *cobra.Command, args []string) error {
	return func(cmd *cobra.Command, args []string) (err error) {
		var clientCtx client.Context

		file, name := args[0], args[1]
		sigFiles := args[2:]

		clientCtx, err = client.GetClientTxContext(cmd)
		if err != nil {
			return err
		}

		txCfg := clientCtx.TxConfig
		txFactory, err := tx.NewFactoryCLI(clientCtx, cmd.Flags())
		if err != nil {
			return err
		}
		if txFactory.SignMode() == signingtypes.SignMode_SIGN_MODE_UNSPECIFIED {
			txFactory = txFactory.WithSignMode(signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
		}

		// reads tx from args[0]
		scanner, err := authclient.ReadTxsFromInput(txCfg, file)
		if err != nil {
			return err
		}

		k, err := clientCtx.Keyring.Key(name)
		if err != nil {
			return errorsmod.Wrap(err, "error getting keybase multisig account")
		}

		var signatureBatch [][]signingtypes.SignatureV2
		for i := 0; i < len(sigFiles); i++ {
			sigs, err := readSignaturesFromFile(clientCtx, sigFiles[i])
			if err != nil {
				return err
			}

			signatureBatch = append(signatureBatch, sigs)
		}

		addr, err := k.GetAddress()
		if err != nil {
			return err
		}

		if !clientCtx.Offline {
			accnum, seq, err := clientCtx.AccountRetriever.GetAccountNumberSequence(clientCtx, addr)
			if err != nil {
				return err
			}

			txFactory = txFactory.WithAccountNumber(accnum).WithSequence(seq)
		}

		// prepare output document
		closeFunc, err := setOutputFile(cmd)
		if err != nil {
			return err
		}

		defer closeFunc()
		clientCtx.WithOutput(cmd.OutOrStdout())

		for i := 0; scanner.Scan(); i++ {
			txBuilder, err := txCfg.WrapTxBuilder(scanner.Tx())
			if err != nil {
				return err
			}
			pubKey, err := k.GetPubKey()
			if err != nil {
				return err
			}
			multisigPub := pubKey.(*kmultisig.LegacyAminoPubKey)
			multisigSig := multisig.NewMultisig(len(multisigPub.PubKeys))

			anyPk, err := codectypes.NewAnyWithValue(multisigPub)
			if err != nil {
				return err
			}
			txSignerData := txsigning.SignerData{
				ChainID:       txFactory.ChainID(),
				AccountNumber: txFactory.AccountNumber(),
				Sequence:      txFactory.Sequence(),
				Address:       sdk.AccAddress(pubKey.Address()).String(),
				PubKey: &anypb.Any{
					TypeUrl: anyPk.TypeUrl,
					Value:   anyPk.Value,
				},
			}

			builtTx := txBuilder.GetTx()
			adaptableTx, ok := builtTx.(signing.V2AdaptableTx)
			if !ok {
				return fmt.Errorf("expected Tx to be signing.V2AdaptableTx, got %T", builtTx)
			}
			txData := adaptableTx.GetSigningTxData()

			for _, sig := range signatureBatch {
				err = signing.VerifySignature(cmd.Context(), sig[i].PubKey, txSignerData, sig[i].Data,
					txCfg.SignModeHandler(), txData)
				if err != nil {
					return fmt.Errorf("couldn't verify signature: %w %v", err, sig)
				}

				if err := multisig.AddSignatureV2(multisigSig, sig[i], multisigPub.GetPubKeys()); err != nil {
					return err
				}
			}

			sigV2 := signingtypes.SignatureV2{
				PubKey:   multisigPub,
				Data:     multisigSig,
				Sequence: txFactory.Sequence(),
			}

			err = txBuilder.SetSignatures(sigV2)
			if err != nil {
				return err
			}

			sigOnly, _ := cmd.Flags().GetBool(flagSigOnly)
			var json []byte
			json, err = marshalSignatureJSON(txCfg, txBuilder.GetTx(), sigOnly)
			if err != nil {
				return err
			}

			err = clientCtx.PrintString(fmt.Sprintf("%s\n", json))
			if err != nil {
				return err
			}

			if viper.GetBool(flagNoAutoIncrement) {
				continue
			}
			sequence := txFactory.Sequence() + 1
			txFactory = txFactory.WithSequence(sequence)
		}

		return scanner.UnmarshalErr()
	}
}

func unmarshalSignatureJSON(clientCtx client.Context, filename string) (sigs []signingtypes.SignatureV2, err error) {
	var bytes []byte
	if bytes, err = os.ReadFile(filename); err != nil {
		return
	}
	return clientCtx.TxConfig.UnmarshalSignatureJSON(bytes)
}

func readSignaturesFromFile(ctx client.Context, filename string) (sigs []signingtypes.SignatureV2, err error) {
	bz, err := os.ReadFile(filename)
	if err != nil {
		return nil, err
	}

	newString := strings.TrimSuffix(string(bz), "\n")
	lines := strings.Split(newString, "\n")

	for _, bz := range lines {
		sig, err := ctx.TxConfig.UnmarshalSignatureJSON([]byte(bz))
		if err != nil {
			return nil, err
		}

		sigs = append(sigs, sig...)
	}
	return sigs, nil
}
