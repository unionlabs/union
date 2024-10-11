package dict

import (
	"math/big"
	"math/rand"
	"testing"
	"time"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/alg/algtest"
	"github.com/mmcloughlin/addchain/alg/contfrac"
	"github.com/mmcloughlin/addchain/internal/bigint"
	"github.com/mmcloughlin/addchain/internal/bigints"
	"github.com/mmcloughlin/addchain/internal/test"
)

func TestDecomposersRandom(t *testing.T) {
	ds := []Decomposer{
		FixedWindow{K: 2},
		FixedWindow{K: 11},
		FixedWindow{K: 16},

		SlidingWindow{K: 2},
		SlidingWindow{K: 7},
		SlidingWindow{K: 12},

		RunLength{T: 0},
		RunLength{T: 1},
		RunLength{T: 3},
		RunLength{T: 7},

		Hybrid{K: 4, T: 7},
		Hybrid{K: 3, T: 0},
	}
	r := rand.New(rand.NewSource(time.Now().UnixNano()))
	for _, d := range ds {
		d := d
		t.Run(d.String(), test.Trials(func(t *testing.T) bool {
			n := bigint.RandBits(r, 256)
			got := d.Decompose(n)
			if !bigint.Equal(got.Int(), n) {
				t.Log(got)
				t.Fatalf("got %v expect %v", got.Int(), n)
			}
			return true
		}))
	}
}

func TestFixedWindow(t *testing.T) {
	n := big.NewInt(0xbeef0 << 3)
	f := FixedWindow{K: 4}
	got := f.Decompose(n)
	expect := Sum{
		{D: big.NewInt(0xf), E: 7},
		{D: big.NewInt(0xe), E: 11},
		{D: big.NewInt(0xe), E: 15},
		{D: big.NewInt(0xb), E: 19},
	}
	if !SumEquals(got, expect) {
		t.Fatalf("got %v expect %v", got, expect)
	}
}

func TestSlidingWindow(t *testing.T) {
	n := big.NewInt(0xf143)
	f := SlidingWindow{K: 4}
	got := f.Decompose(n)
	expect := Sum{
		{D: big.NewInt(0x3), E: 0},
		{D: big.NewInt(0x5), E: 6},
		{D: big.NewInt(0xf), E: 12},
	}
	if !SumEquals(got, expect) {
		t.Fatalf("got %v expect %v", got, expect)
	}
}

func TestRunLength(t *testing.T) {
	cases := []struct {
		T      uint
		X      int64
		Expect Sum
	}{
		{
			T: 4,
			X: 0xff,
			Expect: Sum{
				{D: big.NewInt(0xf), E: 0},
				{D: big.NewInt(0xf), E: 4},
			},
		},
		{
			T: 0,
			X: 0xff,
			Expect: Sum{
				{D: big.NewInt(0xff), E: 0},
			},
		},
		{
			T: 4,
			X: 0xf0f0,
			Expect: Sum{
				{D: big.NewInt(0xf), E: 4},
				{D: big.NewInt(0xf), E: 12},
			},
		},
		{
			T: 0,
			X: 0xf0f0,
			Expect: Sum{
				{D: big.NewInt(0xf), E: 4},
				{D: big.NewInt(0xf), E: 12},
			},
		},
		{
			T: 3,
			X: 0xff,
			Expect: Sum{
				{D: big.NewInt(0x3), E: 0},
				{D: big.NewInt(0x7), E: 2},
				{D: big.NewInt(0x7), E: 5},
			},
		},
	}
	for _, c := range cases {
		d := RunLength{T: c.T}
		if got := d.Decompose(big.NewInt(c.X)); !SumEquals(got, c.Expect) {
			t.Fatalf("Decompose(%#x) = %v; expect %v", c.X, got, c.Expect)
		}
	}
}

func TestHybrid(t *testing.T) {
	n := bigint.MustBinary("11111111_11111111_000_111_000000_1_0_111111_0_11_0")
	f := Hybrid{K: 4, T: 8}
	got := f.Decompose(n)
	expect := Sum{
		{D: big.NewInt(0x3), E: 1},
		{D: big.NewInt(0x3f), E: 4},
		{D: big.NewInt(0x1), E: 11},
		{D: big.NewInt(0x7), E: 18},
		{D: big.NewInt(0xff), E: 24},
		{D: big.NewInt(0xff), E: 32},
	}
	if !SumEquals(got, expect) {
		t.Fatalf("got %v expect %v", got, expect)
	}
}

func TestAlgorithm(t *testing.T) {
	a := NewAlgorithm(
		SlidingWindow{K: 4},
		contfrac.NewAlgorithm(contfrac.DichotomicStrategy{}),
	)
	n := big.NewInt(587257)
	c := algtest.AssertChainAlgorithmProduces(t, a, n)
	t.Log(c)
}

func TestPrimitive(t *testing.T) {
	// These tests are designed to verify that primitive dictionary reduction is
	// doing sensible things, therefore we construct a dictionary algorithm with a
	// decomposer that's going to have obvious results. We'll use the run length
	// decomposer.
	a := NewAlgorithm(
		RunLength{T: 0},
		contfrac.NewAlgorithm(contfrac.TotalStrategy{}),
	)

	// Cases are accompanied by an example of how this chain might be constructed
	// if dictionary reduction is working. We simply confirm that the resulting
	// chain is valid and at least as good as the example.
	cases := []struct {
		N       *big.Int
		Example addchain.Chain
	}{
		{
			N: bigint.MustBinary("1111_0_1_0"),
			Example: bigints.Int64s(
				1, 2, 3, // prep
				6, 12, 15, // << 2 and add 3
				30, 60, 61, // << 2 and add 1
				122, // << 1
			),
		},
		{
			N: bigint.MustBinary("11_0_11111111_0"),
			Example: bigints.Int64s(
				1, 2, 3, 6, 12, 15, // prepare 11 and 1111
				24, 48, 96, 111, // << 5 add 1111
				222, 444, 888, 1776, 1791, // << 4 add 1111
				3582, // << 1
			),
		},
	}

	for _, c := range cases {
		got := algtest.AssertChainAlgorithmProduces(t, a, c.N)
		if err := c.Example.Produces(c.N); err != nil {
			t.Fatalf("example is invalid: %s", err)
		}
		if len(got) > len(c.Example) {
			t.Logf("    got: %v", got)
			t.Logf("example: %v", c.Example)
			t.Errorf("suboptimal result: length %d but possible in %d", len(got), len(c.Example))
		}
	}
}

func SumEquals(a, b Sum) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i].E != b[i].E {
			return false
		}
		if !bigint.Equal(a[i].D, b[i].D) {
			return false
		}
	}
	return true
}
