// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package secp256k1

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
)

func testPointMultiScalarMultIters(t *testing.T, sz int, isVartime bool) {
	scalars, points, check := setupTestMultiScalarMult(sz)

	out := newRcvr()
	switch isVartime {
	case true:
		out.MultiScalarMultVartime(scalars, points)
	case false:
		out.MultiScalarMult(scalars, points)
	}
	requirePointEquals(t, check, out, "MultiScalarMult")
}

func testPointMultiScalarMult(t *testing.T) {
	testSizes := []int{0, 1, 32, 64}
	for _, sz := range testSizes {
		t.Run(fmt.Sprintf("MultiScalarMult/%d", sz), func(t *testing.T) {
			testPointMultiScalarMultIters(t, sz, false)
		})
	}
	require.Panics(t, func() {
		NewIdentityPoint().MultiScalarMult(
			[]*Scalar{NewScalar()},
			[]*Point{NewGeneratorPoint(), NewGeneratorPoint()},
		)
	})

	for _, sz := range testSizes {
		t.Run(fmt.Sprintf("MultiScalarMultVartime/%d", sz), func(t *testing.T) {
			testPointMultiScalarMultIters(t, sz, true)
		})
	}
	require.Panics(t, func() {
		NewIdentityPoint().MultiScalarMultVartime(
			[]*Scalar{NewScalar(), NewScalar()},
			[]*Point{NewGeneratorPoint()},
		)
	})
}

func setupTestMultiScalarMult(sz int) ([]*Scalar, []*Point, *Point) {
	points := make([]*Point, 0, sz)
	scalars := make([]*Scalar, 0, sz)

	check := NewIdentityPoint()
	for i := 0; i < sz; i++ {
		p := newRcvr().DebugMustRandomize()
		s := NewScalar().DebugMustRandomizeNonZero()

		points = append(points, p)
		scalars = append(scalars, s)

		check.Add(check, newRcvr().ScalarMult(s, p))
	}

	return scalars, points, check
}

func benchPointMultiScalarMult(b *testing.B) {
	benchSizes := []int{1, 16, 32, 64, 1024}

	for _, sz := range benchSizes {
		b.Run(fmt.Sprintf("MultiScalarMult/%d", sz), func(b *testing.B) {
			scalars, points, _ := setupTestMultiScalarMult(sz)
			b.ReportAllocs()
			b.ResetTimer()

			for i := 0; i < b.N; i++ {
				_ = newRcvr().MultiScalarMult(scalars, points)
			}
		})
	}

	for _, sz := range benchSizes {
		b.Run(fmt.Sprintf("MultiScalarMultVartime/%d", sz), func(b *testing.B) {
			scalars, points, _ := setupTestMultiScalarMult(sz)
			b.ReportAllocs()
			b.ResetTimer()

			for i := 0; i < b.N; i++ {
				_ = newRcvr().MultiScalarMultVartime(scalars, points)
			}
		})
	}
}
