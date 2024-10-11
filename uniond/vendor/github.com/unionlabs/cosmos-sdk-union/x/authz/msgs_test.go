package authz_test

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/types/known/anypb"

	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/authz"
	banktypes "cosmossdk.io/x/bank/types"
	stakingtypes "cosmossdk.io/x/staking/types"
	txsigning "cosmossdk.io/x/tx/signing"
	"cosmossdk.io/x/tx/signing/aminojson"

	"github.com/cosmos/cosmos-sdk/codec"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/migrations/legacytx"
)

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
	err = m.SetAuthorization(&g)
	require.NoError(err)

	a, err = m.GetAuthorization()
	require.NoError(err)
	require.Equal(a, &g)
}

func TestAminoJSON(t *testing.T) {
	legacyAmino := codec.NewLegacyAmino()
	authz.RegisterLegacyAminoCodec(legacyAmino)
	banktypes.RegisterLegacyAminoCodec(legacyAmino)
	stakingtypes.RegisterLegacyAminoCodec(legacyAmino)
	legacytx.RegressionTestingAminoCodec = legacyAmino
	valAddressCodec := codectestutil.CodecOptions{}.GetValidatorCodec()
	aminoHandler := aminojson.NewSignModeHandler(aminojson.SignModeHandlerOptions{
		FileResolver: proto.HybridResolver,
	})

	tx := legacytx.StdTx{}
	blockTime := time.Date(1, 1, 1, 1, 1, 1, 1, time.UTC)
	expiresAt := blockTime.Add(time.Hour)
	msgSend := banktypes.MsgSend{FromAddress: "cosmos1ghi", ToAddress: "cosmos1jkl"}
	typeURL := sdk.MsgTypeURL(&msgSend)
	msgSendAny, err := cdctypes.NewAnyWithValue(&msgSend)
	require.NoError(t, err)
	grant, err := authz.NewGrant(blockTime, authz.NewGenericAuthorization(typeURL), &expiresAt)
	require.NoError(t, err)
	sendAuthz := banktypes.NewSendAuthorization(sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(1000))), nil, codectestutil.CodecOptions{}.GetValidatorCodec())
	sendGrant, err := authz.NewGrant(blockTime, sendAuthz, &expiresAt)
	require.NoError(t, err)
	valAddr, err := valAddressCodec.StringToBytes("cosmosvaloper1xcy3els9ua75kdm783c3qu0rfa2eples6eavqq")
	require.NoError(t, err)
	stakingAuth, err := stakingtypes.NewStakeAuthorization([]sdk.ValAddress{valAddr}, nil, stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE, &sdk.Coin{Denom: "stake", Amount: sdkmath.NewInt(1000)}, valAddressCodec)
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
	tests := []struct {
		msg sdk.Msg
		exp string
	}{
		{
			msg: &authz.MsgGrant{Granter: "cosmos1abc", Grantee: "cosmos1def", Grant: grant},
			exp: `{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"type":"cosmos-sdk/MsgGrant","value":{"grant":{"authorization":{"type":"cosmos-sdk/GenericAuthorization","value":{"msg":"/cosmos.bank.v1beta1.MsgSend"}},"expiration":"0001-01-01T02:01:01.000000001Z"},"grantee":"cosmos1def","granter":"cosmos1abc"}}],"sequence":"1","timeout_height":"1"}`,
		},
		{
			msg: &authz.MsgGrant{Granter: "cosmos1abc", Grantee: "cosmos1def", Grant: sendGrant},
			exp: `{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"type":"cosmos-sdk/MsgGrant","value":{"grant":{"authorization":{"type":"cosmos-sdk/SendAuthorization","value":{"spend_limit":[{"amount":"1000","denom":"stake"}]}},"expiration":"0001-01-01T02:01:01.000000001Z"},"grantee":"cosmos1def","granter":"cosmos1abc"}}],"sequence":"1","timeout_height":"1"}`,
		},
		{
			msg: &authz.MsgGrant{Granter: "cosmos1abc", Grantee: "cosmos1def", Grant: delegateGrant},
			exp: `{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"type":"cosmos-sdk/MsgGrant","value":{"grant":{"authorization":{"type":"cosmos-sdk/StakeAuthorization","value":{"Validators":{"type":"cosmos-sdk/StakeAuthorization/AllowList","value":{"allow_list":{"address":["cosmosvaloper1xcy3els9ua75kdm783c3qu0rfa2eples6eavqq"]}}},"authorization_type":1,"max_tokens":{"amount":"1000","denom":"stake"}}}},"grantee":"cosmos1def","granter":"cosmos1abc"}}],"sequence":"1","timeout_height":"1"}`,
		},
		{
			msg: &authz.MsgRevoke{Granter: "cosmos1abc", Grantee: "cosmos1def", MsgTypeUrl: typeURL},
			exp: `{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"type":"cosmos-sdk/MsgRevoke","value":{"grantee":"cosmos1def","granter":"cosmos1abc","msg_type_url":"/cosmos.bank.v1beta1.MsgSend"}}],"sequence":"1","timeout_height":"1"}`,
		},
		{
			msg: &authz.MsgExec{Grantee: "cosmos1def", Msgs: []*cdctypes.Any{msgSendAny}},
			exp: `{"account_number":"1","chain_id":"foo","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[{"type":"cosmos-sdk/MsgExec","value":{"grantee":"cosmos1def","msgs":[{"type":"cosmos-sdk/MsgSend","value":{"amount":[],"from_address":"cosmos1ghi","to_address":"cosmos1jkl"}}]}}],"sequence":"1","timeout_height":"1"}`,
		},
	}
	for i, tt := range tests {
		t.Run(fmt.Sprintf("case %d", i), func(t *testing.T) {
			tx.Msgs = []sdk.Msg{tt.msg}
			legacyJSON := string(legacytx.StdSignBytes("foo", 1, 1, 1, legacytx.StdFee{}, []sdk.Msg{tt.msg}, "memo"))
			require.Equal(t, tt.exp, legacyJSON)

			legacyAny, err := cdctypes.NewAnyWithValue(tt.msg)
			require.NoError(t, err)
			anyMsg := &anypb.Any{
				TypeUrl: legacyAny.TypeUrl,
				Value:   legacyAny.Value,
			}
			aminoJSON, err := aminoHandler.GetSignBytes(
				context.TODO(),
				txsigning.SignerData{
					Address:       "foo",
					ChainID:       "foo",
					AccountNumber: 1,
					Sequence:      1,
				},
				txsigning.TxData{
					Body: &txv1beta1.TxBody{
						Memo:          "memo",
						Messages:      []*anypb.Any{anyMsg},
						TimeoutHeight: 1,
					},
					AuthInfo: &txv1beta1.AuthInfo{
						Fee: &txv1beta1.Fee{},
					},
				},
			)
			require.NoError(t, err)
			require.Equal(t, tt.exp, string(aminoJSON))
		})
	}
}
