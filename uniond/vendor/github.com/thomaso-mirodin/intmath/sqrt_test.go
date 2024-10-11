package intmath

import (
	"code.google.com/p/intmath/i32"
	"code.google.com/p/intmath/i64"
	"code.google.com/p/intmath/intgr"
	"code.google.com/p/intmath/u32"
	"code.google.com/p/intmath/u64"
	"code.google.com/p/intmath/uintgr"
	"math"
	"testing"
)

func BenchmarkSqrtIntgr(b *testing.B) {
	k := uint(1)
	if k<<32 != 0 {
		k = 32
	} else {
		k = 0
	}
	for i := 0; i < b.N; i++ {
		for j := 0x10000; j < 0x4000000; j += 0x10000 {
			_ = intgr.Sqrt(j << k)
		}
	}
}

func BenchmarkSqrtUintgr(b *testing.B) {
	k := uint(1)
	if k<<32 != 0 {
		k = 32
	} else {
		k = 0
	}
	for i := 0; i < b.N; i++ {
		for j := uint(0x10000); j < 0x4000000; j += 0x10000 {
			_ = uintgr.Sqrt(j << k)
		}
	}
}

func BenchmarkSqrtInt32(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := int32(0x10000); j < 0x4000000; j += 0x10000 {
			_ = i32.Sqrt(j)
		}
	}
}

func BenchmarkSqrtUint32(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint32(0x10000); j < 0x4000000; j += 0x10000 {
			_ = u32.Sqrt(j)
		}
	}
}

func BenchmarkSqrtShiftU32(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint32(0x10000); j < 0x4000000; j += 0x10000 {
			_ = SqrtShiftU32(j)
		}
	}
}

func BenchmarkSqrtHDU32(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint32(0x10000); j < 0x4000000; j += 0x10000 {
			_ = SqrtHDU32(j)
		}
	}
}

func BenchmarkSqrtInt64(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := int64(0x1000000000000); j < 0x400000000000000; j += 0x1000000000000 {
			_ = i64.Sqrt(j)
		}
	}
}

func BenchmarkSqrtUint64(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint64(0x1000000000000); j < 0x400000000000000; j += 0x1000000000000 {
			_ = u64.Sqrt(j)
		}
	}
}

func BenchmarkSqrtShiftU64(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint64(0x1000000000000); j < 0x400000000000000; j += 0x1000000000000 {
			_ = SqrtShiftU64(j)
		}
	}
}

func BenchmarkSqrtHDU64(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint64(0x1000000000000); j < 0x400000000000000; j += 0x1000000000000 {
			_ = SqrtHDU64(j)
		}
	}
}

func BenchmarkSqrtFloat64ToIntgr(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := 0x10000; j < 0x4000000; j += 0x10000 {
			_ = int(math.Sqrt(float64(j)))
		}
	}
}

func BenchmarkSqrtFloat64ToUintgr(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint(0x10000); j < 0x4000000; j += 0x10000 {
			_ = uint(math.Sqrt(float64(j)))
		}
	}
}

func BenchmarkSqrtFloat64ToInt32(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := int32(0x10000); j < 0x4000000; j += 0x10000 {
			_ = int32(math.Sqrt(float64(j)))
		}
	}
}

func BenchmarkSqrtFloat64ToUint32(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint32(0x10000); j < 0x4000000; j += 0x10000 {
			_ = int64(math.Sqrt(float64(j)))
		}
	}
}

func BenchmarkSqrtFloat64ToInt64(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := int64(0x1000000000000); j < 0x400000000000000; j += 0x1000000000000 {
			_ = int64(math.Sqrt(float64(j)))
		}
	}
}

func BenchmarkSqrtFloat64ToUint64(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint64(0x1000000000000); j < 0x400000000000000; j += 0x1000000000000 {
			_ = uint64(math.Sqrt(float64(j)))
		}
	}
}

func SqrtShiftU64(x uint64) uint64 {
	// The highest set bit of the square root of x is equal to
	// log2(x)/2, and always fits in a 32bit word. Using uint 
	// instead of uint64 guarantees native word width, which 
	// is 7/4 faster on my 32bit netbook.
	r := uint(1 << (u64.Log2(x) >> 1))
	s := r >> 1
	t := r + s
	for s > 0 {
		if uint64(t)*uint64(t) <= x {
			r = t
		}
		s >>= 1
		t = r + s
	}
	return uint64(r)
}

func SqrtShiftU32(x uint32) uint32 {
	r := uint(1 << (u32.Log2(x) >> 1))
	s := r >> 1
	t := r + s
	for s > 0 {
		if t*t <= uint(x) {
			r = t
		}
		s >>= 1
		t = r + s
	}
	return uint32(r)
}

