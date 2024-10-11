// Copyright (c) 2021 Oasis Labs Inc
// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package h2c

import (
	"crypto"
	_ "crypto/sha1" //nolint:gosec // Used for short digest test.
	_ "crypto/sha256"
	"encoding/json"
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/require"

	"gitlab.com/yawning/secp256k1-voi"
	"gitlab.com/yawning/secp256k1-voi/internal/helpers"
)

type h2cSuiteTestDef struct {
	n    string
	file string
	fn   func([]byte, []byte) (*secp256k1.Point, error)
}

type h2cExpandTestDef struct {
	n    string
	file string
	h    crypto.Hash
}

func TestH2C(t *testing.T) {
	expandTestDefs := []h2cExpandTestDef{
		{
			n:    "ExpandMessageXMD/SHA256",
			file: "testdata/expand_message_xmd_SHA256_38.json",
			h:    crypto.SHA256,
		},
		{
			n:    "ExpandMessageXMD/SHA256-LongDST",
			file: "testdata/expand_message_xmd_SHA256_256.json",
			h:    crypto.SHA256,
		},
	}
	for i, expandTest := range expandTestDefs {
		t.Run(expandTest.n, func(t *testing.T) {
			testExpandMessage(t, &expandTestDefs[i])
		})
	}

	t.Run("ExpandMessageXMD/EdgeCases", func(t *testing.T) {
		dst := []byte("QUUX-V01-CS02-failure-tests-DST")

		// Our implementation requires at least a 256-bit digest.
		var out [encodeToCurveSize]byte
		err := expandMessageXMD(out[:], crypto.SHA1, dst, []byte("short hash"))
		require.ErrorIs(t, err, errInvalidDigestSize, "expandMessageXMD - SHA1")

		// Our implementation rejects 0-length DSTs.
		err = expandMessageXMD(out[:], crypto.SHA256, []byte{}, []byte("zero DST"))
		require.ErrorIs(t, err, errInvalidDomainSep, "expandMessageXMD - 0 length dst")

		// Our implementation rejects 0-length output, even if the RFC does not.
		err = expandMessageXMD(out[:0], crypto.SHA256, dst, []byte("zero output"))
		require.ErrorIs(t, err, errInvalidOutputSize, "expandMessageXMD - 0 length output")

		// The RFC calls for rejecting outputs larger than > 2^16-1.
		// Though, this case can never happen (see the ell tests).
		err = expandMessageXMD(make([]byte, 65536), crypto.SHA256, dst, []byte("oversize output"))
		require.ErrorIs(t, err, errInvalidOutputSize, "expandMessageXMD - oversize output")

		// ell = ceil(len_in_bytes / b_in_bytes), up to 255
		err = expandMessageXMD(make([]byte, 8161), crypto.SHA256, dst, []byte("oversize ell"))
		require.ErrorIs(t, err, errEllOutOfRange, "expandMessageXMD - oversize ell")

		err = expandMessageXMD(make([]byte, 8160), crypto.SHA256, dst, []byte("maximum ell"))
		require.NoError(t, err, "expandMessageXMD - maximum ell")
	})

	suiteTestDefs := []h2cSuiteTestDef{
		{
			n:    "Suite/secp256k1_XMD:SHA-256_SSWU_RO_",
			file: "testdata/secp256k1_XMD_SHA-256_SSWU_RO_.json",
			fn:   Secp256k1_XMD_SHA256_SSWU_RO,
		},
		{
			n:    "Suite/secp256k1_XMD:SHA-256_SSWU_NU_",
			file: "testdata/secp256k1_XMD_SHA-256_SSWU_NU_.json",
			fn:   Secp256k1_XMD_SHA256_SSWU_NU,
		},
	}
	for i, suiteTest := range suiteTestDefs {
		t.Run(suiteTest.n, func(t *testing.T) {
			testSuiteH2c(t, &suiteTestDefs[i])
		})
	}

	t.Run("Suite/EdgeCases", func(t *testing.T) {
		m := []byte("zero DST")

		// The only way to force failures is to have a 0-length
		// DST, since every other way expandMessageXMD can fail,
		// will never happen due to hard-coded parameters.

		p, err := Secp256k1_XMD_SHA256_SSWU_RO([]byte{}, m)
		require.Nil(t, p, "RO - 0 length dst")
		require.Error(t, err, "RO - 0 length dst")

		p, err = Secp256k1_XMD_SHA256_SSWU_NU([]byte{}, m)
		require.Nil(t, p, "NU - 0 length dst")
		require.Error(t, err, "NU - 0 length dst")
	})
}

type h2cSuiteTestVectors struct {
	DST     string               `json:"DST"`
	Vectors []h2cSuiteTestVector `json:"vectors"`
}

type h2cSuiteTestVector struct {
	P   h2cSuiteTestPoint
	Msg string `json:"msg"`
}

type h2cSuiteTestPoint struct {
	X string `json:"x"`
	Y string `json:"y"`
}

func (pt *h2cSuiteTestPoint) ToPoint() (*secp256k1.Point, error) {
	x := helpers.MustBytesFromHex(pt.X)
	y := helpers.MustBytesFromHex(pt.Y)

	return secp256k1.NewPointFromCoords((*[secp256k1.CoordSize]byte)(x), (*[secp256k1.CoordSize]byte)(y))
}

func testSuiteH2c(t *testing.T, def *h2cSuiteTestDef) {
	f, err := os.Open(def.file)
	require.NoError(t, err, "os.Open")
	defer f.Close()

	var testVectors h2cSuiteTestVectors

	dec := json.NewDecoder(f)
	err = dec.Decode(&testVectors)
	require.NoError(t, err, "dec.Decode")

	for i, vec := range testVectors.Vectors {
		t.Run(fmt.Sprintf("TestCase/%d", i), func(t *testing.T) {
			expectedP, err := vec.P.ToPoint()
			require.NoError(t, err, "P.ToPoint")

			p, err := def.fn([]byte(testVectors.DST), []byte(vec.Msg))
			require.NoError(t, err, "hash_to_curve")
			require.EqualValues(t, 1, expectedP.Equal(p), "hash_to_curve")
		})
	}
}

type h2cExpandTestVectors struct {
	DST   string                `json:"DST"`
	Tests []h2cExpandTestVector `json:"tests"`
}

type h2cExpandTestVector struct {
	Msg          string `json:"msg"`
	UniformBytes string `json:"uniform_bytes"`
}

func testExpandMessage(t *testing.T, def *h2cExpandTestDef) {
	f, err := os.Open(def.file)
	require.NoError(t, err, "os.Open")
	defer f.Close()

	var testVectors h2cExpandTestVectors

	dec := json.NewDecoder(f)
	err = dec.Decode(&testVectors)
	require.NoError(t, err, "dec.Decode")

	for i, vec := range testVectors.Tests {
		t.Run(fmt.Sprintf("TestCase/%d", i), func(t *testing.T) {
			expectedU := helpers.MustBytesFromHex(vec.UniformBytes)
			out := make([]byte, len(expectedU))

			err := expandMessageXMD(out, def.h, []byte(testVectors.DST), []byte(vec.Msg))
			require.NoError(t, err, "ExpandMessageXMD(out, h, dst, msg)")
			require.Equal(t, expectedU, out, "ExpandMessageXMD(out, h, dst, msg)")
		})
	}
}
