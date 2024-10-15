// Copyright (c) 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package edwards25519 implements group logic for the twisted Edwards curve
//
//	-x^2 + y^2 = 1 + -(121665/121666)*x^2*y^2
//
// This is better known as the Edwards curve equivalent to Curve25519, and is
// the curve used by the Ed25519 signature scheme.
//
// Most users don't need this package, and should instead use crypto/ed25519 for
// signatures, golang.org/x/crypto/curve25519 for Diffie-Hellman, or
// github.com/gtank/ristretto255 for prime order group logic.
//
// However, developers who do need to interact with low-level edwards25519
// operations can use this package, which is an extended version of
// crypto/internal/edwards25519 from the standard library repackaged as
// an importable module.
package edwards25519
