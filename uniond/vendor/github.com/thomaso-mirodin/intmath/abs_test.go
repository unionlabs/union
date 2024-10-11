package intmath

import (
	"testing"
)

// ================================================
// ==== Benchmarking different Abs algorithms ====
// ================================================

// Bitmasking Abs as I found it in package bitbucket.org/SyntaxK/imath. int32 version
func BenchmarkAbs32bit1(b *testing.B) {
	for x := int32(0); -x < int32(b.N); x-- {
		_ = x ^ x>>31 + x>>31&1
	}
}

// Bitmasking Abs that seems simpler to me, used in library. int32 version
// Appears to be of similar speed on my 32bit system
func BenchmarkAbs32bit2(b *testing.B) {
	for x := int32(0); -x < int32(b.N); x-- {
		_ = x ^ x>>31 - x>>31
	}
}

// Bitmasking Abs as I found it in package bitbucket.org/SyntaxK/imath. int64 version
func BenchmarkAbs64bit1(b *testing.B) {
	for x := int64(0); -x < int64(b.N); x-- {
		_ = x ^ x>>63 + x>>63&1
	}
}

// Bitmasking Abs that seems simpler to me, used in library. int64 version
// Appears to be faster in speed on my 32bit system, presumably because 
// int64 is not natively supported, making the other operation more complex
func BenchmarkAbs64bit2(b *testing.B) {
	for x := int64(0); -x < int64(b.N); x-- {
		_ = x ^ x>>63 - x>>63
	}
}
