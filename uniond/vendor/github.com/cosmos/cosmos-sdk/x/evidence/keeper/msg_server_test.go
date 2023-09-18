package keeper_test

import (
	"time"

	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/evidence/types"
)

func (s *KeeperTestSuite) TestSubmitEvidence() {
	pk := ed25519.GenPrivKey()

	e := &types.Equivocation{
		Height:           1,
		Power:            100,
		Time:             time.Now().UTC(),
		ConsensusAddress: sdk.ConsAddress(pk.PubKey().Address().Bytes()).String(),
	}

	validEvidence, err := types.NewMsgSubmitEvidence(sdk.AccAddress(valAddresses[0]), e)
	s.Require().NoError(err)

	e2 := &types.Equivocation{
		Height:           0,
		Power:            100,
		Time:             time.Now().UTC(),
		ConsensusAddress: sdk.ConsAddress(pk.PubKey().Address().Bytes()).String(),
	}

	invalidEvidence, err := types.NewMsgSubmitEvidence(sdk.AccAddress(valAddresses[0]), e2)
	s.Require().NoError(err)

	testCases := []struct {
		name      string
		req       *types.MsgSubmitEvidence
		expErr    bool
		expErrMsg string
	}{
		{
			name:      "invalid evidence with height 0",
			req:       invalidEvidence,
			expErr:    true,
			expErrMsg: "invalid equivocation height",
		},
		{
			name:   "valid evidence",
			req:    validEvidence,
			expErr: false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			_, err := s.msgServer.SubmitEvidence(s.ctx, tc.req)
			if tc.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}
