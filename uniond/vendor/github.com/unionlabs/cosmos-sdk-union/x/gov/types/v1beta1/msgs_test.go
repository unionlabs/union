package v1beta1

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var (
	coinsPos   = sdk.NewCoins(sdk.NewInt64Coin(sdk.DefaultBondDenom, 1000))
	coinsMulti = sdk.NewCoins(sdk.NewInt64Coin(sdk.DefaultBondDenom, 1000), sdk.NewInt64Coin("foo", 10000))
)

func init() {
	coinsMulti.Sort()
}

func TestMsgDepositGetSignBytes(t *testing.T) {
	addr, err := codectestutil.CodecOptions{}.GetAddressCodec().BytesToString(sdk.AccAddress("addr1"))
	require.NoError(t, err)
	msg := NewMsgDeposit(addr, 0, coinsPos)
	pc := codec.NewProtoCodec(types.NewInterfaceRegistry())
	res, err := pc.MarshalAminoJSON(msg)
	require.NoError(t, err)

	expected := `{"type":"cosmos-sdk/MsgDeposit","value":{"amount":[{"amount":"1000","denom":"stake"}],"depositor":"cosmos1v9jxgu33kfsgr5","proposal_id":"0"}}`
	require.Equal(t, expected, string(res))
}

// this tests that Amino JSON MsgSubmitProposal.GetSignBytes() still works with Content as Any using the ModuleCdc
func TestMsgSubmitProposal_GetSignBytes(t *testing.T) {
	addr, err := codectestutil.CodecOptions{}.GetAddressCodec().BytesToString(sdk.AccAddress{})
	require.NoError(t, err)
	msg, err := NewMsgSubmitProposal(NewTextProposal("test", "abcd"), sdk.NewCoins(), addr)
	require.NoError(t, err)
	pc := codec.NewProtoCodec(types.NewInterfaceRegistry())
	bz, err := pc.MarshalAminoJSON(msg)
	require.NoError(t, err)
	require.Equal(t,
		`{"type":"cosmos-sdk/MsgSubmitProposal","value":{"content":{"type":"cosmos-sdk/TextProposal","value":{"description":"abcd","title":"test"}},"initial_deposit":[]}}`,
		string(bz))
}
