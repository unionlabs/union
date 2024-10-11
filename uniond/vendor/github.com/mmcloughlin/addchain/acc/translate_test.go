package acc

import (
	"reflect"
	"testing"

	"github.com/mmcloughlin/addchain/acc/ir"
	"github.com/mmcloughlin/addchain/acc/parse"
)

func TestTranslateBasicOps(t *testing.T) {
	src := "a = 1+1\nd = 2 * 1\n1 << 42"
	expect := &ir.Program{
		Instructions: []*ir.Instruction{
			{
				Output: &ir.Operand{
					Index:      1,
					Identifier: "a",
				},
				Op: ir.Add{
					X: &ir.Operand{Index: 0},
					Y: &ir.Operand{Index: 0},
				},
			},
			{
				Output: &ir.Operand{
					Index:      2,
					Identifier: "d",
				},
				Op: ir.Double{
					X: &ir.Operand{Index: 0},
				},
			},
			{
				Output: &ir.Operand{
					Index: 44,
				},
				Op: ir.Shift{
					X: &ir.Operand{Index: 0},
					S: 42,
				},
			},
		},
	}
	AssertTranslate(t, src, expect)
}

func AssertTranslate(t *testing.T, src string, expect *ir.Program) {
	t.Helper()

	s, err := parse.String(src)
	if err != nil {
		t.Fatal(err)
	}

	p, err := Translate(s)
	if err != nil {
		t.Fatal(err)
	}

	t.Logf("got:\n%s", p)

	if !reflect.DeepEqual(p, expect) {
		t.Logf("expect:\n%s", expect)
		t.Fatal("mismatch")
	}
}
