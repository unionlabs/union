package v1_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	banktypes "cosmossdk.io/x/bank/types"
	v1 "cosmossdk.io/x/gov/types/v1"

	"github.com/cosmos/cosmos-sdk/codec"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var (
	coinsPos   = sdk.NewCoins(sdk.NewInt64Coin(sdk.DefaultBondDenom, 1000))
	coinsMulti = sdk.NewCoins(sdk.NewInt64Coin(sdk.DefaultBondDenom, 1000), sdk.NewInt64Coin("foo", 10000))
	addrs      = []sdk.AccAddress{
		sdk.AccAddress("test1"),
		sdk.AccAddress("test2"),
	}
	addrStrs = []string{
		"cosmos1w3jhxap3gempvr",
		"cosmos1w3jhxapjx2whzu",
	}
)

func init() {
	coinsMulti.Sort()
}

func TestMsgDepositGetSignBytes(t *testing.T) {
	addr := sdk.AccAddress("addr1")
	addrStr, err := codectestutil.CodecOptions{}.GetAddressCodec().BytesToString(addr)
	require.NoError(t, err)
	msg := v1.NewMsgDeposit(addrStr, 0, coinsPos)
	pc := codec.NewProtoCodec(types.NewInterfaceRegistry())
	res, err := pc.MarshalAminoJSON(msg)
	require.NoError(t, err)
	expected := `{"type":"cosmos-sdk/v1/MsgDeposit","value":{"amount":[{"amount":"1000","denom":"stake"}],"depositor":"cosmos1v9jxgu33kfsgr5","proposal_id":"0"}}`
	require.Equal(t, expected, string(res))
}

// this tests that Amino JSON MsgSubmitProposal.GetSignBytes() still works with Content as Any using the ModuleCdc
func TestMsgSubmitProposal_GetSignBytes(t *testing.T) {
	pc := codec.NewProtoCodec(types.NewInterfaceRegistry())
	addr0Str, err := codectestutil.CodecOptions{}.GetAddressCodec().BytesToString(addrs[0])
	require.NoError(t, err)
	testcases := []struct {
		name         string
		proposal     []sdk.Msg
		title        string
		summary      string
		proposalType v1.ProposalType
		expSignBz    string
	}{
		{
			"MsgVote",
			[]sdk.Msg{v1.NewMsgVote(addr0Str, 1, v1.OptionYes, "")},
			"gov/MsgVote",
			"Proposal for a governance vote msg",
			v1.ProposalType_PROPOSAL_TYPE_STANDARD,
			`{"type":"cosmos-sdk/v1/MsgSubmitProposal","value":{"initial_deposit":[],"messages":[{"type":"cosmos-sdk/v1/MsgVote","value":{"option":1,"proposal_id":"1","voter":"cosmos1w3jhxap3gempvr"}}],"proposal_type":1,"summary":"Proposal for a governance vote msg","title":"gov/MsgVote"}}`,
		},
		{
			"MsgSend",
			[]sdk.Msg{banktypes.NewMsgSend(addrStrs[0], addrStrs[0], sdk.NewCoins())},
			"bank/MsgSend",
			"Proposal for a bank msg send",
			v1.ProposalType_PROPOSAL_TYPE_STANDARD,
			fmt.Sprintf(`{"type":"cosmos-sdk/v1/MsgSubmitProposal","value":{"initial_deposit":[],"messages":[{"type":"cosmos-sdk/MsgSend","value":{"amount":[],"from_address":"%s","to_address":"%s"}}],"proposal_type":1,"summary":"Proposal for a bank msg send","title":"bank/MsgSend"}}`, addrStrs[0], addrStrs[0]),
		},
	}

	for _, tc := range testcases {
		t.Run(tc.name, func(t *testing.T) {
			msg, err := v1.NewMsgSubmitProposal(tc.proposal, sdk.NewCoins(), "", "", tc.title, tc.summary, tc.proposalType)
			require.NoError(t, err)
			bz, err := pc.MarshalAminoJSON(msg)
			require.NoError(t, err)
			require.Equal(t, tc.expSignBz, string(bz))
		})
	}
}
