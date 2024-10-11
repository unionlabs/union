// Copyright 2015 Jeffrey Wilcke, Felix Lange, Gustav Simonsson. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build gofuzz || !cgo
// +build gofuzz !cgo

// nolint // this nolint lets us use this file in its original and unmodified form.
package secp256k1

import "math/big"

func (BitCurve *BitCurve) ScalarMult(Bx, By *big.Int, scalar []byte) (*big.Int, *big.Int) {
	panic("ScalarMult is not available when secp256k1 is built without cgo")
}
