package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/upgrade/types"
)

var authority = sdk.AccAddress("authority")

func TestMsgSoftwareUpgrade(t *testing.T) {
	testCases := []struct {
		name   string
		msg    *types.MsgSoftwareUpgrade
		expErr bool
		errMsg string
	}{
		{
			"invalid authority address",
			&types.MsgSoftwareUpgrade{
				Authority: "authority",
				Plan: types.Plan{
					Name:   "all-good",
					Height: 123450000,
				},
			},
			true,
			"authority: decoding bech32 failed",
		},
		{
			"invalid plan",
			&types.MsgSoftwareUpgrade{
				Authority: authority.String(),
				Plan: types.Plan{
					Height: 123450000,
				},
			},
			true,
			"plan",
		},
		{
			"all good",
			&types.MsgSoftwareUpgrade{
				Authority: authority.String(),
				Plan: types.Plan{
					Name:   "all-good",
					Height: 123450000,
				},
			},
			false,
			"",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			err := tc.msg.ValidateBasic()
			if tc.expErr {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.errMsg)
			} else {
				require.NoError(t, err)
				require.Equal(t, tc.msg.Type(), sdk.MsgTypeURL(&types.MsgSoftwareUpgrade{}))
			}
		})
	}
}

func TestMsgCancelUpgrade(t *testing.T) {
	testCases := []struct {
		name   string
		msg    *types.MsgCancelUpgrade
		expErr bool
		errMsg string
	}{
		{
			"invalid authority address",
			&types.MsgCancelUpgrade{
				Authority: "authority",
			},
			true,
			"authority: decoding bech32 failed",
		},
		{
			"all good",
			&types.MsgCancelUpgrade{
				Authority: authority.String(),
			},
			false,
			"",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			err := tc.msg.ValidateBasic()
			if tc.expErr {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.errMsg)
			} else {
				require.NoError(t, err)
				require.Equal(t, tc.msg.Type(), sdk.MsgTypeURL(&types.MsgCancelUpgrade{}))
			}
		})
	}
}
