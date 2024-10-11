// Copyright 2013-2016 The btcsuite developers
// Copyright (c) 2015-2022 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package ecdsa

import (
	"encoding/hex"
	"testing"

	"github.com/decred/dcrd/dcrec/secp256k1/v4"
)

// hexToModNScalar converts the passed hex string into a ModNScalar and will
// panic if there is an error.  This is only provided for the hard-coded
// constants so errors in the source code can be detected. It will only (and
// must only) be called with hard-coded values.
func hexToModNScalar(s string) *secp256k1.ModNScalar {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic("invalid hex in source file: " + s)
	}
	var scalar secp256k1.ModNScalar
	if overflow := scalar.SetByteSlice(b); overflow {
		panic("hex in source file overflows mod N scalar: " + s)
	}
	return &scalar
}

// hexToFieldVal converts the passed hex string into a FieldVal and will panic
// if there is an error.  This is only provided for the hard-coded constants so
// errors in the source code can be detected. It will only (and must only) be
// called with hard-coded values.
func hexToFieldVal(s string) *secp256k1.FieldVal {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic("invalid hex in source file: " + s)
	}
	var f secp256k1.FieldVal
	if overflow := f.SetByteSlice(b); overflow {
		panic("hex in source file overflows mod P: " + s)
	}
	return &f
}

// BenchmarkSigVerify benchmarks how long it takes the secp256k1 curve to
// verify signatures.
func BenchmarkSigVerify(b *testing.B) {
	// Randomly generated keypair.
	// Private key: 9e0699c91ca1e3b7e3c9ba71eb71c89890872be97576010fe593fbf3fd57e66d
	pubKey := secp256k1.NewPublicKey(
		hexToFieldVal("d2e670a19c6d753d1a6d8b20bd045df8a08fb162cf508956c31268c6d81ffdab"),
		hexToFieldVal("ab65528eefbb8057aa85d597258a3fbd481a24633bc9b47a9aa045c91371de52"),
	)

	// Double sha256 of []byte{0x01, 0x02, 0x03, 0x04}
	msgHash := hexToBytes("8de472e2399610baaa7f84840547cd409434e31f5d3bd71e4d947f283874f9c0")
	sig := NewSignature(
		hexToModNScalar("fef45d2892953aa5bbcdb057b5e98b208f1617a7498af7eb765574e29b5d9c2c"),
		hexToModNScalar("d47563f52aac6b04b55de236b7c515eb9311757db01e02cff079c3ca6efb063f"),
	)

	if !sig.Verify(msgHash, pubKey) {
		b.Errorf("Signature failed to verify")
		return
	}

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		sig.Verify(msgHash, pubKey)
	}
}

// BenchmarkSign benchmarks how long it takes to sign a message.
func BenchmarkSign(b *testing.B) {
	// Randomly generated keypair.
	d := hexToModNScalar("9e0699c91ca1e3b7e3c9ba71eb71c89890872be97576010fe593fbf3fd57e66d")
	privKey := secp256k1.NewPrivateKey(d)

	// blake256 of []byte{0x01, 0x02, 0x03, 0x04}.
	msgHash := hexToBytes("c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7")

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		signRFC6979(privKey, msgHash)
	}
}

// BenchmarkSigSerialize benchmarks how long it takes to serialize a typical
// signature with the strict DER encoding.
func BenchmarkSigSerialize(b *testing.B) {
	// Randomly generated keypair.
	// Private key: 9e0699c91ca1e3b7e3c9ba71eb71c89890872be97576010fe593fbf3fd57e66d
	// Signature for double sha256 of []byte{0x01, 0x02, 0x03, 0x04}.
	sig := NewSignature(
		hexToModNScalar("fef45d2892953aa5bbcdb057b5e98b208f1617a7498af7eb765574e29b5d9c2c"),
		hexToModNScalar("d47563f52aac6b04b55de236b7c515eb9311757db01e02cff079c3ca6efb063f"),
	)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		sig.Serialize()
	}
}

// BenchmarkNonceRFC6979 benchmarks how long it takes to generate a
// deterministic nonce according to RFC6979.
func BenchmarkNonceRFC6979(b *testing.B) {
	// Randomly generated keypair.
	// Private key: 9e0699c91ca1e3b7e3c9ba71eb71c89890872be97576010fe593fbf3fd57e66d
	// X: d2e670a19c6d753d1a6d8b20bd045df8a08fb162cf508956c31268c6d81ffdab
	// Y: ab65528eefbb8057aa85d597258a3fbd481a24633bc9b47a9aa045c91371de52
	privKeyStr := "9e0699c91ca1e3b7e3c9ba71eb71c89890872be97576010fe593fbf3fd57e66d"
	privKey := hexToBytes(privKeyStr)

	// BLAKE-256 of []byte{0x01, 0x02, 0x03, 0x04}.
	msgHash := hexToBytes("c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7")

	b.ReportAllocs()
	b.ResetTimer()
	var noElideNonce *secp256k1.ModNScalar
	for i := 0; i < b.N; i++ {
		noElideNonce = secp256k1.NonceRFC6979(privKey, msgHash, nil, nil, 0)
	}
	_ = noElideNonce
}

// BenchmarkSignCompact benchmarks how long it takes to produce a compact
// signature for a message.
func BenchmarkSignCompact(b *testing.B) {
	d := hexToModNScalar("9e0699c91ca1e3b7e3c9ba71eb71c89890872be97576010fe593fbf3fd57e66d")
	privKey := secp256k1.NewPrivateKey(d)

	// blake256 of []byte{0x01, 0x02, 0x03, 0x04}.
	msgHash := hexToBytes("c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7")

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = SignCompact(privKey, msgHash, true)
	}
}

// BenchmarkRecoverCompact benchmarks how long it takes to recover a public key
// given a compact signature and message.
func BenchmarkRecoverCompact(b *testing.B) {
	// Private key: 9e0699c91ca1e3b7e3c9ba71eb71c89890872be97576010fe593fbf3fd57e66d
	wantPubKey := secp256k1.NewPublicKey(
		hexToFieldVal("d2e670a19c6d753d1a6d8b20bd045df8a08fb162cf508956c31268c6d81ffdab"),
		hexToFieldVal("ab65528eefbb8057aa85d597258a3fbd481a24633bc9b47a9aa045c91371de52"),
	)

	compactSig := hexToBytes("205978b7896bc71676ba2e459882a8f52e1299449596c4f" +
		"93c59bf1fbfa2f9d3b76ecd0c99406f61a6de2bb5a8937c061c176ecf381d0231e0d" +
		"af73b922c8952c7")

	// blake256 of []byte{0x01, 0x02, 0x03, 0x04}.
	msgHash := hexToBytes("c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7")

	// Ensure a valid compact signature is being benchmarked.
	pubKey, wasCompressed, err := RecoverCompact(compactSig, msgHash)
	if err != nil {
		b.Fatalf("unexpected err: %v", err)
	}
	if !wasCompressed {
		b.Fatal("recover claims uncompressed pubkey")
	}
	if !pubKey.IsEqual(wantPubKey) {
		b.Fatal("recover returned unexpected pubkey")
	}

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _, _ = RecoverCompact(compactSig, msgHash)
	}
}
