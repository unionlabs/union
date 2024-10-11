package types_test

import (
	"github.com/cosmos/cosmos-sdk/codec/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"

	ibc "github.com/cosmos/ibc-go/v8/modules/core"
	"github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
)

func (suite *MerkleTestSuite) TestCodecTypeRegistration() {
	testCases := []struct {
		name    string
		typeURL string
		expPass bool
	}{
		{
			"success: MerkleRoot",
			sdk.MsgTypeURL(&types.MerkleRoot{}),
			true,
		},
		{
			"success: MerklePrefix",
			sdk.MsgTypeURL(&types.MerklePrefix{}),
			true,
		},
		{
			"success: MerklePath",
			sdk.MsgTypeURL(&types.MerklePath{}),
			true,
		},
		{
			"success: MerkleProof",
			sdk.MsgTypeURL(&types.MerkleProof{}),
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

		suite.Run(tc.name, func() {
			encodingCfg := moduletestutil.MakeTestEncodingConfig(testutil.CodecOptions{}, ibc.AppModule{})
			msg, err := encodingCfg.Codec.InterfaceRegistry().Resolve(tc.typeURL)

			if tc.expPass {
				suite.Require().NotNil(msg)
				suite.Require().NoError(err)
			} else {
				suite.Require().Nil(msg)
				suite.Require().Error(err)
			}
		})
	}
}
