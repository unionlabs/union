// Copyright 2021 ConsenSys AG
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

// A LinearExpression is a linear combination of Term
type LinearExpression []Term

// Clone returns a copy of the underlying slice
func (l LinearExpression) Clone() LinearExpression {
	res := make(LinearExpression, len(l))
	copy(res, l)
	return res
}

func (l LinearExpression) String(r Resolver) string {
	sbb := NewStringBuilder(r)
	sbb.WriteLinearExpression(l)
	return sbb.String()
}

func (l LinearExpression) Compress(to *[]uint32) {
	(*to) = append((*to), uint32(len(l)))
	for i := 0; i < len(l); i++ {
		(*to) = append((*to), l[i].CID, l[i].VID)
	}
}
