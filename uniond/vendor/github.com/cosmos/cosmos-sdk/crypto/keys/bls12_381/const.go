package bls12_381

const (
	// PrivKeyName is the name of the private key as it is stored in the keystore.
	PrivKeyName = "cometbft/PrivKeyBls12_381"
	// PubKeyName is the name of the public key as it is stored in the keystore.
	PubKeyName = "cometbft/PubKeyBls12_381"
	// PubKeySize is the size, in bytes, of public keys as used in this package.
	PubKeySize = 32
	// PrivKeySize is the size, in bytes, of private keys as used in this package.
	PrivKeySize = 64
	// SignatureLength defines the byte length of a BLS signature.
	SignatureLength = 96
	// SeedSize is the size, in bytes, of private key seeds. These are the
	// private key representations used by RFC 8032.
	SeedSize = 32
	// MaxMsgLen defines the maximum length of the message bytes as passed to Sign.
	MaxMsgLen = 32
	// KeyType is the type of key this package provides.
	KeyType = "bls12381"
)
