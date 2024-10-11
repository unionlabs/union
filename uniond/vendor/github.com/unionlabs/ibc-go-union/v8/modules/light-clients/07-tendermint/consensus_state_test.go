package tendermint_test

import (
	"time"

	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
)

func (suite *TendermintTestSuite) TestConsensusStateValidateBasic() {
	testCases := []struct {
		msg            string
		consensusState *ibctm.ConsensusState
		expectPass     bool
	}{
		{
			"success",
			&ibctm.ConsensusState{
				Timestamp:          suite.now,
				Root:               commitmenttypes.NewMerkleRoot([]byte("app_hash")),
				NextValidatorsHash: suite.valsHash,
			},
			true,
		},
		{
			"success with sentinel",
			&ibctm.ConsensusState{
				Timestamp:          suite.now,
				Root:               commitmenttypes.NewMerkleRoot([]byte(ibctm.SentinelRoot)),
				NextValidatorsHash: suite.valsHash,
			},
			true,
		},
		{
			"root is nil",
			&ibctm.ConsensusState{
				Timestamp:          suite.now,
				Root:               commitmenttypes.MerkleRoot{},
				NextValidatorsHash: suite.valsHash,
			},
			false,
		},
		{
			"root is empty",
			&ibctm.ConsensusState{
				Timestamp:          suite.now,
				Root:               commitmenttypes.MerkleRoot{},
				NextValidatorsHash: suite.valsHash,
			},
			false,
		},
		{
			"nextvalshash is invalid",
			&ibctm.ConsensusState{
				Timestamp:          suite.now,
				Root:               commitmenttypes.NewMerkleRoot([]byte("app_hash")),
				NextValidatorsHash: []byte("hi"),
			},
			false,
		},

		{
			"timestamp is zero",
			&ibctm.ConsensusState{
				Timestamp:          time.Time{},
				Root:               commitmenttypes.NewMerkleRoot([]byte("app_hash")),
				NextValidatorsHash: suite.valsHash,
			},
			false,
		},
	}

	for i, tc := range testCases {
		tc := tc

		suite.Run(tc.msg, func() {
			// check just to increase coverage
			suite.Require().Equal(exported.Tendermint, tc.consensusState.ClientType())
			suite.Require().Equal(tc.consensusState.GetRoot(), tc.consensusState.Root)

			err := tc.consensusState.ValidateBasic()
			if tc.expectPass {
				suite.Require().NoError(err, "valid test case %d failed: %s", i, tc.msg)
			} else {
				suite.Require().Error(err, "invalid test case %d passed: %s", i, tc.msg)
			}
		})
	}
}
