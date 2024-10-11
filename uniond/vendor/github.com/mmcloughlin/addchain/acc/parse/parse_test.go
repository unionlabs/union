package parse

import (
	"reflect"
	"testing"

	"github.com/mmcloughlin/addchain/acc/ast"
)

func TestOperands(t *testing.T) {
	exprs := map[string]ast.Expr{
		"1":       ast.Operand(0),
		"[23]":    ast.Operand(23),
		"[0x2a]":  ast.Operand(0x2a),
		"[0644]":  ast.Operand(0o644),
		"(1)":     ast.Operand(0),
		"( [3] )": ast.Operand(3),
		"_101":    ast.Identifier("_101"),
		"x5":      ast.Identifier("x5"),
		"( x5)":   ast.Identifier("x5"),
	}
	for expr, idx := range exprs {
		AssertParseResult(t, expr, idx)
		AssertParseResult(t, expr+"\n", idx)
	}
}

func TestExpressions(t *testing.T) {
	cases := []struct {
		Source string
		Expect ast.Expr
	}{
		{
			Source: "1 + 1",
			Expect: ast.Add{X: ast.Operand(0), Y: ast.Operand(0)},
		},
		{
			Source: "1 add 1",
			Expect: ast.Add{X: ast.Operand(0), Y: ast.Operand(0)},
		},
		{
			Source: "1 + 1 + 1",
			Expect: ast.Add{
				X: ast.Add{X: ast.Operand(0), Y: ast.Operand(0)},
				Y: ast.Operand(0),
			},
		},
		{
			Source: "1 add 1 add 1",
			Expect: ast.Add{
				X: ast.Add{X: ast.Operand(0), Y: ast.Operand(0)},
				Y: ast.Operand(0),
			},
		},
		{
			Source: "1 << 5",
			Expect: ast.Shift{X: ast.Operand(0), S: 5},
		},
		{
			Source: "[3] shl 5 add 1",
			Expect: ast.Add{
				X: ast.Shift{X: ast.Operand(3), S: 5},
				Y: ast.Operand(0),
			},
		},
		{
			Source: "[3] << 5 + [6] << 9",
			Expect: ast.Add{
				X: ast.Shift{X: ast.Operand(3), S: 5},
				Y: ast.Shift{X: ast.Operand(6), S: 9},
			},
		},
		{
			Source: "([3] + [8]) << 2",
			Expect: ast.Shift{
				X: ast.Add{X: ast.Operand(3), Y: ast.Operand(8)},
				S: 2,
			},
		},
		{
			Source: "2 * [3]",
			Expect: ast.Double{
				X: ast.Operand(3),
			},
		},
		{
			Source: "dbl [3] add [5] << 3",
			Expect: ast.Add{
				X: ast.Double{X: ast.Operand(3)},
				Y: ast.Shift{X: ast.Operand(5), S: 3},
			},
		},
	}
	for _, c := range cases {
		AssertParseResult(t, c.Source, c.Expect)
	}
}

func TestChains(t *testing.T) {
	cases := []struct {
		Source string
		Expect *ast.Chain
	}{
		{
			Source: "x = 1 + 1\nreturn x<<3",
			Expect: &ast.Chain{
				Statements: []ast.Statement{
					{
						Name: "x",
						Expr: ast.Add{
							X: ast.Operand(0),
							Y: ast.Operand(0),
						},
					},
					{
						Expr: ast.Shift{
							X: ast.Identifier("x"),
							S: 3,
						},
					},
				},
			},
		},
	}
	for _, c := range cases {
		AssertParse(t, c.Source, c.Expect)
	}
}

func AssertParseResult(t *testing.T, src string, expect ast.Expr) {
	t.Helper()
	AssertParse(t, src, &ast.Chain{
		Statements: []ast.Statement{{Expr: expect}},
	})
}

func AssertParse(t *testing.T, src string, expect *ast.Chain) {
	t.Helper()
	t.Logf("expr=%q", src)
	got, err := String(src)
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(expect, got) {
		t.Fatalf("got %#v; expected %#v", got, expect)
	}
}
