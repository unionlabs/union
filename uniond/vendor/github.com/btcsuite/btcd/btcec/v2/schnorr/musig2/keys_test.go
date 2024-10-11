// Copyright 2013-2022 The btcsuite developers

package musig2

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path"
	"strings"
	"testing"

	"github.com/btcsuite/btcd/btcec/v2"
	"github.com/btcsuite/btcd/btcec/v2/schnorr"
	secp "github.com/decred/dcrd/dcrec/secp256k1/v4"
	"github.com/stretchr/testify/require"
)

const (
	keySortTestVectorFileName = "key_sort_vectors.json"

	keyAggTestVectorFileName = "key_agg_vectors.json"

	keyTweakTestVectorFileName = "tweak_vectors.json"
)

type keySortTestVector struct {
	PubKeys []string `json:"pubkeys"`

	SortedKeys []string `json:"sorted_pubkeys"`
}

// TestMusig2KeySort tests that keys are properly sorted according to the
// musig2 test vectors.
func TestMusig2KeySort(t *testing.T) {
	t.Parallel()

	testVectorPath := path.Join(
		testVectorBaseDir, keySortTestVectorFileName,
	)
	testVectorBytes, err := os.ReadFile(testVectorPath)
	require.NoError(t, err)

	var testCase keySortTestVector
	require.NoError(t, json.Unmarshal(testVectorBytes, &testCase))

	keys := make([]*btcec.PublicKey, len(testCase.PubKeys))
	for i, keyStr := range testCase.PubKeys {
		pubKey, err := btcec.ParsePubKey(mustParseHex(keyStr))
		require.NoError(t, err)

		keys[i] = pubKey
	}

	sortedKeys := sortKeys(keys)

	expectedKeys := make([]*btcec.PublicKey, len(testCase.PubKeys))
	for i, keyStr := range testCase.SortedKeys {
		pubKey, err := btcec.ParsePubKey(mustParseHex(keyStr))
		require.NoError(t, err)

		expectedKeys[i] = pubKey
	}

	require.Equal(t, sortedKeys, expectedKeys)
}

type keyAggValidTest struct {
	Indices  []int  `json:"key_indices"`
	Expected string `json:"expected"`
}

type keyAggError struct {
	Type     string `json:"type"`
	Signer   int    `json:"signer"`
	Contring string `json:"contrib"`
}

type keyAggInvalidTest struct {
	Indices []int `json:"key_indices"`

	TweakIndices []int `json:"tweak_indices"`

	IsXOnly []bool `json:"is_xonly"`

	Comment string `json:"comment"`
}

type keyAggTestVectors struct {
	PubKeys []string `json:"pubkeys"`

	Tweaks []string `json:"tweaks"`

	ValidCases []keyAggValidTest `json:"valid_test_cases"`

	InvalidCases []keyAggInvalidTest `json:"error_test_cases"`
}

func keysFromIndices(t *testing.T, indices []int,
	pubKeys []string) ([]*btcec.PublicKey, error) {

	t.Helper()

	inputKeys := make([]*btcec.PublicKey, len(indices))
	for i, keyIdx := range indices {
		var err error
		inputKeys[i], err = btcec.ParsePubKey(
			mustParseHex(pubKeys[keyIdx]),
		)
		if err != nil {
			return nil, err
		}
	}

	return inputKeys, nil
}

func tweaksFromIndices(t *testing.T, indices []int,
	tweaks []string, isXonly []bool) []KeyTweakDesc {

	t.Helper()

	testTweaks := make([]KeyTweakDesc, len(indices))
	for i, idx := range indices {
		var rawTweak [32]byte
		copy(rawTweak[:], mustParseHex(tweaks[idx]))

		testTweaks[i] = KeyTweakDesc{
			Tweak:   rawTweak,
			IsXOnly: isXonly[i],
		}
	}

	return testTweaks
}

