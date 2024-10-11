package types_test

import (
	"encoding/json"
	"testing"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
	"cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/codec"
	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestDelegationEqual(t *testing.T) {
	codecOpts := codectestutil.CodecOptions{}
	addr, err := codecOpts.GetAddressCodec().BytesToString(valAddr1)
	require.NoError(t, err)
	valAddr, err := codecOpts.GetValidatorCodec().BytesToString(valAddr2)
	require.NoError(t, err)
	d1 := types.NewDelegation(addr, valAddr, math.LegacyNewDec(100))
	d2 := d1

	ok := d1.String() == d2.String()
	require.True(t, ok)

	vAddr3, err := codecOpts.GetValidatorCodec().BytesToString(valAddr2)
	require.NoError(t, err)
	d2.ValidatorAddress = vAddr3
	d2.Shares = math.LegacyNewDec(200)

	ok = d1.String() == d2.String()
	require.False(t, ok)
}

func TestDelegationString(t *testing.T) {
	codecOpts := codectestutil.CodecOptions{}
	addr, err := codecOpts.GetAddressCodec().BytesToString(valAddr1)
	require.NoError(t, err)
	valAddr, err := codecOpts.GetValidatorCodec().BytesToString(valAddr2)
	require.NoError(t, err)
	d := types.NewDelegation(addr, valAddr, math.LegacyNewDec(100))
	require.NotEmpty(t, d.String())
}

func TestUnbondingDelegationEqual(t *testing.T) {
	ubd1 := types.NewUnbondingDelegation(sdk.AccAddress(valAddr1), valAddr2, 0,
		time.Unix(0, 0), math.NewInt(0), 1, addresscodec.NewBech32Codec("cosmosvaloper"), addresscodec.NewBech32Codec("cosmos"))
	ubd2 := ubd1

	ok := ubd1.String() == ubd2.String()
	require.True(t, ok)

	vAddr3, err := codectestutil.CodecOptions{}.GetValidatorCodec().BytesToString(valAddr3)
	require.NoError(t, err)
	ubd2.ValidatorAddress = vAddr3

	ubd2.Entries[0].CompletionTime = time.Unix(20*20*2, 0)
	ok = (ubd1.String() == ubd2.String())
	require.False(t, ok)
}

func TestUnbondingDelegationString(t *testing.T) {
	ubd := types.NewUnbondingDelegation(sdk.AccAddress(valAddr1), valAddr2, 0,
		time.Unix(0, 0), math.NewInt(0), 1, addresscodec.NewBech32Codec("cosmosvaloper"), addresscodec.NewBech32Codec("cosmos"))

	require.NotEmpty(t, ubd.String())
}

func TestRedelegationEqual(t *testing.T) {
	r1 := types.NewRedelegation(sdk.AccAddress(valAddr1), valAddr2, valAddr3, 0,
		time.Unix(0, 0), math.NewInt(0),
		math.LegacyNewDec(0), 1, addresscodec.NewBech32Codec("cosmosvaloper"), addresscodec.NewBech32Codec("cosmos"))
	r2 := types.NewRedelegation(sdk.AccAddress(valAddr1), valAddr2, valAddr3, 0,
		time.Unix(0, 0), math.NewInt(0),
		math.LegacyNewDec(0), 1, addresscodec.NewBech32Codec("cosmosvaloper"), addresscodec.NewBech32Codec("cosmos"))
	require.True(t, proto.Equal(&r1, &r2))

	r2.Entries[0].SharesDst = math.LegacyNewDec(10)
	r2.Entries[0].CompletionTime = time.Unix(20*20*2, 0)
	require.False(t, proto.Equal(&r1, &r2))
}

func TestRedelegationString(t *testing.T) {
	r := types.NewRedelegation(sdk.AccAddress(valAddr1), valAddr2, valAddr3, 0,
		time.Unix(0, 0), math.NewInt(0),
		math.LegacyNewDec(10), 1, addresscodec.NewBech32Codec("cosmosvaloper"), addresscodec.NewBech32Codec("cosmos"))

	require.NotEmpty(t, r.String())
}

func TestDelegationResponses(t *testing.T) {
	cdc := codec.NewLegacyAmino()
	codecOpts := codectestutil.CodecOptions{}
	addr, err := codecOpts.GetAddressCodec().BytesToString(valAddr1)
	require.NoError(t, err)
	valAddr, err := codecOpts.GetValidatorCodec().BytesToString(valAddr2)
	require.NoError(t, err)

	dr1 := types.NewDelegationResp(addr, valAddr, math.LegacyNewDec(5),
		sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(5)))
	dr2 := types.NewDelegationResp(addr, valAddr, math.LegacyNewDec(5),
		sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(5)))
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
	addressCdc := codectestutil.CodecOptions{}.GetAddressCodec()
	addr1, err := addressCdc.BytesToString(valAddr1)
	require.NoError(t, err)
	addr2, err := addressCdc.BytesToString(valAddr2)
	require.NoError(t, err)
	validatorCdc := codectestutil.CodecOptions{}.GetValidatorCodec()
	vAddr1, err := validatorCdc.BytesToString(valAddr1)
	require.NoError(t, err)
	vAddr2, err := validatorCdc.BytesToString(valAddr2)
	require.NoError(t, err)
	vAddr3, err := validatorCdc.BytesToString(valAddr3)
	require.NoError(t, err)

	entries := []types.RedelegationEntryResponse{
		types.NewRedelegationEntryResponse(0, time.Unix(0, 0), math.LegacyNewDec(5), math.NewInt(5), math.NewInt(5), 0),
		types.NewRedelegationEntryResponse(0, time.Unix(0, 0), math.LegacyNewDec(5), math.NewInt(5), math.NewInt(5), 0),
	}
	rdr1 := types.NewRedelegationResponse(addr1, vAddr2, vAddr3, entries)
	rdr2 := types.NewRedelegationResponse(addr2, vAddr1, vAddr3, entries)
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
