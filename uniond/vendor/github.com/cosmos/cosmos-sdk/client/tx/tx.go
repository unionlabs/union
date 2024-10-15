package tx

import (
	"bufio"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"

	gogogrpc "github.com/cosmos/gogoproto/grpc"
	"github.com/spf13/pflag"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/input"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	authsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
)

// GenerateOrBroadcastTxCLI will either generate and print an unsigned transaction
// or sign it and broadcast it returning an error upon failure.
func GenerateOrBroadcastTxCLI(clientCtx client.Context, flagSet *pflag.FlagSet, msgs ...sdk.Msg) error {
	txf, err := NewFactoryCLI(clientCtx, flagSet)
	if err != nil {
		return err
	}
	return GenerateOrBroadcastTxWithFactory(clientCtx, txf, msgs...)
}

// GenerateOrBroadcastTxWithFactory will either generate and print an unsigned transaction
// or sign it and broadcast it returning an error upon failure.
func GenerateOrBroadcastTxWithFactory(clientCtx client.Context, txf Factory, msgs ...sdk.Msg) error {
	// Validate all msgs before generating or broadcasting the tx.
	// We were calling ValidateBasic separately in each CLI handler before.
	// Right now, we're factorizing that call inside this function.
	// ref: https://github.com/cosmos/cosmos-sdk/pull/9236#discussion_r623803504
	for _, msg := range msgs {
		m, ok := msg.(sdk.HasValidateBasic)
		if !ok {
			continue
		}

		if err := m.ValidateBasic(); err != nil {
			return err
		}
	}

	// If the --aux flag is set, we simply generate and print the AuxSignerData.
	if clientCtx.IsAux {
		auxSignerData, err := makeAuxSignerData(clientCtx, txf, msgs...)
		if err != nil {
			return err
		}

		return clientCtx.PrintProto(&auxSignerData)
	}

	if clientCtx.GenerateOnly {
		return txf.PrintUnsignedTx(clientCtx, msgs...)
	}

	return BroadcastTx(clientCtx, txf, msgs...)
}

// BroadcastTx attempts to generate, sign and broadcast a transaction with the
// given set of messages. It will also simulate gas requirements if necessary.
// It will return an error upon failure.
func BroadcastTx(clientCtx client.Context, txf Factory, msgs ...sdk.Msg) error {
	txf, err := txf.Prepare(clientCtx)
	if err != nil {
		return err
	}

	if txf.SimulateAndExecute() || clientCtx.Simulate {
		if clientCtx.Offline {
			return errors.New("cannot estimate gas in offline mode")
		}

		_, adjusted, err := CalculateGas(clientCtx, txf, msgs...)
		if err != nil {
			return err
		}

		txf = txf.WithGas(adjusted)
		_, _ = fmt.Fprintf(os.Stderr, "%s\n", GasEstimateResponse{GasEstimate: txf.Gas()})
	}

	if clientCtx.Simulate {
		return nil
	}

	tx, err := txf.BuildUnsignedTx(msgs...)
	if err != nil {
		return err
	}

	if !clientCtx.SkipConfirm {
		encoder := txf.txConfig.TxJSONEncoder()
		if encoder == nil {
			return errors.New("failed to encode transaction: tx json encoder is nil")
		}

		txBytes, err := encoder(tx.GetTx())
		if err != nil {
			return fmt.Errorf("failed to encode transaction: %w", err)
		}

		if err := clientCtx.PrintRaw(json.RawMessage(txBytes)); err != nil {
			_, _ = fmt.Fprintf(os.Stderr, "error: %v\n%s\n", err, txBytes)
		}

		buf := bufio.NewReader(os.Stdin)
		ok, err := input.GetConfirmation("confirm transaction before signing and broadcasting", buf, os.Stderr)
		if err != nil {
			_, _ = fmt.Fprintf(os.Stderr, "error: %v\ncanceled transaction\n", err)
			return err
		}
		if !ok {
			_, _ = fmt.Fprintln(os.Stderr, "canceled transaction")
			return nil
		}
	}

	if err = Sign(clientCtx, txf, clientCtx.FromName, tx, true); err != nil {
		return err
	}

	txBytes, err := clientCtx.TxConfig.TxEncoder()(tx.GetTx())
	if err != nil {
		return err
	}

	// broadcast to a CometBFT node
	res, err := clientCtx.BroadcastTx(txBytes)
	if err != nil {
		return err
	}

	return clientCtx.PrintProto(res)
}

