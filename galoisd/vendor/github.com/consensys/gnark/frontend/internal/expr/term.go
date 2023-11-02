package expr

import "github.com/consensys/gnark/constraint"

type Term struct {
	VID   int
	Coeff constraint.Element
}

func NewTerm(vID int, cID constraint.Element) Term {
	return Term{Coeff: cID, VID: vID}
}

func (t *Term) SetCoeff(c constraint.Element) {
	t.Coeff = c
}

// TODO @gbotrel make that return a uint32
func (t Term) WireID() int {
	return t.VID
}

func (t Term) HashCode() uint64 {
	return t.Coeff[0]*29 + uint64(t.VID<<12)
}
