package types

import (
	"testing"

	"github.com/cosmos/cosmos-sdk/codec"
)

func FuzzCoinUnmarshalJSON(f *testing.F) {
	if testing.Short() {
		f.Skip()
	}

	cdc := codec.NewLegacyAmino()
	f.Add(`{"denom":"atom","amount":"1000"}`)
	f.Add(`{"denom":"atom","amount":"-1000"}`)
	f.Add(`{"denom":"uatom","amount":"1000111111111111111111111"}`)
	f.Add(`{"denom":"mu","amount":"0"}`)

	f.Fuzz(func(t *testing.T, jsonBlob string) {
		var c Coin
		_ = cdc.UnmarshalJSON([]byte(jsonBlob), &c)
	})
}
