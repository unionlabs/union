package cli

import (
	"errors"
	"fmt"
	"os"

	"github.com/spf13/cobra"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	kmultisig "github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authclient "github.com/cosmos/cosmos-sdk/x/auth/client"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
)

const (
	flagMultisig                  = "multisig"
	flagOverwrite                 = "overwrite"
	flagSigOnly                   = "signature-only"
	flagSkipSignatureVerification = "skip-signature-verification"
	flagNoAutoIncrement           = "no-auto-increment"
	flagAppend                    = "append"
)

// GetSignBatchCommand returns the transaction sign-batch command.
func GetSignBatchCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "sign-batch <file> [<file2>...]",
		Short: "Sign transaction batch files",
		Long: `Sign batch files of transactions generated with --generate-only.
The command processes list of transactions from a file (one StdTx each line), or multiple files.
Then generates signed transactions or signatures and print their JSON encoding, delimited by '\n'.
As the signatures are generated, the command updates the account and sequence number accordingly.

If the --signature-only flag is set, it will output the signature parts only.

The --offline flag makes sure that the client will not reach out to full node.
As a result, the account and the sequence number queries will not be performed and
it is required to set such parameters manually. Note, invalid values will cause
the transaction to fail. The sequence will be incremented automatically for each
transaction that is signed.

If --account-number or --sequence flag is used when offline=false, they are ignored and 
overwritten by the default flag values.

The --multisig=<multisig_key> flag generates a signature on behalf of a multisig
account key. It implies --signature-only.
`,
		PreRun: preSignCmd,
		RunE:   makeSignBatchCmd(),
		Args:   cobra.MinimumNArgs(1),
	}

	cmd.Flags().String(flagMultisig, "", "Address or key name of the multisig account on behalf of which the transaction shall be signed")
	cmd.Flags().String(flags.FlagOutputDocument, "", "The document will be written to the given file instead of STDOUT")
	cmd.Flags().Bool(flagSigOnly, false, "Print only the generated signature, then exit")
	cmd.Flags().Bool(flagAppend, false, "Combine all message and generate single signed transaction for broadcast.")

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

