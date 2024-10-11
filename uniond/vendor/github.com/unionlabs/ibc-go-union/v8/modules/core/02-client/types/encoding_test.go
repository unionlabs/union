package types_test

import (
	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
)

func (suite *TypesTestSuite) TestMarshalHeader() {
	cdc := suite.chainA.App.AppCodec()
	h := &ibctm.Header{
		TrustedHeight: types.NewHeight(4, 100),
	}

	// marshal header
	bz, err := types.MarshalClientMessage(cdc, h)
	suite.Require().NoError(err)

	// unmarshal header
	newHeader, err := types.UnmarshalClientMessage(cdc, bz)
	suite.Require().NoError(err)

	suite.Require().Equal(h, newHeader)

	// use invalid bytes
	invalidHeader, err := types.UnmarshalClientMessage(cdc, []byte("invalid bytes"))
	suite.Require().Error(err)
	suite.Require().Nil(invalidHeader)
}
