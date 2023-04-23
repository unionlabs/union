package signing

import (
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

// SignModeHandler defines a interface to be implemented by types which will handle
// SignMode's by generating sign bytes from a Tx and SignerData
type SignModeHandler interface {
	// DefaultMode is the default mode that is to be used with this handler if no
	// other mode is specified. This can be useful for testing and CLI usage
	DefaultMode() signing.SignMode

	// Modes is the list of modes supporting by this handler
	Modes() []signing.SignMode

	// GetSignBytes returns the sign bytes for the provided SignMode, SignerData and Tx,
	// or an error
	GetSignBytes(mode signing.SignMode, data SignerData, tx sdk.Tx) ([]byte, error)
}

// SignerData is the specific information needed to sign a transaction that generally
// isn't included in the transaction body itself
type SignerData struct {
	// The address of the signer.
	//
	// In case of multisigs, this should be the multisig's address.
	Address string

	// ChainID is the chain that this transaction is targeted
	ChainID string

	// AccountNumber is the account number of the signer.
	//
	// In case of multisigs, this should be the multisig account number.
	AccountNumber uint64

	// Sequence is the account sequence number of the signer that is used
	// for replay protection. This field is only useful for Legacy Amino signing,
	// since in SIGN_MODE_DIRECT the account sequence is already in the signer
	// info.
	//
	// In case of multisigs, this should be the multisig sequence.
	Sequence uint64

	// PubKey is the public key of the signer.
	//
	// In case of multisigs, this should be the pubkey of the member of the
	// multisig that is signing the current sign doc.
	PubKey cryptotypes.PubKey
}
