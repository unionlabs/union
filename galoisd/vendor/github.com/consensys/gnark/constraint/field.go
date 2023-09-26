package constraint

import (
	"encoding/binary"
	"math/big"
)

// Element represents a term coefficient data. It is instantiated by the concrete
// constraint system implementation.
// Most of the scalar field used in gnark are on 4 uint64, so we have a clear memory overhead here.
type Element [6]uint64

// IsZero returns true if coefficient == 0
func (z *Element) IsZero() bool {
	return (z[5] | z[4] | z[3] | z[2] | z[1] | z[0]) == 0
}

// Bytes return the Element as a big-endian byte slice
func (z *Element) Bytes() [48]byte {
	var b [48]byte
	binary.BigEndian.PutUint64(b[40:48], z[0])
	binary.BigEndian.PutUint64(b[32:40], z[1])
	binary.BigEndian.PutUint64(b[24:32], z[2])
	binary.BigEndian.PutUint64(b[16:24], z[3])
	binary.BigEndian.PutUint64(b[8:16], z[4])
	binary.BigEndian.PutUint64(b[0:8], z[5])
	return b
}

// SetBytes sets the Element from a big-endian byte slice
func (z *Element) SetBytes(b [48]byte) {
	z[0] = binary.BigEndian.Uint64(b[40:48])
	z[1] = binary.BigEndian.Uint64(b[32:40])
	z[2] = binary.BigEndian.Uint64(b[24:32])
	z[3] = binary.BigEndian.Uint64(b[16:24])
	z[4] = binary.BigEndian.Uint64(b[8:16])
	z[5] = binary.BigEndian.Uint64(b[0:8])
}

// Field capability to perform arithmetic on Coeff
type Field interface {
	FromInterface(interface{}) Element
	ToBigInt(Element) *big.Int
	Mul(a, b Element) Element
	Add(a, b Element) Element
	Sub(a, b Element) Element
	Neg(a Element) Element
	Inverse(a Element) (Element, bool)
	One() Element
	IsOne(Element) bool
	String(Element) string
	Uint64(Element) (uint64, bool)
}