// CalculateGas simulates the execution of a transaction and returns the
// simulation response obtained by the query and the adjusted gas amount.
func CalculateGas(
	clientCtx gogogrpc.ClientConn, txf Factory, msgs ...sdk.Msg,
) (*tx.SimulateResponse, uint64, error) {
	txBytes, err := txf.BuildSimTx(msgs...)
	if err != nil {
		return nil, 0, err
	}

	txSvcClient := tx.NewServiceClient(clientCtx)
	simRes, err := txSvcClient.Simulate(context.Background(), &tx.SimulateRequest{
		TxBytes: txBytes,
	})
	if err != nil {
		return nil, 0, err
	}

	return simRes, uint64(txf.GasAdjustment() * float64(simRes.GasInfo.GasUsed)), nil
}

// SignWithPrivKey signs a given tx with the given private key, and returns the
// corresponding SignatureV2 if the signing is successful.
func SignWithPrivKey(
	ctx context.Context,
	signMode signing.SignMode, signerData authsigning.SignerData,
	txBuilder client.TxBuilder, priv cryptotypes.PrivKey, txConfig client.TxConfig,
	accSeq uint64,
) (signing.SignatureV2, error) {
	var sigV2 signing.SignatureV2

	// Generate the bytes to be signed.
	signBytes, err := authsigning.GetSignBytesAdapter(
		ctx, txConfig.SignModeHandler(), signMode, signerData, txBuilder.GetTx())
	if err != nil {
		return sigV2, err
	}

	// Sign those bytes
	signature, err := priv.Sign(signBytes)
	if err != nil {
		return sigV2, err
	}

	// Construct the SignatureV2 struct
	sigData := signing.SingleSignatureData{
		SignMode:  signMode,
		Signature: signature,
	}

	sigV2 = signing.SignatureV2{
		PubKey:   priv.PubKey(),
		Data:     &sigData,
		Sequence: accSeq,
	}

	return sigV2, nil
}

// countDirectSigners counts the number of DIRECT signers in a signature data.
func countDirectSigners(data signing.SignatureData) int {
	switch data := data.(type) {
	case *signing.SingleSignatureData:
		if data.SignMode == signing.SignMode_SIGN_MODE_DIRECT {
			return 1
		}

		return 0
	case *signing.MultiSignatureData:
		directSigners := 0
		for _, d := range data.Signatures {
			directSigners += countDirectSigners(d)
		}

		return directSigners
	default:
		panic("unreachable case")
	}
}

// checkMultipleSigners checks that there can be maximum one DIRECT signer in
// a tx.
func checkMultipleSigners(tx authsigning.Tx) error {
	directSigners := 0
	sigsV2, err := tx.GetSignaturesV2()
	if err != nil {
		return err
	}
	for _, sig := range sigsV2 {
		directSigners += countDirectSigners(sig.Data)
		if directSigners > 1 {
			return sdkerrors.ErrNotSupported.Wrap("txs signed with CLI can have maximum 1 DIRECT signer")
		}
	}

	return nil
}

