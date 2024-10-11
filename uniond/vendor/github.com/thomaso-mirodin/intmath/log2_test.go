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

// ================================================
// ==== Benchmarking different Log2 algorithms ====
// ================================================
//
// uint is always set to the native word width, and it is therefore 
// logical to expect that the operation is significantly faster on 
// 32bit systems if we use it instead of uint64. On a netbook with
// an Atom N280 CPU the benchmark confirm that this speeds up the
// algorithm by 25%. The unnecessary cast on 64bit systems was 
// considered worth the trade-off.

func BenchmarkLog2NaiveU32(b *testing.B) {
	for n := 0; n < b.N; n++ {
		for i := uint32(1); i < 32; i++ {
			_ = log2U32(1 << i)
		}

	}
}

func BenchmarkLog2U32(b *testing.B) {
	for n := 0; n < b.N; n++ {
		for i := uint32(1); i < 32; i++ {
			_ = loopU32Log2(1 << i)
		}
	}
}

func BenchmarkLog2bitmaskU32(b *testing.B) {
	for n := 0; n < b.N; n++ {
		for i := uint32(1); i < 32; i++ {
			_ = bitmaskU32Log2(1 << i)
		}
	}
}

func BenchmarkLog2NaiveU64(b *testing.B) {
	for n := 0; n < b.N; n++ {
		for i := uint64(1); i < 64; i++ {
			_ = log2U64(1 << i)
		}
	}
}

func BenchmarkLog2U64(b *testing.B) {
	for n := 0; n < b.N; n++ {
		for i := uint64(1); i < 64; i++ {
			_ = loopU64Log2(1 << i)
		}
	}
}

func BenchmarkLog2bitmaskU64(b *testing.B) {
	for n := 0; n < b.N; n++ {
		for i := uint64(1); i < 64; i++ {
			_ = bitmaskU64Log2(1 << i)
		}
	}
}

func BenchmarkLog2Float64ToU64(b *testing.B) {
	for n := 0; n < b.N; n++ {
		for i := uint64(1); i < 64; i++ {
			_ = uint64(math.Log2(float64(uint64(1 << i))))
		}
	}
}

func log2U32(n uint32) (r uint32) {
	for n > 1 {
		n >>= 1
		r++
	}
	return
}

func loopU32Log2(n uint32) uint32 {
	for i := uint32(25); i >= 0; i -= 8 {
		if n>>i > 0 {
			for ; ; i++ {
				if n>>i == 1 {
					return i
				}
			}
		}
	}
	return 0
}

func bitmaskU32Log2(n uint32) (r uint32) {
	if n >= 1<<16 {
		r += 16
		n >>= 16
	}
	if n >= 1<<8 {
		r += 8
		n >>= 8
	}
	if n >= 1<<4 {
		r += 4
		n >>= 4
	}
	if n >= 1<<2 {
		r += 2
		n >>= 2
	}
	r += n >> 1
	return
}

func log2U64(n uint64) (r uint64) {
	for n > 1 {
		n >>= 1
		r++
	}
	return
}

func loopU64Log2(n uint64) uint64 {
	for i := uint64(56); i >= 0; i -= 8 {
		if n>>i > 0 {
			for ; ; i++ {
				if n>>i == 1 {
					return i
				}
			}
		}
	}
	return 0
}

func bitmaskU64Log2(n uint64) uint64 {
	var r, v uint
	if n >= 1<<32 {
		r += 32
		v = uint(n >> 32)
	} else {
		v = uint(n)
	}
	if v >= 1<<16 {
		r += 16
		v >>= 16
	}
	if v >= 1<<8 {
		r += 8
		v >>= 8
	}
	if v >= 1<<4 {
		r += 4
		v >>= 4
	}
	if v >= 1<<2 {
		r += 2
		v >>= 2
	}
	r += v >> 1
	return uint64(r)
}

// ====================================================
// ==== Testing Log2 algorithm used in the library ====
// ====================================================

func TestLog2(t *testing.T) {
	t.Log("Testing Log2\n")
	iMax := uint(1)
	if iMax<<32 == 0 {
		iMax = 31
	} else {
		iMax = 63
	}
	for i := uint(0); i < iMax; i++ {
		for j := uint(0); j < i; j++ {
			k := int((1 << i) + (1 << j))
			if intgr.Log2(k) != int(i) {
				t.Logf("intgr.Log2(%v) == %v\n", k, intgr.Log2(k))
				t.FailNow()
			}
		}
	}
	if intgr.Log2(-1) != -1 {
		t.Logf("intgr.Log2(-1) == %v\n", intgr.Log2(-1))
		t.FailNow()
	}

	for i := uint(0); i < iMax; i++ {
		for j := uint(0); j < i; j++ {
			k := uint((1 << i) + (1 << j))
			if uintgr.Log2(k) != i {
				t.Logf("intgr.Log2(%v) == %v\n", k, uintgr.Log2(k))
				t.Fail()
			}
		}
	}

	for i := uint32(0); i < uint32(31); i++ {
		for j := uint32(0); j < i; j++ {
			k := int32((1 << i) + (1 << j))
			if i32.Log2(k) != int32(i) {
				t.Logf("i32.Log2(%v) == %v\n", k, i32.Log2(k))
				t.FailNow()
			}
		}
	}
	if i32.Log2(-1) != int32(-1) {
		t.Logf("i32.Log2(-1) == %v\n", i32.Log2(-1))
		t.FailNow()
	}

	for i := uint32(0); i < uint32(31); i++ {
		for j := uint32(0); j < i; j++ {
			k := uint32((1 << i) + (1 << j))
			if u32.Log2(k) != i {
				t.Logf("u32.Log2(%v) == %v\n", k, u32.Log2(k))
				t.FailNow()
			}
		}
	}

	for i := uint64(0); i < uint64(63); i++ {
		for j := uint64(0); j < i; j++ {
			k := int64((1 << i) + (1 << j))
			if i64.Log2(k) != int64(i) {
				t.Logf("i64.Log2(%v) == %v\n", k, i64.Log2(k))
				t.FailNow()
			}
		}
	}
	if i64.Log2(-1) != int64(-1) {
		t.Logf("i64.Log2(-1) == %v\n", i64.Log2(-1))
		t.FailNow()
	}

	for i := uint64(0); i < uint64(63); i++ {
		for j := uint64(0); j < i; j++ {
			k := uint64((1 << i) + (1 << j))
			if u64.Log2(k) != i {
				t.Logf("u64.Log2(%v) == %v\n", k, u64.Log2(k))
				t.FailNow()
			}
		}
	}
}
