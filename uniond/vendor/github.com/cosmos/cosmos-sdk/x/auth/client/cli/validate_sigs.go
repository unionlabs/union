package cli

import (
	"bytes"
	"fmt"

	"github.com/spf13/cobra"
	"google.golang.org/protobuf/types/known/anypb"

	txsigning "cosmossdk.io/x/tx/signing"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authclient "github.com/cosmos/cosmos-sdk/x/auth/client"
	authsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
)

func GetValidateSignaturesCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "validate-signatures [file]",
		Short: "validate transactions signatures",
		Long: `Print the addresses that must sign the transaction, those who have already
signed it, and make sure that signatures are in the correct order.

The command would check whether all required signers have signed the transactions, whether
the signatures were collected in the right order, and if the signature is valid over the
given transaction. If the --offline flag is also set, signature validation over the
transaction will be not be performed as that will require RPC communication with a full node.
`,
		PreRun: preSignCmd,
		RunE:   makeValidateSignaturesCmd(),
		Args:   cobra.ExactArgs(1),
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

func makeValidateSignaturesCmd() func(cmd *cobra.Command, args []string) error {
	return func(cmd *cobra.Command, args []string) error {
		clientCtx, err := client.GetClientTxContext(cmd)
		if err != nil {
			return err
		}
		clientCtx, txBldr, stdTx, err := readTxAndInitContexts(clientCtx, cmd, args[0])
		if err != nil {
			return err
		}

		if !printAndValidateSigs(cmd, clientCtx, txBldr.ChainID(), stdTx, clientCtx.Offline) {
			return fmt.Errorf("signatures validation failed")
		}

		return nil
	}
}

// printAndValidateSigs will validate the signatures of a given transaction over its
// expected signers. In addition, if offline has not been supplied, the signature is
// verified over the transaction sign bytes. Returns false if the validation fails.
func printAndValidateSigs(
	cmd *cobra.Command, clientCtx client.Context, chainID string, tx sdk.Tx, offline bool,
) bool {
	sigTx := tx.(authsigning.SigVerifiableTx)
	signModeHandler := clientCtx.TxConfig.SignModeHandler()
	addrCdc := clientCtx.TxConfig.SigningContext().AddressCodec()

	cmd.Println("Signers:")
	signers, err := sigTx.GetSigners()
	if err != nil {
		panic(err)
	}

	for i, signer := range signers {
		signerStr, err := addrCdc.BytesToString(signer)
		if err != nil {
			panic(err)
		}
		cmd.Printf("  %v: %v\n", i, signerStr)
	}

	success := true
	sigs, err := sigTx.GetSignaturesV2()
	if err != nil {
		panic(err)
	}
	cmd.Println("")
	cmd.Println("Signatures:")

	if len(sigs) != len(signers) {
		success = false
	}

	for i, sig := range sigs {
		var (
			pubKey         = sig.PubKey
			multiSigHeader string
			multiSigMsg    string
			sigAddr        = sdk.AccAddress(pubKey.Address())
			sigSanity      = "OK"
		)

		if i >= len(signers) || !bytes.Equal(sigAddr, signers[i]) {
			sigSanity = "ERROR: signature does not match its respective signer"
			success = false
		}

		// validate the actual signature over the transaction bytes since we can
		// reach out to a full node to query accounts.
		if !offline && success {
			accNum, accSeq, err := clientCtx.AccountRetriever.GetAccountNumberSequence(clientCtx, sigAddr)
			if err != nil {
				cmd.PrintErrf("failed to get account: %s\n", sigAddr)
				return false
			}

			signingData := authsigning.SignerData{
				Address:       sigAddr.String(),
				ChainID:       chainID,
				AccountNumber: accNum,
				Sequence:      accSeq,
				PubKey:        pubKey,
			}
			anyPk, err := codectypes.NewAnyWithValue(pubKey)
			if err != nil {
				cmd.PrintErrf("failed to pack public key: %v", err)
				return false
			}
			txSignerData := txsigning.SignerData{
				ChainID:       signingData.ChainID,
				AccountNumber: signingData.AccountNumber,
				Sequence:      signingData.Sequence,
				Address:       signingData.Address,
				PubKey: &anypb.Any{
					TypeUrl: anyPk.TypeUrl,
					Value:   anyPk.Value,
				},
			}

			adaptableTx, ok := tx.(authsigning.V2AdaptableTx)
			if !ok {
				cmd.PrintErrf("expected V2AdaptableTx, got %T", tx)
				return false
			}
			txData := adaptableTx.GetSigningTxData()

			err = authsigning.VerifySignature(cmd.Context(), pubKey, txSignerData, sig.Data, signModeHandler, txData)
			if err != nil {
				cmd.PrintErrf("failed to verify signature: %v", err)
				return false
			}
		}

		cmd.Printf("  %d: %s\t\t\t[%s]%s%s\n", i, sigAddr.String(), sigSanity, multiSigHeader, multiSigMsg)
	}

	cmd.Println("")

	return success
}

func readTxAndInitContexts(clientCtx client.Context, cmd *cobra.Command, filename string) (client.Context, tx.Factory, sdk.Tx, error) {
	stdTx, err := authclient.ReadTxFromFile(clientCtx, filename)
	if err != nil {
		return clientCtx, tx.Factory{}, nil, err
	}

	txFactory, err := tx.NewFactoryCLI(clientCtx, cmd.Flags())
	if err != nil {
		return clientCtx, tx.Factory{}, nil, err
	}

	return clientCtx, txFactory, stdTx, nil
}