// Sign signs a given tx with a named key. The bytes signed over are canconical.
// The resulting signature will be added to the transaction builder overwriting the previous
// ones if overwrite=true (otherwise, the signature will be appended).
// Signing a transaction with mutltiple signers in the DIRECT mode is not supported and will
// return an error.
// An error is returned upon failure.
func Sign(ctx client.Context, txf Factory, name string, txBuilder client.TxBuilder, overwriteSig bool) error {
	if txf.keybase == nil {
		return errors.New("keybase must be set prior to signing a transaction")
	}

	var err error
	signMode := txf.signMode
	if signMode == signing.SignMode_SIGN_MODE_UNSPECIFIED {
		// use the SignModeHandler's default mode if unspecified
		signMode, err = authsigning.APISignModeToInternal(txf.txConfig.SignModeHandler().DefaultMode())
		if err != nil {
			return err
		}
	}

	k, err := txf.keybase.Key(name)
	if err != nil {
		return err
	}

	pubKey, err := k.GetPubKey()
	if err != nil {
		return err
	}

	addressStr, err := ctx.AddressCodec.BytesToString(pubKey.Address())
	if err != nil {
		return err
	}

	signerData := authsigning.SignerData{
		ChainID:       txf.chainID,
		AccountNumber: txf.accountNumber,
		Sequence:      txf.sequence,
		PubKey:        pubKey,
		Address:       addressStr,
	}

	// For SIGN_MODE_DIRECT, calling SetSignatures calls setSignerInfos on
	// TxBuilder under the hood, and SignerInfos is needed to generated the
	// sign bytes. This is the reason for setting SetSignatures here, with a
	// nil signature.
	//
	// Note: this line is not needed for SIGN_MODE_LEGACY_AMINO, but putting it
	// also doesn't affect its generated sign bytes, so for code's simplicity
	// sake, we put it here.
	sigData := signing.SingleSignatureData{
		SignMode:  signMode,
		Signature: nil,
	}
	sig := signing.SignatureV2{
		PubKey:   pubKey,
		Data:     &sigData,
		Sequence: txf.Sequence(),
	}

	var prevSignatures []signing.SignatureV2
	if !overwriteSig {
		prevSignatures, err = txBuilder.GetTx().GetSignaturesV2()
		if err != nil {
			return err
		}
	}
	// Overwrite or append signer infos.
	var sigs []signing.SignatureV2
	if overwriteSig {
		sigs = []signing.SignatureV2{sig}
	} else {
		sigs = append(sigs, prevSignatures...)
		sigs = append(sigs, sig)
	}
	if err := txBuilder.SetSignatures(sigs...); err != nil {
		return err
	}

	if err := checkMultipleSigners(txBuilder.GetTx()); err != nil {
		return err
	}

	bytesToSign, err := authsigning.GetSignBytesAdapter(ctx.CmdContext, txf.txConfig.SignModeHandler(), signMode, signerData, txBuilder.GetTx())
	if err != nil {
		return err
	}

	// Sign those bytes
	sigBytes, _, err := txf.keybase.Sign(name, bytesToSign, signMode)
	if err != nil {
		return err
	}

	// Construct the SignatureV2 struct
	sigData = signing.SingleSignatureData{
		SignMode:  signMode,
		Signature: sigBytes,
	}
	sig = signing.SignatureV2{
		PubKey:   pubKey,
		Data:     &sigData,
		Sequence: txf.Sequence(),
	}

	if overwriteSig {
		err = txBuilder.SetSignatures(sig)
	} else {
		prevSignatures = append(prevSignatures, sig)
		err = txBuilder.SetSignatures(prevSignatures...)
	}

	if err != nil {
		return fmt.Errorf("unable to set signatures on payload: %w", err)
	}

	// Run optional preprocessing if specified. By default, this is unset
	// and will return nil.
	return txf.PreprocessTx(name, txBuilder)
}

// GasEstimateResponse defines a response definition for tx gas estimation.
type GasEstimateResponse struct {
	GasEstimate uint64 `json:"gas_estimate" yaml:"gas_estimate"`
}

func (gr GasEstimateResponse) String() string {
	return fmt.Sprintf("gas estimate: %d", gr.GasEstimate)
}

// makeAuxSignerData generates an AuxSignerData from the client inputs.
func makeAuxSignerData(clientCtx client.Context, f Factory, msgs ...sdk.Msg) (tx.AuxSignerData, error) {
	b := NewAuxTxBuilder()
	fromAddress, name, _, err := client.GetFromFields(clientCtx, clientCtx.Keyring, clientCtx.From)
	if err != nil {
		return tx.AuxSignerData{}, err
	}

	fromAddrStr, err := clientCtx.AddressCodec.BytesToString(fromAddress)
	if err != nil {
		return tx.AuxSignerData{}, err
	}

	b.SetAddress(fromAddrStr)
	if clientCtx.Offline {
		b.SetAccountNumber(f.accountNumber)
		b.SetSequence(f.sequence)
	} else {
		accNum, seq, err := clientCtx.AccountRetriever.GetAccountNumberSequence(clientCtx, fromAddress)
		if err != nil {
			return tx.AuxSignerData{}, err
		}
		b.SetAccountNumber(accNum)
		b.SetSequence(seq)
	}

	err = b.SetMsgs(msgs...)
	if err != nil {
		return tx.AuxSignerData{}, err
	}

	err = b.SetSignMode(f.SignMode())
	if err != nil {
		return tx.AuxSignerData{}, err
	}

	key, err := clientCtx.Keyring.Key(name)
	if err != nil {
		return tx.AuxSignerData{}, err
	}

	pub, err := key.GetPubKey()
	if err != nil {
		return tx.AuxSignerData{}, err
	}

	err = b.SetPubKey(pub)
	if err != nil {
		return tx.AuxSignerData{}, err
	}

	b.SetChainID(clientCtx.ChainID)
	signBz, err := b.GetSignBytes()
	if err != nil {
		return tx.AuxSignerData{}, err
	}

	sig, _, err := clientCtx.Keyring.Sign(name, signBz, f.signMode)
	if err != nil {
		return tx.AuxSignerData{}, err
	}
	b.SetSignature(sig)

	return b.GetAuxSignerData()
}
