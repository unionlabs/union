package polynomial

import (
	"math/big"
	"testing"

	"github.com/mmcloughlin/addchain/internal/bigint"
)

func TestTermEvaluate(t *testing.T) {
	term := Term{A: 42, N: 3}
	x := big.NewInt(7)
	got := term.Evaluate(x)
	expect := int64(42 * 7 * 7 * 7)
	if !bigint.EqualInt64(got, expect) {
		t.Fatalf("evaluate %s at %v: got %v; expect %v", term, x, got, expect)
	}
}

func TestPolynomialString(t *testing.T) {
	cases := []struct {
		Polynomial Polynomial
		Expect     string
	}{
		{Polynomial{{1, 0}}, "1"},
		{Polynomial{{-1, 0}}, "-1"},
		{Polynomial{{42, 0}}, "42"},
		{Polynomial{{-42, 0}}, "-42"},

		{Polynomial{{1, 1}}, "x"},
		{Polynomial{{-1, 1}}, "-x"},
		{Polynomial{{42, 1}}, "42x"},
		{Polynomial{{-42, 1}}, "-42x"},

		{Polynomial{{1, 2}}, "x^2"},
		{Polynomial{{-1, 2}}, "-x^2"},
		{Polynomial{{42, 2}}, "42x^2"},
		{Polynomial{{-42, 2}}, "-42x^2"},

		{Polynomial{{-7, 0}, {-3, 3}, {1, 4}}, "x^4-3x^3-7"},
	}
	for _, c := range cases {
		if got := c.Polynomial.String(); got != c.Expect {
			t.Errorf("%#v.String() = %s; expect %s", c.Polynomial, got, c.Expect)
		}
	}
}

func TestPolynomialEvaluate(t *testing.T) {
	p := Polynomial{{-11, 0}, {-3, 3}, {1, 4}}
	x := big.NewInt(7)
	got := p.Evaluate(x)
	expect := int64(7*7*7*7 - 3*7*7*7 - 11)
	if !bigint.EqualInt64(got, expect) {
		t.Fatalf("evaluate %s at %v: got %v; expect %v", p, x, got, expect)
	}
}

func TestPolynomialDegree(t *testing.T) {
	p := Polynomial{{-11, 0}, {1, 4}, {2, 3}}
	if p.Degree() != 4 {
		t.FailNow()
	}
}
