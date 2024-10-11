// Copyright (c) 2019 Klaus Post. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package s2

import (
	"bytes"
	"fmt"
	"math"
	"testing"
)

func TestEncodeHuge(t *testing.T) {
	if true {
		t.Skip("Takes too much memory")
	}
	test := func(t *testing.T, data []byte) {
		comp := Encode(make([]byte, MaxEncodedLen(len(data))), data)
		decoded, err := Decode(nil, comp)
		if err != nil {
			t.Error(err)
			return
		}
		if !bytes.Equal(data, decoded) {
			t.Error("block decoder mismatch")
			return
		}
		if mel := MaxEncodedLen(len(data)); len(comp) > mel {
			t.Error(fmt.Errorf("MaxEncodedLen Exceed: input: %d, mel: %d, got %d", len(data), mel, len(comp)))
			return
		}
		comp = EncodeBetter(make([]byte, MaxEncodedLen(len(data))), data)
		decoded, err = Decode(nil, comp)
		if err != nil {
			t.Error(err)
			return
		}
		if !bytes.Equal(data, decoded) {
			t.Error("block decoder mismatch")
			return
		}
		if mel := MaxEncodedLen(len(data)); len(comp) > mel {
			t.Error(fmt.Errorf("MaxEncodedLen Exceed: input: %d, mel: %d, got %d", len(data), mel, len(comp)))
			return
		}

		comp = EncodeBest(make([]byte, MaxEncodedLen(len(data))), data)
		decoded, err = Decode(nil, comp)
		if err != nil {
			t.Error(err)
			return
		}
		if !bytes.Equal(data, decoded) {
			t.Error("block decoder mismatch")
			return
		}
		if mel := MaxEncodedLen(len(data)); len(comp) > mel {
			t.Error(fmt.Errorf("MaxEncodedLen Exceed: input: %d, mel: %d, got %d", len(data), mel, len(comp)))
			return
		}
	}
	test(t, make([]byte, math.MaxInt32))
	if math.MaxInt > math.MaxInt32 {
		x := int64(math.MaxInt32 + math.MaxUint16)
		test(t, make([]byte, x))
	}
	test(t, make([]byte, MaxBlockSize))
}
