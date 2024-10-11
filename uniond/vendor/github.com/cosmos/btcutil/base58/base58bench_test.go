// Copyright (c) 2013-2014 The btcsuite developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package base58_test

import (
	"bytes"
	"testing"

	"github.com/cosmos/btcutil/base58"
)

var (
	raw5k       = bytes.Repeat([]byte{0xff}, 5000)
	raw100k     = bytes.Repeat([]byte{0xff}, 100*1000)
	encoded5k   = base58.Encode(raw5k)
	encoded100k = base58.Encode(raw100k)
)

func BenchmarkBase58Encode_5K(b *testing.B) {
	b.SetBytes(int64(len(raw5k)))
	for i := 0; i < b.N; i++ {
		base58.Encode(raw5k)
	}
}

func BenchmarkBase58Encode_100K(b *testing.B) {
	b.SetBytes(int64(len(raw100k)))
	for i := 0; i < b.N; i++ {
		base58.Encode(raw100k)
	}
}

func BenchmarkBase58Decode_5K(b *testing.B) {
	b.SetBytes(int64(len(encoded5k)))
	for i := 0; i < b.N; i++ {
		base58.Decode(encoded5k)
	}
}

func BenchmarkBase58Decode_100K(b *testing.B) {
	b.SetBytes(int64(len(encoded100k)))
	for i := 0; i < b.N; i++ {
		base58.Decode(encoded100k)
	}
}
