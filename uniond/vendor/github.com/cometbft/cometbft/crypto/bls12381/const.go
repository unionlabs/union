package bls12381

const (
	// PrivKeySize defines the length of the PrivKey byte array.
	PrivKeySize = 32
	// PubKeySize defines the length of the PubKey byte array.
	PubKeySize = 96
	// SignatureLength defines the byte length of a BLS signature.
	SignatureLength = 96
	// KeyType is the string constant for the BLS12-381 algorithm.
	KeyType = "bls12_381"
	// MaxMsgLen defines the maximum length of the message bytes as passed to Sign.
	MaxMsgLen = 32
	// BLS12-381 private key name.
	PrivKeyName = "cometbft/PrivKeyBls12_381"
	// BLS12-381 public key name.
	PubKeyName = "cometbft/PubKeyBls12_381"
)
