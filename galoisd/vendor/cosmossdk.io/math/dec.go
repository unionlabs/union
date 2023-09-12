package math

import (
	"encoding/json"
	"errors"
	"fmt"
	"math/big"
	"strconv"
	"strings"
	"testing"
)

// NOTE: never use new(Dec) or else we will panic unmarshalling into the
// nil embedded big.Int
type LegacyDec struct {
	i *big.Int
}

const (
	// number of decimal places
	LegacyPrecision = 18

	// bits required to represent the above precision
	// Ceiling[Log2[10^Precision - 1]]
	LegacyDecimalPrecisionBits = 60

	// decimalTruncateBits is the minimum number of bits removed
	// by a truncate operation. It is equal to
	// Floor[Log2[10^Precision - 1]].
	decimalTruncateBits = LegacyDecimalPrecisionBits - 1

	maxDecBitLen = MaxBitLen + decimalTruncateBits

	// max number of iterations in ApproxRoot function
	maxApproxRootIterations = 300
)

var (
	precisionReuse       = new(big.Int).Exp(big.NewInt(10), big.NewInt(LegacyPrecision), nil)
	fivePrecision        = new(big.Int).Quo(precisionReuse, big.NewInt(2))
	precisionMultipliers []*big.Int
	zeroInt              = big.NewInt(0)
	oneInt               = big.NewInt(1)
	tenInt               = big.NewInt(10)
)

// Decimal errors
var (
	ErrLegacyEmptyDecimalStr      = errors.New("decimal string cannot be empty")
	ErrLegacyInvalidDecimalLength = errors.New("invalid decimal length")
	ErrLegacyInvalidDecimalStr    = errors.New("invalid decimal string")
)

// Set precision multipliers
func init() {
	precisionMultipliers = make([]*big.Int, LegacyPrecision+1)
	for i := 0; i <= LegacyPrecision; i++ {
		precisionMultipliers[i] = calcPrecisionMultiplier(int64(i))
	}
}

func precisionInt() *big.Int {
	return new(big.Int).Set(precisionReuse)
}

func LegacyZeroDec() LegacyDec     { return LegacyDec{new(big.Int).Set(zeroInt)} }
func LegacyOneDec() LegacyDec      { return LegacyDec{precisionInt()} }
func LegacySmallestDec() LegacyDec { return LegacyDec{new(big.Int).Set(oneInt)} }

// calculate the precision multiplier
func calcPrecisionMultiplier(prec int64) *big.Int {
	if prec < 0 {
		panic(fmt.Sprintf("negative precision %v", prec))
	}

	if prec > LegacyPrecision {
		panic(fmt.Sprintf("too much precision, maximum %v, provided %v", LegacyPrecision, prec))
	}
	zerosToAdd := LegacyPrecision - prec
	multiplier := new(big.Int).Exp(tenInt, big.NewInt(zerosToAdd), nil)
	return multiplier
}

// get the precision multiplier, do not mutate result
func precisionMultiplier(prec int64) *big.Int {
	if prec < 0 {
		panic(fmt.Sprintf("negative precision %v", prec))
	}

	if prec > LegacyPrecision {
		panic(fmt.Sprintf("too much precision, maximum %v, provided %v", LegacyPrecision, prec))
	}
	return precisionMultipliers[prec]
}

// create a new Dec from integer assuming whole number
func LegacyNewDec(i int64) LegacyDec {
	return LegacyNewDecWithPrec(i, 0)
}

// create a new Dec from integer with decimal place at prec
// CONTRACT: prec <= Precision
func LegacyNewDecWithPrec(i, prec int64) LegacyDec {
	return LegacyDec{
		new(big.Int).Mul(big.NewInt(i), precisionMultiplier(prec)),
	}
}

// create a new Dec from big integer assuming whole numbers
// CONTRACT: prec <= Precision
func LegacyNewDecFromBigInt(i *big.Int) LegacyDec {
	return LegacyNewDecFromBigIntWithPrec(i, 0)
}

