package math

import (
	"errors"
	"fmt"
	"math/big"
)

// Uint wraps integer with 256 bit range bound
// Checks overflow, underflow and division by zero
// Exists in range from 0 to 2^256-1
type Uint struct {
	i *big.Int
}

// BigInt converts Uint to big.Int
func (u Uint) BigInt() *big.Int {
	return new(big.Int).Set(u.i)
}

// IsNil returns true if Uint is uninitialized
func (u Uint) IsNil() bool {
	return u.i == nil
}

// NewUintFromBigUint constructs Uint from big.Uint
func NewUintFromBigInt(i *big.Int) Uint {
	u, err := checkNewUint(i)
	if err != nil {
		panic(fmt.Errorf("overflow: %s", err))
	}
	return u
}

// NewUint constructs Uint from int64
func NewUint(n uint64) Uint {
	i := new(big.Int)
	i.SetUint64(n)
	return NewUintFromBigInt(i)
}

// NewUintFromString constructs Uint from string
func NewUintFromString(s string) Uint {
	u, err := ParseUint(s)
	if err != nil {
		panic(err)
	}
	return u
}

// ZeroUint returns unsigned zero.
func ZeroUint() Uint { return Uint{big.NewInt(0)} }

// OneUint returns Uint value with one.
func OneUint() Uint { return Uint{big.NewInt(1)} }

var _ customProtobufType = (*Uint)(nil)

// Uint64 converts Uint to uint64
// Panics if the value is out of range
func (u Uint) Uint64() uint64 {
	if !u.i.IsUint64() {
		panic("Uint64() out of bound")
	}
	return u.i.Uint64()
}

// IsZero returns 1 if the uint equals to 0.
func (u Uint) IsZero() bool { return u.Equal(ZeroUint()) }

// Equal compares two Uints
func (u Uint) Equal(u2 Uint) bool { return equal(u.i, u2.i) }

// GT returns true if first Uint is greater than second
func (u Uint) GT(u2 Uint) bool { return gt(u.i, u2.i) }

// GTE returns true if first Uint is greater than second
func (u Uint) GTE(u2 Uint) bool { return u.GT(u2) || u.Equal(u2) }

// LT returns true if first Uint is lesser than second
func (u Uint) LT(u2 Uint) bool { return lt(u.i, u2.i) }

// LTE returns true if first Uint is lesser than or equal to the second
func (u Uint) LTE(u2 Uint) bool { return !u.GT(u2) }

// Add adds Uint from another
func (u Uint) Add(u2 Uint) Uint { return NewUintFromBigInt(new(big.Int).Add(u.i, u2.i)) }

// Add convert uint64 and add it to Uint
func (u Uint) AddUint64(u2 uint64) Uint { return u.Add(NewUint(u2)) }

// Sub adds Uint from another
func (u Uint) Sub(u2 Uint) Uint { return NewUintFromBigInt(new(big.Int).Sub(u.i, u2.i)) }

// SubUint64 adds Uint from another
func (u Uint) SubUint64(u2 uint64) Uint { return u.Sub(NewUint(u2)) }

// Mul multiplies two Uints
func (u Uint) Mul(u2 Uint) (res Uint) {
	return NewUintFromBigInt(new(big.Int).Mul(u.i, u2.i))
}

// Mul multiplies two Uints
func (u Uint) MulUint64(u2 uint64) (res Uint) { return u.Mul(NewUint(u2)) }

// Quo divides Uint with Uint
func (u Uint) Quo(u2 Uint) (res Uint) { return NewUintFromBigInt(div(u.i, u2.i)) }

// Mod returns remainder after dividing with Uint
func (u Uint) Mod(u2 Uint) Uint {
	if u2.IsZero() {
		panic("division-by-zero")
	}
	return Uint{mod(u.i, u2.i)}
}

// Incr increments the Uint by one.
func (u Uint) Incr() Uint {
	return u.Add(OneUint())
}

// Decr decrements the Uint by one.
// Decr will panic if the Uint is zero.
func (u Uint) Decr() Uint {
	return u.Sub(OneUint())
}

// Quo divides Uint with uint64
func (u Uint) QuoUint64(u2 uint64) Uint { return u.Quo(NewUint(u2)) }

