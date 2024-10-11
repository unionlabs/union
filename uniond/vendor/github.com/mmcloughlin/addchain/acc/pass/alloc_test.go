package pass

import (
	"reflect"
	"strconv"
	"testing"

	"github.com/mmcloughlin/addchain/acc/ir"
)

func TestAllocator(t *testing.T) {
	// Generate a program that will need n live variables.
	n := 13
	p := &ir.Program{}

	// Create n indexes that will be used later.
	for i := 1; i <= n; i++ {
		p.AddInstruction(&ir.Instruction{
			Output: ir.Index(i),
			Op: ir.Add{
				X: ir.One,
				Y: ir.Index(i - 1),
			},
		})
	}

	// Consume the n indicies. This ensures we'll have n live variables at some
	// point.
	for i := n + 1; i <= 2*n; i++ {
		p.AddInstruction(&ir.Instruction{
			Output: ir.Index(i),
			Op: ir.Add{
				X: ir.Index(i - n),
				Y: ir.Index(i - 1),
			},
		})
	}

	// Execute allocation pass.
	t.Logf("pre:\n%s", p)

	a := Allocator{
		Input:  "in",
		Output: "out",
		Format: "tmp%d",
	}
	if err := a.Execute(p); err != nil {
		t.Fatal(err)
	}

	t.Logf("post:\n%s", p)

	// Every operand should have a name.
	for _, operand := range p.Operands {
		if operand.Identifier == "" {
			t.Errorf("operand %s does not have a name", operand)
		}
	}

	// We expect to have n-1 temporaries. Note we do not expect n, since the output
	// variable will be used as a temporary.
	expect := make([]string, n-1)
	for i := 0; i < n-1; i++ {
		expect[i] = "tmp" + strconv.Itoa(i)
	}
	if !reflect.DeepEqual(expect, p.Temporaries) {
		t.Fatalf("got temporaries %v; expect %v", p.Temporaries, expect)
	}

	// Confirm input and output variables are used.
	if p.Operands[0].Identifier != a.Input {
		t.Errorf("unexpected input name: got %q expect %q", p.Operands[0].Identifier, a.Input)
	}

	if p.Output().Identifier != a.Output {
		t.Errorf("unexpected output name: got %q expect %q", p.Output().Identifier, a.Output)
	}
}
