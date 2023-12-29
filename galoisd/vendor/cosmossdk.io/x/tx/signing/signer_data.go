package signing

import "google.golang.org/protobuf/types/known/anypb"

// SignerData is the specific information needed to sign a transaction that generally
// isn't included in the transaction body itself
type SignerData struct {
	// The address of the signer.
	//
	// In case of multisigs, this should be the multisig's address.
	Address string

	// ChainID is the chain that this transaction is targeting.
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
	PubKey *anypb.Any
}
