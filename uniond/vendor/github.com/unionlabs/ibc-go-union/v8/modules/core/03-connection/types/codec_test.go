package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"

	ibc "github.com/cosmos/ibc-go/v8/modules/core"
	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
)

func TestCodecTypeRegistration(t *testing.T) {
	testCases := []struct {
		name    string
		typeURL string
		expPass bool
	}{
		{
			"success: ConnectionEnd",
			sdk.MsgTypeURL(&types.ConnectionEnd{}),
			true,
		},
		{
			"success: Counterparty",
			sdk.MsgTypeURL(&types.Counterparty{}),
			true,
		},
		{
			"success: MsgConnectionOpenInit",
			sdk.MsgTypeURL(&types.MsgConnectionOpenInit{}),
			true,
		},
		{
			"success: MsgConnectionOpenTry",
			sdk.MsgTypeURL(&types.MsgConnectionOpenTry{}),
			true,
		},
		{
			"success: MsgConnectionOpenAck",
			sdk.MsgTypeURL(&types.MsgConnectionOpenAck{}),
			true,
		},
		{
			"success: MsgConnectionOpenConfirm",
			sdk.MsgTypeURL(&types.MsgConnectionOpenConfirm{}),
			true,
		},
		{
			"success: MsgUpdateParams",
			sdk.MsgTypeURL(&types.MsgUpdateParams{}),
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
			encodingCfg := moduletestutil.MakeTestEncodingConfig(testutil.CodecOptions{}, ibc.AppModule{})
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
