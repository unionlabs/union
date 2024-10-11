// Copyright 2020 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package utils

import (
	"math"
	"sync"

	"github.com/ethereum/go-ethereum/common/mclock"
)

// ExpiredValue is a scalar value that is continuously expired (decreased
// exponentially) based on the provided logarithmic expiration offset value.
//
// The formula for value calculation is: base*2^(exp-logOffset). In order to
// simplify the calculation of ExpiredValue, its value is expressed in the form
// of an exponent with a base of 2.
//
// Also here is a trick to reduce a lot of calculations. In theory, when a value X
// decays over time and then a new value Y is added, the final result should be
// X*2^(exp-logOffset)+Y. However it's very hard to represent in memory.
// So the trick is using the idea of inflation instead of exponential decay. At this
// moment the temporary value becomes: X*2^exp+Y*2^logOffset_1, apply the exponential
// decay when we actually want to calculate the value.
//
// e.g.
// t0: V = 100
// t1: add 30, inflationary value is: 100 + 30/0.3, 0.3 is the decay coefficient
// t2: get value, decay coefficient is 0.2 now, final result is: 200*0.2 = 40
type ExpiredValue struct {
	Base, Exp uint64 // rlp encoding works by default
}

// ExpirationFactor is calculated from logOffset. 1 <= Factor < 2 and Factor*2^Exp
// describes the multiplier applicable for additions and the divider for readouts.
// If logOffset changes slowly then it saves some expensive operations to not calculate
// them for each addition and readout but cache this intermediate form for some time.
// It is also useful for structures where multiple values are expired with the same
// Expirer.
type ExpirationFactor struct {
	Exp    uint64
	Factor float64
}

// ExpFactor calculates ExpirationFactor based on logOffset
func ExpFactor(logOffset Fixed64) ExpirationFactor {
	return ExpirationFactor{Exp: logOffset.ToUint64(), Factor: logOffset.Fraction().Pow2()}
}

// Value calculates the expired value based on a floating point base and integer
// power-of-2 exponent. This function should be used by multi-value expired structures.
func (e ExpirationFactor) Value(base float64, exp uint64) float64 {
	return base / e.Factor * math.Pow(2, float64(int64(exp-e.Exp)))
}

// Value calculates the value at the given moment.
func (e ExpiredValue) Value(logOffset Fixed64) uint64 {
	offset := Uint64ToFixed64(e.Exp) - logOffset
	return uint64(float64(e.Base) * offset.Pow2())
}

// Add adds a signed value at the given moment
func (e *ExpiredValue) Add(amount int64, logOffset Fixed64) int64 {
	integer, frac := logOffset.ToUint64(), logOffset.Fraction()
	factor := frac.Pow2()
	base := factor * float64(amount)
	if integer < e.Exp {
		base /= math.Pow(2, float64(e.Exp-integer))
	}
	if integer > e.Exp {
		e.Base >>= (integer - e.Exp)
		e.Exp = integer
	}
	if base >= 0 || uint64(-base) <= e.Base {
		// The conversion from negative float64 to
		// uint64 is undefined in golang, and doesn't
		// work with ARMv8. More details at:
		// https://github.com/golang/go/issues/43047
		if base >= 0 {
			e.Base += uint64(base)
		} else {
			e.Base -= uint64(-base)
		}
		return amount
	}
	net := int64(-float64(e.Base) / factor)
	e.Base = 0
	return net
}

// AddExp adds another ExpiredValue
func (e *ExpiredValue) AddExp(a ExpiredValue) {
	if e.Exp > a.Exp {
		a.Base >>= (e.Exp - a.Exp)
	}
	if e.Exp < a.Exp {
		e.Base >>= (a.Exp - e.Exp)
		e.Exp = a.Exp
	}
	e.Base += a.Base
}

// SubExp subtracts another ExpiredValue
func (e *ExpiredValue) SubExp(a ExpiredValue) {
	if e.Exp > a.Exp {
		a.Base >>= (e.Exp - a.Exp)
	}
	if e.Exp < a.Exp {
		e.Base >>= (a.Exp - e.Exp)
		e.Exp = a.Exp
	}
	if e.Base > a.Base {
		e.Base -= a.Base
	} else {
		e.Base = 0
	}
}

// IsZero returns true if the value is zero
func (e *ExpiredValue) IsZero() bool {
	return e.Base == 0
}

