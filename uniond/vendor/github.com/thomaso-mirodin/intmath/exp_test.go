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

// ===============================================
// ==== Benchmarking different Exp algorithms ====
// ===============================================

// Using i64 package's simple exp function
func BenchmarkExpInt64(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := int64(0); j < 15; j++ {
			for k := int64(0); k < j; k++ {
				_ = i64.Pow(j, k)
			}
		}
	}
}

// Using the standard Math package exp function and casting from/to int64
func BenchmarkExpFloat64ToInt64(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := int64(0); j < 15; j++ {
			for k := int64(0); k < j; k++ {
				_ = int64(math.Pow(float64(j), float64(k)))
			}
		}
	}
}

// Using i32 package's simple exp function
func BenchmarkExpInt32(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := int32(0); j < 15; j++ {
			for k := int32(0); k < j; k++ {
				_ = i32.Pow(j, k)
			}
		}
	}

}

// Using the standard Math package exp function and casting from/to int32
func BenchmarkExpFloat64ToInt32(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := int32(0); j < 15; j++ {
			for k := int32(0); k < j; k++ {
				_ = int32(math.Pow(float64(j), float64(k)))
			}
		}
	}
}

// Using u64 package's simple exp function
func BenchmarkExpUint64(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint64(0); j < 15; j++ {
			for k := uint64(0); k < j; k++ {
				_ = u64.Pow(j, k)
			}
		}
	}
}

// Using the standard Math package exp function and casting from/to uint64
func BenchmarkExpFloat64ToUint64(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint64(0); j < 15; j++ {
			for k := uint64(0); k < j; k++ {
				_ = uint64(math.Pow(float64(j), float64(k)))
			}
		}
	}
}

// Using u32 package's simple exp function
func BenchmarkExpUint32(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint32(0); j < 15; j++ {
			for k := uint32(0); k < j; k++ {
				_ = u32.Pow(j, k)
			}
		}
	}

}

// Using the standard Math package exp function and casting from/to uint32
func BenchmarkExpFloat64ToUint32(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint32(0); j < 15; j++ {
			for k := uint32(0); k < j; k++ {
				_ = uint32(math.Pow(float64(j), float64(k)))
			}
		}
	}
}

// Using intgr package's simple exp function
func BenchmarkExpInt(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := 0; j < 15; j++ {
			for k := 0; k < j; k++ {
				_ = intgr.Pow(j, k)
			}
		}
	}
}

// Using the standard Math package exp function and casting from/to int
func BenchmarkExpFloat64ToInt(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := 0; j < 15; j++ {
			for k := 0; k < j; k++ {
				_ = int(math.Pow(float64(j), float64(k)))
			}
		}
	}
}

// Using uintgr package's simple exp function
func BenchmarkExpUint(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint(0); j < 15; j++ {
			for k := uint(0); k < j; k++ {
				_ = uintgr.Pow(j, k)
			}
		}
	}

}

// Using the standard Math package exp function and casting from/to uint
func BenchmarkExpFloat64ToUint(b *testing.B) {
	for i := 0; i < b.N; i++ {
		for j := uint(0); j < 15; j++ {
			for k := uint(0); k < j; k++ {
				_ = uint(math.Pow(float64(j), float64(k)))
			}
		}
	}
}
