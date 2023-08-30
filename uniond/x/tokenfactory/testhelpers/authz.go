package testhelpers

import (
	"encoding/json"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/authz"
)

var (
	Amino          = codec.NewLegacyAmino()
	AuthzModuleCdc = codec.NewAminoCodec(Amino)
)

func init() {
	cryptocodec.RegisterCrypto(Amino)
	codec.RegisterEvidences(Amino)
	sdk.RegisterLegacyAminoCodec(Amino)
}

func TestMessageAuthzSerialization(t *testing.T, msg sdk.Msg) {
	someDate := time.Date(1, 1, 1, 1, 1, 1, 1, time.UTC)
	const (
		mockGranter string = "cosmos1abc"
		mockGrantee string = "cosmos1xyz"
	)

	var (
		mockMsgGrant  authz.MsgGrant
		mockMsgRevoke authz.MsgRevoke
		mockMsgExec   authz.MsgExec
	)

	// Authz: Grant Msg
	typeURL := sdk.MsgTypeURL(msg)
	later := someDate.Add(time.Hour)
	grant, err := authz.NewGrant(someDate, authz.NewGenericAuthorization(typeURL), &later)
	require.NoError(t, err)

	msgGrant := authz.MsgGrant{Granter: mockGranter, Grantee: mockGrantee, Grant: grant}
	msgGrantBytes := json.RawMessage(sdk.MustSortJSON(AuthzModuleCdc.MustMarshalJSON(&msgGrant)))
	err = AuthzModuleCdc.UnmarshalJSON(msgGrantBytes, &mockMsgGrant)
	require.NoError(t, err)

	// Authz: Revoke Msg
	msgRevoke := authz.MsgRevoke{Granter: mockGranter, Grantee: mockGrantee, MsgTypeUrl: typeURL}
	msgRevokeByte := json.RawMessage(sdk.MustSortJSON(AuthzModuleCdc.MustMarshalJSON(&msgRevoke)))
	err = AuthzModuleCdc.UnmarshalJSON(msgRevokeByte, &mockMsgRevoke)
	require.NoError(t, err)

	// Authz: Exec Msg
	msgAny, err := cdctypes.NewAnyWithValue(msg)
	require.NoError(t, err)
	msgExec := authz.MsgExec{Grantee: mockGrantee, Msgs: []*cdctypes.Any{msgAny}}
	execMsgByte := json.RawMessage(sdk.MustSortJSON(AuthzModuleCdc.MustMarshalJSON(&msgExec)))
	err = AuthzModuleCdc.UnmarshalJSON(execMsgByte, &mockMsgExec)
	require.NoError(t, err)
	require.Equal(t, msgExec.Msgs[0].Value, mockMsgExec.Msgs[0].Value)
}
