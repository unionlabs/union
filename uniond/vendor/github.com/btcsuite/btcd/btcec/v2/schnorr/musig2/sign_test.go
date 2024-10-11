// Copyright 2013-2022 The btcsuite developers

package musig2

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path"
	"strings"
	"testing"

	"github.com/btcsuite/btcd/btcec/v2"
	secp "github.com/decred/dcrd/dcrec/secp256k1/v4"
	"github.com/stretchr/testify/require"
)

const (
	signVerifyTestVectorFileName = "sign_verify_vectors.json"

	sigCombineTestVectorFileName = "sig_agg_vectors.json"
)

type signVerifyValidCase struct {
	Indices []int `json:"key_indices"`

	NonceIndices []int `json:"nonce_indices"`

	AggNonceIndex int `json:"aggnonce_index"`

	MsgIndex int `json:"msg_index"`

	SignerIndex int `json:"signer_index"`

	Expected string `json:"expected"`
}

type signErrorCase struct {
	Indices []int `json:"key_indices"`

	AggNonceIndex int `json:"aggnonce_index"`

	MsgIndex int `json:"msg_index"`

	SecNonceIndex int `json:"secnonce_index"`

	Comment string `json:"comment"`
}

type verifyFailCase struct {
	Sig string `json:"sig"`

	Indices []int `json:"key_indices"`

	NonceIndices []int `json:"nonce_indices"`

	MsgIndex int `json:"msg_index"`

	SignerIndex int `json:"signer_index"`

	Comment string `json:"comment"`
}

type verifyErrorCase struct {
	Sig string `json:"sig"`

	Indices []int `json:"key_indices"`

	NonceIndices []int `json:"nonce_indices"`

	MsgIndex int `json:"msg_index"`

	SignerIndex int `json:"signer_index"`

	Comment string `json:"comment"`
}

type signVerifyTestVectors struct {
	PrivKey string `json:"sk"`

	PubKeys []string `json:"pubkeys"`

	PrivNonces []string `json:"secnonces"`

	PubNonces []string `json:"pnonces"`

	AggNonces []string `json:"aggnonces"`

	Msgs []string `json:"msgs"`

	ValidCases []signVerifyValidCase `json:"valid_test_cases"`

	SignErrorCases []signErrorCase `json:"sign_error_test_cases"`

	VerifyFailCases []verifyFailCase `json:"verify_fail_test_cases"`

	VerifyErrorCases []verifyErrorCase `json:"verify_error_test_cases"`
}

// TestMusig2SignVerify tests that we pass the musig2 verification tests.
func TestMusig2SignVerify(t *testing.T) {
	t.Parallel()

	testVectorPath := path.Join(
		testVectorBaseDir, signVerifyTestVectorFileName,
	)
	testVectorBytes, err := os.ReadFile(testVectorPath)
	require.NoError(t, err)

	var testCases signVerifyTestVectors
	require.NoError(t, json.Unmarshal(testVectorBytes, &testCases))

	privKey, _ := btcec.PrivKeyFromBytes(mustParseHex(testCases.PrivKey))

	for i, testCase := range testCases.ValidCases {
		testCase := testCase

		testName := fmt.Sprintf("valid_case_%v", i)
		t.Run(testName, func(t *testing.T) {
			pubKeys, err := keysFromIndices(
				t, testCase.Indices, testCases.PubKeys,
			)
			require.NoError(t, err)

			pubNonces := pubNoncesFromIndices(
				t, testCase.NonceIndices, testCases.PubNonces,
			)

			combinedNonce, err := AggregateNonces(pubNonces)
			require.NoError(t, err)

			var msg [32]byte
			copy(msg[:], mustParseHex(testCases.Msgs[testCase.MsgIndex]))

			var secNonce [SecNonceSize]byte
			copy(secNonce[:], mustParseHex(testCases.PrivNonces[0]))

			partialSig, err := Sign(
				secNonce, privKey, combinedNonce, pubKeys,
				msg,
			)

			var partialSigBytes [32]byte
			partialSig.S.PutBytesUnchecked(partialSigBytes[:])

			require.Equal(
				t, hex.EncodeToString(partialSigBytes[:]),
				hex.EncodeToString(mustParseHex(testCase.Expected)),
			)
		})
	}

	for _, testCase := range testCases.SignErrorCases {
		testCase := testCase

		testName := fmt.Sprintf("invalid_case_%v",
			strings.ToLower(testCase.Comment))

		t.Run(testName, func(t *testing.T) {
			pubKeys, err := keysFromIndices(
				t, testCase.Indices, testCases.PubKeys,
			)
			if err != nil {
				require.ErrorIs(t, err, secp.ErrPubKeyNotOnCurve)
				return
			}

			var aggNonce [PubNonceSize]byte
			copy(
				aggNonce[:],
				mustParseHex(
					testCases.AggNonces[testCase.AggNonceIndex],
				),
			)

			var msg [32]byte
			copy(msg[:], mustParseHex(testCases.Msgs[testCase.MsgIndex]))

			var secNonce [SecNonceSize]byte
			copy(
				secNonce[:],
				mustParseHex(
					testCases.PrivNonces[testCase.SecNonceIndex],
				),
			)

			_, err = Sign(
				secNonce, privKey, aggNonce, pubKeys,
				msg,
			)
			require.Error(t, err)
		})
	}

	for _, testCase := range testCases.VerifyFailCases {
		testCase := testCase

		testName := fmt.Sprintf("verify_fail_%v",
			strings.ToLower(testCase.Comment))
		t.Run(testName, func(t *testing.T) {
			pubKeys, err := keysFromIndices(
				t, testCase.Indices, testCases.PubKeys,
			)
			require.NoError(t, err)

			pubNonces := pubNoncesFromIndices(
				t, testCase.NonceIndices, testCases.PubNonces,
			)

			combinedNonce, err := AggregateNonces(pubNonces)
			require.NoError(t, err)

			var msg [32]byte
			copy(
				msg[:],
				mustParseHex(testCases.Msgs[testCase.MsgIndex]),
			)

			var secNonce [SecNonceSize]byte
			copy(secNonce[:], mustParseHex(testCases.PrivNonces[0]))

			signerNonce := secNonceToPubNonce(secNonce)

			var partialSig PartialSignature
			err = partialSig.Decode(
				bytes.NewReader(mustParseHex(testCase.Sig)),
			)
			if err != nil && strings.Contains(testCase.Comment, "group size") {
				require.ErrorIs(t, err, ErrPartialSigInvalid)
			}

			err = verifyPartialSig(
				&partialSig, signerNonce, combinedNonce,
				pubKeys, privKey.PubKey().SerializeCompressed(),
				msg,
			)
			require.Error(t, err)
		})
	}

	for _, testCase := range testCases.VerifyErrorCases {
		testCase := testCase

		testName := fmt.Sprintf("verify_error_%v",
			strings.ToLower(testCase.Comment))
		t.Run(testName, func(t *testing.T) {
			switch testCase.Comment {
			case "Invalid pubnonce":
				pubNonces := pubNoncesFromIndices(
					t, testCase.NonceIndices, testCases.PubNonces,
				)
				_, err := AggregateNonces(pubNonces)
				require.ErrorIs(t, err, secp.ErrPubKeyNotOnCurve)

			case "Invalid pubkey":
				_, err := keysFromIndices(
					t, testCase.Indices, testCases.PubKeys,
				)
				require.ErrorIs(t, err, secp.ErrPubKeyNotOnCurve)

			default:
				t.Fatalf("unhandled case: %v", testCase.Comment)
			}
		})
	}

}

