package xxhash

import (
	"strings"
	"testing"
)

var benchmarks = []struct {
	name string
	n    int64
}{
	{"4B", 4},
	{"16B", 16},
	{"100B", 100},
	{"4KB", 4e3},
	{"10MB", 10e6},
}

func BenchmarkSum64(b *testing.B) {
	for _, bb := range benchmarks {
		in := make([]byte, bb.n)
		for i := range in {
			in[i] = byte(i)
		}
		b.Run(bb.name, func(b *testing.B) {
			b.SetBytes(bb.n)
			for i := 0; i < b.N; i++ {
				_ = Sum64(in)
			}
		})
	}
}

func BenchmarkSum64String(b *testing.B) {
	for _, bb := range benchmarks {
		s := strings.Repeat("a", int(bb.n))
		b.Run(bb.name, func(b *testing.B) {
			b.SetBytes(bb.n)
			for i := 0; i < b.N; i++ {
				_ = Sum64String(s)
			}
		})
	}
}

func BenchmarkDigestBytes(b *testing.B) {
	for _, bb := range benchmarks {
		in := make([]byte, bb.n)
		for i := range in {
			in[i] = byte(i)
		}
		b.Run(bb.name, func(b *testing.B) {
			b.SetBytes(bb.n)
			for i := 0; i < b.N; i++ {
				h := New()
				h.Write(in)
				_ = h.Sum64()
			}
		})
	}
}

func BenchmarkDigestString(b *testing.B) {
	for _, bb := range benchmarks {
		s := strings.Repeat("a", int(bb.n))
		b.Run(bb.name, func(b *testing.B) {
			b.SetBytes(bb.n)
			for i := 0; i < b.N; i++ {
				h := New()
				h.WriteString(s)
				_ = h.Sum64()
			}
		})
	}
}
