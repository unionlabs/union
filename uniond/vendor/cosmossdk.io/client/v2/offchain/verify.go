package offchain

import (
	"bytes"
	"context"
	"errors"
	"fmt"

	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/encoding/prototext"
	"google.golang.org/protobuf/types/known/anypb"

	apitx "cosmossdk.io/api/cosmos/tx/v1beta1"
	v2flags "cosmossdk.io/client/v2/internal/flags"
	txsigning "cosmossdk.io/x/tx/signing"

	"github.com/cosmos/cosmos-sdk/client"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

// Verify verifies a digest after unmarshalling it.
func Verify(ctx client.Context, digest []byte, fileFormat string) error {
	tx, err := unmarshal(digest, fileFormat)
	if err != nil {
		return err
	}

	return verify(ctx, tx)
}

// verify verifies given Tx.
func verify(ctx client.Context, tx *apitx.Tx) error {
	sigTx := builder{
		cdc: ctx.Codec,
		tx:  tx,
	}

	signModeHandler := ctx.TxConfig.SignModeHandler()

	signers, err := sigTx.GetSigners()
	if err != nil {
		return err
	}

	sigs, err := sigTx.GetSignatures()
	if err != nil {
		return err
	}

	if len(sigs) != len(signers) {
		return errors.New("mismatch between the number of signatures and signers")
	}

	for i, sig := range sigs {
		pubKey := sig.PubKey
		if !bytes.Equal(pubKey.Address(), signers[i]) {
			return errors.New("signature does not match its respective signer")
		}

		addr, err := ctx.AddressCodec.BytesToString(pubKey.Address())
		if err != nil {
			return err
		}

		anyPk, err := codectypes.NewAnyWithValue(pubKey)
		if err != nil {
			return err
		}

		txSignerData := txsigning.SignerData{
			ChainID:       ExpectedChainID,
			AccountNumber: ExpectedAccountNumber,
			Sequence:      ExpectedSequence,
			Address:       addr,
			PubKey: &anypb.Any{
				TypeUrl: anyPk.TypeUrl,
				Value:   anyPk.Value,
			},
		}

		txData, err := sigTx.GetSigningTxData()
		if err != nil {
			return err
		}

		err = verifySignature(context.Background(), pubKey, txSignerData, sig.Data, signModeHandler, txData)
		if err != nil {
			return err
		}
	}
	return nil
}

// unmarshal unmarshalls a digest to a Tx using protobuf protojson.
func unmarshal(digest []byte, fileFormat string) (*apitx.Tx, error) {
	var err error
	tx := &apitx.Tx{}
	switch fileFormat {
	case v2flags.OutputFormatJSON:
		err = protojson.Unmarshal(digest, tx)
	case v2flags.OutputFormatText:
		err = prototext.Unmarshal(digest, tx)
	default:
		return nil, fmt.Errorf("unsupported file format: %s", fileFormat)
	}
	return tx, err
}

// verifySignature verifies a transaction signature contained in SignatureData abstracting over different signing modes.
func verifySignature(
	ctx context.Context,
	pubKey cryptotypes.PubKey,
	signerData txsigning.SignerData,
	signatureData SignatureData,
	handler *txsigning.HandlerMap,
	txData txsigning.TxData,
) error {
	switch data := signatureData.(type) {
	case *SingleSignatureData:
		signBytes, err := handler.GetSignBytes(ctx, data.SignMode, signerData, txData)
		if err != nil {
			return err
		}
		if !pubKey.VerifySignature(signBytes, data.Signature) {
			return errors.New("unable to verify single signer signature")
		}
		return nil
	default:
		return fmt.Errorf("unexpected SignatureData %T", signatureData)
	}
}
