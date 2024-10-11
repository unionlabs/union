// Copyright 2013-2022 The btcsuite developers

package musig2

import (
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"sync"
	"testing"

	"github.com/btcsuite/btcd/btcec/v2"
)

const (
	testVectorBaseDir = "data"
)

func mustParseHex(str string) []byte {
	b, err := hex.DecodeString(str)
	if err != nil {
		panic(fmt.Errorf("unable to parse hex: %v", err))
	}

	return b
}

type signer struct {
	privKey *btcec.PrivateKey
	pubKey  *btcec.PublicKey

	nonces *Nonces

	partialSig *PartialSignature
}

type signerSet []signer

func (s signerSet) keys() []*btcec.PublicKey {
	keys := make([]*btcec.PublicKey, len(s))
	for i := 0; i < len(s); i++ {
		keys[i] = s[i].pubKey
	}

	return keys
}

func (s signerSet) partialSigs() []*PartialSignature {
	sigs := make([]*PartialSignature, len(s))
	for i := 0; i < len(s); i++ {
		sigs[i] = s[i].partialSig
	}

	return sigs
}

func (s signerSet) pubNonces() [][PubNonceSize]byte {
	nonces := make([][PubNonceSize]byte, len(s))
	for i := 0; i < len(s); i++ {
		nonces[i] = s[i].nonces.PubNonce
	}

	return nonces
}

func (s signerSet) combinedKey() *btcec.PublicKey {
	uniqueKeyIndex := secondUniqueKeyIndex(s.keys(), false)
	key, _, _, _ := AggregateKeys(
		s.keys(), false, WithUniqueKeyIndex(uniqueKeyIndex),
	)
	return key.FinalKey
}

// testMultiPartySign executes a multi-party signing context w/ 100 signers.
func testMultiPartySign(t *testing.T, taprootTweak []byte,
	tweaks ...KeyTweakDesc) {

	const numSigners = 100

	// First generate the set of signers along with their public keys.
	signerKeys := make([]*btcec.PrivateKey, numSigners)
	signSet := make([]*btcec.PublicKey, numSigners)
	for i := 0; i < numSigners; i++ {
		privKey, err := btcec.NewPrivateKey()
		if err != nil {
			t.Fatalf("unable to gen priv key: %v", err)
		}

		pubKey := privKey.PubKey()

		signerKeys[i] = privKey
		signSet[i] = pubKey
	}

	var combinedKey *btcec.PublicKey

	var ctxOpts []ContextOption
	switch {
	case len(taprootTweak) == 0:
		ctxOpts = append(ctxOpts, WithBip86TweakCtx())
	case taprootTweak != nil:
		ctxOpts = append(ctxOpts, WithTaprootTweakCtx(taprootTweak))
	case len(tweaks) != 0:
		ctxOpts = append(ctxOpts, WithTweakedContext(tweaks...))
	}

	ctxOpts = append(ctxOpts, WithKnownSigners(signSet))

	// Now that we have all the signers, we'll make a new context, then
	// generate a new session for each of them(which handles nonce
	// generation).
	signers := make([]*Session, numSigners)
	for i, signerKey := range signerKeys {
		signCtx, err := NewContext(
			signerKey, false, ctxOpts...,
		)
		if err != nil {
			t.Fatalf("unable to generate context: %v", err)
		}

		if combinedKey == nil {
			combinedKey, err = signCtx.CombinedKey()
			if err != nil {
				t.Fatalf("combined key not available: %v", err)
			}
		}

		session, err := signCtx.NewSession()
		if err != nil {
			t.Fatalf("unable to generate new session: %v", err)
		}
		signers[i] = session
	}

	// Next, in the pre-signing phase, we'll send all the nonces to each
	// signer.
	var wg sync.WaitGroup
	for i, signCtx := range signers {
		signCtx := signCtx

		wg.Add(1)
		go func(idx int, signer *Session) {
			defer wg.Done()

			for j, otherCtx := range signers {
				if idx == j {
					continue
				}

				nonce := otherCtx.PublicNonce()
				haveAll, err := signer.RegisterPubNonce(nonce)
				if err != nil {
					t.Fatalf("unable to add public nonce")
				}

				if j == len(signers)-1 && !haveAll {
					t.Fatalf("all public nonces should have been detected")
				}
			}
		}(i, signCtx)
	}

	wg.Wait()

	msg := sha256.Sum256([]byte("let's get taprooty"))

	// In the final step, we'll use the first signer as our combiner, and
	// generate a signature for each signer, and then accumulate that with
	// the combiner.
	combiner := signers[0]
	for i := range signers {
		signer := signers[i]
		partialSig, err := signer.Sign(msg)
		if err != nil {
			t.Fatalf("unable to generate partial sig: %v", err)
		}

		// We don't need to combine the signature for the very first
		// signer, as it already has that partial signature.
		if i != 0 {
			haveAll, err := combiner.CombineSig(partialSig)
			if err != nil {
				t.Fatalf("unable to combine sigs: %v", err)
			}

			if i == len(signers)-1 && !haveAll {
				t.Fatalf("final sig wasn't reconstructed")
			}
		}
	}

	// Finally we'll combined all the nonces, and ensure that it validates
	// as a single schnorr signature.
	finalSig := combiner.FinalSig()
	if !finalSig.Verify(msg[:], combinedKey) {
		t.Fatalf("final sig is invalid!")
	}

	// Verify that if we try to sign again with any of the existing
	// signers, then we'll get an error as the nonces have already been
	// used.
	for _, signer := range signers {
		_, err := signer.Sign(msg)
		if err != ErrSigningContextReuse {
			t.Fatalf("expected to get signing context reuse")
		}
	}
}

