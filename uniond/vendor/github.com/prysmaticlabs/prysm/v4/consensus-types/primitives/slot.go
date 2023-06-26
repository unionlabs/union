package primitives

import (
	"fmt"

	fssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/prysm/v4/math"
)

var _ fssz.HashRoot = (Slot)(0)
var _ fssz.Marshaler = (*Slot)(nil)
var _ fssz.Unmarshaler = (*Slot)(nil)

// Slot represents a single slot.
type Slot uint64

// Mul multiplies slot by x.
// In case of arithmetic issues (overflow/underflow/div by zero) panic is thrown.
func (s Slot) Mul(x uint64) Slot {
	res, err := s.SafeMul(x)
	if err != nil {
		panic(err.Error())
	}
	return res
}

// SafeMul multiplies slot by x.
// In case of arithmetic issues (overflow/underflow/div by zero) error is returned.
func (s Slot) SafeMul(x uint64) (Slot, error) {
	res, err := math.Mul64(uint64(s), x)
	return Slot(res), err
}

// MulSlot multiplies slot by another slot.
// In case of arithmetic issues (overflow/underflow/div by zero) panic is thrown.
func (s Slot) MulSlot(x Slot) Slot {
	return s.Mul(uint64(x))
}

// SafeMulSlot multiplies slot by another slot.
// In case of arithmetic issues (overflow/underflow/div by zero) error is returned.
func (s Slot) SafeMulSlot(x Slot) (Slot, error) {
	return s.SafeMul(uint64(x))
}

// Div divides slot by x.
// In case of arithmetic issues (overflow/underflow/div by zero) panic is thrown.
func (s Slot) Div(x uint64) Slot {
	res, err := s.SafeDiv(x)
	if err != nil {
		panic(err.Error())
	}
	return res
}

// SafeDiv divides slot by x.
// In case of arithmetic issues (overflow/underflow/div by zero) error is returned.
func (s Slot) SafeDiv(x uint64) (Slot, error) {
	res, err := math.Div64(uint64(s), x)
	return Slot(res), err
}

// DivSlot divides slot by another slot.
// In case of arithmetic issues (overflow/underflow/div by zero) panic is thrown.
func (s Slot) DivSlot(x Slot) Slot {
	return s.Div(uint64(x))
}

// SafeDivSlot divides slot by another slot.
// In case of arithmetic issues (overflow/underflow/div by zero) error is returned.
func (s Slot) SafeDivSlot(x Slot) (Slot, error) {
	return s.SafeDiv(uint64(x))
}

// Add increases slot by x.
// In case of arithmetic issues (overflow/underflow/div by zero) panic is thrown.
func (s Slot) Add(x uint64) Slot {
	res, err := s.SafeAdd(x)
	if err != nil {
		panic(err.Error())
	}
	return res
}

// SafeAdd increases slot by x.
// In case of arithmetic issues (overflow/underflow/div by zero) error is returned.
func (s Slot) SafeAdd(x uint64) (Slot, error) {
	res, err := math.Add64(uint64(s), x)
	return Slot(res), err
}

// AddSlot increases slot by another slot.
// In case of arithmetic issues (overflow/underflow/div by zero) panic is thrown.
func (s Slot) AddSlot(x Slot) Slot {
	return s.Add(uint64(x))
}

// SafeAddSlot increases slot by another slot.
// In case of arithmetic issues (overflow/underflow/div by zero) error is returned.
func (s Slot) SafeAddSlot(x Slot) (Slot, error) {
	return s.SafeAdd(uint64(x))
}

// Sub subtracts x from the slot.
// In case of arithmetic issues (overflow/underflow/div by zero) panic is thrown.
func (s Slot) Sub(x uint64) Slot {
	res, err := s.SafeSub(x)
	if err != nil {
		panic(err.Error())
	}
	return res
}

// SafeSub subtracts x from the slot.
// In case of arithmetic issues (overflow/underflow/div by zero) error is returned.
func (s Slot) SafeSub(x uint64) (Slot, error) {
	res, err := math.Sub64(uint64(s), x)
	return Slot(res), err
}

// SubSlot finds difference between two slot values.
// In case of arithmetic issues (overflow/underflow/div by zero) panic is thrown.
func (s Slot) SubSlot(x Slot) Slot {
	return s.Sub(uint64(x))
}

// SafeSubSlot finds difference between two slot values.
// In case of arithmetic issues (overflow/underflow/div by zero) error is returned.
func (s Slot) SafeSubSlot(x Slot) (Slot, error) {
	return s.SafeSub(uint64(x))
}

// Mod returns result of `slot % x`.
// In case of arithmetic issues (overflow/underflow/div by zero) panic is thrown.
func (s Slot) Mod(x uint64) Slot {
	res, err := s.SafeMod(x)
	if err != nil {
		panic(err.Error())
	}
	return res
}

// SafeMod returns result of `slot % x`.
// In case of arithmetic issues (overflow/underflow/div by zero) error is returned.
func (s Slot) SafeMod(x uint64) (Slot, error) {
	res, err := math.Mod64(uint64(s), x)
	return Slot(res), err
}

// ModSlot returns result of `slot % slot`.
// In case of arithmetic issues (overflow/underflow/div by zero) panic is thrown.
func (s Slot) ModSlot(x Slot) Slot {
	return s.Mod(uint64(x))
}

// SafeModSlot returns result of `slot % slot`.
// In case of arithmetic issues (overflow/underflow/div by zero) error is returned.
func (s Slot) SafeModSlot(x Slot) (Slot, error) {
	return s.SafeMod(uint64(x))
}

// HashTreeRoot --
func (s Slot) HashTreeRoot() ([32]byte, error) {
	return fssz.HashWithDefaultHasher(s)
}

// HashTreeRootWith --
func (s Slot) HashTreeRootWith(hh *fssz.Hasher) error {
	hh.PutUint64(uint64(s))
	return nil
}

// UnmarshalSSZ --
func (s *Slot) UnmarshalSSZ(buf []byte) error {
	if len(buf) != s.SizeSSZ() {
		return fmt.Errorf("expected buffer of length %d received %d", s.SizeSSZ(), len(buf))
	}
	*s = Slot(fssz.UnmarshallUint64(buf))
	return nil
}

// MarshalSSZTo --
func (s *Slot) MarshalSSZTo(dst []byte) ([]byte, error) {
	marshalled, err := s.MarshalSSZ()
	if err != nil {
		return nil, err
	}
	return append(dst, marshalled...), nil
}

// MarshalSSZ --
func (s *Slot) MarshalSSZ() ([]byte, error) {
	marshalled := fssz.MarshalUint64([]byte{}, uint64(*s))
	return marshalled, nil
}

// SizeSSZ --
func (s *Slot) SizeSSZ() int {
	return 8
}
