//go:build gofuzz || go1.18

package tests

import (
	"testing"

	"github.com/cosmos/cosmos-sdk/types"
)

func FuzzTypesParseDecCoin(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		_, _ = types.ParseDecCoin(string(data))
	})
}
