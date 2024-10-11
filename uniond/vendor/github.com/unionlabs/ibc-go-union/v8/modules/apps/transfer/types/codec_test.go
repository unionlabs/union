package types_test

import (
	"strings"

	"github.com/cosmos/cosmos-sdk/codec/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer"
	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
)

// TestMustMarshalProtoJSON tests that the memo field is only emitted (marshalled) if it is populated
func (suite *TypesTestSuite) TestMustMarshalProtoJSON() {
	memo := "memo"
	packetData := types.NewFungibleTokenPacketData(sdk.DefaultBondDenom, "1", suite.chainA.SenderAccount.GetAddress().String(), suite.chainB.SenderAccount.GetAddress().String(), memo)

	bz := packetData.GetBytes()
	exists := strings.Contains(string(bz), memo)
	suite.Require().True(exists)

	packetData.Memo = ""

	bz = packetData.GetBytes()
	exists = strings.Contains(string(bz), memo)
	suite.Require().False(exists)
}

func (suite *TypesTestSuite) TestCodecTypeRegistration() {
	testCases := []struct {
		name    string
		typeURL string
		expPass bool
	}{
		{
			"success: MsgTransfer",
			sdk.MsgTypeURL(&types.MsgTransfer{}),
			true,
		},
		{
			"success: MsgUpdateParams",
			sdk.MsgTypeURL(&types.MsgUpdateParams{}),
			true,
		},
		{
			"success: TransferAuthorization",
			sdk.MsgTypeURL(&types.TransferAuthorization{}),
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
			encodingCfg := moduletestutil.MakeTestEncodingConfig(testutil.CodecOptions{}, transfer.AppModule{})
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
