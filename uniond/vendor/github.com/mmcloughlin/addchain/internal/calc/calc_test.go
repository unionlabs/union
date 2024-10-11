package calc

import (
	"testing"

	"github.com/mmcloughlin/addchain/internal/bigint"
)

func TestEval(t *testing.T) {
	cases := []struct {
		Expr   string
		Expect int64
	}{
		// Numeric literals.
		{"2", 2},
		{"34534", 34534},
		{"-42", -42},
		{"-0xab", -0xab},
		{"0b1011", 11},

		// Single operators.
		{"15-2", 15 - 2},
		{"15+2", 15 + 2},
		{"15/2", 15 / 2},
		{"15*2", 15 * 2},
		{"2^10", 1 << 10},

		// Whitespace.
		{" 2   ^ 10   ", 1 << 10},

		// Operator combinations.
		{"15+2-9", 15 + 2 - 9},
		{"15+2*9", 15 + 2*9},
		{"15+9/2", 15 + 9/2},
		{"15+2^9", 15 + (1 << 9)},
		{"15*2+9", 15*2 + 9},
		{"15/2+9", 15/2 + 9},
		{"15^2+9", 15*15 + 9},
		{"15^2*9+40/2^2", 15*15*9 + 10},
	}
	for _, c := range cases {
		x, err := Eval(c.Expr)
		if err != nil {
			t.Fatalf("Eval(%v) returned error %q", c.Expr, err)
		}
		if !bigint.EqualInt64(x, c.Expect) {
			t.Errorf("Eval(%v) = %v; expect %v", c.Expr, x, c.Expect)
		}
	}
}

func TestEvalErrors(t *testing.T) {
	exprs := []string{
		"",
		"10 +",
		"1 + +",
		" + ",
		" + abc",
		"10 20 +",
		"10 20 3 + -",
	}
	for _, expr := range exprs {
		_, err := Eval(expr)
		if err == nil {
			t.Errorf("Eval(%v): expected error", expr)
		}
	}
}