// Return the minimum of the Uints
func MinUint(u1, u2 Uint) Uint { return NewUintFromBigInt(min(u1.i, u2.i)) }

// Return the maximum of the Uints
func MaxUint(u1, u2 Uint) Uint { return NewUintFromBigInt(max(u1.i, u2.i)) }

// Human readable string
func (u Uint) String() string { return u.i.String() }

// MarshalJSON defines custom encoding scheme
func (u Uint) MarshalJSON() ([]byte, error) {
	if u.i == nil { // Necessary since default Uint initialization has i.i as nil
		u.i = new(big.Int)
	}
	return marshalJSON(u.i)
}

// UnmarshalJSON defines custom decoding scheme
func (u *Uint) UnmarshalJSON(bz []byte) error {
	if u.i == nil { // Necessary since default Uint initialization has i.i as nil
		u.i = new(big.Int)
	}
	return unmarshalJSON(u.i, bz)
}

// Marshal implements the gogo proto custom type interface.
func (u Uint) Marshal() ([]byte, error) {
	if u.i == nil {
		u.i = new(big.Int)
	}
	return u.i.MarshalText()
}

// MarshalTo implements the gogo proto custom type interface.
func (u *Uint) MarshalTo(data []byte) (n int, err error) {
	if u.i == nil {
		u.i = new(big.Int)
	}
	if u.i.BitLen() == 0 { // The value 0
		n = copy(data, []byte{0x30})
		return n, nil
	}

	bz, err := u.Marshal()
	if err != nil {
		return 0, err
	}

	n = copy(data, bz)
	return n, nil
}

// Unmarshal implements the gogo proto custom type interface.
func (u *Uint) Unmarshal(data []byte) error {
	if len(data) == 0 {
		u = nil
		return nil
	}

	if u.i == nil {
		u.i = new(big.Int)
	}

	if err := u.i.UnmarshalText(data); err != nil {
		return err
	}

	// Finally check for overflow.
	return UintOverflow(u.i)
}

// Size implements the gogo proto custom type interface.
func (u *Uint) Size() int {
	bz, _ := u.Marshal()
	return len(bz)
}

// Override Amino binary serialization by proxying to protobuf.
func (u Uint) MarshalAmino() ([]byte, error)   { return u.Marshal() }
func (u *Uint) UnmarshalAmino(bz []byte) error { return u.Unmarshal(bz) }

// UintOverflow returns true if a given unsigned integer overflows and false
// otherwise.
func UintOverflow(i *big.Int) error {
	if i.Sign() < 0 {
		return errors.New("non-positive integer")
	}

	if g, w := i.BitLen(), MaxBitLen; g > w {
		return fmt.Errorf("integer out of range; got: %d, max: %d", g, w)
	}
	return nil
}

// ParseUint reads a string-encoded Uint value and return a Uint.
func ParseUint(s string) (Uint, error) {
	i, ok := new(big.Int).SetString(s, 0)
	if !ok {
		return Uint{}, fmt.Errorf("cannot convert %q to big.Int", s)
	}
	return checkNewUint(i)
}

func checkNewUint(i *big.Int) (Uint, error) {
	if err := UintOverflow(i); err != nil {
		return Uint{}, err
	}
	return Uint{i}, nil
}

// RelativePow raises x to the power of n, where x (and the result, z) are scaled by factor b
// for example, RelativePow(210, 2, 100) = 441 (2.1^2 = 4.41)
func RelativePow(x, n, b Uint) (z Uint) {
	if x.IsZero() {
		if n.IsZero() {
			z = b // 0^0 = 1
			return
		}
		z = ZeroUint() // otherwise 0^a = 0
		return
	}

	z = x
	if n.Mod(NewUint(2)).Equal(ZeroUint()) {
		z = b
	}

	halfOfB := b.Quo(NewUint(2))
	n = n.Quo(NewUint(2))

	for n.GT(ZeroUint()) {
		xSquared := x.Mul(x)
		xSquaredRounded := xSquared.Add(halfOfB)

		x = xSquaredRounded.Quo(b)

		if n.Mod(NewUint(2)).Equal(OneUint()) {
			zx := z.Mul(x)
			zxRounded := zx.Add(halfOfB)
			z = zxRounded.Quo(b)
		}
		n = n.Quo(NewUint(2))
	}
	return z
}
