package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func TestMsgUpdateParamsValidateBasic(t *testing.T) {
	testCases := []struct {
		name    string
		msg     *types.MsgUpdateParams
		expPass bool
	}{
		{
			"success: valid signer address",
			types.NewMsgUpdateParams(sdk.AccAddress(ibctesting.TestAccAddress).String(), types.DefaultParams()),
			true,
		},
		{
			"failure: invalid signer address",
			types.NewMsgUpdateParams("signer", types.DefaultParams()),
			false,
		},
		{
			"failure: invalid allowed message",
			types.NewMsgUpdateParams("signer", types.Params{
				AllowMessages: []string{""},
			}),
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		err := tc.msg.ValidateBasic()
		if tc.expPass {
			require.NoError(t, err)
		} else {
			require.Error(t, err)
		}
	}
}

func TestMsgUpdateParamsGetSigners(t *testing.T) {
	testCases := []struct {
		name    string
		address sdk.AccAddress
		expPass bool
	}{
		{"success: valid address", sdk.AccAddress(ibctesting.TestAccAddress), true},
		{"failure: nil address", nil, false},
	}

	for _, tc := range testCases {
		tc := tc

		msg := types.NewMsgUpdateParams(tc.address.String(), types.DefaultParams())
		if tc.expPass {
			require.Equal(t, []sdk.AccAddress{tc.address}, msg.GetSigners())
		} else {
			require.Panics(t, func() {
				msg.GetSigners()
			})
		}
	}
}

func TestMsgModuleQuerySafeValidateBasic(t *testing.T) {
	queryRequest := &types.QueryRequest{
		Path: "/cosmos.bank.v1beta1.Query/Balance",
		Data: []byte{},
	}

	testCases := []struct {
		name   string
		msg    *types.MsgModuleQuerySafe
		expErr error
	}{
		{
			"success: valid signer address",
			types.NewMsgModuleQuerySafe(sdk.AccAddress(ibctesting.TestAccAddress).String(), []*types.QueryRequest{queryRequest}),
			nil,
		},
		{
			"failure: invalid signer address",
			types.NewMsgModuleQuerySafe("signer", []*types.QueryRequest{queryRequest}),
			ibcerrors.ErrInvalidAddress,
		},
		{
			"failure: empty query requests",
			types.NewMsgModuleQuerySafe(sdk.AccAddress(ibctesting.TestAccAddress).String(), []*types.QueryRequest{}),
			ibcerrors.ErrInvalidRequest,
		},
	}

	for _, tc := range testCases {
		tc := tc

		err := tc.msg.ValidateBasic()

		expPass := tc.expErr == nil
		if expPass {
			require.NoError(t, err)
		} else {
			require.Error(t, err)
			require.ErrorIs(t, err, tc.expErr)
		}
	}
}

func TestMsgModuleQuerySafeGetSigners(t *testing.T) {
	testCases := []struct {
		name    string
		address sdk.AccAddress
		expPass bool
	}{
		{"success: valid address", sdk.AccAddress(ibctesting.TestAccAddress), true},
		{"failure: nil address", nil, false},
	}

	for _, tc := range testCases {
		tc := tc

		msg := types.NewMsgModuleQuerySafe(tc.address.String(), []*types.QueryRequest{})
		if tc.expPass {
			require.Equal(t, []sdk.AccAddress{tc.address}, msg.GetSigners())
		} else {
			require.Panics(t, func() {
				msg.GetSigners()
			})
		}
	}
}
