package authz_test

import (
	"testing"
	"time"

	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"

	"github.com/stretchr/testify/require"

	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/migrations/legacytx"
	"github.com/cosmos/cosmos-sdk/x/authz"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
)

var (
	coinsPos = sdk.NewCoins(sdk.NewInt64Coin("steak", 100))
	granter  = sdk.AccAddress("_______granter______")
	grantee  = sdk.AccAddress("_______grantee______")
)

func TestMsgExecAuthorized(t *testing.T) {
	tests := []struct {
		title      string
		grantee    sdk.AccAddress
		msgs       []sdk.Msg
		expectPass bool
	}{
		{"nil grantee address", nil, []sdk.Msg{}, false},
		{"zero-messages test: should fail", grantee, []sdk.Msg{}, false},
		{"invalid nested msg", grantee, []sdk.Msg{
			&banktypes.MsgSend{
				Amount:      sdk.NewCoins(sdk.NewInt64Coin("steak", 2)),
				FromAddress: "invalid_from_address",
				ToAddress:   grantee.String(),
			},
		}, false},
		{"valid test: msg type", grantee, []sdk.Msg{
			&banktypes.MsgSend{
				Amount:      sdk.NewCoins(sdk.NewInt64Coin("steak", 2)),
				FromAddress: granter.String(),
				ToAddress:   grantee.String(),
			},
		}, true},
	}
	for i, tc := range tests {
		msg := authz.NewMsgExec(tc.grantee, tc.msgs)
		if tc.expectPass {
			require.NoError(t, msg.ValidateBasic(), "test: %v", i)
		} else {
			require.Error(t, msg.ValidateBasic(), "test: %v", i)
		}
	}
}

func TestMsgRevokeAuthorization(t *testing.T) {
	tests := []struct {
		title            string
		granter, grantee sdk.AccAddress
		msgType          string
		expectPass       bool
	}{
		{"nil Granter address", nil, grantee, "hello", false},
		{"nil Grantee address", granter, nil, "hello", false},
		{"nil Granter and Grantee address", nil, nil, "hello", false},
		{"valid test case", granter, grantee, "hello", true},
	}
	for i, tc := range tests {
		msg := authz.NewMsgRevoke(tc.granter, tc.grantee, tc.msgType)
		if tc.expectPass {
			require.NoError(t, msg.ValidateBasic(), "test: %v", i)
		} else {
			require.Error(t, msg.ValidateBasic(), "test: %v", i)
		}
	}
}

// add time interval to a time object and returns a pointer
func addDatePtr(t *time.Time, months, days int) *time.Time {
	t2 := t.AddDate(0, months, days)
	return &t2
}

func TestMsgGrantAuthorization(t *testing.T) {
	now := time.Now()
	tests := []struct {
		name             string
		granter, grantee sdk.AccAddress
		authorization    authz.Authorization
		expiration       *time.Time
		expectErr        bool
		valBasic         bool
	}{
		{
			"nil granter address",
			nil, grantee, &banktypes.SendAuthorization{SpendLimit: coinsPos}, &now, false, false,
		},
		{
			"nil grantee address",
			granter, nil, &banktypes.SendAuthorization{SpendLimit: coinsPos}, &now, false, false,
		},
		{
			"nil granter and grantee address",
			nil, nil, &banktypes.SendAuthorization{SpendLimit: coinsPos}, &now, false, false,
		},
		{
			"nil authorization should fail",
			granter, grantee, nil, &now, true, false,
		},
		{
			"valid test case",
			granter, grantee, &banktypes.SendAuthorization{SpendLimit: coinsPos}, addDatePtr(&now, 1, 0), false, true,
		},
		{
			"valid test case with nil expire time",
			granter, grantee, &banktypes.SendAuthorization{SpendLimit: coinsPos}, nil, false, true,
		},
		// we don't access the block time / nor time.Now, so we don't know if it's in the past at this level.
		{
			"past expire time should not fail",
			granter, grantee, &banktypes.SendAuthorization{SpendLimit: coinsPos}, addDatePtr(&now, 0, -1), false, true,
		},
	}
	for _, tc := range tests {
		msg, err := authz.NewMsgGrant(
			tc.granter, tc.grantee, tc.authorization, tc.expiration,
		)
		if !tc.expectErr {
			require.NoError(t, err, "test: %v", tc.name)
		} else {
			require.Error(t, err, "test: %v", tc.name)
			continue
		}
		if tc.valBasic {
			require.NoError(t, msg.ValidateBasic(), "test: %v", tc.name)
		} else {
			require.Error(t, msg.ValidateBasic(), "test: %v", tc.name)
		}
	}
}

func TestMsgGrantGetAuthorization(t *testing.T) {
	require := require.New(t)

	m := authz.MsgGrant{}
	require.Nil(m.GetAuthorization())

	g := authz.GenericAuthorization{Msg: "some_type"}
	var err error
	m.Grant.Authorization, err = cdctypes.NewAnyWithValue(&g)
	require.NoError(err)

	a, err := m.GetAuthorization()
	require.NoError(err)
	require.Equal(a, &g)

	g = authz.GenericAuthorization{Msg: "some_type2"}
	m.SetAuthorization(&g)
	a, err = m.GetAuthorization()
	require.NoError(err)
	require.Equal(a, &g)
}

