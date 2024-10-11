package solomachine_test

import (
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	solomachine "github.com/cosmos/ibc-go/v8/modules/light-clients/06-solomachine"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *SoloMachineTestSuite) TestConsensusState() {
	consensusState := suite.solomachine.ConsensusState()

	suite.Require().Equal(exported.Solomachine, consensusState.ClientType())
	suite.Require().Equal(suite.solomachine.Time, consensusState.GetTimestamp())
}

func (suite *SoloMachineTestSuite) TestConsensusStateValidateBasic() {
	// test singlesig and multisig public keys
	for _, sm := range []*ibctesting.Solomachine{suite.solomachine, suite.solomachineMulti} {

		testCases := []struct {
			name           string
			consensusState *solomachine.ConsensusState
			expPass        bool
		}{
			{
				"valid consensus state",
				sm.ConsensusState(),
				true,
			},
			{
				"timestamp is zero",
				&solomachine.ConsensusState{
					PublicKey:   sm.ConsensusState().PublicKey,
					Timestamp:   0,
					Diversifier: sm.Diversifier,
				},
				false,
			},
			{
				"diversifier is blank",
				&solomachine.ConsensusState{
					PublicKey:   sm.ConsensusState().PublicKey,
					Timestamp:   sm.Time,
					Diversifier: " ",
				},
				false,
			},
			{
				"pubkey is nil",
				&solomachine.ConsensusState{
					Timestamp:   sm.Time,
					Diversifier: sm.Diversifier,
					PublicKey:   nil,
				},
				false,
			},
		}

		for _, tc := range testCases {
			tc := tc

			suite.Run(tc.name, func() {
				err := tc.consensusState.ValidateBasic()

				if tc.expPass {
					suite.Require().NoError(err)
				} else {
					suite.Require().Error(err)
				}
			})
		}
	}
}
