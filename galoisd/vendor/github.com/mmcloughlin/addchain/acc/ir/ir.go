// Package ir declares an intermediate representation for acc programs.
package ir

import (
	"fmt"
	"strings"

	"github.com/mmcloughlin/addchain"
)

// Program is a sequence of acc instructions.
type Program struct {
	Instructions []*Instruction

	// Pass/analysis results.
	Operands    map[int]*Operand
	ReadCount   map[int]int
	Program     addchain.Program
	Chain       addchain.Chain
	Temporaries []string
}

// AddInstruction appends an instruction to the program.
func (p *Program) AddInstruction(i *Instruction) {
	p.Instructions = append(p.Instructions, i)
}

// Output returns the output of the last instruction.
func (p Program) Output() *Operand {
	last := len(p.Instructions) - 1
	return p.Instructions[last].Output
}

// Clone returns a copy of p. Pass results are not copied and would need to be
// rerun on the clone.
func (p Program) Clone() *Program {
	c := &Program{}
	for _, inst := range p.Instructions {
		c.Instructions = append(c.Instructions, inst.Clone())
	}
	return c
}

func (p Program) String() string {
	var b strings.Builder
	for _, i := range p.Instructions {
		fmt.Fprintln(&b, i)
	}
	return b.String()
}

// Operand represents an element of an addition chain, with an optional
// identifier.
type Operand struct {
	Identifier string
	Index      int
}

// NewOperand builds a named operand for index i.
func NewOperand(name string, i int) *Operand {
	return &Operand{
		Identifier: name,
		Index:      i,
	}
}

// Index builds an unnamed operand for index i.
func Index(i int) *Operand {
	return NewOperand("", i)
}

// One is the first element in the addition chain, which by definition always
// has the value 1.
var One = Index(0)

// Clone returns a copy of the operand.
func (o Operand) Clone() *Operand {
	clone := o
	return &clone
}

func (o Operand) String() string {
	if len(o.Identifier) > 0 {
		return o.Identifier
	}
	return fmt.Sprintf("[%d]", o.Index)
}

// Instruction assigns the result of an operation to an operand.
type Instruction struct {
	Output *Operand
	Op     Op
}

// Operands returns the input and output operands.
func (i Instruction) Operands() []*Operand {
	return append(i.Op.Inputs(), i.Output)
}

// Clone returns a copy of the instruction.
func (i Instruction) Clone() *Instruction {
	return &Instruction{
		Output: i.Output.Clone(),
		Op:     i.Op.Clone(),
	}
}

func (i Instruction) String() string {
	return fmt.Sprintf("%s \u2190 %s", i.Output, i.Op)
}

// Op is an operation.
type Op interface {
	Inputs() []*Operand
	Clone() Op
	String() string
}

// Add is an addition operation.
type Add struct {
	X, Y *Operand
}

// Inputs returns the addends.
func (a Add) Inputs() []*Operand {
	return []*Operand{a.X, a.Y}
}

// Clone returns a copy of the operation.
func (a Add) Clone() Op {
	return Add{
		X: a.X.Clone(),
		Y: a.Y.Clone(),
	}
}

func (a Add) String() string {
	return fmt.Sprintf("%s + %s", a.X, a.Y)
}

// Double is a double operation.
type Double struct {
	X *Operand
}

// Inputs returns the operand.
func (d Double) Inputs() []*Operand {
	return []*Operand{d.X}
}

// Clone returns a copy of the operation.
func (d Double) Clone() Op {
	return Double{
		X: d.X.Clone(),
	}
}

func (d Double) String() string {
	return fmt.Sprintf("2 * %s", d.X)
}

// Shift represents a shift-left operation, equivalent to repeat doubling.
type Shift struct {
	X *Operand
	S uint
}

// Inputs returns the operand to be shifted.
func (s Shift) Inputs() []*Operand {
	return []*Operand{s.X}
}

// Clone returns a copy of the operation.
func (s Shift) Clone() Op {
	return Shift{
		X: s.X.Clone(),
		S: s.S,
	}
}

func (s Shift) String() string {
	return fmt.Sprintf("%s \u226a %d", s.X, s.S)
}

// HasInput reports whether the given operation takes idx as an input.
func HasInput(op Op, idx int) bool {
	for _, input := range op.Inputs() {
		if input.Index == idx {
			return true
		}
	}
	return false
}