// create a new Dec from big integer assuming whole numbers
// CONTRACT: prec <= Precision
func LegacyNewDecFromBigIntWithPrec(i *big.Int, prec int64) LegacyDec {
	return LegacyDec{
		new(big.Int).Mul(i, precisionMultiplier(prec)),
	}
}

// create a new Dec from big integer assuming whole numbers
// CONTRACT: prec <= Precision
func LegacyNewDecFromInt(i Int) LegacyDec {
	return LegacyNewDecFromIntWithPrec(i, 0)
}

// create a new Dec from big integer with decimal place at prec
// CONTRACT: prec <= Precision
func LegacyNewDecFromIntWithPrec(i Int, prec int64) LegacyDec {
	return LegacyDec{
		new(big.Int).Mul(i.BigInt(), precisionMultiplier(prec)),
	}
}

// create a decimal from an input decimal string.
// valid must come in the form:
//
//	(-) whole integers (.) decimal integers
//
// examples of acceptable input include:
//
//	-123.456
//	456.7890
//	345
//	-456789
//
// NOTE - An error will return if more decimal places
// are provided in the string than the constant Precision.
//
// CONTRACT - This function does not mutate the input str.
func LegacyNewDecFromStr(str string) (LegacyDec, error) {
	// first extract any negative symbol
	neg := false
	if len(str) > 0 && str[0] == '-' {
		neg = true
		str = str[1:]
	}

	if len(str) == 0 {
		return LegacyDec{}, ErrLegacyEmptyDecimalStr
	}

	strs := strings.Split(str, ".")
	lenDecs := 0
	combinedStr := strs[0]

	if len(strs) == 2 { // has a decimal place
		lenDecs = len(strs[1])
		if lenDecs == 0 || len(combinedStr) == 0 {
			return LegacyDec{}, ErrLegacyInvalidDecimalLength
		}
		combinedStr += strs[1]
	} else if len(strs) > 2 {
		return LegacyDec{}, ErrLegacyInvalidDecimalStr
	}

	if lenDecs > LegacyPrecision {
		return LegacyDec{}, fmt.Errorf("value '%s' exceeds max precision by %d decimal places: max precision %d", str, LegacyPrecision-lenDecs, LegacyPrecision)
	}

	// add some extra zero's to correct to the Precision factor
	zerosToAdd := LegacyPrecision - lenDecs
	zeros := strings.Repeat("0", zerosToAdd)
	combinedStr += zeros

	combined, ok := new(big.Int).SetString(combinedStr, 10) // base 10
	if !ok {
		return LegacyDec{}, fmt.Errorf("failed to set decimal string with base 10: %s", combinedStr)
	}
	if combined.BitLen() > maxDecBitLen {
		return LegacyDec{}, fmt.Errorf("decimal '%s' out of range; bitLen: got %d, max %d", str, combined.BitLen(), maxDecBitLen)
	}
	if neg {
		combined = new(big.Int).Neg(combined)
	}

	return LegacyDec{combined}, nil
}

// Decimal from string, panic on error
func LegacyMustNewDecFromStr(s string) LegacyDec {
	dec, err := LegacyNewDecFromStr(s)
	if err != nil {
		panic(err)
	}
	return dec
}

