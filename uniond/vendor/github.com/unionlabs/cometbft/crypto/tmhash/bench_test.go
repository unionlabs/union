package tmhash

import (
	"bytes"
	"crypto/sha256"
	"strings"
	"testing"
)

var sink any

var manySlices = []struct {
	name string
	in   [][]byte
	want [32]byte
}{
	{
		name: "all empty",
		in:   [][]byte{[]byte(""), []byte("")},
		want: sha256.Sum256(nil),
	},
	{
		name: "ax6",
		in:   [][]byte{[]byte("aaaa"), []byte("😎"), []byte("aaaa")},
		want: sha256.Sum256([]byte("aaaa😎aaaa")),
	},
	{
		name: "composite joined",
		in:   [][]byte{bytes.Repeat([]byte("a"), 1<<10), []byte("AA"), bytes.Repeat([]byte("z"), 100)},
		want: sha256.Sum256([]byte(strings.Repeat("a", 1<<10) + "AA" + strings.Repeat("z", 100))),
	},
}

func BenchmarkSHA256Many(b *testing.B) {
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		for _, tt := range manySlices {
			got := SumMany(tt.in[0], tt.in[1:]...)
			if !bytes.Equal(got, tt.want[:]) {
				b.Fatalf("Outward checksum mismatch for %q\n\tGot:  %x\n\tWant: %x", tt.name, got, tt.want)
			}
			sink = got
		}
	}

	if sink == nil {
		b.Fatal("Benchmark did not run!")
	}

	sink = nil
}