func SqrtHDU64(x uint64) uint64 {
	var b, r uint64
	//p highest power of 4 equal or less to x 
	//p := uint64(1 << ((uint(u64.Log2(x)) >> 1) << 1))
	p := x
	var n, v uint
	if p < 1<<32 {
		v = uint(p)
	} else {
		v = uint(p >> 32)
		n = 32
	}

	if v >= 1<<16 {
		v >>= 16
		n += 16
	}
	if v >= 1<<8 {
		v >>= 8
		n += 8
	}
	if v >= 1<<4 {
		v >>= 4
		n += 4
	}
	if v >= 1<<2 {
		v >>= 2
		n += 2
	}
	p = 1 << n

	for ; p != 0; p >>= 2 {
		b = r | p
		r >>= 1
		if x >= b {
			x -= b
			r |= p
		}
	}
	return r
}

func SqrtHDU32(x uint32) uint32 {
	//Using uint guarantees native word width
	var b, r uint
	t := uint(x)
	//Fast way to make p highest power of 4 <= x
	p := t
	if p >= 1<<16 {
		p >>= 16
		b = 16
	}
	if p >= 1<<8 {
		p >>= 8
		b += 8
	}
	if p >= 1<<4 {
		p >>= 4
		b += 4
	}
	if p >= 1<<2 {
		b += 2
	}
	p = 1 << b
	for ; p != 0; p >>= 2 {
		b = r | p
		r >>= 1
		if t >= b {
			t -= b
			r |= p
		}
	}
	return uint32(r)
}

func TestSqrt(t *testing.T) {
	t.Logf("Testing Sqrt\n")
	i32Max := int32(0x7FFF)
	i64Max := int64(0x7FFFFFFF)
	u32Max := uint32(0xFFFF)
	u64Max := uint64(0xFFFFFFFF)

	iMax := 1
	var uMax uint
	if iMax<<32 == 0 {
		iMax = int(i32Max)
		uMax = uint(u32Max)
	} else {
		iMax = int(i64Max)
		uMax = uint(u64Max)
	}

	for i := iMax; i > 0; i >>= 1 {
		if intgr.Sqrt(i*i) != i {
			t.Logf("intgr.Sqrt(%X*%X) == %X\n", i, i, intgr.Sqrt(i))
			t.Fail()
		}
	}
	for i := uMax; i > 0; i >>= 1 {
		if uintgr.Sqrt(i*i) != i {
			t.Logf("uintgr.Sqrt(%X*%X) == %X\n", i, i, uintgr.Sqrt(i))
			t.Fail()
		}
	}
	for i := i32Max; i > 0; i >>= 1 {
		if i32.Sqrt(i*i) != i {
			t.Logf("i32.Sqrt(%X*%X) == %X\n", i, i, i32.Sqrt(i))
			t.Fail()
		}
	}
	for i := u32Max; i > 0; i >>= 1 {
		if u32.Sqrt(i*i) != i {
			t.Logf("u32.Sqrt(%X*%X) == %X\n", i, i, u32.Sqrt(i))
			t.Fail()
		}
	}
	for i := i64Max; i > 0; i >>= 1 {
		if i64.Sqrt(i*i) != i {
			t.Logf("i64.Sqrt(%X*%X) == %X\n", i, i, i64.Sqrt(i))
			t.Fail()
		}
	}
	for i := u64Max; i > 0; i >>= 1 {
		if u64.Sqrt(i*i) != i {
			t.Logf("u64.Sqrt(%X*%X) == %X\n", i, i, u64.Sqrt(i))
			t.Fail()
		}
	}

	for i := u32Max; i > 0; i >>= 1 {
		if SqrtShiftU32(i*i) != i {
			t.Logf("SqrtShiftU32(%X*%X) == %X\n", i, i, SqrtShiftU32(i*i))
			t.Fail()
		}
	}
	for i := u64Max; i > 0; i >>= 1 {
		if SqrtShiftU64(i*i) != i {
			t.Logf("SqrtShiftU64(%X*%X) == %X\n", i, i, SqrtShiftU64(i*i))
			t.Fail()
		}
	}

	for i := u32Max; i > 0; i >>= 1 {
		if SqrtHDU32(i*i) != i {
			t.Logf("SqrtHDU32(%X*%X) == %X\n", i, i, SqrtHDU32(i*i))
			t.Fail()
		}
	}
	for i := u64Max; i > 0; i >>= 1 {
		if SqrtHDU64(i*i) != i {
			t.Logf("SqrtHDU64(%X*%X) == %X\n", i, i, SqrtHDU64(i*i))
			t.Fail()
		}
	}
}
