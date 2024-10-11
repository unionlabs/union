package tendermint_test

import (
	"time"

	cmtprotocrypto "github.com/cometbft/cometbft/api/cometbft/crypto/v1"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
)

func (suite *TendermintTestSuite) TestGetHeight() {
	header := suite.chainA.LastHeader
	suite.Require().NotEqual(uint64(0), header.GetHeight())
}

func (suite *TendermintTestSuite) TestGetTime() {
	header := suite.chainA.LastHeader
	suite.Require().NotEqual(time.Time{}, header.GetTime())
}

func (suite *TendermintTestSuite) TestHeaderValidateBasic() {
	var header *ibctm.Header
	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{"valid header", func() {}, true},
		{"header is nil", func() {
			header.Header = nil
		}, false},
		{"signed header is nil", func() {
			header.SignedHeader = nil
		}, false},
		{"SignedHeaderFromProto failed", func() {
			header.SignedHeader.Commit.Height = -1
		}, false},
		{"signed header failed tendermint ValidateBasic", func() {
			header = suite.chainA.LastHeader
			header.SignedHeader.Commit = nil
		}, false},
		{"trusted height is equal to header height", func() {
			header.TrustedHeight = header.GetHeight().(clienttypes.Height)
		}, false},
		{"validator set nil", func() {
			header.ValidatorSet = nil
		}, false},
		{"ValidatorSetFromProto failed", func() {
			header.ValidatorSet.Validators[0].PubKey = &cmtprotocrypto.PublicKey{}
		}, false},
		{"header validator hash does not equal hash of validator set", func() {
			// use chainB's randomly generated validator set
			header.ValidatorSet = suite.chainB.LastHeader.ValidatorSet
		}, false},
	}

	suite.Require().Equal(exported.Tendermint, suite.header.ClientType())

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest()

			header = suite.chainA.LastHeader // must be explicitly changed in malleate

			tc.malleate()

			err := header.ValidateBasic()

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}
