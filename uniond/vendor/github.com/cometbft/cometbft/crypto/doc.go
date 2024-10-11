// crypto is a customized/convenience cryptography package for CometBFT.
//
// It wraps select functionality of equivalent functions in the
// Go standard library, for easy usage with our libraries.
//
// Keys:
//
// All key generation functions return an instance of the PrivKey interface
// which implements methods:
//
//	type PrivKey interface {
//		Bytes() []byte
//		Sign(msg []byte) ([]byte, error)
//		PubKey() PubKey
//		Type() string
//	}
//
// From the above method we can retrieve the public key if needed:
//
//	privKey, err := ed25519.GenPrivKey()
//	if err != nil {
//		panic(err)
//	}
//	pubKey := privKey.PubKey()
//
// The resulting public key is an instance of the PubKey interface:
//
//	type PubKey interface {
//		Address() Address
//		Bytes() []byte
//		VerifySignature(msg []byte, sig []byte) bool
//		Type() string
//	}
package crypto