func makeSignBatchCmd() func(cmd *cobra.Command, args []string) error {
	return func(cmd *cobra.Command, args []string) error {
		clientCtx, err := client.GetClientTxContext(cmd)
		if err != nil {
			return err
		}
		txFactory, err := tx.NewFactoryCLI(clientCtx, cmd.Flags())
		if err != nil {
			return err
		}
		txCfg := clientCtx.TxConfig

		multisigKey, err := cmd.Flags().GetString(flagMultisig)
		if err != nil {
			return err
		}

		// prepare output document
		closeFunc, err := setOutputFile(cmd)
		if err != nil {
			return err
		}
		defer closeFunc()
		clientCtx.WithOutput(cmd.OutOrStdout())

		// reads tx from args
		scanner, err := authclient.ReadTxsFromInput(txCfg, args...)
		if err != nil {
			return err
		}

		if !clientCtx.Offline && multisigKey == "" {
			from, err := cmd.Flags().GetString(flags.FlagFrom)
			if err != nil {
				return err
			}

			fromAddr, _, _, err := client.GetFromFields(clientCtx, txFactory.Keybase(), from)
			if err != nil {
				return err
			}

			fromAcc, err := txFactory.AccountRetriever().GetAccount(clientCtx, fromAddr)
			if err != nil {
				return err
			}

			txFactory = txFactory.WithAccountNumber(fromAcc.GetAccountNumber()).WithSequence(fromAcc.GetSequence())
		}

		appendMessagesToSingleTx, _ := cmd.Flags().GetBool(flagAppend)
		// Combines all tx msgs and create single signed transaction
		if appendMessagesToSingleTx {
			var totalFees sdk.Coins
			txBuilder := txCfg.NewTxBuilder()
			msgs := make([]sdk.Msg, 0)
			newGasLimit := uint64(0)

			for scanner.Scan() {
				unsignedStdTx := scanner.Tx()
				fe, err := txCfg.WrapTxBuilder(unsignedStdTx)
				if err != nil {
					return err
				}
				// increment the gas
				newGasLimit += fe.GetTx().GetGas()
				// Individual fee values from each transaction need to be
				// aggregated to calculate the total fee for the batch of transactions.
				// https://github.com/cosmos/cosmos-sdk/issues/18064
				unmergedFees := fe.GetTx().GetFee()
				for _, fee := range unmergedFees {
					totalFees = totalFees.Add(fee)
				}
				// append messages
				msgs = append(msgs, unsignedStdTx.GetMsgs()...)
			}
			// set the new appended msgs into builder
			err = txBuilder.SetMsgs(msgs...)
			if err != nil {
				return err
			}

			// set the memo,fees,feeGranter,feePayer from cmd flags
			txBuilder.SetMemo(txFactory.Memo())
			txBuilder.SetFeeGranter(clientCtx.FeeGranter)
			txBuilder.SetFeePayer(clientCtx.FeePayer)

			// set the gasLimit
			txBuilder.SetGasLimit(newGasLimit)

			// set the feeAmount
			txBuilder.SetFeeAmount(totalFees)

			// sign the txs
			from, _ := cmd.Flags().GetString(flags.FlagFrom)
			err := sigTxOrMultisig(clientCtx, txBuilder, txFactory, from, multisigKey)
			if err != nil {
				return err
			}

			sigOnly, _ := cmd.Flags().GetBool(flagSigOnly)
			json, err := marshalSignatureJSON(txCfg, txBuilder.GetTx(), sigOnly)
			if err != nil {
				return err
			}

			cmd.Printf("%s\n", json)
		} else {
			// It will generate signed tx for each tx
			for sequence := txFactory.Sequence(); scanner.Scan(); sequence++ {
				unsignedStdTx := scanner.Tx()
				txFactory = txFactory.WithSequence(sequence)
				txBuilder, err := txCfg.WrapTxBuilder(unsignedStdTx)
				if err != nil {
					return err
				}

				// sign the txs
				from, _ := cmd.Flags().GetString(flags.FlagFrom)
				err = sigTxOrMultisig(clientCtx, txBuilder, txFactory, from, multisigKey)
				if err != nil {
					return err
				}

				printSigOnly, _ := cmd.Flags().GetBool(flagSigOnly)
				json, err := marshalSignatureJSON(txCfg, txBuilder.GetTx(), printSigOnly)
				if err != nil {
					return err
				}
				cmd.Printf("%s\n", json)
			}
		}

		return scanner.UnmarshalErr()
	}
}

func sigTxOrMultisig(clientCtx client.Context, txBuilder client.TxBuilder, txFactory tx.Factory, from, multisigKey string) (err error) {
	if multisigKey == "" {
		err = sign(clientCtx, txBuilder, txFactory, from)
	} else {
		err = multisigSign(clientCtx, txBuilder, txFactory, multisigKey)
	}
	return err
}

func sign(clientCtx client.Context, txBuilder client.TxBuilder, txFactory tx.Factory, from string) error {
	_, fromName, _, err := client.GetFromFields(clientCtx, txFactory.Keybase(), from)
	if err != nil {
		return fmt.Errorf("error getting account from keybase: %w", err)
	}

	if err = authclient.SignTx(txFactory, clientCtx, fromName, txBuilder, true, true); err != nil {
		return err
	}

	return nil
}

