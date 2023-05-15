/*
Copyright © 2020 ConsenSys

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package frontend

import (
	"github.com/consensys/gnark/frontend/internal/expr"
)

// Variable represents a variable in the circuit. Any integer type (e.g. int, *big.Int, fr.Element)
// can be assigned to it. It is also allowed to set a base-10 encoded string representing an integer value.
// The only purpose of putting this definition here is to avoid the import cycles (cs/plonk <-> frontend) and (cs/r1cs <-> frontend)
type Variable interface{}

// IsCanonical returns true if the Variable has been normalized in a (internal) LinearExpression
// by one of the constraint system builder. In other words, if the Variable is a circuit input OR
// returned by the API.
func IsCanonical(v Variable) bool {
	switch v.(type) {
	case expr.LinearExpression, *expr.LinearExpression, expr.Term, *expr.Term:
		return true
	}
	return false
}
