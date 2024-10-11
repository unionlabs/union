package types

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
)

func TestValidate(t *testing.T) {
	params := DefaultParams()
	err := params.Validate()
	require.NoError(t, err)

	params2 := NewParams(
		params.MintDenom,
		params.InflationRateChange,
		params.InflationMax,
		params.InflationMin,
		params.GoalBonded,
		params.BlocksPerYear,
		params.MaxSupply,
	)
	err = params2.Validate()
	require.NoError(t, err)
	require.Equal(t, params, params2)

	params.MintDenom = ""
	err = params.Validate()
	require.Error(t, err)

	params.MintDenom = "asd/$%!@#"
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.InflationRateChange = math.LegacyNewDec(123)
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.InflationRateChange = math.LegacyNewDec(-123)
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.InflationRateChange = math.LegacyDec{}
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.InflationMax = math.LegacyNewDec(123)
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.InflationMax = math.LegacyNewDec(-123)
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.InflationMax = math.LegacyDec{}
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.InflationMin = math.LegacyNewDec(123)
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.InflationMin = math.LegacyNewDec(-123)
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.InflationMin = math.LegacyDec{}
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.GoalBonded = math.LegacyNewDec(123)
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.GoalBonded = math.LegacyNewDec(-123)
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.GoalBonded = math.LegacyDec{}
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.BlocksPerYear = 0
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.MaxSupply = math.NewInt(-1)
	err = params.Validate()
	require.Error(t, err)

	params = DefaultParams()
	params.InflationMax = math.LegacyNewDecWithPrec(1, 2)
	params.InflationMin = math.LegacyNewDecWithPrec(2, 2)
	err = params.Validate()
	require.Error(t, err)
}

func Test_validateInflationFields(t *testing.T) {
	fns := []func(dec math.LegacyDec) error{
		validateInflationRateChange,
		validateInflationMax,
		validateInflationMin,
		validateGoalBonded,
	}
	tests := []struct {
		name    string
		v       math.LegacyDec
		wantErr bool
	}{
		{
			name: "valid",
			v:    math.LegacyNewDecWithPrec(12, 2),
		},
		{
			name:    "nil",
			v:       math.LegacyDec{},
			wantErr: true,
		},
		{
			name:    "negative",
			v:       math.LegacyNewDec(-1),
			wantErr: true,
		},
		{
			name:    "greater than one",
			v:       math.LegacyOneDec().Add(math.LegacyOneDec()),
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			for _, fn := range fns {
				if err := fn(tt.v); (err != nil) != tt.wantErr {
					t.Errorf("validateInflationRateChange() error = %v, wantErr %v", err, tt.wantErr)
				}
			}
		})
	}
}
