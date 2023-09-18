//go:build gofuzz || go1.18

package tests

import (
	"testing"

	"github.com/cosmos/cosmos-sdk/types"
)

func FuzzTypesVerifyAddressFormat(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		types.VerifyAddressFormat(data)
	})
}