type sigCombineValidCase struct {
	AggNonce string `json:"aggnonce"`

	NonceIndices []int `json:"nonce_indices"`

	Indices []int `json:"key_indices"`

	TweakIndices []int `json:"tweak_indices"`

	IsXOnly []bool `json:"is_xonly"`

	PSigIndices []int `json:"psig_indices"`

	Expected string `json:"expected"`
}

type sigCombineTestVectors struct {
	PubKeys []string `json:"pubkeys"`

	PubNonces []string `json:"pnonces"`

	Tweaks []string `json:"tweaks"`

	Psigs []string `json:"psigs"`

	Msg string `json:"msg"`

	ValidCases []sigCombineValidCase `json:"valid_test_cases"`
}

func pSigsFromIndices(t *testing.T, sigs []string, indices []int) []*PartialSignature {
	pSigs := make([]*PartialSignature, len(indices))
	for i, idx := range indices {
		var pSig PartialSignature
		err := pSig.Decode(bytes.NewReader(mustParseHex(sigs[idx])))
		require.NoError(t, err)

		pSigs[i] = &pSig
	}

	return pSigs
}

// TestMusig2SignCombine tests that we pass the musig2 sig combination tests.
func TestMusig2SignCombine(t *testing.T) {
	t.Parallel()

	testVectorPath := path.Join(
		testVectorBaseDir, sigCombineTestVectorFileName,
	)
	testVectorBytes, err := os.ReadFile(testVectorPath)
	require.NoError(t, err)

	var testCases sigCombineTestVectors
	require.NoError(t, json.Unmarshal(testVectorBytes, &testCases))

	var msg [32]byte
	copy(msg[:], mustParseHex(testCases.Msg))

	for i, testCase := range testCases.ValidCases {
		testCase := testCase

		testName := fmt.Sprintf("valid_case_%v", i)
		t.Run(testName, func(t *testing.T) {
			pubKeys, err := keysFromIndices(
				t, testCase.Indices, testCases.PubKeys,
			)
			require.NoError(t, err)

			pubNonces := pubNoncesFromIndices(
				t, testCase.NonceIndices, testCases.PubNonces,
			)

			partialSigs := pSigsFromIndices(
				t, testCases.Psigs, testCase.PSigIndices,
			)

			var (
				combineOpts []CombineOption
				keyOpts     []KeyAggOption
			)
			if len(testCase.TweakIndices) > 0 {
				tweaks := tweaksFromIndices(
					t, testCase.TweakIndices,
					testCases.Tweaks, testCase.IsXOnly,
				)

				combineOpts = append(combineOpts, WithTweakedCombine(
					msg, pubKeys, tweaks, false,
				))

				keyOpts = append(keyOpts, WithKeyTweaks(tweaks...))
			}

			combinedKey, _, _, err := AggregateKeys(
				pubKeys, false, keyOpts...,
			)
			require.NoError(t, err)

			combinedNonce, err := AggregateNonces(pubNonces)
			require.NoError(t, err)

			finalNonceJ, _, err := computeSigningNonce(
				combinedNonce, combinedKey.FinalKey, msg,
			)

			finalNonceJ.ToAffine()
			finalNonce := btcec.NewPublicKey(
				&finalNonceJ.X, &finalNonceJ.Y,
			)

			combinedSig := CombineSigs(
				finalNonce, partialSigs, combineOpts...,
			)
			require.Equal(t,
				strings.ToLower(testCase.Expected),
				hex.EncodeToString(combinedSig.Serialize()),
			)
		})
	}
}
