// Copyright 2020 ConsenSys AG
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package constraint

import (
	"errors"
	"strconv"
	"strings"
)

type SparseR1CS interface {
	ConstraintSystem

	// AddConstraint adds a constraint to the sytem and returns its id
	// This does not check for validity of the constraint.
	// If a debugInfo parameter is provided, it will be appended to the debug info structure
	// and will grow the memory usage of the constraint system.
	AddConstraint(c SparseR1C, debugInfo ...DebugInfo) int

	// GetConstraint return a pointer to the constraint at index i, or nil if out of bounds.
	GetConstraint(i int) *SparseR1C

	// GetCoefficient returns coefficient with given id in the coeff table.
	// calls panic if i is out of bounds, because this is called in the hot path of the compiler.
	GetCoefficient(i int) Coeff

	// GetConstraints return the list of SparseR1C and a helper for pretty printing.
	// See StringBuilder for more info.
	// ! this is an experimental API.
	GetConstraints() ([]SparseR1C, Resolver)
}

// R1CS describes a set of SparseR1C constraint
// TODO @gbotrel maybe SparseR1CSCore and R1CSCore should go in code generation directly to avoid confusing this package.
type SparseR1CSCore struct {
	System
	Constraints []SparseR1C
}

// GetNbConstraints returns the number of constraints
func (cs *SparseR1CSCore) GetNbConstraints() int {
	return len(cs.Constraints)
}

func (cs *SparseR1CSCore) UpdateLevel(cID int, c Iterable) {
	cs.updateLevel(cID, c)
}

func (system *SparseR1CSCore) CheckUnconstrainedWires() error {
	// TODO @gbotrel add unit test for that.

	inputConstrained := make([]bool, system.GetNbSecretVariables()+system.GetNbPublicVariables())
	cptInputs := len(inputConstrained)
	if cptInputs == 0 {
		return errors.New("invalid constraint system: no input defined")
	}

	cptHints := len(system.MHints)
	mHintsConstrained := make(map[int]bool)

	// for each constraint, we check the terms and mark our inputs / hints as constrained
	processTerm := func(t Term) {

		// L and M[0] handles the same wire but with a different coeff
		vID := t.WireID()
		if t.CoeffID() != CoeffIdZero {
			if vID < len(inputConstrained) {
				if !inputConstrained[vID] {
					inputConstrained[vID] = true
					cptInputs--
				}
			} else {
				// internal variable, let's check if it's a hint
				if _, ok := system.MHints[vID]; ok {
					vID -= (system.GetNbPublicVariables() + system.GetNbSecretVariables())
					if !mHintsConstrained[vID] {
						mHintsConstrained[vID] = true
						cptHints--
					}
				}
			}
		}

	}
	for _, c := range system.Constraints {
		processTerm(c.L)
		processTerm(c.R)
		processTerm(c.M[0])
		processTerm(c.M[1])
		processTerm(c.O)
		if cptHints|cptInputs == 0 {
			return nil // we can stop.
		}

	}

	// something is a miss, we build the error string
	var sbb strings.Builder
	if cptInputs != 0 {
		sbb.WriteString(strconv.Itoa(cptInputs))
		sbb.WriteString(" unconstrained input(s):")
		sbb.WriteByte('\n')
		for i := 0; i < len(inputConstrained) && cptInputs != 0; i++ {
			if !inputConstrained[i] {
				if i < len(system.Public) {
					sbb.WriteString(system.Public[i])
				} else {
					sbb.WriteString(system.Secret[i-len(system.Public)])
				}
				sbb.WriteByte('\n')
				cptInputs--
			}
		}
		sbb.WriteByte('\n')
	}

	if cptHints != 0 {
		sbb.WriteString(strconv.Itoa(cptHints))
		sbb.WriteString(" unconstrained hints")
		sbb.WriteByte('\n')
		// TODO we may add more debug info here → idea, in NewHint, take the debug stack, and store in the hint map some
		// debugInfo to find where a hint was declared (and not constrained)
	}
	return errors.New(sbb.String())
}

type CommitmentConstraint byte

const (
	NOT        CommitmentConstraint = 0
	COMMITTED  CommitmentConstraint = 1
	COMMITMENT CommitmentConstraint = 2
)

// SparseR1C used to compute the wires
// L+R+M[0]M[1]+O+k-committed?*PI2-commitment?*commitmentValue=0
// if a Term is zero, it means the field doesn't exist (ex M=[0,0] means there is no multiplicative term)
type SparseR1C struct {
	L, R, O    Term
	M          [2]Term
	K          int // stores only the ID of the constant term that is used
	Commitment CommitmentConstraint
}

// WireIterator implements constraint.Iterable
func (c *SparseR1C) WireIterator() func() int {
	curr := 0
	return func() int {
		switch curr {
		case 0:
			curr++
			return c.L.WireID()
		case 1:
			curr++
			return c.R.WireID()
		case 2:
			curr++
			return c.O.WireID()
		}
		return -1
	}
}

// String formats the constraint as qL⋅xa + qR⋅xb + qO⋅xc + qM⋅(xaxb) + qC == 0
func (c *SparseR1C) String(r Resolver) string {
	sbb := NewStringBuilder(r)
	sbb.WriteTerm(c.L)
	sbb.WriteString(" + ")
	sbb.WriteTerm(c.R)
	sbb.WriteString(" + ")
	sbb.WriteTerm(c.O)
	if qM := sbb.CoeffToString(c.M[0].CoeffID()); qM != "0" {
		xa := sbb.VariableToString(c.M[0].WireID())
		xb := sbb.VariableToString(c.M[1].WireID())
		sbb.WriteString(" + ")
		sbb.WriteString(qM)
		sbb.WriteString("⋅(")
		sbb.WriteString(xa)
		sbb.WriteString("×")
		sbb.WriteString(xb)
		sbb.WriteByte(')')
	}
	sbb.WriteString(" + ")
	sbb.WriteString(r.CoeffToString(c.K))
	sbb.WriteString(" == 0")
	return sbb.String()
}
