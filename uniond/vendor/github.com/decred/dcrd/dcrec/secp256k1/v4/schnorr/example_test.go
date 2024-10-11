// Copyright (c) 2020-2021 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package schnorr_test

import (
	"encoding/hex"
	"fmt"

	"github.com/decred/dcrd/crypto/blake256"
	"github.com/decred/dcrd/dcrec/secp256k1/v4"
	"github.com/decred/dcrd/dcrec/secp256k1/v4/schnorr"
)

// This example demonstrates signing a message with the EC-Schnorr-DCRv0 scheme
// using a secp256k1 private key that is first parsed from raw bytes and
// serializing the generated signature.
func ExampleSign() {
	// Decode a hex-encoded private key.
	pkBytes, err := hex.DecodeString("22a47fa09a223f2aa079edf85a7c2d4f8720ee6" +
		"3e502ee2869afab7de234b80c")
	if err != nil {
		fmt.Println(err)
		return
	}
	privKey := secp256k1.PrivKeyFromBytes(pkBytes)

	// Sign a message using the private key.
	message := "test message"
	messageHash := blake256.Sum256([]byte(message))
	signature, err := schnorr.Sign(privKey, messageHash[:])
	if err != nil {
		fmt.Println(err)
		return
	}

	// Serialize and display the signature.
	fmt.Printf("Serialized Signature: %x\n", signature.Serialize())

	// Verify the signature for the message using the public key.
	pubKey := privKey.PubKey()
	verified := signature.Verify(messageHash[:], pubKey)
	fmt.Printf("Signature Verified? %v\n", verified)

	// Output:
	// Serialized Signature: 970603d8ccd2475b1ff66cfb3ce7e622c5938348304c5a7bc2e6015fb98e3b457d4e912fcca6ca87c04390aa5e6e0e613bbbba7ffd6f15bc59f95bbd92ba50f0
	// Signature Verified? true
}

// This example demonstrates verifying an EC-Schnorr-DCRv0 signature against a
// public key that is first parsed from raw bytes.  The signature is also parsed
// from raw bytes.
func ExampleSignature_Verify() {
	// Decode hex-encoded serialized public key.
	pubKeyBytes, err := hex.DecodeString("02a673638cb9587cb68ea08dbef685c6f2d" +
		"2a751a8b3c6f2a7e9a4999e6e4bfaf5")
	if err != nil {
		fmt.Println(err)
		return
	}
	pubKey, err := schnorr.ParsePubKey(pubKeyBytes)
	if err != nil {
		fmt.Println(err)
		return
	}

	// Decode hex-encoded serialized signature.
	sigBytes, err := hex.DecodeString("970603d8ccd2475b1ff66cfb3ce7e622c59383" +
		"48304c5a7bc2e6015fb98e3b457d4e912fcca6ca87c04390aa5e6e0e613bbbba7ffd" +
		"6f15bc59f95bbd92ba50f0")
	if err != nil {
		fmt.Println(err)
		return
	}
	signature, err := schnorr.ParseSignature(sigBytes)
	if err != nil {
		fmt.Println(err)
		return
	}

	// Verify the signature for the message using the public key.
	message := "test message"
	messageHash := blake256.Sum256([]byte(message))
	verified := signature.Verify(messageHash[:], pubKey)
	fmt.Println("Signature Verified?", verified)

	// Output:
	// Signature Verified? true
}
