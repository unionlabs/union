package test

import (
	"math/big"

	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/internal/utils"
)

// blueprintSolver is a constraint.Solver that can be used to test a circuit
// it is a separate type to avoid method collisions with the engine.
type blueprintSolver struct {
	internalVariables []*big.Int
	q                 *big.Int
}

// implements constraint.Solver

func (s *blueprintSolver) SetValue(vID uint32, f constraint.Element) {
	if int(vID) > len(s.internalVariables) {
		panic("out of bounds")
	}
	v := s.ToBigInt(f)
	s.internalVariables[vID].Set(v)
}

func (s *blueprintSolver) GetValue(cID, vID uint32) constraint.Element {
	panic("not implemented in test.Engine")
}
func (s *blueprintSolver) GetCoeff(cID uint32) constraint.Element {
	panic("not implemented in test.Engine")
}

func (s *blueprintSolver) IsSolved(vID uint32) bool {
	panic("not implemented in test.Engine")
}

// implements constraint.Field

func (s *blueprintSolver) FromInterface(i interface{}) constraint.Element {
	b := utils.FromInterface(i)
	return s.toElement(&b)
}

func (s *blueprintSolver) ToBigInt(f constraint.Element) *big.Int {
	r := new(big.Int)
	fBytes := f.Bytes()
	r.SetBytes(fBytes[:])
	return r
}
func (s *blueprintSolver) Mul(a, b constraint.Element) constraint.Element {
	ba, bb := s.ToBigInt(a), s.ToBigInt(b)
	ba.Mul(ba, bb).Mod(ba, s.q)
	return s.toElement(ba)
}
func (s *blueprintSolver) Add(a, b constraint.Element) constraint.Element {
	ba, bb := s.ToBigInt(a), s.ToBigInt(b)
	ba.Add(ba, bb).Mod(ba, s.q)
	return s.toElement(ba)
}
func (s *blueprintSolver) Sub(a, b constraint.Element) constraint.Element {
	ba, bb := s.ToBigInt(a), s.ToBigInt(b)
	ba.Sub(ba, bb).Mod(ba, s.q)
	return s.toElement(ba)
}
func (s *blueprintSolver) Neg(a constraint.Element) constraint.Element {
	ba := s.ToBigInt(a)
	ba.Neg(ba).Mod(ba, s.q)
	return s.toElement(ba)
}
func (s *blueprintSolver) Inverse(a constraint.Element) (constraint.Element, bool) {
	ba := s.ToBigInt(a)
	r := ba.ModInverse(ba, s.q)
	return s.toElement(ba), r != nil
}
func (s *blueprintSolver) One() constraint.Element {
	b := new(big.Int).SetUint64(1)
	return s.toElement(b)
}
func (s *blueprintSolver) IsOne(a constraint.Element) bool {
	b := s.ToBigInt(a)
	return b.IsUint64() && b.Uint64() == 1
}

func (s *blueprintSolver) String(a constraint.Element) string {
	b := s.ToBigInt(a)
	return b.String()
}

func (s *blueprintSolver) Uint64(a constraint.Element) (uint64, bool) {
	b := s.ToBigInt(a)
	return b.Uint64(), b.IsUint64()
}

func (s *blueprintSolver) Read(calldata []uint32) (constraint.Element, int) {
	// We encoded big.Int as constraint.Element on 12 uint32 words.
	var r constraint.Element
	for i := 0; i < len(r); i++ {
		index := i * 2
		r[i] = uint64(calldata[index])<<32 | uint64(calldata[index+1])
	}
	return r, len(r) * 2
}

func (s *blueprintSolver) toElement(b *big.Int) constraint.Element {
	return bigIntToElement(b)
}

func bigIntToElement(b *big.Int) constraint.Element {
	if b.Sign() == -1 {
		panic("negative value")
	}
	bytes := b.Bytes()
	if len(bytes) > 48 {
		panic("value too big")
	}
	var paddedBytes [48]byte
	copy(paddedBytes[48-len(bytes):], bytes[:])

	var r constraint.Element
	r.SetBytes(paddedBytes)

	return r
}

// wrappedBigInt is a wrapper around big.Int to implement the frontend.CanonicalVariable interface
type wrappedBigInt struct {
	*big.Int
}

func (w wrappedBigInt) Compress(to *[]uint32) {
	// convert to Element.
	e := bigIntToElement(w.Int)

	// append the uint32 words to the slice
	for i := 0; i < len(e); i++ {
		*to = append(*to, uint32(e[i]>>32))
		*to = append(*to, uint32(e[i]&0xffffffff))
	}
}