func (d LegacyDec) IsNil() bool                { return d.i == nil }                       // is decimal nil
func (d LegacyDec) IsZero() bool               { return (d.i).Sign() == 0 }                // is equal to zero
func (d LegacyDec) IsNegative() bool           { return (d.i).Sign() == -1 }               // is negative
func (d LegacyDec) IsPositive() bool           { return (d.i).Sign() == 1 }                // is positive
func (d LegacyDec) Equal(d2 LegacyDec) bool    { return (d.i).Cmp(d2.i) == 0 }             // equal decimals
func (d LegacyDec) GT(d2 LegacyDec) bool       { return (d.i).Cmp(d2.i) > 0 }              // greater than
func (d LegacyDec) GTE(d2 LegacyDec) bool      { return (d.i).Cmp(d2.i) >= 0 }             // greater than or equal
func (d LegacyDec) LT(d2 LegacyDec) bool       { return (d.i).Cmp(d2.i) < 0 }              // less than
func (d LegacyDec) LTE(d2 LegacyDec) bool      { return (d.i).Cmp(d2.i) <= 0 }             // less than or equal
func (d LegacyDec) Neg() LegacyDec             { return LegacyDec{new(big.Int).Neg(d.i)} } // reverse the decimal sign
func (d LegacyDec) NegMut() LegacyDec          { d.i.Neg(d.i); return d }                  // reverse the decimal sign, mutable
func (d LegacyDec) Abs() LegacyDec             { return LegacyDec{new(big.Int).Abs(d.i)} } // absolute value
func (d LegacyDec) AbsMut() LegacyDec          { d.i.Abs(d.i); return d }                  // absolute value, mutable
func (d LegacyDec) Set(d2 LegacyDec) LegacyDec { d.i.Set(d2.i); return d }                 // set to existing dec value
func (d LegacyDec) Clone() LegacyDec           { return LegacyDec{new(big.Int).Set(d.i)} } // clone new dec

// BigInt returns a copy of the underlying big.Int.
func (d LegacyDec) BigInt() *big.Int {
	if d.IsNil() {
		return nil
	}

	cp := new(big.Int)
	return cp.Set(d.i)
}

func (d LegacyDec) ImmutOp(op func(LegacyDec, LegacyDec) LegacyDec, d2 LegacyDec) LegacyDec {
	return op(d.Clone(), d2)
}

func (d LegacyDec) ImmutOpInt(op func(LegacyDec, Int) LegacyDec, d2 Int) LegacyDec {
	return op(d.Clone(), d2)
}

func (d LegacyDec) ImmutOpInt64(op func(LegacyDec, int64) LegacyDec, d2 int64) LegacyDec {
	// TODO: use already allocated operand bigint to avoid
	// newint each time, add mutex for race condition
	// Issue: https://github.com/cosmos/cosmos-sdk/issues/11166
	return op(d.Clone(), d2)
}

func (d LegacyDec) SetInt64(i int64) LegacyDec {
	d.i.SetInt64(i)
	d.i.Mul(d.i, precisionReuse)
	return d
}

// addition
func (d LegacyDec) Add(d2 LegacyDec) LegacyDec {
	return d.ImmutOp(LegacyDec.AddMut, d2)
}

// mutable addition
func (d LegacyDec) AddMut(d2 LegacyDec) LegacyDec {
	d.i.Add(d.i, d2.i)

	if d.i.BitLen() > maxDecBitLen {
		panic("Int overflow")
	}
	return d
}

// subtraction
func (d LegacyDec) Sub(d2 LegacyDec) LegacyDec {
	return d.ImmutOp(LegacyDec.SubMut, d2)
}

// mutable subtraction
func (d LegacyDec) SubMut(d2 LegacyDec) LegacyDec {
	d.i.Sub(d.i, d2.i)

	if d.i.BitLen() > maxDecBitLen {
		panic("Int overflow")
	}
	return d
}

// multiplication
func (d LegacyDec) Mul(d2 LegacyDec) LegacyDec {
	return d.ImmutOp(LegacyDec.MulMut, d2)
}

// mutable multiplication
func (d LegacyDec) MulMut(d2 LegacyDec) LegacyDec {
	d.i.Mul(d.i, d2.i)
	chopped := chopPrecisionAndRound(d.i)

	if chopped.BitLen() > maxDecBitLen {
		panic("Int overflow")
	}
	*d.i = *chopped
	return d
}

// multiplication truncate
func (d LegacyDec) MulTruncate(d2 LegacyDec) LegacyDec {
	return d.ImmutOp(LegacyDec.MulTruncateMut, d2)
}

// mutable multiplication truncage
func (d LegacyDec) MulTruncateMut(d2 LegacyDec) LegacyDec {
	d.i.Mul(d.i, d2.i)
	chopPrecisionAndTruncate(d.i)

	if d.i.BitLen() > maxDecBitLen {
		panic("Int overflow")
	}
	return d
}

// multiplication round up at precision end.
func (d LegacyDec) MulRoundUp(d2 LegacyDec) LegacyDec {
	return d.ImmutOp(LegacyDec.MulRoundUpMut, d2)
}

