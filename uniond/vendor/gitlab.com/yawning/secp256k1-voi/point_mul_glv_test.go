// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package secp256k1

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	fiat "gitlab.com/yawning/secp256k1-voi/internal/fiat/secp256k1montgomeryscalar"
)

func testScalarSplit(t *testing.T) {
	// Lambda = 0x5363ad4cc05c30e0a5261c028812645a122e22ea20816678df02967c1b23bd72
	lambda := newScalarFromCanonicalHex("0x5363ad4cc05c30e0a5261c028812645a122e22ea20816678df02967c1b23bd72")

	for i, v := range []*Scalar{
		NewScalar(),
		scOne,
		NewScalar().DebugMustRandomizeNonZero(),

		// Test cases from libsecp256k1
		newScalarFromCanonicalHex("0xd938a5667f479e3eb5b3c7faefdb37493aa0585cc5ea2367e1b660db0209e6fc"),
		newScalarFromCanonicalHex("0xd938a5667f479e3eb5b3c7faefdb37493aa0585cc5ea2367e1b660db0209e6fd"),
		newScalarFromCanonicalHex("0xd938a5667f479e3eb5b3c7faefdb37493aa0585cc5ea2367e1b660db0209e6fe"),
		newScalarFromCanonicalHex("0xd938a5667f479e3eb5b3c7faefdb37493aa0585cc5ea2367e1b660db0209e6ff"),
		newScalarFromCanonicalHex("0x2c9c52b33fa3cf1f5ad9e3fd77ed9ba5b294b8933722e9a500e698ca4cf7632d"),
		newScalarFromCanonicalHex("0x2c9c52b33fa3cf1f5ad9e3fd77ed9ba5b294b8933722e9a500e698ca4cf7632e"),
		newScalarFromCanonicalHex("0x2c9c52b33fa3cf1f5ad9e3fd77ed9ba5b294b8933722e9a500e698ca4cf7632f"),
		newScalarFromCanonicalHex("0x2c9c52b33fa3cf1f5ad9e3fd77ed9ba5b294b8933722e9a500e698ca4cf76330"),
		newScalarFromCanonicalHex("0x7fffffffffffffffffffffffffffffffd576e73557a4501ddfe92f46681b209f"),
		newScalarFromCanonicalHex("0x7fffffffffffffffffffffffffffffffd576e73557a4501ddfe92f46681b20a0"),
		newScalarFromCanonicalHex("0x7fffffffffffffffffffffffffffffffd576e73557a4501ddfe92f46681b20a1"),
		newScalarFromCanonicalHex("0x7fffffffffffffffffffffffffffffffd576e73557a4501ddfe92f46681b20a2"),
		newScalarFromCanonicalHex("0xd363ad4cc05c30e0a5261c0288126459f85915d77825b696beebc5c2833ede11"),
		newScalarFromCanonicalHex("0xd363ad4cc05c30e0a5261c0288126459f85915d77825b696beebc5c2833ede12"),
		newScalarFromCanonicalHex("0xd363ad4cc05c30e0a5261c0288126459f85915d77825b696beebc5c2833ede13"),
		newScalarFromCanonicalHex("0xd363ad4cc05c30e0a5261c0288126459f85915d77825b696beebc5c2833ede14"),
		newScalarFromCanonicalHex("0x26c75a9980b861c14a4c38051024c8b4704d760ee95e7cd3de1bfdb1ce2c5a42"),
		newScalarFromCanonicalHex("0x26c75a9980b861c14a4c38051024c8b4704d760ee95e7cd3de1bfdb1ce2c5a43"),
		newScalarFromCanonicalHex("0x26c75a9980b861c14a4c38051024c8b4704d760ee95e7cd3de1bfdb1ce2c5a44"),
		newScalarFromCanonicalHex("0x26c75a9980b861c14a4c38051024c8b4704d760ee95e7cd3de1bfdb1ce2c5a45"),
	} {
		t.Run(fmt.Sprintf("Case %d", i), func(t *testing.T) {
			k1, k2 := v.splitGLV()

			// k = k1 + k2 * lambda mod n
			k := NewScalar().Multiply(k2, lambda)
			k.Add(k, k1)
			require.EqualValues(t, 1, v.Equal(k), "k = k1 + k2 * lambda mod n")

			// The split scalars (or their negatives) are < 2^128.
			var k1Neg, k2Neg bool
			if k1.IsGreaterThanHalfN() == 1 {
				k1.Negate(k1)
				k1Neg = true
			}
			if k2.IsGreaterThanHalfN() == 1 {
				k2.Negate(k2)
				k2Neg = true
			}

			var tmp1, tmp2 fiat.NonMontgomeryDomainFieldElement
			fiat.FromMontgomery(&tmp1, &k1.m)
			fiat.FromMontgomery(&tmp2, &k2.m)

			require.Zero(t, tmp1[3], "k1 limb 3 == 0")
			require.Zero(t, tmp1[2], "k1 limb 2 == 0")

			require.Zero(t, tmp2[3], "k2 limb 3 == 0")
			require.Zero(t, tmp2[2], "k2 limb 2 == 0")

			// k * P = k1 * P + k2 * lambda * P
			p := newRcvr().DebugMustRandomize()
			kP := newRcvr().scalarMultTrivial(v, p)

			var k1p, k2p *Point
			if !k1Neg {
				k1p = newRcvr().ScalarMult(k1, p)
			} else {
				k1p = newRcvr().ScalarMult(k1, newRcvr().Negate(p))
			}

			pPrime := newRcvr().mulBeta(p)
			if !k2Neg {
				k2p = newRcvr().ScalarMult(k2, pPrime)
			} else {
				k2p = newRcvr().ScalarMult(k2, newRcvr().Negate(pPrime))
			}

			sum := newRcvr().Add(k1p, k2p)
			requirePointEquals(t, kP, sum, "k * P = k1 * P + k2 * lambda * P")
		})
	}
}
