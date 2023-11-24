// Package math provides helper functions for doing mathematical calculations and parsing for the group module.
package math

import (
	"fmt"

	"github.com/cockroachdb/apd/v2"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/x/group/errors"
)

// Dec is a wrapper struct around apd.Decimal that does no mutation of apd.Decimal's when performing
// arithmetic, instead creating a new apd.Decimal for every operation ensuring usage is safe.
//
// Using apd.Decimal directly can be unsafe because apd operations mutate the underlying Decimal,
// but when copying the big.Int structure can be shared between Decimal instances causing corruption.
// This was originally discovered in regen0-network/mainnet#15.
type Dec struct {
	dec apd.Decimal
}

func NewPositiveDecFromString(s string) (Dec, error) {
	d, err := NewDecFromString(s)
	if err != nil {
		return Dec{}, errors.ErrInvalidDecString.Wrap(err.Error())
	}
	if !d.IsPositive() {
		return Dec{}, errors.ErrInvalidDecString.Wrapf("expected a positive decimal, got %s", s)
	}
	return d, nil
}

func NewNonNegativeDecFromString(s string) (Dec, error) {
	d, err := NewDecFromString(s)
	if err != nil {
		return Dec{}, errors.ErrInvalidDecString.Wrap(err.Error())
	}
	if d.IsNegative() {
		return Dec{}, errors.ErrInvalidDecString.Wrapf("expected a non-negative decimal, got %s", s)
	}
	return d, nil
}

func (x Dec) IsPositive() bool {
	return !x.dec.Negative && !x.dec.IsZero()
}

// NewDecFromString returns a new Dec from a string
// It only support finite numbers, not NaN, +Inf, -Inf
func NewDecFromString(s string) (Dec, error) {
	d, _, err := apd.NewFromString(s)
	if err != nil {
		return Dec{}, errors.ErrInvalidDecString.Wrap(err.Error())
	}

	if d.Form != apd.Finite {
		return Dec{}, errors.ErrInvalidDecString.Wrapf("expected a finite decimal, got %s", s)
	}

	return Dec{*d}, nil
}

func (x Dec) String() string {
	return x.dec.Text('f')
}

func NewDecFromInt64(x int64) Dec {
	var res Dec
	res.dec.SetInt64(x)
	return res
}

// Add returns a new Dec with value `x+y` without mutating any argument and error if
// there is an overflow.
func (x Dec) Add(y Dec) (Dec, error) {
	var z Dec
	_, err := apd.BaseContext.Add(&z.dec, &x.dec, &y.dec)
	return z, errorsmod.Wrap(err, "decimal addition error")
}

// Sub returns a new Dec with value `x-y` without mutating any argument and error if
// there is an overflow.
func (x Dec) Sub(y Dec) (Dec, error) {
	var z Dec
	_, err := apd.BaseContext.Sub(&z.dec, &x.dec, &y.dec)
	return z, errorsmod.Wrap(err, "decimal subtraction error")
}

func (x Dec) Int64() (int64, error) {
	return x.dec.Int64()
}

func (x Dec) Cmp(y Dec) int {
	return x.dec.Cmp(&y.dec)
}

func (x Dec) Equal(y Dec) bool {
	return x.dec.Cmp(&y.dec) == 0
}

func (x Dec) IsNegative() bool {
	return x.dec.Negative && !x.dec.IsZero()
}

// Add adds x and y
func Add(x, y Dec) (Dec, error) {
	return x.Add(y)
}

var dec128Context = apd.Context{
	Precision:   34,
	MaxExponent: apd.MaxExponent,
	MinExponent: apd.MinExponent,
	Traps:       apd.DefaultTraps,
}

// Quo returns a new Dec with value `x/y` (formatted as decimal128, 34 digit precision) without mutating any
// argument and error if there is an overflow.
func (x Dec) Quo(y Dec) (Dec, error) {
	var z Dec
	_, err := dec128Context.Quo(&z.dec, &x.dec, &y.dec)
	return z, errorsmod.Wrap(err, "decimal quotient error")
}

func (x Dec) IsZero() bool {
	return x.dec.IsZero()
}

// SubNonNegative subtracts the value of y from x and returns the result with
// arbitrary precision. Returns an error if the result is negative.
func SubNonNegative(x, y Dec) (Dec, error) {
	z, err := x.Sub(y)
	if err != nil {
		return Dec{}, err
	}

	if z.IsNegative() {
		return z, fmt.Errorf("result negative during non-negative subtraction")
	}

	return z, nil
}
