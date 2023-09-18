package keys

import (
	"testing"

	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestParseKey(t *testing.T) {
	bech32str := "cosmos104ytdpvrx9284zd50v9ep8c6j7pua7dkk0x3ek"
	hexstr := "EB5AE9872103497EC092EF901027049E4F39200C60040D3562CD7F104A39F62E6E5A39A818F4"

	config := sdk.NewConfig()

	tests := []struct {
		name    string
		args    []string
		wantErr bool
	}{
		{"empty input", []string{""}, true},
		{"invalid input", []string{"invalid"}, true},
		{"bech32", []string{bech32str}, false},
		{"hex", []string{hexstr}, false},
	}
	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			require.Equal(t, tt.wantErr, doParseKey(ParseKeyStringCommand(), config, tt.args) != nil)
		})
	}
}