// LinearExpiredValue is very similar with the expiredValue which the value
// will continuously expired. But the different part is it's expired linearly.
type LinearExpiredValue struct {
	Offset uint64         // The latest time offset
	Val    uint64         // The remaining value, can never be negative
	Rate   mclock.AbsTime `rlp:"-"` // Expiration rate(by nanosecond), will ignored by RLP
}

// Value calculates the value at the given moment. This function always has the
// assumption that the given timestamp shouldn't less than the recorded one.
func (e LinearExpiredValue) Value(now mclock.AbsTime) uint64 {
	offset := uint64(now / e.Rate)
	if e.Offset < offset {
		diff := offset - e.Offset
		if e.Val >= diff {
			e.Val -= diff
		} else {
			e.Val = 0
		}
	}
	return e.Val
}

// Add adds a signed value at the given moment. This function always has the
// assumption that the given timestamp shouldn't less than the recorded one.
func (e *LinearExpiredValue) Add(amount int64, now mclock.AbsTime) uint64 {
	offset := uint64(now / e.Rate)
	if e.Offset < offset {
		diff := offset - e.Offset
		if e.Val >= diff {
			e.Val -= diff
		} else {
			e.Val = 0
		}
		e.Offset = offset
	}
	if amount < 0 && uint64(-amount) > e.Val {
		e.Val = 0
	} else {
		e.Val = uint64(int64(e.Val) + amount)
	}
	return e.Val
}

// ValueExpirer controls value expiration rate
type ValueExpirer interface {
	SetRate(now mclock.AbsTime, rate float64)
	SetLogOffset(now mclock.AbsTime, logOffset Fixed64)
	LogOffset(now mclock.AbsTime) Fixed64
}

// Expirer changes logOffset with a linear rate which can be changed during operation.
// It is not thread safe, if access by multiple goroutines is needed then it should be
// encapsulated into a locked structure.
// Note that if neither SetRate nor SetLogOffset are used during operation then LogOffset
// is thread safe.
type Expirer struct {
	lock       sync.RWMutex
	logOffset  Fixed64
	rate       float64
	lastUpdate mclock.AbsTime
}

// SetRate changes the expiration rate which is the inverse of the time constant in
// nanoseconds.
func (e *Expirer) SetRate(now mclock.AbsTime, rate float64) {
	e.lock.Lock()
	defer e.lock.Unlock()

	dt := now - e.lastUpdate
	if dt > 0 {
		e.logOffset += Fixed64(logToFixedFactor * float64(dt) * e.rate)
	}
	e.lastUpdate = now
	e.rate = rate
}

// SetLogOffset sets logOffset instantly.
func (e *Expirer) SetLogOffset(now mclock.AbsTime, logOffset Fixed64) {
	e.lock.Lock()
	defer e.lock.Unlock()

	e.lastUpdate = now
	e.logOffset = logOffset
}

// LogOffset returns the current logarithmic offset.
func (e *Expirer) LogOffset(now mclock.AbsTime) Fixed64 {
	e.lock.RLock()
	defer e.lock.RUnlock()

	dt := now - e.lastUpdate
	if dt <= 0 {
		return e.logOffset
	}
	return e.logOffset + Fixed64(logToFixedFactor*float64(dt)*e.rate)
}

// fixedFactor is the fixed point multiplier factor used by Fixed64.
const fixedFactor = 0x1000000

// Fixed64 implements 64-bit fixed point arithmetic functions.
type Fixed64 int64

// Uint64ToFixed64 converts uint64 integer to Fixed64 format.
func Uint64ToFixed64(f uint64) Fixed64 {
	return Fixed64(f * fixedFactor)
}

// Float64ToFixed64 converts float64 to Fixed64 format.
func Float64ToFixed64(f float64) Fixed64 {
	return Fixed64(f * fixedFactor)
}

// ToUint64 converts Fixed64 format to uint64.
func (f64 Fixed64) ToUint64() uint64 {
	return uint64(f64) / fixedFactor
}

// Fraction returns the fractional part of a Fixed64 value.
func (f64 Fixed64) Fraction() Fixed64 {
	return f64 % fixedFactor
}

var (
	logToFixedFactor = float64(fixedFactor) / math.Log(2)
	fixedToLogFactor = math.Log(2) / float64(fixedFactor)
)

// Pow2 returns the base 2 power of the fixed point value.
func (f64 Fixed64) Pow2() float64 {
	return math.Exp(float64(f64) * fixedToLogFactor)
}
