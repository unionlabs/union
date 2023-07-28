package expr

import (
	"github.com/consensys/gnark/constraint"
)

// TODO @gbotrel --> storing a UUID in the linear expressions would enable better perf
// in the frontend -> check a linear expression is boolean, or has been converted to a
// "backend" constraint.LinearExpresion ... and avoid duplicating work would be interesting.

type LinearExpression []Term

func (l LinearExpression) Clone() LinearExpression {
	res := make(LinearExpression, len(l))
	copy(res, l)
	return res
}

// NewLinearExpression helper to initialize a linear expression with one term
func NewLinearExpression(vID int, cID constraint.Coeff) LinearExpression {
	return LinearExpression{Term{Coeff: cID, VID: vID}}
}

func NewTerm(vID int, cID constraint.Coeff) Term {
	return Term{Coeff: cID, VID: vID}
}

type Term struct {
	VID   int
	Coeff constraint.Coeff
}

func (t *Term) SetCoeff(c constraint.Coeff) {
	t.Coeff = c
}
func (t Term) WireID() int {
	return t.VID
}

func (t Term) HashCode() uint64 {
	return t.Coeff[0]*29 + uint64(t.VID<<12)
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
