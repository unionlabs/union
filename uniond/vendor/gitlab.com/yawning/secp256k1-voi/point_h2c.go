// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package secp256k1

import (
	"gitlab.com/yawning/secp256k1-voi/internal/field"
	"gitlab.com/yawning/secp256k1-voi/internal/swu"
)

// SetUniformBytes sets `v = map_to_curve(OS2IP(src) mod p)`, where
// `src` MUST have a length in the range `[32,64]`-bytes, and returns
// `v`.
//
// If called with exactly 48-bytes of data, this can be used to
// implement `encode_to_curve` and `hash_to_curve`, per "Hashing to
// Elliptic Curves".  With a cryptographically insignificant probability,
// the result may be the point at infinity.
//
// Most users SHOULD use a higher-level `encode_to_curve` or
// `hash_to_curve` implementation instead.
func (v *Point) SetUniformBytes(src []byte) *Point {
	// The spec notes that there is an optimization opportunity for
	// the random oracle suites to save a call to `iso_map` by
	// doing the point addition in E'.
	//
	// We will forgo this, as while inversion is expensive, this
	// will neccecitate implementing additional point addition formula
	// that is only used for this routine.

	u := field.NewElement().SetWideBytes(src)

	// 6.6.3. Simplified SWU for AB == 0

	// 1. (x', y') = map_to_curve_simple_swu(u)    # (x', y') is on E'
	xP, yP := swu.MapToCurveSimpleSWU(u)

	// 2. (x, y) = iso_map(x', y')               # (x, y) is on E
	x, y, isOnCurve := swu.IsoMap(xP, yP)

	// 3. return (x, y)
	v.x.Set(x)
	v.y.Set(y)
	v.z.One()
	v.isValid = true

	// map_to_curve_simple_swu handles its exceptional cases.
	// Exceptional cases of iso_map are inputs that cause the
	// denominator of either rational function to evaluate to zero;
	// such cases MUST return the identity point on E.
	v.ConditionalSelect(NewIdentityPoint(), v, isOnCurve)

	return v
}
