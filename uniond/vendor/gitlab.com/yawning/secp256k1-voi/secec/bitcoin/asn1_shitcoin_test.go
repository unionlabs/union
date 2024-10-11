// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package bitcoin

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/require"

	"gitlab.com/yawning/secp256k1-voi"
	"gitlab.com/yawning/secp256k1-voi/internal/helpers"
	"gitlab.com/yawning/secp256k1-voi/secec"
)

var errInvalidBIP0066Sig = errors.New("secp256k1/secec/bitcoin: invalid BIP-0066 signature")

type bip0066ValidCase struct {
	DER string `json:"DER"`
	R   string `json:"r"`
	S   string `json:"s"`
}

type bip0066InvalidDecodeCase struct {
	Exception string `json:"exception"`
	DER       string `json:"DER"`
}

type bip0066TestVectors struct {
	Valid   []bip0066ValidCase `json:"valid"`
	Invalid struct {
		Encode json.RawMessage            `json:"encode"`
		Decode []bip0066InvalidDecodeCase `json:"decode"`
	} `json:"invalid"`
}

func TestBIP0066(t *testing.T) {
	f, err := os.Open("testdata/bip-0066-test-vectors.json")
	require.NoError(t, err, "Open")
	defer f.Close()

	var testVectors bip0066TestVectors

	dec := json.NewDecoder(f)
	err = dec.Decode(&testVectors)
	require.NoError(t, err, "dec.Decode")

	for i, testCase := range testVectors.Valid {
		n := fmt.Sprintf("TestVectors/Valid/%d", i)
		t.Run(n, func(t *testing.T) {
			b := helpers.MustBytesFromHex(testCase.DER)
			b = append(b, 69) // Append the sighash byte.
			ok := IsValidSignatureEncodingBIP0066(b)

			require.True(t, ok, "IsValidSignatureEncodingBIP0066")

			rBytes := helpers.MustBytesFromHex(testCase.R)
			sBytes := helpers.MustBytesFromHex(testCase.S)

			rBytes = bytes.TrimLeft(rBytes, string([]byte{0x00}))
			sBytes = bytes.TrimLeft(sBytes, string([]byte{0x00}))
			rBytes = append(bytes.Repeat([]byte{0x00}, secp256k1.ScalarSize-len(rBytes)), rBytes...)
			sBytes = append(bytes.Repeat([]byte{0x00}, secp256k1.ScalarSize-len(sBytes)), sBytes...)

			expectedR, err := secp256k1.NewScalarFromCanonicalBytes((*[secp256k1.ScalarSize]byte)(rBytes))
			require.NoError(t, err, "NewScalarFromCanonicalBytes - rBytes")
			expectedS, err := secp256k1.NewScalarFromCanonicalBytes((*[secp256k1.ScalarSize]byte)(sBytes))
			require.NoError(t, err, "NewScalarFromCanonicalBytes - sBytes")

			r, s, err := parseASN1SignatureShitcoin(b)
			switch i {
			case 8: // Test case has bad r + s
				require.EqualValues(t, 1, expectedR.IsZero())
				require.EqualValues(t, 1, expectedS.IsZero())
				require.Error(t, err, "parseASN1SignatureShitcoin")
			default:
				require.NoError(t, err, "parseASN1SignatureShitcoin")
				require.EqualValues(t, 1, expectedR.Equal(r))
				require.EqualValues(t, 1, expectedS.Equal(s))
			}
		})
	}
	for i, testCase := range testVectors.Invalid.Decode {
		n := fmt.Sprintf("TestVectors/Invalid/%d", i)
		t.Run(n, func(t *testing.T) {
			t.Log(testCase.Exception)

			b := helpers.MustBytesFromHex(testCase.DER)
			b = append(b, 69) // Append the sighash byte.
			ok := IsValidSignatureEncodingBIP0066(b)

			require.False(t, ok, "IsValidSignatureEncodingBIP0066")

			_, _, err := parseASN1SignatureShitcoin(b)
			require.ErrorIs(t, err, errInvalidBIP0066Sig, "parseASN1SignatureShitcoin")
		})
	}
}

func parseASN1SignatureShitcoin(data []byte) (*secp256k1.Scalar, *secp256k1.Scalar, error) {
	if !IsValidSignatureEncodingBIP0066(data) {
		return nil, nil, errInvalidBIP0066Sig
	}

	return secec.ParseASN1Signature(data[:len(data)-1]) // Ignore the sighash
}
