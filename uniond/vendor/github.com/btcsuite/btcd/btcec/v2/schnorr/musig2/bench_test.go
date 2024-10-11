// Copyright 2013-2022 The btcsuite developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package musig2

import (
	"encoding/hex"
	"fmt"
	"testing"

	"github.com/btcsuite/btcd/btcec/v2"
	"github.com/btcsuite/btcd/btcec/v2/schnorr"
)

var (
	testPrivBytes = hexToModNScalar("9e0699c91ca1e3b7e3c9ba71eb71c89890872be97576010fe593fbf3fd57e66d")

	testMsg = hexToBytes("c301ba9de5d6053caad9f5eb46523f007702add2c62fa39de03146a36b8026b7")
)

func hexToBytes(s string) []byte {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic("invalid hex in source file: " + s)
	}
	return b
}

func hexToModNScalar(s string) *btcec.ModNScalar {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic("invalid hex in source file: " + s)
	}
	var scalar btcec.ModNScalar
	if overflow := scalar.SetByteSlice(b); overflow {
		panic("hex in source file overflows mod N scalar: " + s)
	}
	return &scalar
}

func genSigner(t *testing.B) signer {
	privKey, err := btcec.NewPrivateKey()
	if err != nil {
		t.Fatalf("unable to gen priv key: %v", err)
	}

	pubKey := privKey.PubKey()

	nonces, err := GenNonces(WithPublicKey(pubKey))
	if err != nil {
		t.Fatalf("unable to gen nonces: %v", err)
	}

	return signer{
		privKey: privKey,
		pubKey:  pubKey,
		nonces:  nonces,
	}
}

var (
	testSig *PartialSignature
	testErr error
)

// BenchmarkPartialSign benchmarks how long it takes to generate a partial
// signature factoring in if the keys are sorted and also if we're in fast sign
// mode.
func BenchmarkPartialSign(b *testing.B) {
	for _, numSigners := range []int{10, 100} {
		for _, fastSign := range []bool{true, false} {
			for _, sortKeys := range []bool{true, false} {
				name := fmt.Sprintf("num_signers=%v/fast_sign=%v/sort=%v",
					numSigners, fastSign, sortKeys)

				signers := make(signerSet, numSigners)
				for i := 0; i < numSigners; i++ {
					signers[i] = genSigner(b)
				}

				combinedNonce, err := AggregateNonces(signers.pubNonces())
				if err != nil {
					b.Fatalf("unable to generate combined nonce: %v", err)
				}

				var sig *PartialSignature

				var msg [32]byte
				copy(msg[:], testMsg[:])

				keys := signers.keys()

				b.Run(name, func(b *testing.B) {
					var signOpts []SignOption
					if fastSign {
						signOpts = append(signOpts, WithFastSign())
					}
					if sortKeys {
						signOpts = append(signOpts, WithSortedKeys())
					}

					b.ResetTimer()
					b.ReportAllocs()

					for i := 0; i < b.N; i++ {
						sig, err = Sign(
							signers[0].nonces.SecNonce, signers[0].privKey,
							combinedNonce, keys, msg, signOpts...,
						)
						if err != nil {
							b.Fatalf("unable to generate sig: %v", err)
						}
					}

					testSig = sig
					testErr = err
				})
			}
		}
	}
}

// TODO(roasbeef): add impact of sorting ^

var sigOk bool

