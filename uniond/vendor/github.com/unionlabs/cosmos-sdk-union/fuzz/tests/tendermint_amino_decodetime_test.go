//go:build gofuzz || go1.18

package tests

import (
	"fmt"
	"testing"

	"github.com/tendermint/go-amino"
)

func FuzzTendermintAminoDecodeTime(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		if len(data) == 0 {
			return
		}
		_, n, err := amino.DecodeTime(data)
		if err != nil {
			return
		}
		if n < 0 {
			panic(fmt.Sprintf("n=%d < 0", n))
		}
	})
}
