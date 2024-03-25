// Package uints implements optimised byte and long integer operations.
//
// Usually arithmetic in a circuit is performed in the native field, which is of
// prime order. However, for compatibility with native operations we rely on
// operating on smaller primitive types as 8-bit, 32-bit and 64-bit integer.
// Naively, these operations have to be implemented bitwise as there are no
// closed equations for boolean operations (XOR, AND, OR).
//
// However, the bitwise approach is very inefficient and leads to several
// constraints per bit. Accumulating over a long integer, it leads to very
// inefficients circuits.
//
// This package performs boolean operations using lookup tables on bytes. So,
// long integers are split into 4 or 8 bytes and we perform the operations
// bytewise. In the lookup tables, we store results for all possible 2^8×2^8
// inputs. With this approach, every bytewise operation costs as single lookup,
// which depending on the backend is relatively cheap (one to three
// constraints).
//
// NB! The package is still work in progress. The interfaces and implementation
// details most certainly changes over time. We cannot ensure the soundness of
// the operations.
package uints

import (
	"fmt"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/internal/logderivprecomp"
	"github.com/consensys/gnark/std/math/bitslice"
	"github.com/consensys/gnark/std/rangecheck"
)

// TODO: if internal then enforce range check!

// TODO: all operations can take rand linear combinations instead. Then instead
// of one check can perform multiple at the same time.

// TODO: implement versions which take multiple inputs. Maybe can combine multiple together

// TODO: instantiate tables only when we first query. Maybe do not need to build!

// TODO: maybe can store everything in a single table? Later! Or if we have a
// lot of queries then makes sense to extract into separate table?

// TODO: in ValueOf ensure consistency

// TODO: distinguish between when we set constant in-circuit or witness
// assignment. For constant we don't have to range check but for witness
// assignment we have to.

// TODO: add something which allows to store array in native element

// TODO: add methods for checking if U8/Long is constant.

// TODO: should something for byte-only ops. Implement a type and then embed it in BinaryField

// TODO: add helper method to call hints which allows to pass in uint8s (bytes)
// and returns bytes. Then can to byte array manipluation nicely. It is useful
// for X509. For the implementation we want to pack as much bytes into a field
// element as possible.

// TODO: methods for converting uint array into emulated element and native
// element. Most probably should add the implementation for non-native in its
// package, but for native we should add it here.

type U8 struct {
	Val      frontend.Variable
	internal bool
}

// GnarkInitHook describes how to initialise the element.
func (e *U8) GnarkInitHook() {
	if e.Val == nil {
		e.Val = 0
		e.internal = false // we need to constrain in later.
	}
}

type U64 [8]U8
type U32 [4]U8

type Long interface{ U32 | U64 }

type BinaryField[T U32 | U64] struct {
	api        frontend.API
	xorT, andT *logderivprecomp.Precomputed
	rchecker   frontend.Rangechecker
	allOne     U8
}

func New[T Long](api frontend.API) (*BinaryField[T], error) {
	xorT, err := logderivprecomp.New(api, xorHint, []uint{8})
	if err != nil {
		return nil, fmt.Errorf("new xor table: %w", err)
	}
	andT, err := logderivprecomp.New(api, andHint, []uint{8})
	if err != nil {
		return nil, fmt.Errorf("new and table: %w", err)
	}
	rchecker := rangecheck.New(api)
	bf := &BinaryField[T]{
		api:      api,
		xorT:     xorT,
		andT:     andT,
		rchecker: rchecker,
	}
	// TODO: this is const. add way to init constants
	allOne := bf.ByteValueOf(0xff)
	bf.allOne = allOne
	return bf, nil
}

func NewU8(v uint8) U8 {
	// TODO: don't have to check constants
	return U8{Val: v, internal: true}
}

func NewU32(v uint32) U32 {
	return [4]U8{
		NewU8(uint8((v >> (0 * 8)) & 0xff)),
		NewU8(uint8((v >> (1 * 8)) & 0xff)),
		NewU8(uint8((v >> (2 * 8)) & 0xff)),
		NewU8(uint8((v >> (3 * 8)) & 0xff)),
	}
}

func NewU64(v uint64) U64 {
	return [8]U8{
		NewU8(uint8((v >> (0 * 8)) & 0xff)),
		NewU8(uint8((v >> (1 * 8)) & 0xff)),
		NewU8(uint8((v >> (2 * 8)) & 0xff)),
		NewU8(uint8((v >> (3 * 8)) & 0xff)),
		NewU8(uint8((v >> (4 * 8)) & 0xff)),
		NewU8(uint8((v >> (5 * 8)) & 0xff)),
		NewU8(uint8((v >> (6 * 8)) & 0xff)),
		NewU8(uint8((v >> (7 * 8)) & 0xff)),
	}
}

func NewU8Array(v []uint8) []U8 {
	ret := make([]U8, len(v))
	for i := range v {
		ret[i] = NewU8(v[i])
	}
	return ret
}

func NewU32Array(v []uint32) []U32 {
	ret := make([]U32, len(v))
	for i := range v {
		ret[i] = NewU32(v[i])
	}
	return ret
}

func NewU64Array(v []uint64) []U64 {
	ret := make([]U64, len(v))
	for i := range v {
		ret[i] = NewU64(v[i])
	}
	return ret
}

func (bf *BinaryField[T]) ByteValueOf(a frontend.Variable) U8 {
	bf.rchecker.Check(a, 8)
	return U8{Val: a, internal: true}
}

func (bf *BinaryField[T]) ValueOf(a frontend.Variable) T {
	var r T
	bts, err := bf.api.Compiler().NewHint(toBytes, len(r), len(r), a)
	if err != nil {
		panic(err)
	}
	// TODO: add constraint which ensures that map back to
	for i := range bts {
		r[i] = bf.ByteValueOf(bts[i])
	}
	return r
}