func multisigSign(clientCtx client.Context, txBuilder client.TxBuilder, txFactory tx.Factory, multisig string) error {
	multisigAddr, multisigName, _, err := client.GetFromFields(clientCtx, txFactory.Keybase(), multisig)
	if err != nil {
		return fmt.Errorf("error getting account from keybase: %w", err)
	}

	fromRecord, err := clientCtx.Keyring.Key(clientCtx.FromName)
	if err != nil {
		return fmt.Errorf("error getting account from keybase: %w", err)
	}

	fromPubKey, err := fromRecord.GetPubKey()
	if err != nil {
		return err
	}

	multisigkey, err := clientCtx.Keyring.Key(multisigName)
	if err != nil {
		return err
	}

	multisigPubKey, err := multisigkey.GetPubKey()
	if err != nil {
		return err
	}

	isSigner, err := isMultisigSigner(clientCtx, multisigPubKey, fromPubKey)
	if err != nil {
		return err
	}

	if !isSigner {
		return errors.New("signing key is not a part of multisig key")
	}

	if err = authclient.SignTxWithSignerAddress(
		txFactory,
		clientCtx,
		multisigAddr,
		clientCtx.FromName,
		txBuilder,
		clientCtx.Offline,
		true,
	); err != nil {
		return err
	}

	return nil
}

// isMultisigSigner checks if the given pubkey is a signer in the multisig or in
// any of the nested multisig signers.
func isMultisigSigner(clientCtx client.Context, multisigPubKey, fromPubKey cryptotypes.PubKey) (bool, error) {
	multisigLegacyPub := multisigPubKey.(*kmultisig.LegacyAminoPubKey)

	var found bool
	for _, pubkey := range multisigLegacyPub.GetPubKeys() {
		if pubkey.Equals(fromPubKey) {
			found = true
			break
		}

		if nestedMultisig, ok := pubkey.(*kmultisig.LegacyAminoPubKey); ok {
			var err error
			found, err = isMultisigSigner(clientCtx, nestedMultisig, fromPubKey)
			if err != nil {
				return false, err
			}
			if found {
				break
			}
		}
	}

	return found, nil
}

func setOutputFile(cmd *cobra.Command) (func(), error) {
	outputDoc, _ := cmd.Flags().GetString(flags.FlagOutputDocument)
	if outputDoc == "" {
		return func() {}, nil
	}

	fp, err := os.OpenFile(outputDoc, os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0o644)
	if err != nil {
		return func() {}, err
	}

	cmd.SetOut(fp)

	return func() { fp.Close() }, nil
}

// GetSignCommand returns the transaction sign command.
func GetSignCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "sign <file>",
		Short: "Sign a transaction generated offline",
		Long: `Sign a transaction created with the --generate-only flag.
It will read a transaction from [file], sign it, and print its JSON encoding.

If the --signature-only flag is set, it will output the signature parts only.

The --offline flag makes sure that the client will not reach out to full node.
As a result, the account and sequence number queries will not be performed and
it is required to set such parameters manually. Note, invalid values will cause
the transaction to fail.

The --multisig=<multisig_key> flag generates a signature on behalf of a multisig account
key. It implies --signature-only. Full multisig signed transactions may eventually
be generated via the 'multisign' command.
`,
		PreRun: preSignCmd,
		RunE:   makeSignCmd(),
		Args:   cobra.ExactArgs(1),
	}

	cmd.Flags().String(flagMultisig, "", "Address or key name of the multisig account on behalf of which the transaction shall be signed")
	cmd.Flags().Bool(flagOverwrite, false, "Overwrite existing signatures with a new one. If disabled, new signature will be appended")
	cmd.Flags().Bool(flagSigOnly, false, "Print only the signatures")
	cmd.Flags().String(flags.FlagOutputDocument, "", "The document will be written to the given file instead of STDOUT")
	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

func preSignCmd(cmd *cobra.Command, _ []string) {
	// Conditionally mark the account and sequence numbers required as no RPC
	// query will be done.
	if offline, _ := cmd.Flags().GetBool(flags.FlagOffline); offline {
		err := cmd.MarkFlagRequired(flags.FlagAccountNumber)
		if err != nil {
			panic(err)
		}
		err = cmd.MarkFlagRequired(flags.FlagSequence)
		if err != nil {
			panic(err)
		}
	}
}