// TestMuSigMultiParty tests that for a given set of 100 signers, we're able to
// properly generate valid sub signatures, which ultimately can be combined
// into a single valid signature.
func TestMuSigMultiParty(t *testing.T) {
	t.Parallel()

	testTweak := [32]byte{
		0xE8, 0xF7, 0x91, 0xFF, 0x92, 0x25, 0xA2, 0xAF,
		0x01, 0x02, 0xAF, 0xFF, 0x4A, 0x9A, 0x72, 0x3D,
		0x96, 0x12, 0xA6, 0x82, 0xA2, 0x5E, 0xBE, 0x79,
		0x80, 0x2B, 0x26, 0x3C, 0xDF, 0xCD, 0x83, 0xBB,
	}

	t.Run("no_tweak", func(t *testing.T) {
		t.Parallel()

		testMultiPartySign(t, nil)
	})

	t.Run("tweaked", func(t *testing.T) {
		t.Parallel()

		testMultiPartySign(t, nil, KeyTweakDesc{
			Tweak: testTweak,
		})
	})

	t.Run("tweaked_x_only", func(t *testing.T) {
		t.Parallel()

		testMultiPartySign(t, nil, KeyTweakDesc{
			Tweak:   testTweak,
			IsXOnly: true,
		})
	})

	t.Run("taproot_tweaked_x_only", func(t *testing.T) {
		t.Parallel()

		testMultiPartySign(t, testTweak[:])
	})

	t.Run("taproot_bip_86", func(t *testing.T) {
		t.Parallel()

		testMultiPartySign(t, []byte{})
	})
}

