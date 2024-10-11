// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package tuplehash

import (
	"bytes"
	"encoding/binary"
	"encoding/hex"
	"encoding/json"
	"hash"
	"math/big"
	"os"
	"testing"

	"github.com/stretchr/testify/require"
)

// The JSON format test vectors were converted manually from:
// https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/TupleHash_samples.pdf
// https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/TupleHashXOF_samples.pdf

type tupleHashTestVector struct {
	Name             string   `json:"name"`
	SecurityStrength int      `json:"security_strength"`
	OutputLength     int      `json:"output_length"`
	S                string   `json:"s"`
	Tuples           []string `json:"tuples"`
	Outval           string   `json:"outval"`
	IsXOF            bool     `json:"is_xof"`
}

func (tv *tupleHashTestVector) Run(t *testing.T) {
	var h *Hasher
	switch tv.IsXOF {
	case false:
		switch tv.SecurityStrength {
		case 128:
			h = NewTupleHash128([]byte(tv.S), tv.OutputLength/8)
		case 256:
			h = NewTupleHash256([]byte(tv.S), tv.OutputLength/8)
		}
	case true:
		switch tv.SecurityStrength {
		case 128:
			h = NewTupleHashXOF128([]byte(tv.S))
		case 256:
			h = NewTupleHashXOF256([]byte(tv.S))
		}
	}
	require.NotNil(t, h, "NewTupleHash(XOF)")

	switch tv.SecurityStrength {
	case 128:
		require.Equal(t, rate128, h.BlockSize())
	case 256:
		require.Equal(t, rate256, h.BlockSize())
	}
	switch tv.IsXOF {
	case false:
		require.Equal(t, tv.OutputLength/8, h.Size())
	case true:
		require.Panics(t, func() { h.Size() })
	}

	for _, tuple := range tv.Tuples {
		b := mustUnhex(tuple)
		n, err := h.Write(b)
		require.Equal(t, len(b), n, "h.Write")
		require.NoError(t, err, "h.Write")
	}

	var out []byte
	expected := mustUnhex(tv.Outval)
	switch tv.IsXOF {
	case false:
		out = h.Sum(nil)
	case true:
		out = make([]byte, len(expected))
		n, err := h.Read(out)
		require.Equal(t, len(out), n, "h.Read")
		require.NoError(t, err, "h.Read")
	}
	require.Equal(t, expected, out, "h.Sum/h.Read")
}

func TestNISTVectors(t *testing.T) {
	f, err := os.Open("testdata/tuplehash.json")
	require.NoError(t, err, "os.Open")
	defer f.Close()

	var testVectors []tupleHashTestVector

	dec := json.NewDecoder(f)
	err = dec.Decode(&testVectors)
	require.NoError(t, err, "dec.Decode")

	for _, vec := range testVectors {
		t.Run(vec.Name, vec.Run)
	}
}

type encodeTestVector struct {
	Value string `json:"value"`
	Left  string `json:"left"`
	Right string `json:"right"`
}

func (tv *encodeTestVector) Run(t *testing.T) {
	expectedLeft := mustUnhex(tv.Left)
	expectedRight := mustUnhex(tv.Right)

	v, _ := new(big.Int).SetString(tv.Value, 0)

	var buf [2 * 8]byte
	v.FillBytes(buf[:])

	hi := binary.BigEndian.Uint64(buf[0:])
	lo := binary.BigEndian.Uint64(buf[8:])

	w := bytes.NewBuffer(nil)
	leftEncode(w, hi, lo)
	require.EqualValues(t, expectedLeft, w.Bytes())

	w.Reset()

	rightEncode(w, hi, lo)
	require.EqualValues(t, expectedRight, w.Bytes())
}

func TestEncode(t *testing.T) {
	f, err := os.Open("testdata/lr_encode.json")
	require.NoError(t, err, "os.Open")
	defer f.Close()

	var testVectors []encodeTestVector

	dec := json.NewDecoder(f)
	err = dec.Decode(&testVectors)
	require.NoError(t, err, "dec.Decode")

	for _, vec := range testVectors {
		t.Run(vec.Value, vec.Run)
	}
}

func TestAPI(t *testing.T) {
	var (
		testSep    = []byte("yawning/tuplehash/tests")
		testTuple  = []byte("this is my tuple")
		testTuple2 = []byte("there are many others like it")
	)

	t.Run("Hash", func(t *testing.T) {
		h := NewTupleHash128(testSep, 32)
		_ = (hash.Hash)(h)

		_, _ = h.Write(testTuple)

		require.Panics(t, func() {
			var tmp [10]byte
			_, _ = h.Read(tmp[:])
		})

		out := h.Sum(nil)

		dst := make([]byte, 0, 32)
		out2 := h.Sum(dst)
		require.Equal(t, out, out2)

		h.Reset()
		_, _ = h.Write(testTuple)
		out3 := h.Sum(nil)
		require.Equal(t, out, out3)
	})

	t.Run("ShakeHash", func(t *testing.T) {
		xof := NewTupleHashXOF128(testSep)

		_, _ = xof.Write(testTuple)

		require.Panics(t, func() { xof.Sum(nil) })

		xof2 := xof.Clone()

		_, _ = xof.Write(testTuple2)
		dst := make([]byte, 64)
		_, _ = xof.Read(dst)

		_, _ = xof2.Write(testTuple2)
		dst2 := make([]byte, 64)
		_, _ = xof2.Read(dst2)

		require.EqualValues(t, dst, dst2)
	})

	require.Panics(t, func() { NewTupleHash128(nil, -1) })
}

func mustUnhex(x string) []byte {
	b, err := hex.DecodeString(x)
	if err != nil {
		panic(err)
	}
	return b
}
