package types_test

import (
	"encoding/json"
	"testing"
	"time"

	"cosmossdk.io/math"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

func TestDelegationEqual(t *testing.T) {
	d1 := types.NewDelegation(sdk.AccAddress(valAddr1), valAddr2, math.LegacyNewDec(100))
	d2 := d1

	ok := d1.String() == d2.String()
	require.True(t, ok)

	d2.ValidatorAddress = valAddr3.String()
	d2.Shares = math.LegacyNewDec(200)

	ok = d1.String() == d2.String()
	require.False(t, ok)
}

func TestDelegationString(t *testing.T) {
	d := types.NewDelegation(sdk.AccAddress(valAddr1), valAddr2, math.LegacyNewDec(100))
	require.NotEmpty(t, d.String())
}

func TestUnbondingDelegationEqual(t *testing.T) {
	ubd1 := types.NewUnbondingDelegation(sdk.AccAddress(valAddr1), valAddr2, 0,
		time.Unix(0, 0), sdk.NewInt(0), 1)
	ubd2 := ubd1

	ok := ubd1.String() == ubd2.String()
	require.True(t, ok)

	ubd2.ValidatorAddress = valAddr3.String()

	ubd2.Entries[0].CompletionTime = time.Unix(20*20*2, 0)
	ok = (ubd1.String() == ubd2.String())
	require.False(t, ok)
}

func TestUnbondingDelegationString(t *testing.T) {
	ubd := types.NewUnbondingDelegation(sdk.AccAddress(valAddr1), valAddr2, 0,
		time.Unix(0, 0), sdk.NewInt(0), 1)

	require.NotEmpty(t, ubd.String())
}

func TestRedelegationEqual(t *testing.T) {
	r1 := types.NewRedelegation(sdk.AccAddress(valAddr1), valAddr2, valAddr3, 0,
		time.Unix(0, 0), sdk.NewInt(0),
		math.LegacyNewDec(0), 1)
	r2 := types.NewRedelegation(sdk.AccAddress(valAddr1), valAddr2, valAddr3, 0,
		time.Unix(0, 0), sdk.NewInt(0),
		math.LegacyNewDec(0), 2)
	require.False(t, r1.String() == r2.String())
	r2 = types.NewRedelegation(sdk.AccAddress(valAddr1), valAddr2, valAddr3, 0,
		time.Unix(0, 0), sdk.NewInt(0),
		math.LegacyNewDec(0), 1)
	require.True(t, r1.String() == r2.String())

	r2.Entries[0].SharesDst = math.LegacyNewDec(10)
	r2.Entries[0].CompletionTime = time.Unix(20*20*2, 0)
	require.False(t, r1.String() == r2.String())
}

func TestRedelegationString(t *testing.T) {
	r := types.NewRedelegation(sdk.AccAddress(valAddr1), valAddr2, valAddr3, 0,
		time.Unix(0, 0), sdk.NewInt(0),
		math.LegacyNewDec(10), 1)

	require.NotEmpty(t, r.String())
}

func TestDelegationResponses(t *testing.T) {
	cdc := codec.NewLegacyAmino()
	dr1 := types.NewDelegationResp(sdk.AccAddress(valAddr1), valAddr2, math.LegacyNewDec(5),
		sdk.NewCoin(sdk.DefaultBondDenom, sdk.NewInt(5)))
	dr2 := types.NewDelegationResp(sdk.AccAddress(valAddr1), valAddr3, math.LegacyNewDec(5),
		sdk.NewCoin(sdk.DefaultBondDenom, sdk.NewInt(5)))
	drs := types.DelegationResponses{dr1, dr2}

	bz1, err := json.Marshal(dr1)
	require.NoError(t, err)

	bz2, err := cdc.MarshalJSON(dr1)
	require.NoError(t, err)

	require.Equal(t, bz1, bz2)

	bz1, err = json.Marshal(drs)
	require.NoError(t, err)

	bz2, err = cdc.MarshalJSON(drs)
	require.NoError(t, err)

	require.Equal(t, bz1, bz2)

	var drs2 types.DelegationResponses
	require.NoError(t, cdc.UnmarshalJSON(bz2, &drs2))
	require.Equal(t, drs, drs2)
}

func TestRedelegationResponses(t *testing.T) {
	cdc := codec.NewLegacyAmino()
	entries := []types.RedelegationEntryResponse{
		types.NewRedelegationEntryResponse(0, time.Unix(0, 0), math.LegacyNewDec(5), sdk.NewInt(5), sdk.NewInt(5), 0),
		types.NewRedelegationEntryResponse(0, time.Unix(0, 0), math.LegacyNewDec(5), sdk.NewInt(5), sdk.NewInt(5), 0),
	}
	rdr1 := types.NewRedelegationResponse(sdk.AccAddress(valAddr1), valAddr2, valAddr3, entries)
	rdr2 := types.NewRedelegationResponse(sdk.AccAddress(valAddr2), valAddr1, valAddr3, entries)
	rdrs := types.RedelegationResponses{rdr1, rdr2}

	bz1, err := json.Marshal(rdr1)
	require.NoError(t, err)

	bz2, err := cdc.MarshalJSON(rdr1)
	require.NoError(t, err)

	require.Equal(t, bz1, bz2)

	bz1, err = json.Marshal(rdrs)
	require.NoError(t, err)

	bz2, err = cdc.MarshalJSON(rdrs)
	require.NoError(t, err)

	require.Equal(t, bz1, bz2)

	var rdrs2 types.RedelegationResponses
	require.NoError(t, cdc.UnmarshalJSON(bz2, &rdrs2))

	bz3, err := cdc.MarshalJSON(rdrs2)
	require.NoError(t, err)

	require.Equal(t, bz2, bz3)
}
