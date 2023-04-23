package addchain

import (
	"fmt"
	"math/big"

	"github.com/mmcloughlin/addchain/internal/bigint"
)

// Op is an instruction to add positions I and J in a chain.
type Op struct{ I, J int }

// IsDouble returns whether this operation is a doubling.
func (o Op) IsDouble() bool { return o.I == o.J }

// Operands returns the indicies used in this operation. This will contain one
// or two entries depending on whether this is a doubling.
func (o Op) Operands() []int {
	if o.IsDouble() {
		return []int{o.I}
	}
	return []int{o.I, o.J}
}

// Uses reports whether the given index is one of the operands.
func (o Op) Uses(i int) bool {
	return o.I == i || o.J == i
}

// Program is a sequence of operations.
type Program []Op

// Shift appends a sequence of operations that bitwise shift index i left by s,
// equivalent to s double operations. Returns the index of the result.
func (p *Program) Shift(i int, s uint) (int, error) {
	for ; s > 0; s-- {
		next, err := p.Double(i)
		if err != nil {
			return 0, err
		}
		i = next
	}
	return i, nil
}

// Double appends an operation that doubles index i. Returns the index of the
// result.
func (p *Program) Double(i int) (int, error) {
	return p.Add(i, i)
}

// Add appends an operation that adds indices i and j. Returns the index of the
// result.
func (p *Program) Add(i, j int) (int, error) {
	if err := p.boundscheck(i); err != nil {
		return 0, err
	}
	if err := p.boundscheck(j); err != nil {
		return 0, err
	}
	*p = append(*p, Op{i, j})
	return len(*p), nil
}

// boundscheck returns an error if i is out of bounds.
func (p Program) boundscheck(i int) error {
	// Note the corresponding chain is one longer than the program.
	n := len(p)
	switch {
	case i < 0:
		return fmt.Errorf("negative index %d", i)
	case i > n:
		return fmt.Errorf("index %d out of bounds", i)
	}
	return nil
}

// Doubles returns the number of doubles in the program.
func (p Program) Doubles() int {
	doubles, _ := p.Count()
	return doubles
}

// Adds returns the number of adds in the program.
func (p Program) Adds() int {
	_, adds := p.Count()
	return adds
}

// Count returns the number of doubles and adds in the program.
func (p Program) Count() (doubles, adds int) {
	for _, op := range p {
		if op.IsDouble() {
			doubles++
		} else {
			adds++
		}
	}
	return
}

// Evaluate executes the program and returns the resulting chain.
func (p Program) Evaluate() Chain {
	c := New()
	for _, op := range p {
		sum := new(big.Int).Add(c[op.I], c[op.J])
		c = append(c, sum)
	}
	return c
}

// ReadCounts returns how many times each index is read in the program.
func (p Program) ReadCounts() []int {
	reads := make([]int, len(p)+1)
	for _, op := range p {
		for _, i := range op.Operands() {
			reads[i]++
		}
	}
	return reads
}

// Dependencies returns an array of bitsets where each bitset contains the set
// of indicies that contributed to that position.
func (p Program) Dependencies() []*big.Int {
	bitsets := []*big.Int{bigint.One()}
	for i, op := range p {
		bitset := new(big.Int).Or(bitsets[op.I], bitsets[op.J])
		bitset.SetBit(bitset, i+1, 1)
		bitsets = append(bitsets, bitset)
	}
	return bitsets
}