// mutable multiplication with round up at precision end.
func (d LegacyDec) MulRoundUpMut(d2 LegacyDec) LegacyDec {
	d.i.Mul(d.i, d2.i)
	chopPrecisionAndRoundUp(d.i)

	if d.i.BitLen() > maxDecBitLen {
		panic("Int overflow")
	}
	return d
}

// multiplication
func (d LegacyDec) MulInt(i Int) LegacyDec {
	return d.ImmutOpInt(LegacyDec.MulIntMut, i)
}

func (d LegacyDec) MulIntMut(i Int) LegacyDec {
	d.i.Mul(d.i, i.BigInt())
	if d.i.BitLen() > maxDecBitLen {
		panic("Int overflow")
	}
	return d
}

// MulInt64 - multiplication with int64
func (d LegacyDec) MulInt64(i int64) LegacyDec {
	return d.ImmutOpInt64(LegacyDec.MulInt64Mut, i)
}

func (d LegacyDec) MulInt64Mut(i int64) LegacyDec {
	d.i.Mul(d.i, big.NewInt(i))

	if d.i.BitLen() > maxDecBitLen {
		panic("Int overflow")
	}
	return d
}

// quotient
func (d LegacyDec) Quo(d2 LegacyDec) LegacyDec {
	return d.ImmutOp(LegacyDec.QuoMut, d2)
}

var squaredPrecisionReuse = new(big.Int).Mul(precisionReuse, precisionReuse)

// mutable quotient
func (d LegacyDec) QuoMut(d2 LegacyDec) LegacyDec {
	// multiply by precision twice
	d.i.Mul(d.i, squaredPrecisionReuse)
	d.i.Quo(d.i, d2.i)

	chopPrecisionAndRound(d.i)
	if d.i.BitLen() > maxDecBitLen {
		panic("Int overflow")
	}
	return d
}

// quotient truncate
func (d LegacyDec) QuoTruncate(d2 LegacyDec) LegacyDec {
	return d.ImmutOp(LegacyDec.QuoTruncateMut, d2)
}

// mutable quotient truncate
func (d LegacyDec) QuoTruncateMut(d2 LegacyDec) LegacyDec {
	// multiply precision twice
	d.i.Mul(d.i, squaredPrecisionReuse)
	d.i.Quo(d.i, d2.i)

	chopPrecisionAndTruncate(d.i)
	if d.i.BitLen() > maxDecBitLen {
		panic("Int overflow")
	}
	return d
}

// quotient, round up
func (d LegacyDec) QuoRoundUp(d2 LegacyDec) LegacyDec {
	return d.ImmutOp(LegacyDec.QuoRoundupMut, d2)
}

// mutable quotient, round up
func (d LegacyDec) QuoRoundupMut(d2 LegacyDec) LegacyDec {
	// multiply precision twice
	d.i.Mul(d.i, squaredPrecisionReuse)
	d.i.Quo(d.i, d2.i)

	chopPrecisionAndRoundUp(d.i)
	if d.i.BitLen() > maxDecBitLen {
		panic("Int overflow")
	}
	return d
}

// quotient
func (d LegacyDec) QuoInt(i Int) LegacyDec {
	return d.ImmutOpInt(LegacyDec.QuoIntMut, i)
}

func (d LegacyDec) QuoIntMut(i Int) LegacyDec {
	d.i.Quo(d.i, i.BigInt())
	return d
}

// QuoInt64 - quotient with int64
func (d LegacyDec) QuoInt64(i int64) LegacyDec {
	return d.ImmutOpInt64(LegacyDec.QuoInt64Mut, i)
}

func (d LegacyDec) QuoInt64Mut(i int64) LegacyDec {
	d.i.Quo(d.i, big.NewInt(i))
	return d
}