// BenchmarkPartialVerify benchmarks how long it takes to verify a partial
// signature.
func BenchmarkPartialVerify(b *testing.B) {
	for _, numSigners := range []int{10, 100} {
		for _, sortKeys := range []bool{true, false} {
			name := fmt.Sprintf("sort_keys=%v/num_signers=%v",
				sortKeys, numSigners)

			signers := make(signerSet, numSigners)
			for i := 0; i < numSigners; i++ {
				signers[i] = genSigner(b)
			}

			combinedNonce, err := AggregateNonces(
				signers.pubNonces(),
			)
			if err != nil {
				b.Fatalf("unable to generate combined "+
					"nonce: %v", err)
			}

			var sig *PartialSignature

			var msg [32]byte
			copy(msg[:], testMsg[:])

			b.ReportAllocs()
			b.ResetTimer()

			sig, err = Sign(
				signers[0].nonces.SecNonce, signers[0].privKey,
				combinedNonce, signers.keys(), msg,
			)
			if err != nil {
				b.Fatalf("unable to generate sig: %v", err)
			}

			keys := signers.keys()
			pubKey := signers[0].pubKey

			b.Run(name, func(b *testing.B) {
				var signOpts []SignOption
				if sortKeys {
					signOpts = append(
						signOpts, WithSortedKeys(),
					)
				}

				b.ResetTimer()
				b.ReportAllocs()

				var ok bool
				for i := 0; i < b.N; i++ {
					ok = sig.Verify(
						signers[0].nonces.PubNonce, combinedNonce,
						keys, pubKey, msg, signOpts...,
					)
					if !ok {
						b.Fatalf("generated invalid sig!")
					}
				}
				sigOk = ok
			})

		}
	}
}

var finalSchnorrSig *schnorr.Signature

// BenchmarkCombineSigs benchmarks how long it takes to combine a set amount of
// signatures.
func BenchmarkCombineSigs(b *testing.B) {

	for _, numSigners := range []int{10, 100} {
		signers := make(signerSet, numSigners)
		for i := 0; i < numSigners; i++ {
			signers[i] = genSigner(b)
		}

		combinedNonce, err := AggregateNonces(signers.pubNonces())
		if err != nil {
			b.Fatalf("unable to generate combined nonce: %v", err)
		}

		var msg [32]byte
		copy(msg[:], testMsg[:])

		var finalNonce *btcec.PublicKey
		for i := range signers {
			signer := signers[i]
			partialSig, err := Sign(
				signer.nonces.SecNonce, signer.privKey,
				combinedNonce, signers.keys(), msg,
			)
			if err != nil {
				b.Fatalf("unable to generate partial sig: %v",
					err)
			}

			signers[i].partialSig = partialSig

			if finalNonce == nil {
				finalNonce = partialSig.R
			}
		}

		sigs := signers.partialSigs()

		name := fmt.Sprintf("num_signers=%v", numSigners)
		b.Run(name, func(b *testing.B) {
			b.ResetTimer()
			b.ReportAllocs()

			finalSig := CombineSigs(finalNonce, sigs)

			finalSchnorrSig = finalSig
		})
	}
}

var testNonce [PubNonceSize]byte

// BenchmarkAggregateNonces benchmarks how long it takes to combine nonces.
func BenchmarkAggregateNonces(b *testing.B) {
	for _, numSigners := range []int{10, 100} {
		signers := make(signerSet, numSigners)
		for i := 0; i < numSigners; i++ {
			signers[i] = genSigner(b)
		}

		nonces := signers.pubNonces()

		name := fmt.Sprintf("num_signers=%v", numSigners)
		b.Run(name, func(b *testing.B) {
			b.ResetTimer()
			b.ReportAllocs()

			pubNonce, err := AggregateNonces(nonces)
			if err != nil {
				b.Fatalf("unable to generate nonces: %v", err)
			}

			testNonce = pubNonce
		})
	}
}

var testKey *btcec.PublicKey

// BenchmarkAggregateKeys benchmarks how long it takes to aggregate public
// keys.
func BenchmarkAggregateKeys(b *testing.B) {
	for _, numSigners := range []int{10, 100} {
		for _, sortKeys := range []bool{true, false} {
			signers := make(signerSet, numSigners)
			for i := 0; i < numSigners; i++ {
				signers[i] = genSigner(b)
			}

			signerKeys := signers.keys()

			name := fmt.Sprintf("num_signers=%v/sort_keys=%v",
				numSigners, sortKeys)

			uniqueKeyIndex := secondUniqueKeyIndex(signerKeys, false)

			b.Run(name, func(b *testing.B) {
				b.ResetTimer()
				b.ReportAllocs()

				aggKey, _, _, _ := AggregateKeys(
					signerKeys, sortKeys,
					WithUniqueKeyIndex(uniqueKeyIndex),
				)

				testKey = aggKey.FinalKey
			})
		}
	}
}
