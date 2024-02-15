// Package math includes important helpers for Ethereum such as fast integer square roots.
package math

import (
	"errors"
	stdmath "math"
	"math/big"
	"math/bits"
	"sync"

	"github.com/thomaso-mirodin/intmath/u64"
)

func init() {
	// The Int function assumes that the operating system is 64 bit. In any case, Ethereum
	// consensus layer uses 64 bit values almost exclusively so 64 bit OS requirement should
	// already be established. This panic is a strict fail fast feedback to alert 32 bit users
	// that they are not supported.
	if stdmath.MaxUint < stdmath.MaxUint64 {
		panic("Prysm is only supported on 64 bit OS")
	}
}

// ErrOverflow occurs when an operation exceeds max or minimum values.
var (
	ErrOverflow     = errors.New("integer overflow")
	ErrDivByZero    = errors.New("integer divide by zero")
	ErrMulOverflow  = errors.New("multiplication overflows")
	ErrAddOverflow  = errors.New("addition overflows")
	ErrSubUnderflow = errors.New("subtraction underflow")

	// Sensible guess for 500 000 validators
	cachedSquareRoot = struct {
		sync.Mutex
		squareRoot, balance uint64
	}{squareRoot: 126491106, balance: 15999999897103236}
)

// Common square root values.
var squareRootTable = map[uint64]uint64{
	4:       2,
	16:      4,
	64:      8,
	256:     16,
	1024:    32,
	4096:    64,
	16384:   128,
	65536:   256,
	262144:  512,
	1048576: 1024,
	4194304: 2048,
}

// CachedSquareRoot implements Newton's algorithm to compute the square root of
// the given uint64 starting from the last cached value
func CachedSquareRoot(balance uint64) uint64 {
	if balance == 0 {
		return 0
	}
	cachedSquareRoot.Lock()
	defer cachedSquareRoot.Unlock()
	if balance == cachedSquareRoot.balance {
		return cachedSquareRoot.squareRoot
	}
	cachedSquareRoot.balance = balance
	val := balance / cachedSquareRoot.squareRoot
	for {
		cachedSquareRoot.squareRoot = (cachedSquareRoot.squareRoot + val) / 2
		val = balance / cachedSquareRoot.squareRoot
		if cachedSquareRoot.squareRoot <= val {
			return cachedSquareRoot.squareRoot
		}
	}
}

// IntegerSquareRoot defines a function that returns the
// largest possible integer root of a number using go's standard library.
func IntegerSquareRoot(n uint64) uint64 {
	if v, ok := squareRootTable[n]; ok {
		return v
	}

	// Golang floating point precision may be lost above 52 bits, so we use a
	// non floating point method. u64.Sqrt is about x2.5 slower than math.Sqrt.
	if n >= 1<<52 {
		return u64.Sqrt(n)
	}

	return uint64(stdmath.Sqrt(float64(n)))
}

// CeilDiv8 divides the input number by 8
// and takes the ceiling of that number.
func CeilDiv8(n int) int {
	ret := n / 8
	if n%8 > 0 {
		ret++
	}

	return ret
}

// IsPowerOf2 returns true if n is an
// exact power of two. False otherwise.
func IsPowerOf2(n uint64) bool {
	return n != 0 && (n&(n-1)) == 0
}

// PowerOf2 returns an integer that is the provided
// exponent of 2. Can only return powers of 2 till 63,
// after that it overflows
func PowerOf2(n uint64) uint64 {
	if n >= 64 {
		panic("integer overflow")
	}
	return 1 << n
}

// Max returns the larger integer of the two
// given ones.This is used over the Max function
// in the standard math library because that max function
// has to check for some special floating point cases
// making it slower by a magnitude of 10.
func Max(a, b uint64) uint64 {
	if a > b {
		return a
	}
	return b
}

// Min returns the smaller integer of the two
// given ones. This is used over the Min function
// in the standard math library because that min function
// has to check for some special floating point cases
// making it slower by a magnitude of 10.
func Min(a, b uint64) uint64 {
	if a < b {
		return a
	}
	return b
}

// Mul64 multiples 2 64-bit unsigned integers and checks if they
// lead to an overflow. If they do not, it returns the result
// without an error.
func Mul64(a, b uint64) (uint64, error) {
	overflows, val := bits.Mul64(a, b)
	if overflows > 0 {
		return 0, errors.New("multiplication overflows")
	}
	return val, nil
}

// Div64 divides two 64-bit unsigned integers and checks for errors.
func Div64(a, b uint64) (uint64, error) {
	if b == 0 {
		return 0, ErrDivByZero
	}
	val, _ := bits.Div64(0, a, b)
	return val, nil
}

// Add64 adds 2 64-bit unsigned integers and checks if they
// lead to an overflow. If they do not, it returns the result
// without an error.
func Add64(a, b uint64) (uint64, error) {
	res, carry := bits.Add64(a, b, 0 /* carry */)
	if carry > 0 {
		return 0, errors.New("addition overflows")
	}
	return res, nil
}

// Sub64 subtracts two 64-bit unsigned integers and checks for errors.
func Sub64(a, b uint64) (uint64, error) {
	res, borrow := bits.Sub64(a, b, 0 /* borrow */)
	if borrow > 0 {
		return 0, errors.New("subtraction underflow")
	}
	return res, nil
}

// Mod64 finds remainder of division of two 64-bit unsigned integers and checks for errors.
func Mod64(a, b uint64) (uint64, error) {
	if b == 0 {
		return 0, ErrDivByZero
	}
	_, val := bits.Div64(0, a, b)
	return val, nil
}

// Int returns the integer value of the uint64 argument. If there is an overflow, then an error is
// returned.
func Int(u uint64) (int, error) {
	if u > stdmath.MaxInt {
		return 0, ErrOverflow
	}
	return int(u), nil // lint:ignore uintcast -- This is the preferred method of casting uint64 to int.
}

// AddInt adds two or more integers and checks for integer overflows.
func AddInt(i ...int) (int, error) {
	var sum int
	for _, ii := range i {
		if ii > 0 && sum > stdmath.MaxInt-ii {
			return 0, ErrOverflow
		} else if ii < 0 && sum < stdmath.MinInt-ii {
			return 0, ErrOverflow
		}

		sum += ii
	}
	return sum, nil
}

// Wei is the smallest unit of Ether, represented as a pointer to a bigInt.
type Wei *big.Int

// Gwei is a denomination of 1e9 Wei represented as an uint64.
type Gwei uint64

// WeiToGwei converts big int wei to uint64 gwei.
// The input `v` is copied before being modified.
func WeiToGwei(v Wei) Gwei {
	if v == nil {
		return 0
	}
	gweiPerEth := big.NewInt(1e9)
	copied := big.NewInt(0).Set(v)
	copied.Div(copied, gweiPerEth)
	return Gwei(copied.Uint64())
}

// IsValidUint256 given a bigint checks if the value is a valid Uint256
func IsValidUint256(bi *big.Int) bool {
	return bi.Cmp(big.NewInt(0)) >= 0 && bi.BitLen() <= 256
}