// ApproxRoot returns an approximate estimation of a Dec's positive real nth root
// using Newton's method (where n is positive). The algorithm starts with some guess and
// computes the sequence of improved guesses until an answer converges to an
// approximate answer.  It returns `|d|.ApproxRoot() * -1` if input is negative.
// A maximum number of 100 iterations is used a backup boundary condition for
// cases where the answer never converges enough to satisfy the main condition.
func (d LegacyDec) ApproxRoot(root uint64) (guess LegacyDec, err error) {
	defer func() {
		if r := recover(); r != nil {
			var ok bool
			err, ok = r.(error)
			if !ok {
				err = errors.New("out of bounds")
			}
		}
	}()

	if d.IsNegative() {
		absRoot, err := d.Neg().ApproxRoot(root)
		return absRoot.NegMut(), err
	}

	// One decimal, that we invalidate later. Helps us save a heap allocation.
	scratchOneDec := LegacyOneDec()
	if root == 1 || d.IsZero() || d.Equal(scratchOneDec) {
		return d, nil
	}

	if root == 0 {
		return scratchOneDec, nil
	}

	guess, delta := scratchOneDec, LegacyOneDec()
	smallestDec := LegacySmallestDec()

	for iter := 0; delta.AbsMut().GT(smallestDec) && iter < maxApproxRootIterations; iter++ {
		// Set prev = guess^{root - 1}, with an optimization for sqrt
		// where root=2 => prev = guess. (And thus no extra heap allocations)
		prev := guess
		if root != 2 {
			prev = guess.Power(root - 1)
		}
		if prev.IsZero() {
			prev = smallestDec
		}
		delta.Set(d).QuoMut(prev)
		delta.SubMut(guess)
		// delta = delta / root.
		// We optimize for sqrt, where root=2 => delta = delta >> 1
		if root == 2 {
			delta.i.Rsh(delta.i, 1)
		} else {
			delta.QuoInt64Mut(int64(root))
		}

		guess.AddMut(delta)
	}

	return guess, nil
}

// Power returns a the result of raising to a positive integer power
func (d LegacyDec) Power(power uint64) LegacyDec {
	res := LegacyDec{new(big.Int).Set(d.i)}
	return res.PowerMut(power)
}

func (d LegacyDec) PowerMut(power uint64) LegacyDec {
	if power == 0 {
		// Set to 1 with the correct precision.
		d.i.Set(precisionReuse)
		return d
	}
	tmp := LegacyOneDec()

	for i := power; i > 1; {
		if i%2 != 0 {
			tmp.MulMut(d)
		}
		i /= 2
		d.MulMut(d)
	}

	return d.MulMut(tmp)
}

// ApproxSqrt is a wrapper around ApproxRoot for the common special case
// of finding the square root of a number. It returns -(sqrt(abs(d)) if input is negative.
func (d LegacyDec) ApproxSqrt() (LegacyDec, error) {
	return d.ApproxRoot(2)
}

// is integer, e.g. decimals are zero
func (d LegacyDec) IsInteger() bool {
	return new(big.Int).Rem(d.i, precisionReuse).Sign() == 0
}

// format decimal state
func (d LegacyDec) Format(s fmt.State, verb rune) {
	_, err := s.Write([]byte(d.String()))
	if err != nil {
		panic(err)
	}
}

func (d LegacyDec) String() string {
	if d.i == nil {
		return d.i.String()
	}

	isNeg := d.IsNegative()

	if isNeg {
		d = d.Neg()
	}

	bzInt, err := d.i.MarshalText()
	if err != nil {
		return ""
	}
	inputSize := len(bzInt)

	var bzStr []byte

	// TODO: Remove trailing zeros
	// case 1, purely decimal
	if inputSize <= LegacyPrecision {
		bzStr = make([]byte, LegacyPrecision+2)

		// 0. prefix
		bzStr[0] = byte('0')
		bzStr[1] = byte('.')

		// set relevant digits to 0
		for i := 0; i < LegacyPrecision-inputSize; i++ {
			bzStr[i+2] = byte('0')
		}

		// set final digits
		copy(bzStr[2+(LegacyPrecision-inputSize):], bzInt)
	} else {
		// inputSize + 1 to account for the decimal point that is being added
		bzStr = make([]byte, inputSize+1)
		decPointPlace := inputSize - LegacyPrecision

		copy(bzStr, bzInt[:decPointPlace])                   // pre-decimal digits
		bzStr[decPointPlace] = byte('.')                     // decimal point
		copy(bzStr[decPointPlace+1:], bzInt[decPointPlace:]) // post-decimal digits
	}

	if isNeg {
		return "-" + string(bzStr)
	}

	return string(bzStr)
}

