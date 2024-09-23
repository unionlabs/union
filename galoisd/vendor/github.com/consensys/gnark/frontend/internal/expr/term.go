package expr

import (
	"encoding/binary"

	"github.com/consensys/gnark/constraint"
	"golang.org/x/crypto/blake2b"
)

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

// HashCode returns a collision resistant hash code identifier for the term.
func (t Term) HashCode() [16]byte {
	h, err := blake2b.New256(nil)
	if err != nil {
		panic(err)
	}
	h.Write(binary.BigEndian.AppendUint64(nil, uint64(t.VID)))
	for i := range t.Coeff {
		h.Write(binary.BigEndian.AppendUint64(nil, uint64(t.Coeff[i])))
	}
	crc := h.Sum(nil)
	return [16]byte(crc[:16])
}
