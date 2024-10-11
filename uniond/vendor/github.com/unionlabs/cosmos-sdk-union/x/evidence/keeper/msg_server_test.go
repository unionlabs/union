package keeper_test

import (
	"time"

	"cosmossdk.io/x/evidence/types"

	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
)

func (s *KeeperTestSuite) TestSubmitEvidence() {
	pk := ed25519.GenPrivKey()
	consAddr, err := s.consAddressCodec.BytesToString(pk.PubKey().Address())
	s.Require().NoError(err)

	e := &types.Equivocation{
		Height:           1,
		Power:            100,
		Time:             time.Now().UTC(),
		ConsensusAddress: consAddr,
	}

	accAddr, err := s.addressCodec.BytesToString(valAddress)
	s.Require().NoError(err)

	validEvidence, err := types.NewMsgSubmitEvidence(accAddr, e)
	s.Require().NoError(err)

	e2 := &types.Equivocation{
		Height:           0,
		Power:            100,
		Time:             time.Now().UTC(),
		ConsensusAddress: consAddr,
	}

	invalidEvidence, err := types.NewMsgSubmitEvidence(accAddr, e2)
	s.Require().NoError(err)

	testCases := []struct {
		name      string
		req       *types.MsgSubmitEvidence
		expErr    bool
		expErrMsg string
	}{
		{
			name:      "invalid address",
			req:       &types.MsgSubmitEvidence{},
			expErr:    true,
			expErrMsg: "invalid submitter address: empty address string is not allowed",
		},
		{
			name: "missing evidence",
			req: &types.MsgSubmitEvidence{
				Submitter: accAddr,
			},
			expErr:    true,
			expErrMsg: "missing evidence: invalid evidence",
		},
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