func makeSignCmd() func(cmd *cobra.Command, args []string) error {
	return func(cmd *cobra.Command, args []string) (err error) {
		var clientCtx client.Context

		clientCtx, err = client.GetClientTxContext(cmd)
		if err != nil {
			return err
		}

		clientCtx, txF, newTx, err := readTxAndInitContexts(clientCtx, cmd, args[0])
		if err != nil {
			return err
		}

		return signTx(cmd, clientCtx, txF, newTx)
	}
}

func signTx(cmd *cobra.Command, clientCtx client.Context, txFactory tx.Factory, newTx sdk.Tx) error {
	f := cmd.Flags()
	txCfg := clientCtx.TxConfig
	txBuilder, err := txCfg.WrapTxBuilder(newTx)
	if err != nil {
		return err
	}

	sigOnly, err := cmd.Flags().GetBool(flagSigOnly)
	if err != nil {
		return err
	}

	multisigKey, err := cmd.Flags().GetString(flagMultisig)
	if err != nil {
		return err
	}

	from, err := cmd.Flags().GetString(flags.FlagFrom)
	if err != nil {
		return err
	}

	_, fromName, _, err := client.GetFromFields(clientCtx, txFactory.Keybase(), from)
	if err != nil {
		return fmt.Errorf("error getting account from keybase: %w", err)
	}

	overwrite, err := f.GetBool(flagOverwrite)
	if err != nil {
		return err
	}

	if multisigKey != "" {
		sigOnly = true

		// get the multisig key by name
		// Bech32 decode error, maybe it's a name, we try to fetch from keyring
		multisigAddr, multisigName, _, err := client.GetFromFields(clientCtx, txFactory.Keybase(), multisigKey)
		if err != nil {
			return fmt.Errorf("error getting account from keybase: %w", err)
		}
		multisigkey, err := clientCtx.Keyring.Key(multisigName)
		if err != nil {
			return errorsmod.Wrap(err, "error getting keybase multisig account")
		}
		multisigPubKey, err := multisigkey.GetPubKey()
		if err != nil {
			return err
		}

		fromRecord, err := clientCtx.Keyring.Key(fromName)
		if err != nil {
			return fmt.Errorf("error getting account from keybase: %w", err)
		}
		fromPubKey, err := fromRecord.GetPubKey()
		if err != nil {
			return err
		}

		isSigner, err := isMultisigSigner(clientCtx, multisigPubKey, fromPubKey)
		if err != nil {
			return err
		}
		if !isSigner {
			return errors.New("signing key is not a part of multisig key")
		}

		err = authclient.SignTxWithSignerAddress(
			txFactory, clientCtx, multisigAddr, fromName, txBuilder, clientCtx.Offline, overwrite)
		if err != nil {
			return err
		}
	} else {
		err = authclient.SignTx(txFactory, clientCtx, clientCtx.FromName, txBuilder, clientCtx.Offline, overwrite)
	}
	if err != nil {
		return err
	}

	// set output
	closeFunc, err := setOutputFile(cmd)
	if err != nil {
		return err
	}

	defer closeFunc()
	clientCtx.WithOutput(cmd.OutOrStdout())

	var json []byte
	json, err = marshalSignatureJSON(txCfg, txBuilder.GetTx(), sigOnly)
	if err != nil {
		return err
	}

	cmd.Printf("%s\n", json)

	return err
}

func marshalSignatureJSON(txConfig client.TxConfig, tx signing.Tx, signatureOnly bool) ([]byte, error) {
	if signatureOnly {
		sigs, err := tx.GetSignaturesV2()
		if err != nil {
			return nil, err
		}
		return txConfig.MarshalSignatureJSON(sigs)
	}

	return txConfig.TxJSONEncoder()(tx)
}