func TestAminoJSON(t *testing.T) {
	tx := legacytx.StdTx{}
	var msg legacytx.LegacyMsg
	blockTime := time.Date(1, 1, 1, 1, 1, 1, 1, time.UTC)
	expiresAt := blockTime.Add(time.Hour)
	msgSend := banktypes.MsgSend{FromAddress: "cosmos1ghi", ToAddress: "cosmos1jkl"}
	typeURL := sdk.MsgTypeURL(&msgSend)
	msgSendAny, err := cdctypes.NewAnyWithValue(&msgSend)
	require.NoError(t, err)
	grant, err := authz.NewGrant(blockTime, authz.NewGenericAuthorization(typeURL), &expiresAt)
	require.NoError(t, err)
	sendAuthz := banktypes.NewSendAuthorization(sdk.NewCoins(sdk.NewCoin("stake", sdk.NewInt(1000))), nil)
	sendGrant, err := authz.NewGrant(blockTime, sendAuthz, &expiresAt)
	require.NoError(t, err)
	valAddr, err := sdk.ValAddressFromBech32("cosmosvaloper1xcy3els9ua75kdm783c3qu0rfa2eples6eavqq")
	require.NoError(t, err)
	stakingAuth, err := stakingtypes.NewStakeAuthorization([]sdk.ValAddress{valAddr}, nil, stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE, &sdk.Coin{Denom: "stake", Amount: sdk.NewInt(1000)})
	require.NoError(t, err)
	delegateGrant, err := authz.NewGrant(blockTime, stakingAuth, nil)
	require.NoError(t, err)

	// Amino JSON encoding has changed in authz since v0.46.
	// Before, it was outputting something like:
	// `{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"grant":{"authorization":{"msg":"/cosmos.bank.v1beta1.MsgSend"},"expiration":"0001-01-01T02:01:01.000000001Z"},"grantee":"cosmos1def","granter":"cosmos1abc"}],"sequence":"1","timeout_height":"1"}`
	//
	// This was a bug. Now, it's as below, See how there's `type` & `value` fields.
	// ref: https://github.com/cosmos/cosmos-sdk/issues/11190
	// ref: https://github.com/cosmos/cosmjs/issues/1026
	msg = &authz.MsgGrant{Granter: "cosmos1abc", Grantee: "cosmos1def", Grant: grant}
	tx.Msgs = []sdk.Msg{msg}
	require.Equal(t,
		`{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"type":"cosmos-sdk/MsgGrant","value":{"grant":{"authorization":{"type":"cosmos-sdk/GenericAuthorization","value":{"msg":"/cosmos.bank.v1beta1.MsgSend"}},"expiration":"0001-01-01T02:01:01.000000001Z"},"grantee":"cosmos1def","granter":"cosmos1abc"}}],"sequence":"1","timeout_height":"1"}`,
		string(legacytx.StdSignBytes("foo", 1, 1, 1, legacytx.StdFee{}, []sdk.Msg{msg}, "memo", nil)),
	)

	msg = &authz.MsgGrant{Granter: "cosmos1abc", Grantee: "cosmos1def", Grant: sendGrant}
	tx.Msgs = []sdk.Msg{msg}
	require.Equal(t,
		`{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"type":"cosmos-sdk/MsgGrant","value":{"grant":{"authorization":{"type":"cosmos-sdk/SendAuthorization","value":{"spend_limit":[{"amount":"1000","denom":"stake"}]}},"expiration":"0001-01-01T02:01:01.000000001Z"},"grantee":"cosmos1def","granter":"cosmos1abc"}}],"sequence":"1","timeout_height":"1"}`,
		string(legacytx.StdSignBytes("foo", 1, 1, 1, legacytx.StdFee{}, []sdk.Msg{msg}, "memo", nil)),
	)

	msg = &authz.MsgGrant{Granter: "cosmos1abc", Grantee: "cosmos1def", Grant: delegateGrant}
	tx.Msgs = []sdk.Msg{msg}
	require.Equal(t,
		`{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"type":"cosmos-sdk/MsgGrant","value":{"grant":{"authorization":{"type":"cosmos-sdk/StakeAuthorization","value":{"Validators":{"type":"cosmos-sdk/StakeAuthorization/AllowList","value":{"allow_list":{"address":["cosmosvaloper1xcy3els9ua75kdm783c3qu0rfa2eples6eavqq"]}}},"authorization_type":1,"max_tokens":{"amount":"1000","denom":"stake"}}}},"grantee":"cosmos1def","granter":"cosmos1abc"}}],"sequence":"1","timeout_height":"1"}`,
		string(legacytx.StdSignBytes("foo", 1, 1, 1, legacytx.StdFee{}, []sdk.Msg{msg}, "memo", nil)),
	)

	msg = &authz.MsgRevoke{Granter: "cosmos1abc", Grantee: "cosmos1def", MsgTypeUrl: typeURL}
	tx.Msgs = []sdk.Msg{msg}
	require.Equal(t,
		`{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"type":"cosmos-sdk/MsgRevoke","value":{"grantee":"cosmos1def","granter":"cosmos1abc","msg_type_url":"/cosmos.bank.v1beta1.MsgSend"}}],"sequence":"1","timeout_height":"1"}`,
		string(legacytx.StdSignBytes("foo", 1, 1, 1, legacytx.StdFee{}, []sdk.Msg{msg}, "memo", nil)),
	)

	msg = &authz.MsgExec{Grantee: "cosmos1def", Msgs: []*cdctypes.Any{msgSendAny}}
	tx.Msgs = []sdk.Msg{msg}
	require.Equal(t,
		`{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"type":"cosmos-sdk/MsgExec","value":{"grantee":"cosmos1def","msgs":[{"type":"cosmos-sdk/MsgSend","value":{"amount":[],"from_address":"cosmos1ghi","to_address":"cosmos1jkl"}}]}}],"sequence":"1","timeout_height":"1"}`,
		string(legacytx.StdSignBytes("foo", 1, 1, 1, legacytx.StdFee{}, []sdk.Msg{msg}, "memo", nil)),
	)
}
