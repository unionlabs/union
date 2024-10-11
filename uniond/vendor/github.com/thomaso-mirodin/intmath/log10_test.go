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

// Using i64 package's Log10 function
func BenchmarkLog10Int64(b *testing.B) {
	var k int64
	for i := 0; i < b.N; i++ {
		for j := uint(0); j < 64; j++ {
			k = int64(1) << j
			_ = i64.Log10(k)
		}
	}
}

// Using Log10 from the standard Math package and casting from/to int64
func BenchmarkLog10Float64ToInt64(b *testing.B) {
	var k uint64
	for i := 0; i < b.N; i++ {
		for j := uint(0); j < 64; j++ {
			k = uint64(1) << j
			_ = int64(math.Log10(float64(k)))
		}
	}
}

// Using u64 package's Log10 function
func BenchmarkLog10Uint64(b *testing.B) {
	var k uint64
	for i := 0; i < b.N; i++ {
		for j := uint(0); j < 64; j++ {
			k = uint64(1) << j
			_ = u64.Log10(k)
		}
	}
}

// Using Log10 from the standard Math package and casting from/to uint64
func BenchmarkLog10Float64ToUint64(b *testing.B) {
	var k uint64
	for i := 0; i < b.N; i++ {
		for j := uint(0); j < 64; j++ {
			k = uint64(1) << j
			_ = uint64(math.Log10(float64(k)))
		}
	}
}

// Using i32 package's Log10 function
func BenchmarkLog10Int32(b *testing.B) {
	var k int32
	for i := 0; i < b.N; i++ {
		for j := uint(0); j < 32; j++ {
			k = int32(1) << j
			_ = i32.Log10(k)
		}
	}
}

// Using Log10 from the standard Math package and casting from/to int32
func BenchmarkLog10Float64ToInt32(b *testing.B) {
	var k int32
	for i := 0; i < b.N; i++ {
		for j := uint(0); j < 32; j++ {
			k = int32(1) << j
			_ = int32(math.Log10(float64(k)))
		}
	}
}

// Using u32 package's Log10 function
func BenchmarkLog10Uint32(b *testing.B) {
	var k uint32
	for i := 0; i < b.N; i++ {
		for j := uint(0); j < 32; j++ {
			k = uint32(1) << j
			_ = u32.Log10(k)
		}
	}
}

// Using Log10 from the standard Math package and casting from/to uint32
func BenchmarkLog10Float64ToUint32(b *testing.B) {
	var k uint32
	for i := 0; i < b.N; i++ {
		for j := uint(0); j < 32; j++ {
			k = uint32(1) << j
			_ = uint32(math.Log10(float64(k)))
		}
	}
}

// Using intgr package's Log10 function
func BenchmarkLog10Int(b *testing.B) {
	jMax := uint(1)
	if jMax<<32 == 0 {
		jMax = 32
	} else {
		jMax = 64
	}
	var k int
	for i := 0; i < b.N; i++ {
		for j := uint(0); j < jMax; j++ {
			k = int(1) << j
			_ = intgr.Log10(k)
		}
	}
}

// Using Log10 from the standard Math package and casting from/to int
func BenchmarkLog10Float64ToInt(b *testing.B) {
	jMax := uint(1)
	if jMax<<32 == 0 {
		jMax = 32
	} else {
		jMax = 64
	}
	var k int
	for i := 0; i < b.N; i++ {
		for j := uint(0); j < jMax; j++ {
			k = int(1) << j
			_ = int(math.Log10(float64(k)))
		}
	}
}

// Using uintgr package's Log10 function
func BenchmarkLog10Uint(b *testing.B) {
	jMax := uint(1)
	if jMax<<32 == 0 {
		jMax = 32
	} else {
		jMax = 64
	}
	var k uint
	for i := 0; i < b.N; i++ {
		for j := uint(0); j < jMax; j++ {
			k = uint(1) << j
			_ = uintgr.Log10(k)
		}
	}
}

// Using Log10 from the standard Math package and casting from/to uint
func BenchmarkLog10Float64ToUint(b *testing.B) {
	jMax := uint(1)
	if jMax<<32 == 0 {
		jMax = 32
	} else {
		jMax = 64
	}
	var k uint
	for i := 0; i < b.N; i++ {
		for j := uint(0); j < jMax; j++ {
			k = uint(1) << j
			_ = uint32(math.Log10(float64(k)))
		}
	}
}
