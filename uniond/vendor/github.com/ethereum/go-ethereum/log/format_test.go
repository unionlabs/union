package log

import (
	"math/rand"
	"testing"
)

var sink string

func BenchmarkPrettyInt64Logfmt(b *testing.B) {
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		sink = FormatLogfmtInt64(rand.Int63())
	}
}

func BenchmarkPrettyUint64Logfmt(b *testing.B) {
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		sink = FormatLogfmtUint64(rand.Uint64())
	}
}
