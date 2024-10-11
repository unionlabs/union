package ecc

import (
	"math/big"
	"testing"
)

func TestNafDecomposition(t *testing.T) {
	t.Parallel()
	// TODO write a real test...
	exp := big.NewInt(13)
	var result [400]int8
	lExp := NafDecomposition(exp, result[:])
	dec := result[:lExp]

	res := [5]int8{1, 0, -1, 0, 1}
	for i, v := range dec {
		if v != res[i] {
			t.Error("Error in NafDecomposition")
		}
	}
}

func TestSplitting(t *testing.T) {
	t.Parallel()

	var lambda, r, s, _s, zero big.Int
	var l Lattice

	r.SetString("21888242871839275222246405745257275088548364400416034343698204186575808495617", 10)
	lambda.SetString("4407920970296243842393367215006156084916469457145843978461", 10)

	PrecomputeLattice(&r, &lambda, &l)

	s.SetString("183927522224640574525727508854836440041603434369820418657580", 10)

	v := SplitScalar(&s, &l)
	_s.Mul(&v[1], &lambda).Add(&_s, &v[0]).Sub(&_s, &s)
	_s.Mod(&_s, &r)
	if _s.Cmp(&zero) != 0 {
		t.Fatal("Error split scalar")
	}

}

func BenchmarkSplitting256(b *testing.B) {

	var lambda, r, s big.Int
	var l Lattice

	r.SetString("21888242871839275222246405745257275088548364400416034343698204186575808495617", 10)
	lambda.SetString("4407920970296243842393367215006156084916469457145843978461", 10)
	PrecomputeLattice(&r, &lambda, &l)
	s.SetString("183927522224640574525727508854836440041603434369820418657580", 10)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		SplitScalar(&s, &l)
	}

}
