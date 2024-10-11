package encoding

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"math"
	"testing"
)

var encValues = []int64{
	-1, -100, -1 << 32,
	0, 1, 100, 1 << 32,
	-1 << 52, 1 << 52, 17,
	19, 28, 37, 388888888,
	-99999999999, 99999999999,
	math.MaxInt64, math.MinInt64,
}

// This tests that the results from directly invoking binary.PutVarint match
// exactly those that we get from invoking EncodeVarint and its internals.
func TestEncodeVarintParity(t *testing.T) {
	buf := new(bytes.Buffer)
	var board [binary.MaxVarintLen64]byte

	for _, val := range encValues {
		val := val
		name := fmt.Sprintf("%d", val)

		buf.Reset()
		t.Run(name, func(t *testing.T) {
			if err := EncodeVarint(buf, val); err != nil {
				t.Fatal(err)
			}

			n := binary.PutVarint(board[:], val)
			got := buf.Bytes()
			want := board[:n]
			if !bytes.Equal(got, want) {
				t.Fatalf("Result mismatch\n\tGot:  %d\n\tWant: %d", got, want)
			}
		})
	}
}

func BenchmarkEncodeVarint(b *testing.B) {
	buf := new(bytes.Buffer)
	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		for _, val := range encValues {
			if err := EncodeVarint(buf, val); err != nil {
				b.Fatal(err)
			}
			buf.Reset()
		}
	}
}
