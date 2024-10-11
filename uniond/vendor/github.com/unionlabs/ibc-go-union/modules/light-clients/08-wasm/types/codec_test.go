package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"

	wasm "github.com/cosmos/ibc-go/modules/light-clients/08-wasm"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

func TestCodecTypeRegistration(t *testing.T) {
	testCases := []struct {
		name    string
		typeURL string
		expPass bool
	}{
		{
			"success: ClientState",
			sdk.MsgTypeURL(&types.ClientState{}),
			true,
		},
		{
			"success: ConsensusState",
			sdk.MsgTypeURL(&types.ConsensusState{}),
			true,
		},
		{
			"success: ClientMessage",
			sdk.MsgTypeURL(&types.ClientMessage{}),
			true,
		},
		{
			"success: MsgStoreCode",
			sdk.MsgTypeURL(&types.MsgStoreCode{}),
			true,
		},
		{
			"success: MsgMigrateContract",
			sdk.MsgTypeURL(&types.MsgMigrateContract{}),
			true,
		},
		{
			"success: MsgRemoveChecksum",
			sdk.MsgTypeURL(&types.MsgRemoveChecksum{}),
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
			encodingCfg := moduletestutil.MakeTestEncodingConfig(wasm.AppModuleBasic{})
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
