package types_test

import (
	"testing"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/vesting/types"
)

func TestPeriodsString(t *testing.T) {
	tests := []struct {
		name    string
		periods types.Periods
		want    string
	}{
		{
			"empty slice",
			nil,
			"Vesting Periods:",
		},
		{
			"1 period",
			types.Periods{
				{Length: int64(12 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin("feeatom", 500), sdk.NewInt64Coin("statom", 50)}},
			},
			"Vesting Periods:\n\t\t" + `length:43200 amount:<denom:"feeatom" amount:"500" > amount:<denom:"statom" amount:"50" >`,
		},
		{
			"many",
			types.Periods{
				{Length: int64(12 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}},
				{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
				{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 100), sdk.NewInt64Coin(stakeDenom, 15)}},
			},
			"Vesting Periods:\n\t\t" + `length:43200 amount:<denom:"fee" amount:"500" > amount:<denom:"stake" amount:"50" > , ` +
				`length:21600 amount:<denom:"fee" amount:"250" > amount:<denom:"stake" amount:"25" > , ` +
				`length:21600 amount:<denom:"fee" amount:"100" > amount:<denom:"stake" amount:"15" >`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := tt.periods.String()
			if got != tt.want {
				t.Fatalf("Mismatch in values:\n\tGot:  %q\n\tWant: %q", got, tt.want)
			}
		})
	}
}
