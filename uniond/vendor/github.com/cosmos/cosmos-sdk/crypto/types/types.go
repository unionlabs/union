package types

import (
	cmtcrypto "github.com/cometbft/cometbft/crypto"
	proto "github.com/cosmos/gogoproto/proto"
)

// PubKey defines a public key and extends proto.Message.
type PubKey interface {
	proto.Message

	Address() Address
	Bytes() []byte
	VerifySignature(msg, sig []byte) bool
	Equals(PubKey) bool
	Type() string
}

// LedgerPrivKey defines a private key that is not a proto message. For now,
// LedgerSecp256k1 keys are not converted to proto.Message yet, this is why
// they use LedgerPrivKey instead of PrivKey. All other keys must use PrivKey
// instead of LedgerPrivKey.
// TODO https://github.com/cosmos/cosmos-sdk/issues/7357.
type LedgerPrivKey interface {
	Bytes() []byte
	Sign(msg []byte) ([]byte, error)
	PubKey() PubKey
	Equals(LedgerPrivKey) bool
	Type() string
}

// LedgerPrivKeyAminoJSON is a Ledger PrivKey type that supports signing with
// SIGN_MODE_LEGACY_AMINO_JSON. It is added as a non-breaking change, instead of directly
// on the LedgerPrivKey interface (whose Sign method will sign with TEXTUAL),
// and will be deprecated/removed once LEGACY_AMINO_JSON is removed.
type LedgerPrivKeyAminoJSON interface {
	LedgerPrivKey
	// SignLedgerAminoJSON signs a messages on the Ledger device using
	// SIGN_MODE_LEGACY_AMINO_JSON.
	SignLedgerAminoJSON(msg []byte) ([]byte, error)
}

// PrivKey defines a private key and extends proto.Message. For now, it extends
// LedgerPrivKey (see godoc for LedgerPrivKey). Ultimately, we should remove
// LedgerPrivKey and add its methods here directly.
// TODO https://github.com/cosmos/cosmos-sdk/issues/7357.
type PrivKey interface {
	proto.Message
	LedgerPrivKey
}

type (
	Address = cmtcrypto.Address
)
