package solomachine_test

import (
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	solomachine "github.com/cosmos/ibc-go/v8/modules/light-clients/06-solomachine"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *SoloMachineTestSuite) TestMisbehaviour() {
	misbehaviour := suite.solomachine.CreateMisbehaviour()

	suite.Require().Equal(exported.Solomachine, misbehaviour.ClientType())
}

func (suite *SoloMachineTestSuite) TestMisbehaviourValidateBasic() {
	// test singlesig and multisig public keys
	for _, sm := range []*ibctesting.Solomachine{suite.solomachine, suite.solomachineMulti} {

		testCases := []struct {
			name                 string
			malleateMisbehaviour func(misbehaviour *solomachine.Misbehaviour)
			expPass              bool
		}{
			{
				"valid misbehaviour",
				func(*solomachine.Misbehaviour) {},
				true,
			},
			{
				"sequence is zero",
				func(misbehaviour *solomachine.Misbehaviour) {
					misbehaviour.Sequence = 0
				},
				false,
			},
			{
				"signature one sig is empty",
				func(misbehaviour *solomachine.Misbehaviour) {
					misbehaviour.SignatureOne.Signature = []byte{}
				},
				false,
			},
			{
				"signature two sig is empty",
				func(misbehaviour *solomachine.Misbehaviour) {
					misbehaviour.SignatureTwo.Signature = []byte{}
				},
				false,
			},
			{
				"signature one data is empty",
				func(misbehaviour *solomachine.Misbehaviour) {
					misbehaviour.SignatureOne.Data = nil
				},
				false,
			},
			{
				"signature two data is empty",
				func(misbehaviour *solomachine.Misbehaviour) {
					misbehaviour.SignatureTwo.Data = []byte{}
				},
				false,
			},
			{
				"signatures are identical",
				func(misbehaviour *solomachine.Misbehaviour) {
					misbehaviour.SignatureTwo.Signature = misbehaviour.SignatureOne.Signature
				},
				false,
			},
			{
				"data signed is identical but path differs",
				func(misbehaviour *solomachine.Misbehaviour) {
					misbehaviour.SignatureTwo.Data = misbehaviour.SignatureOne.Data
				},
				true,
			},
			{
				"data signed and path are identical",
				func(misbehaviour *solomachine.Misbehaviour) {
					misbehaviour.SignatureTwo.Path = misbehaviour.SignatureOne.Path
					misbehaviour.SignatureTwo.Data = misbehaviour.SignatureOne.Data
				},
				false,
			},
			{
				"data path for SignatureOne is unspecified",
				func(misbehaviour *solomachine.Misbehaviour) {
					misbehaviour.SignatureOne.Path = []byte{}
				}, false,
			},
			{
				"data path for SignatureTwo is unspecified",
				func(misbehaviour *solomachine.Misbehaviour) {
					misbehaviour.SignatureTwo.Path = []byte{}
				}, false,
			},
			{
				"timestamp for SignatureOne is zero",
				func(misbehaviour *solomachine.Misbehaviour) {
					misbehaviour.SignatureOne.Timestamp = 0
				}, false,
			},
			{
				"timestamp for SignatureTwo is zero",
				func(misbehaviour *solomachine.Misbehaviour) {
					misbehaviour.SignatureTwo.Timestamp = 0
				}, false,
			},
		}

		for _, tc := range testCases {
			tc := tc

			suite.Run(tc.name, func() {
				misbehaviour := sm.CreateMisbehaviour()
				tc.malleateMisbehaviour(misbehaviour)

				err := misbehaviour.ValidateBasic()

				if tc.expPass {
					suite.Require().NoError(err)
				} else {
					suite.Require().Error(err)
				}
			})
		}
	}
}
