//go:build gofuzz || go1.18

package tests

import (
	"testing"

	"github.com/cosmos/cosmos-sdk/crypto/hd"
)

func FuzzCryptoHDNewParamsFromPath(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		_, _ = hd.NewParamsFromPath(string(data))
	})
}
