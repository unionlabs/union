package testdata

import (
	"math"
	"time"
)

// Uint64CastToInt --
func Uint64CastToInt() {
	a := uint64(math.MaxUint64)
	b := int(a) // want "Unsafe cast from uint64 to int."

	_ = b
}

// Uint64CastToIntIfStatement --
func Uint64CastToIntIfStatement() {
	var b []string
	a := uint64(math.MaxUint64)

	if len(b) < int(a) { // want "Unsafe cast from uint64 to int."
		return
	}
}

type Slot = uint64

// BaseTypes should alert on alias like Slot.
func BaseTypes() {
	var slot Slot
	bad := int(slot) // want "Unsafe cast from uint64 to int."
	_ = bad
}

func Uint64CastInStruct() {
	type S struct {
		a int
	}
	s := S{
		a: int(uint64(5)), // want "Unsafe cast from uint64 to int."
	}
	_ = s
}

func Uint64CastFunctionReturn() {
	fn := func() uint64 {
		return 5
	}
	a := int(fn()) // want "Unsafe cast from uint64 to int."
	_ = a
}

// IgnoredResult should not report an error.
func IgnoredResult() {
	a := uint64(math.MaxUint64)
	b := int(a) // lint:ignore uintcast -- test code

	_ = b
}

// IgnoredIfStatement should not report an error.
func IgnoredIfStatement() {
	var balances []int
	var numDeposits uint64
	var i int
	var balance int

	// lint:ignore uintcast -- test code
	if len(balances) == int(numDeposits) {
		balance = balances[i]
	}

	_ = balance
}

func IgnoreInFunctionCall() bool {
	var timestamp uint64
	var timeout time.Time
	return time.Unix(int64(timestamp), 0).Before(timeout) // lint:ignore uintcast -- test code
}

func IgnoreWithOtherComments() bool {
	var timestamp uint64
	var timeout time.Time
	// I plan to live forever. Maybe we should not do this?
	return time.Unix(int64(timestamp), 0).Before(timeout) // lint:ignore uintcast -- timestamp will not exceed int64 in your lifetime.
}
