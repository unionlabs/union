// Copyright (c) 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package field

import (
	"math/big"
	"testing"
	"testing/quick"
)

var bigP = new(big.Int).Sub(new(big.Int).Lsh(big.NewInt(1), 255), big.NewInt(19))

func TestSetWideBytes(t *testing.T) {
	f1 := func(in [64]byte, fe Element) bool {
		fe1 := new(Element).Set(&fe)

		if out, err := fe.SetWideBytes([]byte{42}); err == nil || out != nil ||
			fe.Equal(fe1) != 1 {
			return false
		}

		if out, err := fe.SetWideBytes(in[:]); err != nil || out != &fe {
			return false
		}

		b := new(big.Int).SetBytes(swapEndianness(in[:]))
		fe1.fromBig(b.Mod(b, bigP))

		return fe.Equal(fe1) == 1 && isInBounds(&fe) && isInBounds(fe1)
	}
	if err := quick.Check(f1, nil); err != nil {
		t.Error(err)
	}

}
