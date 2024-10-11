package logfmt

import (
	"bytes"
	"testing"
)

func BenchmarkDecodeKeyval(b *testing.B) {
	const rows = 10000
	data := []byte{}
	for i := 0; i < rows; i++ {
		data = append(data, "a=1 b=\"bar\" ƒ=2h3s r=\"esc\\tmore stuff\" d x=sf   \n"...)
	}

	b.SetBytes(int64(len(data)))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var (
			dec = NewDecoder(bytes.NewReader(data))
			j   = 0
		)
		for dec.ScanRecord() {
			for dec.ScanKeyval() {
			}
			j++
		}
		if err := dec.Err(); err != nil {
			b.Errorf("got %v, want %v", err, nil)
		}
		if j != rows {
			b.Errorf("got %v, want %v", j, rows)
		}
	}
}
