//go:build gofuzz || go1.18

package tests

import (
	"testing"

	"github.com/cosmos/cosmos-sdk/crypto/types"
)

func FuzzCryptoTypesCompactbitarrayMarshalUnmarshal(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		cba, err := types.CompactUnmarshal(data)
		if err != nil {
			return
		}
		if cba == nil && string(data) != "null" {
			panic("Inconsistency, no error, yet BitArray is nil")
		}
		if cba.SetIndex(-1, true) {
			panic("Set negative index success")
		}
		if cba.GetIndex(-1) {
			panic("Get negative index success")
		}
	})
}
