package types

import (
	"testing"

	"cosmossdk.io/math"
	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func Test_validateAuxFuncs(t *testing.T) {
	type args struct {
		i interface{}
	}
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{"wrong type", args{10.5}, true},
		{"empty sdk.Dec", args{sdk.Dec{}}, true},
		{"negative", args{math.LegacyNewDec(-1)}, true},
		{"one dec", args{math.LegacyNewDec(1)}, false},
		{"two dec", args{math.LegacyNewDec(2)}, true},
	}
	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			require.Equal(t, tt.wantErr, validateCommunityTax(tt.args.i) != nil)
		})
	}
}
