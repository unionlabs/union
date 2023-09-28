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

type R1CS interface {
	ConstraintSystem

	// AddR1C adds a constraint to the system and returns its id
	// This does not check for validity of the constraint.
	AddR1C(r1c R1C, bID BlueprintID) int

	// GetR1Cs return the list of R1C
	// See StringBuilder for more info.
	// ! this is an experimental API.
	GetR1Cs() []R1C

	// GetR1CIterator returns an R1CIterator to iterate on the R1C constraints of the system.
	GetR1CIterator() R1CIterator
}

// R1CIterator facilitates iterating through R1C constraints.
type R1CIterator struct {
	R1C
	cs *System
	n  int
}

// Next returns the next R1C or nil if end. Caller must not store the result since the
// same memory space is re-used for subsequent calls to Next.
func (it *R1CIterator) Next() *R1C {
	if it.n >= it.cs.GetNbInstructions() {
		return nil
	}
	inst := it.cs.Instructions[it.n]
	it.n++
	blueprint := it.cs.Blueprints[inst.BlueprintID]
	if bc, ok := blueprint.(BlueprintR1C); ok {
		bc.DecompressR1C(&it.R1C, inst.Unpack(it.cs))
		return &it.R1C
	}
	return it.Next()
}

// // IsValid perform post compilation checks on the Variables
// //
// // 1. checks that all user inputs are referenced in at least one constraint
// // 2. checks that all hints are constrained
// func (r1cs *R1CSCore) CheckUnconstrainedWires() error {
// 	return nil

// 	// TODO @gbotrel add unit test for that.

// 	inputConstrained := make([]bool, r1cs.GetNbSecretVariables()+r1cs.GetNbPublicVariables())
// 	// one wire does not need to be constrained
// 	inputConstrained[0] = true
// 	cptInputs := len(inputConstrained) - 1 // marking 1 wire as already constrained // TODO @gbotrel check that
// 	if cptInputs == 0 {
// 		return errors.New("invalid constraint system: no input defined")
// 	}

// 	cptHints := len(r1cs.MHints)
// 	mHintsConstrained := make(map[int]bool)

// 	// for each constraint, we check the linear expressions and mark our inputs / hints as constrained
// 	processLinearExpression := func(l LinearExpression) {
// 		for _, t := range l {
// 			if t.CoeffID() == CoeffIdZero {
// 				// ignore zero coefficient, as it does not constraint the Variable
// 				// though, we may want to flag that IF the Variable doesn't appear else where
// 				continue
// 			}
// 			vID := t.WireID()
// 			if vID < len(inputConstrained) {
// 				if !inputConstrained[vID] {
// 					inputConstrained[vID] = true
// 					cptInputs--
// 				}
// 			} else {
// 				// internal variable, let's check if it's a hint
// 				if _, ok := r1cs.MHints[vID]; ok {
// 					if !mHintsConstrained[vID] {
// 						mHintsConstrained[vID] = true
// 						cptHints--
// 					}
// 				}
// 			}

// 		}
// 	}
// 	for _, r1c := range r1cs.Constraints {
// 		processLinearExpression(r1c.L)
// 		processLinearExpression(r1c.R)
// 		processLinearExpression(r1c.O)

// 		if cptHints|cptInputs == 0 {
// 			return nil // we can stop.
// 		}

// 	}

// 	// something is a miss, we build the error string
// 	var sbb strings.Builder
// 	if cptInputs != 0 {
// 		sbb.WriteString(strconv.Itoa(cptInputs))
// 		sbb.WriteString(" unconstrained input(s):")
// 		sbb.WriteByte('\n')
// 		for i := 0; i < len(inputConstrained) && cptInputs != 0; i++ {
// 			if !inputConstrained[i] {
// 				if i < len(r1cs.Public) {
// 					sbb.WriteString(r1cs.Public[i])
// 				} else {
// 					sbb.WriteString(r1cs.Secret[i-len(r1cs.Public)])
// 				}

// 				sbb.WriteByte('\n')
// 				cptInputs--
// 			}
// 		}
// 		sbb.WriteByte('\n')
// 		return errors.New(sbb.String())
// 	}

// 	if cptHints != 0 {
// 		// TODO @gbotrel @ivokub investigate --> emulated hints seems to go in this path a lot.
// 		sbb.WriteString(strconv.Itoa(cptHints))
// 		sbb.WriteString(" unconstrained hints; i.e. wire created through NewHint() but doesn't not appear in the constraint system")
// 		sbb.WriteByte('\n')
// 		log := logger.Logger()
// 		log.Warn().Err(errors.New(sbb.String())).Send()
// 		return nil
// 		// TODO we may add more debug info here → idea, in NewHint, take the debug stack, and store in the hint map some
// 		// debugInfo to find where a hint was declared (and not constrained)
// 	}
// 	return errors.New(sbb.String())
// }

// R1C used to compute the wires
type R1C struct {
	L, R, O LinearExpression
}

// String formats a R1C as L⋅R == O
func (r1c *R1C) String(r Resolver) string {
	sbb := NewStringBuilder(r)
	sbb.WriteLinearExpression(r1c.L)
	sbb.WriteString(" ⋅ ")
	sbb.WriteLinearExpression(r1c.R)
	sbb.WriteString(" == ")
	sbb.WriteLinearExpression(r1c.O)
	return sbb.String()
}
