// crypto is a customized/convenience cryptography package for supporting
// CometBFT.

// It wraps select functionality of equivalent functions in the
// Go standard library, for easy usage with our libraries.

// Keys:

// All key generation functions return an instance of the PrivKey interface
// which implements methods

//     AssertIsPrivKeyInner()
//     Bytes() []byte
//     Sign(msg []byte) Signature
//     PubKey() PubKey
//     Equals(PrivKey) bool
//     Wrap() PrivKey

// From the above method we can:
// a) Retrieve the public key if needed

//     pubKey := key.PubKey()

// For example:
//     privKey, err := ed25519.GenPrivKey()
//     if err != nil {
// 	...
//     }
//     pubKey := privKey.PubKey()
//     ...
//     // And then you can use the private and public key
//     doSomething(privKey, pubKey)

// We also provide hashing wrappers around algorithms:

// Sha256
//     sum := crypto.Sha256([]byte("This is CometBFT"))
//     fmt.Printf("%x\n", sum)

package crypto

// TODO: Add more docs in here
