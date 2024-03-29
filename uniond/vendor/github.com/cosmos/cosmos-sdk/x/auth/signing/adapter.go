package signing

import (
	"context"
	"fmt"

	"google.golang.org/protobuf/types/known/anypb"

	txsigning "cosmossdk.io/x/tx/signing"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

// V2AdaptableTx is an interface that wraps the GetSigningTxData method.
// GetSigningTxData returns an x/tx/signing.TxData representation of a transaction for use in signing
// interoperability with x/tx.
type V2AdaptableTx interface {
	GetSigningTxData() txsigning.TxData
}

// GetSignBytesAdapter returns the sign bytes for a given transaction and sign mode.  It accepts the arguments expected
// for signing in x/auth/tx and converts them to the arguments expected by the txsigning.HandlerMap, then applies
// HandlerMap.GetSignBytes to get the sign bytes.
func GetSignBytesAdapter(
	ctx context.Context,
	handlerMap *txsigning.HandlerMap,
	mode signing.SignMode,
	signerData SignerData,
	tx sdk.Tx,
) ([]byte, error) {
	adaptableTx, ok := tx.(V2AdaptableTx)
	if !ok {
		return nil, fmt.Errorf("expected tx to be V2AdaptableTx, got %T", tx)
	}
	txData := adaptableTx.GetSigningTxData()

	txSignMode, err := internalSignModeToAPI(mode)
	if err != nil {
		return nil, err
	}

	var pubKey *anypb.Any
	if signerData.PubKey != nil {
		anyPk, err := codectypes.NewAnyWithValue(signerData.PubKey)
		if err != nil {
			return nil, err
		}

		pubKey = &anypb.Any{
			TypeUrl: anyPk.TypeUrl,
			Value:   anyPk.Value,
		}
	}
	txSignerData := txsigning.SignerData{
		ChainID:       signerData.ChainID,
		AccountNumber: signerData.AccountNumber,
		Sequence:      signerData.Sequence,
		Address:       signerData.Address,
		PubKey:        pubKey,
	}
	// Generate the bytes to be signed.
	return handlerMap.GetSignBytes(ctx, txSignMode, txSignerData, txData)
}
