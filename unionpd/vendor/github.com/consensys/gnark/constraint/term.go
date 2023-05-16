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
