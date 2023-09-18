//go:build gofuzz || go1.18

package tests

import (
	"strings"
	"testing"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func FuzzTypesDecSetString(f *testing.F) {
	f.Fuzz(func(t *testing.T, b []byte) {
		dec, err := sdk.NewDecFromStr(string(b))
		if err != nil {
			return
		}
		if !dec.IsZero() {
			return
		}
		switch s := string(b); {
		case strings.TrimLeft(s, "-+0") == "":
		case strings.TrimRight(strings.TrimLeft(s, "-+0"), "0") == ".":
		default:
			panic("no error yet is zero")
		}
	})
}
