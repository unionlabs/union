package feegrant_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
	"cosmossdk.io/x/feegrant"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/migrations/legacytx"
)

func TestAminoJSON(t *testing.T) {
	legacyAmino := codec.NewLegacyAmino()
	feegrant.RegisterLegacyAminoCodec(legacyAmino)
	legacytx.RegressionTestingAminoCodec = legacyAmino
	tx := legacytx.StdTx{}
	var msg sdk.Msg
	allowanceAny, err := codectypes.NewAnyWithValue(&feegrant.BasicAllowance{SpendLimit: sdk.NewCoins(sdk.NewCoin("foo", math.NewInt(100)))})
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
		string(legacytx.StdSignBytes("foo", 1, 1, 1, legacytx.StdFee{}, []sdk.Msg{msg}, "memo")),
	)

	msg = &feegrant.MsgRevokeAllowance{Granter: "cosmos1abc", Grantee: "cosmos1def"}
	tx.Msgs = []sdk.Msg{msg}
	require.Equal(t,
		`{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"type":"cosmos-sdk/MsgRevokeAllowance","value":{"grantee":"cosmos1def","granter":"cosmos1abc"}}],"sequence":"1","timeout_height":"1"}`,
		string(legacytx.StdSignBytes("foo", 1, 1, 1, legacytx.StdFee{}, []sdk.Msg{msg}, "memo")),
	)
}
