// Copyright (c) 2014 The btcsuite developers
// Copyright (c) 2015-2020 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package secp256k1_test

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/sha256"
	"encoding/binary"
	"encoding/hex"
	"fmt"

	"github.com/decred/dcrd/dcrec/secp256k1/v4"
)

// This example demonstrates use of GenerateSharedSecret to encrypt a message
// for a recipient's public key, and subsequently decrypt the message using the
// recipient's private key.
func Example_encryptDecryptMessage() {
	newAEAD := func(key []byte) (cipher.AEAD, error) {
		block, err := aes.NewCipher(key)
		if err != nil {
			return nil, err
		}
		return cipher.NewGCM(block)
	}

	// Decode the hex-encoded pubkey of the recipient.
	pubKeyBytes, err := hex.DecodeString("04115c42e757b2efb7671c578530ec191a1" +
		"359381e6a71127a9d37c486fd30dae57e76dc58f693bd7e7010358ce6b165e483a29" +
		"21010db67ac11b1b51b651953d2") // uncompressed pubkey
	if err != nil {
		fmt.Println(err)
		return
	}
	pubKey, err := secp256k1.ParsePubKey(pubKeyBytes)
	if err != nil {
		fmt.Println(err)
		return
	}

	// Derive an ephemeral public/private keypair for performing ECDHE with
	// the recipient.
	ephemeralPrivKey, err := secp256k1.GeneratePrivateKey()
	if err != nil {
		fmt.Println(err)
		return
	}
	ephemeralPubKey := ephemeralPrivKey.PubKey().SerializeCompressed()

	// Using ECDHE, derive a shared symmetric key for encryption of the plaintext.
	cipherKey := sha256.Sum256(secp256k1.GenerateSharedSecret(ephemeralPrivKey, pubKey))

	// Seal the message using an AEAD.  Here we use AES-256-GCM.
	// The ephemeral public key must be included in this message, and becomes
	// the authenticated data for the AEAD.
	//
	// Note that unless a unique nonce can be guaranteed, the ephemeral
	// and/or shared keys must not be reused to encrypt different messages.
	// Doing so destroys the security of the scheme.  Random nonces may be
	// used if XChaCha20-Poly1305 is used instead, but the message must then
	// also encode the nonce (which we don't do here).
	//
	// Since a new ephemeral key is generated for every message ensuring there
	// is no key reuse and AES-GCM permits the nonce to be used as a counter,
	// the nonce is intentionally initialized to all zeros so it acts like the
	// first (and only) use of a counter.
	plaintext := []byte("test message")
	aead, err := newAEAD(cipherKey[:])
	if err != nil {
		fmt.Println(err)
		return
	}
	nonce := make([]byte, aead.NonceSize())
	ciphertext := make([]byte, 4+len(ephemeralPubKey))
	binary.LittleEndian.PutUint32(ciphertext, uint32(len(ephemeralPubKey)))
	copy(ciphertext[4:], ephemeralPubKey)
	ciphertext = aead.Seal(ciphertext, nonce, plaintext, ephemeralPubKey)

	// The remainder of this example is performed by the recipient on the
	// ciphertext shared by the sender.

	// Decode the hex-encoded private key.
	pkBytes, err := hex.DecodeString("a11b0a4e1a132305652ee7a8eb7848f6ad" +
		"5ea381e3ce20a2c086a2e388230811")
	if err != nil {
		fmt.Println(err)
		return
	}
	privKey := secp256k1.PrivKeyFromBytes(pkBytes)

	// Read the sender's ephemeral public key from the start of the message.
	// Error handling for inappropriate pubkey lengths is elided here for
	// brevity.
	pubKeyLen := binary.LittleEndian.Uint32(ciphertext[:4])
	senderPubKeyBytes := ciphertext[4 : 4+pubKeyLen]
	senderPubKey, err := secp256k1.ParsePubKey(senderPubKeyBytes)
	if err != nil {
		fmt.Println(err)
		return
	}

	// Derive the key used to seal the message, this time from the
	// recipient's private key and the sender's public key.
	recoveredCipherKey := sha256.Sum256(secp256k1.GenerateSharedSecret(privKey, senderPubKey))

	// Open the sealed message.
	aead, err = newAEAD(recoveredCipherKey[:])
	if err != nil {
		fmt.Println(err)
		return
	}
	nonce = make([]byte, aead.NonceSize())
	recoveredPlaintext, err := aead.Open(nil, nonce, ciphertext[4+pubKeyLen:], senderPubKeyBytes)
	if err != nil {
		fmt.Println(err)
		return
	}

	fmt.Println(string(recoveredPlaintext))

	// Output:
	// test message
}