// Float64 returns the float64 representation of a Dec.
// Will return the error if the conversion failed.
func (d LegacyDec) Float64() (float64, error) {
	return strconv.ParseFloat(d.String(), 64)
}

// MustFloat64 returns the float64 representation of a Dec.
// Would panic if the conversion failed.
func (d LegacyDec) MustFloat64() float64 {
	if value, err := strconv.ParseFloat(d.String(), 64); err != nil {
		panic(err)
	} else {
		return value
	}
}

//     ____
//  __|    |__   "chop 'em
//       ` \     round!"
// ___||  ~  _     -bankers
// |         |      __
// |       | |   __|__|__
// |_____:  /   | $$$    |
//              |________|

// Remove a Precision amount of rightmost digits and perform bankers rounding
// on the remainder (gaussian rounding) on the digits which have been removed.
//
// Mutates the input. Use the non-mutative version if that is undesired
func chopPrecisionAndRound(d *big.Int) *big.Int {
	// remove the negative and add it back when returning
	if d.Sign() == -1 {
		// make d positive, compute chopped value, and then un-mutate d
		d = d.Neg(d)
		d = chopPrecisionAndRound(d)
		d = d.Neg(d)
		return d
	}

	// get the truncated quotient and remainder
	quo, rem := d, big.NewInt(0)
	quo, rem = quo.QuoRem(d, precisionReuse, rem)

	if rem.Sign() == 0 { // remainder is zero
		return quo
	}

	switch rem.Cmp(fivePrecision) {
	case -1:
		return quo
	case 1:
		return quo.Add(quo, oneInt)
	default: // bankers rounding must take place
		// always round to an even number
		if quo.Bit(0) == 0 {
			return quo
		}
		return quo.Add(quo, oneInt)
	}
}

func chopPrecisionAndRoundUp(d *big.Int) *big.Int {
	// remove the negative and add it back when returning
	if d.Sign() == -1 {
		// make d positive, compute chopped value, and then un-mutate d
		d = d.Neg(d)
		// truncate since d is negative...
		chopPrecisionAndTruncate(d)
		d = d.Neg(d)
		return d
	}

	// get the truncated quotient and remainder
	quo, rem := d, big.NewInt(0)
	quo, rem = quo.QuoRem(d, precisionReuse, rem)

	if rem.Sign() == 0 { // remainder is zero
		return quo
	}

	return quo.Add(quo, oneInt)
}

func chopPrecisionAndRoundNonMutative(d *big.Int) *big.Int {
	tmp := new(big.Int).Set(d)
	return chopPrecisionAndRound(tmp)
}

// RoundInt64 rounds the decimal using bankers rounding
func (d LegacyDec) RoundInt64() int64 {
	chopped := chopPrecisionAndRoundNonMutative(d.i)
	if !chopped.IsInt64() {
		panic("Int64() out of bound")
	}
	return chopped.Int64()
}

// RoundInt round the decimal using bankers rounding
func (d LegacyDec) RoundInt() Int {
	return NewIntFromBigInt(chopPrecisionAndRoundNonMutative(d.i))
}

// chopPrecisionAndTruncate is similar to chopPrecisionAndRound,
// but always rounds down. It does not mutate the input.
func chopPrecisionAndTruncate(d *big.Int) {
	d.Quo(d, precisionReuse)
}

func chopPrecisionAndTruncateNonMutative(d *big.Int) *big.Int {
	tmp := new(big.Int).Set(d)
	chopPrecisionAndTruncate(tmp)
	return tmp
}

// TruncateInt64 truncates the decimals from the number and returns an int64
func (d LegacyDec) TruncateInt64() int64 {
	chopped := chopPrecisionAndTruncateNonMutative(d.i)
	if !chopped.IsInt64() {
		panic("Int64() out of bound")
	}
	return chopped.Int64()
}

