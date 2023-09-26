package expr

import (
	"github.com/consensys/gnark/constraint"
)

type LinearExpression []Term

// NewLinearExpression helper to initialize a linear expression with one term
func NewLinearExpression(vID int, cID constraint.Element) LinearExpression {
	return LinearExpression{Term{Coeff: cID, VID: vID}}
}

func (l LinearExpression) Clone() LinearExpression {
	res := make(LinearExpression, len(l))
	copy(res, l)
	return res
}

// Len return the length of the Variable (implements Sort interface)
func (l LinearExpression) Len() int {
	return len(l)
}

// Equals returns true if both SORTED expressions are the same
//
// pre conditions: l and o are sorted
func (l LinearExpression) Equal(o LinearExpression) bool {
	if len(l) != len(o) {
		return false
	}
	if (l == nil) != (o == nil) {
		return false
	}
	for i := 0; i < len(l); i++ {
		if l[i] != o[i] {
			return false
		}
	}
	return true
}

// Swap swaps terms in the Variable (implements Sort interface)
func (l LinearExpression) Swap(i, j int) {
	l[i], l[j] = l[j], l[i]
}

// Less returns true if variableID for term at i is less than variableID for term at j (implements Sort interface)
func (l LinearExpression) Less(i, j int) bool {
	iID := l[i].WireID()
	jID := l[j].WireID()
	return iID < jID
}

// HashCode returns a fast-to-compute but NOT collision resistant hash code identifier for the linear
// expression
func (l LinearExpression) HashCode() uint64 {
	h := uint64(17)
	for _, val := range l {
		h = h*23 + val.HashCode() // TODO @gbotrel revisit
	}
	return h
}
