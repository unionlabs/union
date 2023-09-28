package constraint

import (
	"fmt"
)

// TODO @gbotrel this shouldn't be there, but we need to figure out a clean way to serialize
// blueprints

// BlueprintLookupHint is a blueprint that facilitates the lookup of values in a table.
// It is essentially a hint to the solver, but enables storing the table entries only once.
type BlueprintLookupHint struct {
	EntriesCalldata []uint32
}

// ensures BlueprintLookupHint implements the BlueprintSolvable interface
var _ BlueprintSolvable = (*BlueprintLookupHint)(nil)

func (b *BlueprintLookupHint) Solve(s Solver, inst Instruction) error {
	nbEntries := int(inst.Calldata[1])
	entries := make([]Element, nbEntries)

	// read the static entries from the blueprint
	// TODO @gbotrel cache that.
	offset, delta := 0, 0
	for i := 0; i < nbEntries; i++ {
		entries[i], delta = s.Read(b.EntriesCalldata[offset:])
		offset += delta
	}

	nbInputs := int(inst.Calldata[2])

	// read the inputs from the instruction
	inputs := make([]Element, nbInputs)
	offset = 3
	for i := 0; i < nbInputs; i++ {
		inputs[i], delta = s.Read(inst.Calldata[offset:])
		offset += delta
	}

	// set the outputs
	nbOutputs := nbInputs

	for i := 0; i < nbOutputs; i++ {
		idx, isUint64 := s.Uint64(inputs[i])
		if !isUint64 || idx >= uint64(len(entries)) {
			return fmt.Errorf("lookup query too large")
		}
		// we set the output wire to the value of the entry
		s.SetValue(uint32(i+int(inst.WireOffset)), entries[idx])
	}
	return nil
}

func (b *BlueprintLookupHint) CalldataSize() int {
	// variable size
	return -1
}
func (b *BlueprintLookupHint) NbConstraints() int {
	return 0
}

// NbOutputs return the number of output wires this blueprint creates.
func (b *BlueprintLookupHint) NbOutputs(inst Instruction) int {
	return int(inst.Calldata[2])
}

// Wires returns a function that walks the wires appearing in the blueprint.
// This is used by the level builder to build a dependency graph between instructions.
func (b *BlueprintLookupHint) WireWalker(inst Instruction) func(cb func(wire uint32)) {
	return func(cb func(wire uint32)) {
		// depend on the table UP to the number of entries at time of instruction creation.
		nbEntries := int(inst.Calldata[1])

		// invoke the callback on each wire appearing in the table
		j := 0
		for i := 0; i < nbEntries; i++ {
			// first we have the length of the linear expression
			n := int(b.EntriesCalldata[j])
			j++
			for k := 0; k < n; k++ {
				t := Term{CID: b.EntriesCalldata[j], VID: b.EntriesCalldata[j+1]}
				if !t.IsConstant() {
					cb(t.VID)
				}
				j += 2
			}
		}

		// invoke the callback on each wire appearing in the inputs
		nbInputs := int(inst.Calldata[2])
		j = 3
		for i := 0; i < nbInputs; i++ {
			// first we have the length of the linear expression
			n := int(inst.Calldata[j])
			j++
			for k := 0; k < n; k++ {
				t := Term{CID: inst.Calldata[j], VID: inst.Calldata[j+1]}
				if !t.IsConstant() {
					cb(t.VID)
				}
				j += 2
			}
		}

		// finally we have the outputs
		for i := 0; i < nbInputs; i++ {
			cb(uint32(i + int(inst.WireOffset)))
		}
	}
}
