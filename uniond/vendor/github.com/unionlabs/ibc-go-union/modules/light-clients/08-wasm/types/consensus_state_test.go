package types_test

import (
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

func (suite *TypesTestSuite) TestConsensusStateValidateBasic() {
	testCases := []struct {
		name           string
		consensusState *types.ConsensusState
		expectPass     bool
	}{
		{
			"success",
			types.NewConsensusState([]byte("data")),
			true,
		},
		{
			"data is nil",
			types.NewConsensusState(nil),
			false,
		},
		{
			"data is empty",
			types.NewConsensusState([]byte{}),
			false,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			// check just to increase coverage
			suite.Require().Equal(types.Wasm, tc.consensusState.ClientType())

			err := tc.consensusState.ValidateBasic()
			if tc.expectPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}
