//go:build gofuzz || go1.18

package tests

import (
	"bytes"
	"fmt"
	"testing"

	"github.com/cosmos/cosmos-sdk/types"
)

func FuzzTypesParseTimeBytes(f *testing.F) {
	f.Fuzz(func(t *testing.T, bin []byte) {
		// Normalize input, reject invalid timestamps.
		ti, err := types.ParseTimeBytes(bin)
		if err != nil {
			return
		}
		brt := types.FormatTimeBytes(ti)
		// Check that roundtripping a normalized timestamp doesn't change it.
		ti2, err := types.ParseTimeBytes(brt)
		if err != nil {
			panic(fmt.Errorf("failed to parse formatted time %q: %w", brt, err))
		}
		brt2 := types.FormatTimeBytes(ti2)
		if !bytes.Equal(brt, brt2) {
			panic(fmt.Sprintf("Roundtrip failure, got\n%q\nwant\n%q", brt, brt2))
		}
	})
}
