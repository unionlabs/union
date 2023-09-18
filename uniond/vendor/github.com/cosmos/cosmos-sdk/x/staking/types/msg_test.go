package types_test

import (
	"testing"
	"time"

	"cosmossdk.io/math"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

var (
	coinPos  = sdk.NewInt64Coin(sdk.DefaultBondDenom, 1000)
	coinZero = sdk.NewInt64Coin(sdk.DefaultBondDenom, 0)
)

func TestMsgDecode(t *testing.T) {
	registry := codectypes.NewInterfaceRegistry()
	cryptocodec.RegisterInterfaces(registry)
	types.RegisterInterfaces(registry)
	cdc := codec.NewProtoCodec(registry)

	// firstly we start testing the pubkey serialization

	pk1bz, err := cdc.MarshalInterface(pk1)
	require.NoError(t, err)
	var pkUnmarshaled cryptotypes.PubKey
	err = cdc.UnmarshalInterface(pk1bz, &pkUnmarshaled)
	require.NoError(t, err)
	require.True(t, pk1.Equals(pkUnmarshaled.(*ed25519.PubKey)))

	// now let's try to serialize the whole message

	commission1 := types.NewCommissionRates(math.LegacyZeroDec(), math.LegacyZeroDec(), math.LegacyZeroDec())
	msg, err := types.NewMsgCreateValidator(valAddr1, pk1, coinPos, types.Description{}, commission1, math.OneInt())
	require.NoError(t, err)
	msgSerialized, err := cdc.MarshalInterface(msg)
	require.NoError(t, err)

	var msgUnmarshaled sdk.Msg
	err = cdc.UnmarshalInterface(msgSerialized, &msgUnmarshaled)
	require.NoError(t, err)
	msg2, ok := msgUnmarshaled.(*types.MsgCreateValidator)
	require.True(t, ok)
	require.True(t, msg.Value.IsEqual(msg2.Value))
	require.True(t, msg.Pubkey.Equal(msg2.Pubkey))
}

// test ValidateBasic for MsgCreateValidator
func TestMsgCreateValidator(t *testing.T) {
	commission1 := types.NewCommissionRates(math.LegacyZeroDec(), math.LegacyZeroDec(), math.LegacyZeroDec())
	commission2 := types.NewCommissionRates(math.LegacyNewDec(5), math.LegacyNewDec(5), math.LegacyNewDec(5))

	tests := []struct {
		name, moniker, identity, website, securityContact, details string
		CommissionRates                                            types.CommissionRates
		minSelfDelegation                                          math.Int
		validatorAddr                                              sdk.ValAddress
		pubkey                                                     cryptotypes.PubKey
		bond                                                       sdk.Coin
		expectPass                                                 bool
	}{
		{"basic good", "a", "b", "c", "d", "e", commission1, math.OneInt(), valAddr1, pk1, coinPos, true},
		{"partial description", "", "", "c", "", "", commission1, math.OneInt(), valAddr1, pk1, coinPos, true},
		{"empty description", "", "", "", "", "", commission2, math.OneInt(), valAddr1, pk1, coinPos, false},
		{"empty address", "a", "b", "c", "d", "e", commission2, math.OneInt(), emptyAddr, pk1, coinPos, false},
		{"empty pubkey", "a", "b", "c", "d", "e", commission1, math.OneInt(), valAddr1, emptyPubkey, coinPos, false},
		{"empty bond", "a", "b", "c", "d", "e", commission2, math.OneInt(), valAddr1, pk1, coinZero, false},
		{"nil bond", "a", "b", "c", "d", "e", commission2, math.OneInt(), valAddr1, pk1, sdk.Coin{}, false},
		{"zero min self delegation", "a", "b", "c", "d", "e", commission1, math.ZeroInt(), valAddr1, pk1, coinPos, false},
		{"negative min self delegation", "a", "b", "c", "d", "e", commission1, sdk.NewInt(-1), valAddr1, pk1, coinPos, false},
		{"delegation less than min self delegation", "a", "b", "c", "d", "e", commission1, coinPos.Amount.Add(math.OneInt()), valAddr1, pk1, coinPos, false},
	}

	for _, tc := range tests {
		description := types.NewDescription(tc.moniker, tc.identity, tc.website, tc.securityContact, tc.details)
		msg, err := types.NewMsgCreateValidator(tc.validatorAddr, tc.pubkey, tc.bond, description, tc.CommissionRates, tc.minSelfDelegation)
		require.NoError(t, err)
		if tc.expectPass {
			require.Nil(t, msg.ValidateBasic(), "test: %v", tc.name)
		} else {
			require.NotNil(t, msg.ValidateBasic(), "test: %v", tc.name)
		}
	}
}

// test ValidateBasic for MsgEditValidator
func TestMsgEditValidator(t *testing.T) {
	tests := []struct {
		name, moniker, identity, website, securityContact, details string
		validatorAddr                                              sdk.ValAddress
		expectPass                                                 bool
		minSelfDelegation                                          math.Int
	}{
		{"basic good", "a", "b", "c", "d", "e", valAddr1, true, math.OneInt()},
		{"partial description", "", "", "c", "", "", valAddr1, true, math.OneInt()},
		{"empty description", "", "", "", "", "", valAddr1, false, math.OneInt()},
		{"empty address", "a", "b", "c", "d", "e", emptyAddr, false, math.OneInt()},
		{"nil int", "a", "b", "c", "d", "e", emptyAddr, false, math.Int{}},
	}

	for _, tc := range tests {
		description := types.NewDescription(tc.moniker, tc.identity, tc.website, tc.securityContact, tc.details)
		newRate := math.LegacyZeroDec()

		msg := types.NewMsgEditValidator(tc.validatorAddr, description, &newRate, &tc.minSelfDelegation)
		if tc.expectPass {
			require.Nil(t, msg.ValidateBasic(), "test: %v", tc.name)
		} else {
			require.NotNil(t, msg.ValidateBasic(), "test: %v", tc.name)
		}
	}
}

// test ValidateBasic for MsgDelegate
func TestMsgDelegate(t *testing.T) {
	tests := []struct {
		name          string
		delegatorAddr sdk.AccAddress
		validatorAddr sdk.ValAddress
		bond          sdk.Coin
		expectPass    bool
	}{
		{"basic good", sdk.AccAddress(valAddr1), valAddr2, coinPos, true},
		{"self bond", sdk.AccAddress(valAddr1), valAddr1, coinPos, true},
		{"empty delegator", sdk.AccAddress(emptyAddr), valAddr1, coinPos, false},
		{"empty validator", sdk.AccAddress(valAddr1), emptyAddr, coinPos, false},
		{"empty bond", sdk.AccAddress(valAddr1), valAddr2, coinZero, false},
		{"nil bold", sdk.AccAddress(valAddr1), valAddr2, sdk.Coin{}, false},
	}

	for _, tc := range tests {
		msg := types.NewMsgDelegate(tc.delegatorAddr, tc.validatorAddr, tc.bond)
		if tc.expectPass {
			require.Nil(t, msg.ValidateBasic(), "test: %v", tc.name)
		} else {
			require.NotNil(t, msg.ValidateBasic(), "test: %v", tc.name)
		}
	}
}

// test ValidateBasic for MsgUnbond
func TestMsgBeginRedelegate(t *testing.T) {
	tests := []struct {
		name             string
		delegatorAddr    sdk.AccAddress
		validatorSrcAddr sdk.ValAddress
		validatorDstAddr sdk.ValAddress
		amount           sdk.Coin
		expectPass       bool
	}{
		{"regular", sdk.AccAddress(valAddr1), valAddr2, valAddr3, sdk.NewInt64Coin(sdk.DefaultBondDenom, 1), true},
		{"zero amount", sdk.AccAddress(valAddr1), valAddr2, valAddr3, sdk.NewInt64Coin(sdk.DefaultBondDenom, 0), false},
		{"nil amount", sdk.AccAddress(valAddr1), valAddr2, valAddr3, sdk.Coin{}, false},
		{"empty delegator", sdk.AccAddress(emptyAddr), valAddr1, valAddr3, sdk.NewInt64Coin(sdk.DefaultBondDenom, 1), false},
		{"empty source validator", sdk.AccAddress(valAddr1), emptyAddr, valAddr3, sdk.NewInt64Coin(sdk.DefaultBondDenom, 1), false},
		{"empty destination validator", sdk.AccAddress(valAddr1), valAddr2, emptyAddr, sdk.NewInt64Coin(sdk.DefaultBondDenom, 1), false},
	}

	for _, tc := range tests {
		msg := types.NewMsgBeginRedelegate(tc.delegatorAddr, tc.validatorSrcAddr, tc.validatorDstAddr, tc.amount)
		if tc.expectPass {
			require.Nil(t, msg.ValidateBasic(), "test: %v", tc.name)
		} else {
			require.NotNil(t, msg.ValidateBasic(), "test: %v", tc.name)
		}
	}
}

// test ValidateBasic for MsgUnbond
func TestMsgUndelegate(t *testing.T) {
	tests := []struct {
		name          string
		delegatorAddr sdk.AccAddress
		validatorAddr sdk.ValAddress
		amount        sdk.Coin
		expectPass    bool
	}{
		{"regular", sdk.AccAddress(valAddr1), valAddr2, sdk.NewInt64Coin(sdk.DefaultBondDenom, 1), true},
		{"zero amount", sdk.AccAddress(valAddr1), valAddr2, sdk.NewInt64Coin(sdk.DefaultBondDenom, 0), false},
		{"nil amount", sdk.AccAddress(valAddr1), valAddr2, sdk.Coin{}, false},
		{"empty delegator", sdk.AccAddress(emptyAddr), valAddr1, sdk.NewInt64Coin(sdk.DefaultBondDenom, 1), false},
		{"empty validator", sdk.AccAddress(valAddr1), emptyAddr, sdk.NewInt64Coin(sdk.DefaultBondDenom, 1), false},
	}

	for _, tc := range tests {
		msg := types.NewMsgUndelegate(tc.delegatorAddr, tc.validatorAddr, tc.amount)
		if tc.expectPass {
			require.Nil(t, msg.ValidateBasic(), "test: %v", tc.name)
		} else {
			require.NotNil(t, msg.ValidateBasic(), "test: %v", tc.name)
		}
	}
}

func TestMsgUpdateParams(t *testing.T) {
	msg := types.MsgUpdateParams{
		Authority: authtypes.NewModuleAddress(govtypes.ModuleName).String(),
		Params:    types.DefaultParams(),
	}

	require.Equal(t, []sdk.AccAddress{authtypes.NewModuleAddress(govtypes.ModuleName)}, msg.GetSigners())
}

func TestMsgUpdateParamsValidateBasic(t *testing.T) {
	tests := []struct {
		name            string
		msgUpdateParams types.MsgUpdateParams
		expFail         bool
		expError        string
	}{
		{
			"valid msg",
			types.MsgUpdateParams{
				Authority: authtypes.NewModuleAddress(govtypes.ModuleName).String(),
				Params:    types.DefaultParams(),
			},
			false,
			"",
		},
		{
			"negative unbounding time",
			types.MsgUpdateParams{
				Authority: authtypes.NewModuleAddress(govtypes.ModuleName).String(),
				Params: types.Params{
					UnbondingTime:     time.Hour * 24 * 7 * 3 * -1,
					MaxEntries:        types.DefaultMaxEntries,
					MaxValidators:     types.DefaultMaxValidators,
					HistoricalEntries: types.DefaultHistoricalEntries,
					MinCommissionRate: types.DefaultMinCommissionRate,
					BondDenom:         "denom",
				},
			},
			true,
			"unbonding time must be positive:",
		},
		{
			"cero value max validator",
			types.MsgUpdateParams{
				Authority: authtypes.NewModuleAddress(govtypes.ModuleName).String(),
				Params: types.Params{
					UnbondingTime:     time.Hour * 24 * 7 * 3,
					MaxEntries:        types.DefaultMaxEntries,
					MaxValidators:     0,
					HistoricalEntries: types.DefaultHistoricalEntries,
					MinCommissionRate: types.DefaultMinCommissionRate,
					BondDenom:         "denom",
				},
			},
			true,
			"max validators must be positive:",
		},
		{
			"cero value max validator",
			types.MsgUpdateParams{
				Authority: authtypes.NewModuleAddress(govtypes.ModuleName).String(),
				Params: types.Params{
					UnbondingTime:     time.Hour * 24 * 7 * 3,
					MaxEntries:        0,
					MaxValidators:     types.DefaultMaxValidators,
					HistoricalEntries: types.DefaultHistoricalEntries,
					MinCommissionRate: types.DefaultMinCommissionRate,
					BondDenom:         "denom",
				},
			},
			true,
			"max entries must be positive:",
		},
		{
			"negative commission rate",
			types.MsgUpdateParams{
				Authority: authtypes.NewModuleAddress(govtypes.ModuleName).String(),
				Params: types.Params{
					UnbondingTime:     time.Hour * 24 * 7 * 3,
					MaxEntries:        types.DefaultMaxEntries,
					MaxValidators:     types.DefaultMaxValidators,
					HistoricalEntries: types.DefaultHistoricalEntries,
					MinCommissionRate: math.LegacyNewDec(-1),
					BondDenom:         "denom",
				},
			},
			true,
			"minimum commission rate cannot be negative:",
		},
		{
			"negative commission rate",
			types.MsgUpdateParams{
				Authority: authtypes.NewModuleAddress(govtypes.ModuleName).String(),
				Params: types.Params{
					UnbondingTime:     time.Hour * 24 * 7 * 3,
					MaxEntries:        types.DefaultMaxEntries,
					MaxValidators:     types.DefaultMaxValidators,
					HistoricalEntries: types.DefaultHistoricalEntries,
					MinCommissionRate: math.LegacyNewDec(2),
					BondDenom:         "denom",
				},
			},
			true,
			"minimum commission rate cannot be greater than 100",
		},
		{
			"blank bonddenom",
			types.MsgUpdateParams{
				Authority: authtypes.NewModuleAddress(govtypes.ModuleName).String(),
				Params: types.Params{
					UnbondingTime:     time.Hour * 24 * 7 * 3,
					MaxEntries:        types.DefaultMaxEntries,
					MaxValidators:     types.DefaultMaxValidators,
					HistoricalEntries: types.DefaultHistoricalEntries,
					MinCommissionRate: types.DefaultMinCommissionRate,
					BondDenom:         "",
				},
			},
			true,
			"bond denom cannot be blank",
		},
		{
			"Invalid authority",
			types.MsgUpdateParams{
				Authority: "invalid",
				Params: types.Params{
					UnbondingTime:     time.Hour * 24 * 7 * 3,
					MaxEntries:        types.DefaultMaxEntries,
					MaxValidators:     types.DefaultMaxValidators,
					HistoricalEntries: types.DefaultHistoricalEntries,
					MinCommissionRate: types.DefaultMinCommissionRate,
					BondDenom:         "denom",
				},
			},
			true,
			"invalid authority address",
		},
	}

	for _, tc := range tests {
		err := tc.msgUpdateParams.ValidateBasic()
		if tc.expFail {
			require.Error(t, err)
			require.Contains(t, err.Error(), tc.expError)
		} else {
			require.NoError(t, err)
		}
	}
}
