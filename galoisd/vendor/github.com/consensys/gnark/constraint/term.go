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
	"math"
)

// ids of the coefficients with simple values in any cs.coeffs slice.
const (
	CoeffIdZero = iota
	CoeffIdOne
	CoeffIdTwo
	CoeffIdMinusOne
	CoeffIdMinusTwo
)

// Term represents a coeff * variable in a constraint system
type Term struct {
	CID, VID uint32
}

func (t *Term) MarkConstant() {
	t.VID = math.MaxUint32
}

func (t *Term) IsConstant() bool {
	return t.VID == math.MaxUint32
}

func (t *Term) WireID() int {
	return int(t.VID)
}

func (t *Term) CoeffID() int {
	return int(t.CID)
}

func (t Term) String(r Resolver) string {
	sbb := NewStringBuilder(r)
	sbb.WriteTerm(t)
	return sbb.String()
}

// implements constraint.Compressible

// Compress compresses the term into a slice of uint32 words.
// For compatibility with test engine and LinearExpression, the term is encoded as:
// 1, CID, VID (i.e a LinearExpression with a single term)
func (t Term) Compress(to *[]uint32) {
	(*to) = append((*to), 1, t.CID, t.VID)
}
