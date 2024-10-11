//go:build ignore
// +build ignore

package main

import (
	"fmt"
	"log"
	"testing"

	"github.com/cespare/xxhash/v2"
)

const (
	in   = "Call me Ishmael. Some years ago--never mind how long precisely-"
	want = uint64(0x02a2e85470d6fd96)
)

func TestSum(t *testing.T) {
	got := xxhash.Sum64String(in)
	if got != want {
		t.Fatalf("Sum64String: got 0x%x; want 0x%x", got, want)
	}
}

func TestDigest(t *testing.T) {
	for chunkSize := 1; chunkSize <= len(in); chunkSize++ {
		name := fmt.Sprintf("[chunkSize=%d]", chunkSize)
		t.Run(name, func(t *testing.T) {
			d := xxhash.New()
			for i := 0; i < len(in); i += chunkSize {
				chunk := in[i:]
				if len(chunk) > chunkSize {
					chunk = chunk[:chunkSize]
				}
				n, err := d.WriteString(chunk)
				if err != nil || n != len(chunk) {
					t.Fatalf("Digest.WriteString: got (%d, %v); want (%d, nil)",
						n, err, len(chunk))
				}
			}
			if got := d.Sum64(); got != want {
				log.Fatalf("Digest.Sum64: got 0x%x; want 0x%x", got, want)
			}
		})
	}
}