// TestMuSig2KeyAggTestVectors tests that this implementation of musig2 key
// aggregation lines up with the secp256k1-zkp test vectors.
func TestMuSig2KeyAggTestVectors(t *testing.T) {
	t.Parallel()

	testVectorPath := path.Join(
		testVectorBaseDir, keyAggTestVectorFileName,
	)
	testVectorBytes, err := os.ReadFile(testVectorPath)
	require.NoError(t, err)

	var testCases keyAggTestVectors
	require.NoError(t, json.Unmarshal(testVectorBytes, &testCases))

	tweaks := make([][]byte, len(testCases.Tweaks))
	for i := range testCases.Tweaks {
		tweaks[i] = mustParseHex(testCases.Tweaks[i])
	}

	for i, testCase := range testCases.ValidCases {
		testCase := testCase

		// Assemble the set of keys we'll pass in based on their key
		// index. We don't use sorting to ensure we send the keys in
		// the exact same order as the test vectors do.
		inputKeys, err := keysFromIndices(
			t, testCase.Indices, testCases.PubKeys,
		)
		require.NoError(t, err)

		t.Run(fmt.Sprintf("test_case=%v", i), func(t *testing.T) {
			uniqueKeyIndex := secondUniqueKeyIndex(inputKeys, false)
			opts := []KeyAggOption{WithUniqueKeyIndex(uniqueKeyIndex)}

			combinedKey, _, _, err := AggregateKeys(
				inputKeys, false, opts...,
			)
			require.NoError(t, err)

			require.Equal(
				t, schnorr.SerializePubKey(combinedKey.FinalKey),
				mustParseHex(testCase.Expected),
			)
		})
	}

	for _, testCase := range testCases.InvalidCases {
		testCase := testCase

		testName := fmt.Sprintf("invalid_%v",
			strings.ToLower(testCase.Comment))
		t.Run(testName, func(t *testing.T) {
			// For each test, we'll extract the set of input keys
			// as well as the tweaks since this set of cases also
			// exercises error cases related to the set of tweaks.
			inputKeys, err := keysFromIndices(
				t, testCase.Indices, testCases.PubKeys,
			)

			// In this set of test cases, we should only get this
			// for the very first vector.
			if err != nil {
				switch testCase.Comment {
				case "Invalid public key":
					require.ErrorIs(
						t, err,
						secp.ErrPubKeyNotOnCurve,
					)

				case "Public key exceeds field size":
					require.ErrorIs(
						t, err, secp.ErrPubKeyXTooBig,
					)

				case "First byte of public key is not 2 or 3":
					require.ErrorIs(
						t, err,
						secp.ErrPubKeyInvalidFormat,
					)

				default:
					t.Fatalf("uncaught err: %v", err)
				}

				return
			}

			var tweaks []KeyTweakDesc
			if len(testCase.TweakIndices) != 0 {
				tweaks = tweaksFromIndices(
					t, testCase.TweakIndices, testCases.Tweaks,
					testCase.IsXOnly,
				)
			}

			uniqueKeyIndex := secondUniqueKeyIndex(inputKeys, false)
			opts := []KeyAggOption{
				WithUniqueKeyIndex(uniqueKeyIndex),
			}

			if len(tweaks) != 0 {
				opts = append(opts, WithKeyTweaks(tweaks...))
			}

			_, _, _, err = AggregateKeys(
				inputKeys, false, opts...,
			)
			require.Error(t, err)

			switch testCase.Comment {
			case "Tweak is out of range":
				require.ErrorIs(t, err, ErrTweakedKeyOverflows)

			case "Intermediate tweaking result is point at infinity":

				require.ErrorIs(t, err, ErrTweakedKeyIsInfinity)

			default:
				t.Fatalf("uncaught err: %v", err)
			}
		})
	}
}

type keyTweakInvalidTest struct {
	Indices []int `json:"key_indices"`

	NonceIndices []int `json:"nonce_indices"`

	TweakIndices []int `json:"tweak_indices"`

	IsXOnly []bool `json:"is_only"`

	SignerIndex int `json:"signer_index"`

	Comment string `json:"comment"`
}

type keyTweakValidTest struct {
	Indices []int `json:"key_indices"`

	NonceIndices []int `json:"nonce_indices"`

	TweakIndices []int `json:"tweak_indices"`

	IsXOnly []bool `json:"is_xonly"`

	SignerIndex int `json:"signer_index"`

	Expected string `json:"expected"`

	Comment string `json:"comment"`
}

type keyTweakVector struct {
	PrivKey string `json:"sk"`

	PubKeys []string `json:"pubkeys"`

	PrivNonce string `json:"secnonce"`

	PubNonces []string `json:"pnonces"`

	AggNnoce string `json:"aggnonce"`

	Tweaks []string `json:"tweaks"`

	Msg string `json:"msg"`

	ValidCases []keyTweakValidTest `json:"valid_test_cases"`

	InvalidCases []keyTweakInvalidTest `json:"error_test_cases"`
}

func pubNoncesFromIndices(t *testing.T, nonceIndices []int, pubNonces []string) [][PubNonceSize]byte {

	nonces := make([][PubNonceSize]byte, len(nonceIndices))

	for i, idx := range nonceIndices {
		var pubNonce [PubNonceSize]byte
		copy(pubNonce[:], mustParseHex(pubNonces[idx]))

		nonces[i] = pubNonce
	}

	return nonces
}

// TestMuSig2TweakTestVectors tests that we properly handle the various edge
// cases related to tweaking public keys.
func TestMuSig2TweakTestVectors(t *testing.T) {
	t.Parallel()

	testVectorPath := path.Join(
		testVectorBaseDir, keyTweakTestVectorFileName,
	)
	testVectorBytes, err := os.ReadFile(testVectorPath)
	require.NoError(t, err)

	var testCases keyTweakVector
	require.NoError(t, json.Unmarshal(testVectorBytes, &testCases))

	privKey, _ := btcec.PrivKeyFromBytes(mustParseHex(testCases.PrivKey))

	var msg [32]byte
	copy(msg[:], mustParseHex(testCases.Msg))

	var secNonce [SecNonceSize]byte
	copy(secNonce[:], mustParseHex(testCases.PrivNonce))

	for _, testCase := range testCases.ValidCases {
		testCase := testCase

		testName := fmt.Sprintf("valid_%v",
			strings.ToLower(testCase.Comment))
		t.Run(testName, func(t *testing.T) {
			pubKeys, err := keysFromIndices(
				t, testCase.Indices, testCases.PubKeys,
			)
			require.NoError(t, err)

			var tweaks []KeyTweakDesc
			if len(testCase.TweakIndices) != 0 {
				tweaks = tweaksFromIndices(
					t, testCase.TweakIndices,
					testCases.Tweaks, testCase.IsXOnly,
				)
			}

			pubNonces := pubNoncesFromIndices(
				t, testCase.NonceIndices, testCases.PubNonces,
			)

			combinedNonce, err := AggregateNonces(pubNonces)
			require.NoError(t, err)

			var opts []SignOption
			if len(tweaks) != 0 {
				opts = append(opts, WithTweaks(tweaks...))
			}

			partialSig, err := Sign(
				secNonce, privKey, combinedNonce, pubKeys,
				msg, opts...,
			)

			var partialSigBytes [32]byte
			partialSig.S.PutBytesUnchecked(partialSigBytes[:])

			require.Equal(
				t, hex.EncodeToString(partialSigBytes[:]),
				hex.EncodeToString(mustParseHex(testCase.Expected)),
			)

		})
	}
}
