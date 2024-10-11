package types

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
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
		{"empty math.LegacyDec", args{math.LegacyDec{}}, true},
		{"negative", args{math.LegacyNewDec(-1)}, true},
		{"one dec", args{math.LegacyNewDec(1)}, false},
		{"two dec", args{math.LegacyNewDec(2)}, true},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			require.Equal(t, tt.wantErr, validateCommunityTax(tt.args.i) != nil)
		})
	}
}