// TruncateInt truncates the decimals from the number and returns an Int
func (d LegacyDec) TruncateInt() Int {
	return NewIntFromBigInt(chopPrecisionAndTruncateNonMutative(d.i))
}

// TruncateDec truncates the decimals from the number and returns a Dec
func (d LegacyDec) TruncateDec() LegacyDec {
	return LegacyNewDecFromBigInt(chopPrecisionAndTruncateNonMutative(d.i))
}

// Ceil returns the smallest interger value (as a decimal) that is greater than
// or equal to the given decimal.
func (d LegacyDec) Ceil() LegacyDec {
	tmp := new(big.Int).Set(d.i)

	quo, rem := tmp, big.NewInt(0)
	quo, rem = quo.QuoRem(tmp, precisionReuse, rem)

	// no need to round with a zero remainder regardless of sign
	if rem.Cmp(zeroInt) == 0 {
		return LegacyNewDecFromBigInt(quo)
	}

	if rem.Sign() == -1 {
		return LegacyNewDecFromBigInt(quo)
	}

	return LegacyNewDecFromBigInt(quo.Add(quo, oneInt))
}

// LegacyMaxSortableDec is the largest Dec that can be passed into SortableDecBytes()
// Its negative form is the least Dec that can be passed in.
var LegacyMaxSortableDec LegacyDec

func init() {
	LegacyMaxSortableDec = LegacyOneDec().Quo(LegacySmallestDec())
}

// ValidSortableDec ensures that a Dec is within the sortable bounds,
// a Dec can't have a precision of less than 10^-18.
// Max sortable decimal was set to the reciprocal of SmallestDec.
func LegacyValidSortableDec(dec LegacyDec) bool {
	return dec.Abs().LTE(LegacyMaxSortableDec)
}

// SortableDecBytes returns a byte slice representation of a Dec that can be sorted.
// Left and right pads with 0s so there are 18 digits to left and right of the decimal point.
// For this reason, there is a maximum and minimum value for this, enforced by ValidSortableDec.
func LegacySortableDecBytes(dec LegacyDec) []byte {
	if !LegacyValidSortableDec(dec) {
		panic("dec must be within bounds")
	}
	// Instead of adding an extra byte to all sortable decs in order to handle max sortable, we just
	// makes its bytes be "max" which comes after all numbers in ASCIIbetical order
	if dec.Equal(LegacyMaxSortableDec) {
		return []byte("max")
	}
	// For the same reason, we make the bytes of minimum sortable dec be --, which comes before all numbers.
	if dec.Equal(LegacyMaxSortableDec.Neg()) {
		return []byte("--")
	}
	// We move the negative sign to the front of all the left padded 0s, to make negative numbers come before positive numbers
	if dec.IsNegative() {
		return append([]byte("-"), []byte(fmt.Sprintf(fmt.Sprintf("%%0%ds", LegacyPrecision*2+1), dec.Abs().String()))...)
	}
	return []byte(fmt.Sprintf(fmt.Sprintf("%%0%ds", LegacyPrecision*2+1), dec.String()))
}

// reuse nil values
var nilJSON []byte

func init() {
	empty := new(big.Int)
	bz, _ := empty.MarshalText()
	nilJSON, _ = json.Marshal(string(bz))
}

// MarshalJSON marshals the decimal
func (d LegacyDec) MarshalJSON() ([]byte, error) {
	if d.i == nil {
		return nilJSON, nil
	}
	return json.Marshal(d.String())
}

// UnmarshalJSON defines custom decoding scheme
func (d *LegacyDec) UnmarshalJSON(bz []byte) error {
	if d.i == nil {
		d.i = new(big.Int)
	}

	var text string
	err := json.Unmarshal(bz, &text)
	if err != nil {
		return err
	}

	// TODO: Reuse dec allocation
	newDec, err := LegacyNewDecFromStr(text)
	if err != nil {
		return err
	}

	d.i = newDec.i
	return nil
}

