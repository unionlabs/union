package pass

import (
	"reflect"
	"testing"

	"github.com/mmcloughlin/addchain/acc/ir"
	"github.com/mmcloughlin/addchain/internal/assert"
)

func TestCanonicalizeOperands(t *testing.T) {
	// Index 1 is referenced multiple times from various operand types. Only one of
	// them has a name. Expect them to be replaced with a single version that has a
	// name.
	p := &ir.Program{
		Instructions: []*ir.Instruction{
			{
				Output: ir.Index(1), // unnamed
				Op: ir.Double{
					X: ir.One,
				},
			},
			{
				Output: ir.Index(2),
				Op: ir.Add{
					X: ir.One,
					Y: ir.NewOperand("a", 1), // named
				},
			},
			{
				Output: ir.Index(3),
				Op: ir.Double{
					X: ir.Index(1), // unnamed
				},
			},
			{
				Output: ir.Index(4),
				Op: ir.Shift{
					X: ir.Index(1), // unnamed
					S: 1,
				},
			},
		},
	}

	t.Logf("pre:\n%s", p)

	err := CanonicalizeOperands(p)
	if err != nil {
		t.Fatal(err)
	}

	t.Logf("post:\n%s", p)

	// Check all operands are canonical.
	for _, i := range p.Instructions {
		for _, operand := range i.Operands() {
			if operand != p.Operands[operand.Index] {
				t.Fatal("non canonical operand found")
			}
		}
	}

	// Check that the canonical version has a name.
	if p.Operands[1].Identifier != "a" {
		t.Fatal("operand 1 should have a name")
	}
}

func TestCanonicalizeOperandsIdentifierConflict(t *testing.T) {
	// Construct a program with two names for index 1.
	p := &ir.Program{
		Instructions: []*ir.Instruction{
			{
				Output: ir.NewOperand("a", 1),
				Op: ir.Double{
					X: ir.One,
				},
			},
			{
				Output: ir.NewOperand("c", 2),
				Op: ir.Add{
					X: ir.One,
					Y: ir.NewOperand("b", 1),
				},
			},
		},
	}

	err := CanonicalizeOperands(p)
	assert.ErrorContains(t, err, "conflict")
}

func TestReadCounts(t *testing.T) {
	p := &ir.Program{
		Instructions: []*ir.Instruction{
			{
				Output: ir.Index(1),
				Op: ir.Double{
					X: ir.Index(0),
				},
			},
			{
				Output: ir.Index(2),
				Op: ir.Add{
					X: ir.Index(0),
					Y: ir.Index(1),
				},
			},
			{
				Output: ir.Index(5),
				Op: ir.Shift{
					X: ir.Index(1),
					S: 3,
				},
			},
			{
				Output: ir.Index(6),
				Op: ir.Add{
					X: ir.Index(0),
					Y: ir.Index(5),
				},
			},
		},
	}
	expect := map[int]int{
		0: 3,
		1: 2,
		5: 1,
	}

	if err := ReadCounts(p); err != nil {
		t.Fatal(p)
	}

	if !reflect.DeepEqual(expect, p.ReadCount) {
		t.FailNow()
	}
}
