package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"

	fee "github.com/cosmos/ibc-go/v8/modules/apps/29-fee"
	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
)

func TestCodecTypeRegistration(t *testing.T) {
	testCases := []struct {
		name    string
		typeURL string
		expPass bool
	}{
		{
			"success: MsgPayPacketFee",
			sdk.MsgTypeURL(&types.MsgPayPacketFee{}),
			true,
		},
		{
			"success: MsgPayPacketFeeAsync",
			sdk.MsgTypeURL(&types.MsgPayPacketFeeAsync{}),
			true,
		},
		{
			"success: MsgRegisterPayee",
			sdk.MsgTypeURL(&types.MsgRegisterPayee{}),
			true,
		},
		{
			"success: MsgRegisterCounterpartyPayee",
			sdk.MsgTypeURL(&types.MsgRegisterCounterpartyPayee{}),
			true,
		},
		{
			"type not registered on codec",
			"ibc.invalid.MsgTypeURL",
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		t.Run(tc.name, func(t *testing.T) {
			encodingCfg := moduletestutil.MakeTestEncodingConfig(testutil.CodecOptions{}, fee.AppModule{})
			msg, err := encodingCfg.Codec.InterfaceRegistry().Resolve(tc.typeURL)

			if tc.expPass {
				require.NotNil(t, msg)
				require.NoError(t, err)
			} else {
				require.Nil(t, msg)
				require.Error(t, err)
			}
		})
	}
}