// MarshalYAML returns the YAML representation.
func (d LegacyDec) MarshalYAML() (interface{}, error) {
	return d.String(), nil
}

// Marshal implements the gogo proto custom type interface.
func (d LegacyDec) Marshal() ([]byte, error) {
	i := d.i
	if i == nil {
		i = new(big.Int)
	}
	return i.MarshalText()
}

// MarshalTo implements the gogo proto custom type interface.
func (d *LegacyDec) MarshalTo(data []byte) (n int, err error) {
	i := d.i
	if i == nil {
		i = new(big.Int)
	}

	if i.Cmp(zeroInt) == 0 {
		copy(data, []byte{0x30})
		return 1, nil
	}

	bz, err := d.Marshal()
	if err != nil {
		return 0, err
	}

	copy(data, bz)
	return len(bz), nil
}

// Unmarshal implements the gogo proto custom type interface.
func (d *LegacyDec) Unmarshal(data []byte) error {
	if len(data) == 0 {
		d = nil
		return nil
	}

	if d.i == nil {
		d.i = new(big.Int)
	}

	if err := d.i.UnmarshalText(data); err != nil {
		return err
	}

	if d.i.BitLen() > maxDecBitLen {
		return fmt.Errorf("decimal out of range; got: %d, max: %d", d.i.BitLen(), maxDecBitLen)
	}

	return nil
}

// Size implements the gogo proto custom type interface.
func (d *LegacyDec) Size() int {
	bz, _ := d.Marshal()
	return len(bz)
}

// Override Amino binary serialization by proxying to protobuf.
func (d LegacyDec) MarshalAmino() ([]byte, error)   { return d.Marshal() }
func (d *LegacyDec) UnmarshalAmino(bz []byte) error { return d.Unmarshal(bz) }

// helpers

// test if two decimal arrays are equal
func LegacyDecsEqual(d1s, d2s []LegacyDec) bool {
	if len(d1s) != len(d2s) {
		return false
	}

	for i, d1 := range d1s {
		if !d1.Equal(d2s[i]) {
			return false
		}
	}
	return true
}

// minimum decimal between two
func LegacyMinDec(d1, d2 LegacyDec) LegacyDec {
	if d1.LT(d2) {
		return d1
	}
	return d2
}

// maximum decimal between two
func LegacyMaxDec(d1, d2 LegacyDec) LegacyDec {
	if d1.LT(d2) {
		return d2
	}
	return d1
}

// intended to be used with require/assert:  require.True(DecEq(...))
func LegacyDecEq(t *testing.T, exp, got LegacyDec) (*testing.T, bool, string, string, string) {
	t.Helper()
	return t, exp.Equal(got), "expected:\t%v\ngot:\t\t%v", exp.String(), got.String()
}

func LegacyDecApproxEq(t *testing.T, d1, d2, tol LegacyDec) (*testing.T, bool, string, string, string) {
	t.Helper()
	diff := d1.Sub(d2).Abs()
	return t, diff.LTE(tol), "expected |d1 - d2| <:\t%v\ngot |d1 - d2| = \t\t%v", tol.String(), diff.String()
}

// FormatDec formats a decimal (as encoded in protobuf) into a value-rendered
// string following ADR-050. This function operates with string manipulation
// (instead of manipulating the sdk.Dec object).
func FormatDec(v string) (string, error) {
	parts := strings.Split(v, ".")
	if len(parts) > 2 {
		return "", fmt.Errorf("invalid decimal: too many points in %s", v)
	}

	intPart, err := FormatInt(parts[0])
	if err != nil {
		return "", err
	}

	if len(parts) == 1 {
		return intPart, nil
	}

	decPart := strings.TrimRight(parts[1], "0")
	if len(decPart) == 0 {
		return intPart, nil
	}

	// Ensure that the decimal part has only digits.
	// https://github.com/cosmos/cosmos-sdk/issues/12811
	if !hasOnlyDigits(decPart) {
		return "", fmt.Errorf("non-digits detected after decimal point in: %q", decPart)
	}

	return intPart + "." + decPart, nil
}