// TestMuSigEarlyNonce tests that for protocols where nonces need to be
// exchanged before all signers are known, the context API works as expected.
func TestMuSigEarlyNonce(t *testing.T) {
	t.Parallel()

	privKey1, err := btcec.NewPrivateKey()
	if err != nil {
		t.Fatalf("unable to gen priv key: %v", err)
	}
	privKey2, err := btcec.NewPrivateKey()
	if err != nil {
		t.Fatalf("unable to gen priv key: %v", err)
	}

	// If we try to make a context, with just the private key and sorting
	// value, we should get an error.
	_, err = NewContext(privKey1, true)
	if !errors.Is(err, ErrSignersNotSpecified) {
		t.Fatalf("unexpected ctx error: %v", err)
	}

	signers := []*btcec.PublicKey{privKey1.PubKey(), privKey2.PubKey()}
	numSigners := len(signers)

	ctx1, err := NewContext(
		privKey1, true, WithNumSigners(numSigners), WithEarlyNonceGen(),
	)
	if err != nil {
		t.Fatalf("unable to make ctx: %v", err)
	}
	pubKey1 := ctx1.PubKey()

	ctx2, err := NewContext(
		privKey2, true, WithKnownSigners(signers), WithEarlyNonceGen(),
	)
	if err != nil {
		t.Fatalf("unable to make ctx: %v", err)
	}
	pubKey2 := ctx2.PubKey()

	// At this point, the combined key shouldn't be available for signer 1,
	// but should be for signer 2, as they know about all signers.
	if _, err := ctx1.CombinedKey(); !errors.Is(err, ErrNotEnoughSigners) {
		t.Fatalf("unepxected error: %v", err)
	}
	_, err = ctx2.CombinedKey()
	if err != nil {
		t.Fatalf("unable to get combined key: %v", err)
	}

	// The early nonces _should_ be available at this point.
	nonce1, err := ctx1.EarlySessionNonce()
	if err != nil {
		t.Fatalf("session nonce not available: %v", err)
	}
	nonce2, err := ctx2.EarlySessionNonce()
	if err != nil {
		t.Fatalf("session nonce not available: %v", err)
	}

	// The number of registered signers should still be 1 for both parties.
	if ctx1.NumRegisteredSigners() != 1 {
		t.Fatalf("expected 1 signer, instead have: %v",
			ctx1.NumRegisteredSigners())
	}
	if ctx2.NumRegisteredSigners() != 2 {
		t.Fatalf("expected 2 signers, instead have: %v",
			ctx2.NumRegisteredSigners())
	}

	// If we try to make a session, we should get an error since we dn't
	// have all the signers yet.
	if _, err := ctx1.NewSession(); !errors.Is(err, ErrNotEnoughSigners) {
		t.Fatalf("unexpected session key error: %v", err)
	}

	// The combined key should also be unavailable as well.
	if _, err := ctx1.CombinedKey(); !errors.Is(err, ErrNotEnoughSigners) {
		t.Fatalf("unexpected combined key error: %v", err)
	}

	// We'll now register the other signer for party 1.
	done, err := ctx1.RegisterSigner(&pubKey2)
	if err != nil {
		t.Fatalf("unable to register signer: %v", err)
	}
	if !done {
		t.Fatalf("signer 1 doesn't have all keys")
	}

	// If we try to register the signer again, we should get an error.
	_, err = ctx2.RegisterSigner(&pubKey1)
	if !errors.Is(err, ErrAlreadyHaveAllSigners) {
		t.Fatalf("should not be able to register too many signers")
	}

	// We should be able to create the session at this point.
	session1, err := ctx1.NewSession()
	if err != nil {
		t.Fatalf("unable to create new session: %v", err)
	}
	session2, err := ctx2.NewSession()
	if err != nil {
		t.Fatalf("unable to create new session: %v", err)
	}

	msg := sha256.Sum256([]byte("let's get taprooty, LN style"))

	// If we try to sign before we have the combined nonce, we should get
	// an error.
	_, err = session1.Sign(msg)
	if !errors.Is(err, ErrCombinedNonceUnavailable) {
		t.Fatalf("unable to gen sig: %v", err)
	}

	// Now we can exchange nonces to continue with the rest of the signing
	// process as normal.
	done, err = session1.RegisterPubNonce(nonce2.PubNonce)
	if err != nil {
		t.Fatalf("unable to register nonce: %v", err)
	}
	if !done {
		t.Fatalf("signer 1 doesn't have all nonces")
	}
	done, err = session2.RegisterPubNonce(nonce1.PubNonce)
	if err != nil {
		t.Fatalf("unable to register nonce: %v", err)
	}
	if !done {
		t.Fatalf("signer 2 doesn't have all nonces")
	}

	// Registering the nonce again should error out.
	_, err = session2.RegisterPubNonce(nonce1.PubNonce)
	if !errors.Is(err, ErrAlredyHaveAllNonces) {
		t.Fatalf("shouldn't be able to register nonces twice")
	}

	// Sign the message and combine the two partial sigs into one.
	_, err = session1.Sign(msg)
	if err != nil {
		t.Fatalf("unable to gen sig: %v", err)
	}
	sig2, err := session2.Sign(msg)
	if err != nil {
		t.Fatalf("unable to gen sig: %v", err)
	}
	done, err = session1.CombineSig(sig2)
	if err != nil {
		t.Fatalf("unable to combine sig: %v", err)
	}
	if !done {
		t.Fatalf("all sigs should be known now: %v", err)
	}

	// If we try to combine another sig, then we should get an error.
	_, err = session1.CombineSig(sig2)
	if !errors.Is(err, ErrAlredyHaveAllSigs) {
		t.Fatalf("shouldn't be able to combine again")
	}

	// Finally, verify that the final signature is valid.
	combinedKey, err := ctx1.CombinedKey()
	if err != nil {
		t.Fatalf("unexpected combined key error: %v", err)
	}
	finalSig := session1.FinalSig()
	if !finalSig.Verify(msg[:], combinedKey) {
		t.Fatalf("final sig is invalid!")
	}
}

type memsetRandReader struct {
	i int
}

func (mr *memsetRandReader) Read(buf []byte) (n int, err error) {
	for i := range buf {
		buf[i] = byte(mr.i)
	}
	return len(buf), nil
}
