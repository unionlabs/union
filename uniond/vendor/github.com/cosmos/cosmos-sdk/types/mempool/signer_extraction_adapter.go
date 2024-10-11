package mempool

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
)

// SignerData contains canonical useful information about the signer of a transaction
type SignerData struct {
	Signer   sdk.AccAddress
	Sequence uint64
}

// NewSignerData returns a new SignerData instance.
func NewSignerData(signer sdk.AccAddress, sequence uint64) SignerData {
	return SignerData{
		Signer:   signer,
		Sequence: sequence,
	}
}

// String implements the fmt.Stringer interface.
func (s SignerData) String() string {
	return fmt.Sprintf("SignerData{Signer: %s, Sequence: %d}", s.Signer, s.Sequence)
}

// SignerExtractionAdapter is an interface used to determine how the signers of a transaction should be extracted
// from the transaction.
type SignerExtractionAdapter interface {
	GetSigners(sdk.Tx) ([]SignerData, error)
}

var _ SignerExtractionAdapter = DefaultSignerExtractionAdapter{}

// DefaultSignerExtractionAdapter is the default implementation of SignerExtractionAdapter. It extracts the signers
// from a cosmos-sdk tx via GetSignaturesV2.
type DefaultSignerExtractionAdapter struct{}

// NewDefaultSignerExtractionAdapter constructs a new DefaultSignerExtractionAdapter instance
func NewDefaultSignerExtractionAdapter() DefaultSignerExtractionAdapter {
	return DefaultSignerExtractionAdapter{}
}

// GetSigners implements the Adapter interface
func (DefaultSignerExtractionAdapter) GetSigners(tx sdk.Tx) ([]SignerData, error) {
	sigTx, ok := tx.(signing.SigVerifiableTx)
	if !ok {
		return nil, fmt.Errorf("tx of type %T does not implement SigVerifiableTx", tx)
	}

	sigs, err := sigTx.GetSignaturesV2()
	if err != nil {
		return nil, err
	}

	signers := make([]SignerData, len(sigs))
	for i, sig := range sigs {
		signers[i] = NewSignerData(
			sig.PubKey.Address().Bytes(),
			sig.Sequence,
		)
	}

	return signers, nil
}
