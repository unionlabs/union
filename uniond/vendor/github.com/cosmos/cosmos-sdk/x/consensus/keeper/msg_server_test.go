package keeper_test

import (
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	tmtypes "github.com/cometbft/cometbft/types"
	"github.com/cosmos/cosmos-sdk/x/consensus/types"
)

func (s *KeeperTestSuite) TestUpdateParams() {
	defaultConsensusParams := tmtypes.DefaultConsensusParams().ToProto()
	testCases := []struct {
		name      string
		input     *types.MsgUpdateParams
		expErr    bool
		expErrMsg string
	}{
		{
			name: "valid params",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
			},
			expErr:    false,
			expErrMsg: "",
		},
		{
			name: "invalid  params",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     &tmproto.BlockParams{MaxGas: -10, MaxBytes: -10},
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
			},
			expErr:    true,
			expErrMsg: "block.MaxBytes must be greater than 0. Got -10",
		},
		{
			name: "invalid authority",
			input: &types.MsgUpdateParams{
				Authority: "invalid",
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
			},
			expErr:    true,
			expErrMsg: "invalid authority",
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			s.SetupTest()
			_, err := s.msgServer.UpdateParams(s.ctx, tc.input)
			if tc.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}
