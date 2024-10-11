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

func BenchmarkCbrtIntgr(b *testing.B) {
	k := uint(1)
	if k<<32 != 0 {
		k = 32
	} else {
		k = 0
	}
	for i := 0; i < b.N; i++ {
		for j := 0x10000; j < 0x4000000; j += 0x10000 {
			_ = intgr.Cbrt(j << k)
		}
	}
}

func BenchmarkCbrtUintgr(b *testing.B) {
	k := uint(1)
	if k<<32 != 0 {
		k = 32
	} else {
		k = 0
	}
	for i := 0; i < b.N; i++ {
		for j := uint(0x10000); j < 0x4000000; j += 0x10000 {
			_ = uintgr.Cbrt(j << k)
		}
	}
}

func BenchmarkCbrtInt32(b *testing.B) {
	k := uint(0) //to prevent unfair advantage over other benchmarks
	for i := 0; i < b.N; i++ {
		for j := int32(0x10000); j < 0x4000000; j += 0x10000 {
			_ = i32.Cbrt(j << k)
		}
	}
}

func BenchmarkCbrtUint32(b *testing.B) {
	k := uint(0) //to prevent unfair advantage over other benchmarks
	for i := 0; i < b.N; i++ {
		for j := uint32(0x10000); j < 0x4000000; j += 0x10000 {
			_ = u32.Cbrt(j << k)
		}
	}
}

func BenchmarkCbrtInt64(b *testing.B) {
	k := uint(32)
	for i := 0; i < b.N; i++ {
		for j := int64(0x10000); j < 0x4000000; j += 0x10000 {
			_ = i64.Cbrt(j << k)
		}
	}
}

func BenchmarkCbrtUint64(b *testing.B) {
	k := uint(32)
	for i := 0; i < b.N; i++ {
		for j := uint64(0x10000); j < 0x4000000; j += 0x10000 {
			_ = u64.Cbrt(j << k)
		}
	}
}

func BenchmarkCbrtFloat64ToInt(b *testing.B) {
	k := uint(1)
	if k<<32 != 0 {
		k = 32
	} else {
		k = 0
	}
	for i := 0; i < b.N; i++ {
		for j := 0x10000; j < 0x4000000; j += 0x10000 {
			_ = int(math.Cbrt(float64(j << k)))
		}
	}
}

func BenchmarkCbrtFloat64ToUint(b *testing.B) {
	k := uint(1)
	if k<<32 != 0 {
		k = 32
	} else {
		k = 0
	}
	for i := 0; i < b.N; i++ {
		for j := uint(0x10000); j < 0x4000000; j += 0x10000 {
			_ = uint(math.Cbrt(float64(j << k)))
		}
	}
}

func BenchmarkCbrtFloat64ToInt32(b *testing.B) {
	k := uint(0)
	for i := 0; i < b.N; i++ {
		for j := int32(0x10000); j < 0x4000000; j += 0x10000 {
			_ = int32(math.Cbrt(float64(j << k)))
		}
	}
}

func BenchmarkCbrtFloat64ToUint32(b *testing.B) {
	k := uint(0)
	for i := 0; i < b.N; i++ {
		for j := uint32(0x10000); j < 0x4000000; j += 0x10000 {
			_ = uint32(math.Cbrt(float64(j << k)))
		}
	}
}

func BenchmarkCbrtFloat64ToInt64(b *testing.B) {
	k := uint(32)
	for i := 0; i < b.N; i++ {
		for j := int64(0x10000); j < 0x4000000; j += 0x10000 {
			_ = int64(math.Cbrt(float64(j << k)))
		}
	}
}

func BenchmarkCbrtFloat64ToUint64(b *testing.B) {
	k := uint(32)
	for i := 0; i < b.N; i++ {
		for j := uint64(0x10000); j < 0x4000000; j += 0x10000 {
			_ = uint64(math.Cbrt(float64(j << k)))
		}
	}
}

func TestCrt(t *testing.T) {
	t.Logf("Testing Crt\n")
	// 32 bits -> 10 bits max number
	// 64 bits -> 21 bits max number
	i32Max := int32(1<<10 - 1)
	i64Max := int64(1<<21 - 1)
	u32Max := uint32(1<<10 - 1)
	u64Max := uint64(1<<21 - 1)

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
		k := intgr.Cbrt(i * i * i)
		if k != i {
			t.Logf("intgr.Cbrt(%X*%X*%X) == %X\n", i, i, i, k)
			t.Fail()
		}
		k = intgr.Cbrt(-(i * i * i))
		if k != -i {
			t.Logf("intgr.Cbrt(%X*%X*%X) == %X\n", -i, -i, -i, k)
			t.Fail()
		}
	}
	for i := uMax; i > 0; i >>= 1 {
		k := uintgr.Cbrt(i * i * i)
		if k != i {
			t.Logf("uintgr.Cbrt(%X*%X*%X) == %X\n", i, i, i, k)
			t.Fail()
		}
	}
	for i := i32Max; i > 0; i >>= 1 {
		k := i32.Cbrt(i * i * i)
		if k != i {
			t.Logf("i32.Cbrt(%X*%X*%X) == %X\n", i, i, i, k)
			t.Fail()
		}
		k = i32.Cbrt(-(i * i * i))
		if k != -i {
			t.Logf("i32.Cbrt(%X*%X*%X) == %X\n", -i, -i, -i, k)
			t.Fail()
		}
	}
	for i := u32Max; i > 0; i >>= 1 {
		k := u32.Cbrt(i * i * i)
		if k != i {
			t.Logf("u32.Cbrt(%X*%X*%X) == %X\n", i, i, i, k)
			t.Fail()
		}
	}
	for i := i64Max; i > 0; i >>= 1 {
		k := i64.Cbrt(i * i * i)
		if k != i {
			t.Logf("i64.Cbrt(%X*%X*%X) == %X\n", i, i, i, k)
			t.Fail()
		}
		k = i64.Cbrt(-(i * i * i))
		if k != -i {
			t.Logf("i64.Cbrt(%X*%X*%X) == %X\n", -i, -i, -i, k)
			t.Fail()
		}
	}
	for i := u64Max; i > 0; i >>= 1 {
		k := u64.Cbrt(i * i * i)
		if k != i {
			t.Logf("u64.Cbrt(%X*%X*%X) == %X\n", i, i, i, k)
			t.Fail()
		}
	}
}