func (bf *BinaryField[T]) ToValue(a T) frontend.Variable {
	v := make([]frontend.Variable, len(a))
	for i := range v {
		v[i] = bf.api.Mul(a[i].Val, 1<<(i*8))
	}
	vv := bf.api.Add(v[0], v[1], v[2:]...)
	return vv
}

func (bf *BinaryField[T]) PackMSB(a ...U8) T {
	var ret T
	for i := range a {
		ret[len(a)-i-1] = a[i]
	}
	return ret
}

func (bf *BinaryField[T]) PackLSB(a ...U8) T {
	var ret T
	for i := range a {
		ret[i] = a[i]
	}
	return ret
}

func (bf *BinaryField[T]) UnpackMSB(a T) []U8 {
	ret := make([]U8, len(a))
	for i := 0; i < len(a); i++ {
		ret[len(a)-i-1] = a[i]
	}
	return ret
}

func (bf *BinaryField[T]) UnpackLSB(a T) []U8 {
	// cannot deduce that a can be cast to []U8
	ret := make([]U8, len(a))
	for i := 0; i < len(a); i++ {
		ret[i] = a[i]
	}
	return ret
}

func (bf *BinaryField[T]) twoArgFn(tbl *logderivprecomp.Precomputed, a ...U8) U8 {
	ret := tbl.Query(a[0].Val, a[1].Val)[0]
	for i := 2; i < len(a); i++ {
		ret = tbl.Query(ret, a[i].Val)[0]
	}
	return U8{Val: ret}
}

func (bf *BinaryField[T]) twoArgWideFn(tbl *logderivprecomp.Precomputed, a ...T) T {
	var r T
	for i, v := range reslice(a) {
		r[i] = bf.twoArgFn(tbl, v...)
	}
	return r
}

func (bf *BinaryField[T]) And(a ...T) T { return bf.twoArgWideFn(bf.andT, a...) }
func (bf *BinaryField[T]) Xor(a ...T) T { return bf.twoArgWideFn(bf.xorT, a...) }

func (bf *BinaryField[T]) not(a U8) U8 {
	ret := bf.xorT.Query(a.Val, bf.allOne.Val)
	return U8{Val: ret[0]}
}

func (bf *BinaryField[T]) Not(a T) T {
	var r T
	for i := 0; i < len(a); i++ {
		r[i] = bf.not(a[i])
	}
	return r
}

func (bf *BinaryField[T]) Add(a ...T) T {
	va := make([]frontend.Variable, len(a))
	for i := range a {
		va[i] = bf.ToValue(a[i])
	}
	vres := bf.api.Add(va[0], va[1], va[2:]...)
	res := bf.ValueOf(vres)
	// TODO: should also check the that carry we omitted is correct.
	return res
}

func (bf *BinaryField[T]) Lrot(a T, c int) T {
	l := len(a)
	if c < 0 {
		c = l*8 + c
	}
	shiftBl := c / 8
	shiftBt := c % 8
	revShiftBt := 8 - shiftBt
	if revShiftBt == 8 {
		revShiftBt = 0
	}
	partitioned := make([][2]frontend.Variable, l)
	for i := range partitioned {
		lower, upper := bitslice.Partition(bf.api, a[i].Val, uint(revShiftBt), bitslice.WithNbDigits(8))
		partitioned[i] = [2]frontend.Variable{lower, upper}
	}
	var ret T
	for i := 0; i < l; i++ {
		if shiftBt != 0 {
			ret[(i+shiftBl)%l].Val = bf.api.Add(bf.api.Mul(1<<(shiftBt), partitioned[i][0]), partitioned[(i+l-1)%l][1])
		} else {
			ret[(i+shiftBl)%l].Val = partitioned[i][1]
		}
	}
	return ret
}

func (bf *BinaryField[T]) Rshift(a T, c int) T {
	shiftBl := c / 8
	shiftBt := c % 8
	partitioned := make([][2]frontend.Variable, len(a)-shiftBl)
	for i := range partitioned {
		lower, upper := bitslice.Partition(bf.api, a[i+shiftBl].Val, uint(shiftBt), bitslice.WithNbDigits(8))
		partitioned[i] = [2]frontend.Variable{lower, upper}
	}
	var ret T
	for i := 0; i < len(a)-shiftBl-1; i++ {
		if shiftBt != 0 {
			ret[i].Val = bf.api.Add(partitioned[i][1], bf.api.Mul(1<<(8-shiftBt), partitioned[i+1][0]))
		} else {
			ret[i].Val = partitioned[i][1]
		}
	}
	ret[len(a)-shiftBl-1].Val = partitioned[len(a)-shiftBl-1][1]
	for i := len(a) - shiftBl; i < len(ret); i++ {
		ret[i] = NewU8(0)
	}
	return ret
}

func (bf *BinaryField[T]) ByteAssertEq(a, b U8) {
	bf.api.AssertIsEqual(a.Val, b.Val)
}

func (bf *BinaryField[T]) AssertEq(a, b T) {
	for i := 0; i < len(a); i++ {
		bf.ByteAssertEq(a[i], b[i])
	}
}

func reslice[T U32 | U64](in []T) [][]U8 {
	if len(in) == 0 {
		panic("zero-length input")
	}
	ret := make([][]U8, len(in[0]))
	for i := range ret {
		ret[i] = make([]U8, len(in))
	}
	for i := 0; i < len(in); i++ {
		for j := 0; j < len(in[0]); j++ {
			ret[j][i] = in[i][j]
		}
	}
	return ret
}
