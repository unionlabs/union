package feegrant_test

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/migrations/legacytx"
	"github.com/cosmos/cosmos-sdk/x/feegrant"
)

func TestMsgGrantAllowance(t *testing.T) {
	cdc := codec.NewProtoCodec(codectypes.NewInterfaceRegistry())
	addr, _ := sdk.AccAddressFromBech32("cosmos1aeuqja06474dfrj7uqsvukm6rael982kk89mqr")
	addr2, _ := sdk.AccAddressFromBech32("cosmos1nph3cfzk6trsmfxkeu943nvach5qw4vwstnvkl")
	atom := sdk.NewCoins(sdk.NewInt64Coin("atom", 555))
	threeHours := time.Now().Add(3 * time.Hour)
	basic := &feegrant.BasicAllowance{
		SpendLimit: atom,
		Expiration: &threeHours,
	}

	cases := map[string]struct {
		grantee sdk.AccAddress
		granter sdk.AccAddress
		grant   *feegrant.BasicAllowance
		valid   bool
	}{
		"valid": {
			grantee: addr,
			granter: addr2,
			grant:   basic,
			valid:   true,
		},
		"no grantee": {
			granter: addr2,
			grantee: sdk.AccAddress{},
			grant:   basic,
			valid:   false,
		},
		"no granter": {
			granter: sdk.AccAddress{},
			grantee: addr,
			grant:   basic,
			valid:   false,
		},
		"grantee == granter": {
			grantee: addr,
			granter: addr,
			grant:   basic,
			valid:   false,
		},
	}

	for _, tc := range cases {
		msg, err := feegrant.NewMsgGrantAllowance(tc.grant, tc.granter, tc.grantee)
		require.NoError(t, err)
		err = msg.ValidateBasic()

		if tc.valid {
			require.NoError(t, err)

			addrSlice := msg.GetSigners()
			require.True(t, tc.granter.Equals(addrSlice[0]))

			allowance, err := msg.GetFeeAllowanceI()
			require.NoError(t, err)
			require.Equal(t, tc.grant, allowance)

			err = msg.UnpackInterfaces(cdc)
			require.NoError(t, err)
		} else {
			require.Error(t, err)
		}
	}
}

func TestMsgRevokeAllowance(t *testing.T) {
	addr, _ := sdk.AccAddressFromBech32("cosmos1aeuqja06474dfrj7uqsvukm6rael982kk89mqr")
	addr2, _ := sdk.AccAddressFromBech32("cosmos1nph3cfzk6trsmfxkeu943nvach5qw4vwstnvkl")
	atom := sdk.NewCoins(sdk.NewInt64Coin("atom", 555))
	threeHours := time.Now().Add(3 * time.Hour)

	basic := &feegrant.BasicAllowance{
		SpendLimit: atom,
		Expiration: &threeHours,
	}
	cases := map[string]struct {
		grantee sdk.AccAddress
		granter sdk.AccAddress
		grant   *feegrant.BasicAllowance
		valid   bool
	}{
		"valid": {
			grantee: addr,
			granter: addr2,
			grant:   basic,
			valid:   true,
		},
		"no grantee": {
			granter: addr2,
			grantee: sdk.AccAddress{},
			grant:   basic,
			valid:   false,
		},
		"no granter": {
			granter: sdk.AccAddress{},
			grantee: addr,
			grant:   basic,
			valid:   false,
		},
		"grantee == granter": {
			grantee: addr,
			granter: addr,
			grant:   basic,
			valid:   false,
		},
	}

	for _, tc := range cases {
		msg := feegrant.NewMsgRevokeAllowance(tc.granter, tc.grantee)
		err := msg.ValidateBasic()
		if tc.valid {
			require.NoError(t, err)
			addrSlice := msg.GetSigners()
			require.True(t, tc.granter.Equals(addrSlice[0]))
		} else {
			require.Error(t, err)
		}
	}
}

func TestAminoJSON(t *testing.T) {
	tx := legacytx.StdTx{}
	var msg legacytx.LegacyMsg
	allowanceAny, err := codectypes.NewAnyWithValue(&feegrant.BasicAllowance{SpendLimit: sdk.NewCoins(sdk.NewCoin("foo", sdk.NewInt(100)))})
	require.NoError(t, err)

	// Amino JSON encoding has changed in feegrant since v0.46.
	// Before, it was outputting something like:
	// `{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"allowance":{"spend_limit":[{"amount":"100","denom":"foo"}]},"grantee":"cosmos1def","granter":"cosmos1abc"}],"sequence":"1","timeout_height":"1"}`
	//
	// This was a bug. Now, it's as below, See how there's `type` & `value` fields.
	// ref: https://github.com/cosmos/cosmos-sdk/issues/11190
	// ref: https://github.com/cosmos/cosmjs/issues/1026
	msg = &feegrant.MsgGrantAllowance{Granter: "cosmos1abc", Grantee: "cosmos1def", Allowance: allowanceAny}
	tx.Msgs = []sdk.Msg{msg}
	require.Equal(t,
		`{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"type":"cosmos-sdk/MsgGrantAllowance","value":{"allowance":{"type":"cosmos-sdk/BasicAllowance","value":{"spend_limit":[{"amount":"100","denom":"foo"}]}},"grantee":"cosmos1def","granter":"cosmos1abc"}}],"sequence":"1","timeout_height":"1"}`,
		string(legacytx.StdSignBytes("foo", 1, 1, 1, legacytx.StdFee{}, []sdk.Msg{msg}, "memo", nil)),
	)

	msg = &feegrant.MsgRevokeAllowance{Granter: "cosmos1abc", Grantee: "cosmos1def"}
	tx.Msgs = []sdk.Msg{msg}
	require.Equal(t,
		`{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"type":"cosmos-sdk/MsgRevokeAllowance","value":{"grantee":"cosmos1def","granter":"cosmos1abc"}}],"sequence":"1","timeout_height":"1"}`,
		string(legacytx.StdSignBytes("foo", 1, 1, 1, legacytx.StdFee{}, []sdk.Msg{msg}, "memo", nil)),
	)
}
