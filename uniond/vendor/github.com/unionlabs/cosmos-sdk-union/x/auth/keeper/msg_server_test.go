package keeper_test

import (
	"context"

	"github.com/cosmos/gogoproto/proto"
	any "github.com/cosmos/gogoproto/types/any"
	"github.com/golang/mock/gomock"
	"google.golang.org/protobuf/runtime/protoiface"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

func (s *KeeperTestSuite) TestUpdateParams() {
	testCases := []struct {
		name      string
		req       *types.MsgUpdateParams
		expectErr bool
		expErrMsg string
	}{
		{
			name: "set invalid authority",
			req: &types.MsgUpdateParams{
				Authority: "foo",
			},
			expectErr: true,
			expErrMsg: "invalid authority",
		},
		{
			name: "set invalid max memo characters",
			req: &types.MsgUpdateParams{
				Authority: s.accountKeeper.GetAuthority(),
				Params: types.Params{
					MaxMemoCharacters:      0,
					TxSigLimit:             9,
					TxSizeCostPerByte:      5,
					SigVerifyCostED25519:   694,
					SigVerifyCostSecp256k1: 511,
				},
			},
			expectErr: true,
			expErrMsg: "invalid max memo characters",
		},
		{
			name: "set invalid tx sig limit",
			req: &types.MsgUpdateParams{
				Authority: s.accountKeeper.GetAuthority(),
				Params: types.Params{
					MaxMemoCharacters:      140,
					TxSigLimit:             0,
					TxSizeCostPerByte:      5,
					SigVerifyCostED25519:   694,
					SigVerifyCostSecp256k1: 511,
				},
			},
			expectErr: true,
			expErrMsg: "invalid tx signature limit",
		},
		{
			name: "set invalid tx size cost per bytes",
			req: &types.MsgUpdateParams{
				Authority: s.accountKeeper.GetAuthority(),
				Params: types.Params{
					MaxMemoCharacters:      140,
					TxSigLimit:             9,
					TxSizeCostPerByte:      0,
					SigVerifyCostED25519:   694,
					SigVerifyCostSecp256k1: 511,
				},
			},
			expectErr: true,
			expErrMsg: "invalid tx size cost per byte",
		},
		{
			name: "set invalid sig verify cost ED25519",
			req: &types.MsgUpdateParams{
				Authority: s.accountKeeper.GetAuthority(),
				Params: types.Params{
					MaxMemoCharacters:      140,
					TxSigLimit:             9,
					TxSizeCostPerByte:      5,
					SigVerifyCostED25519:   0,
					SigVerifyCostSecp256k1: 511,
				},
			},
			expectErr: true,
			expErrMsg: "invalid ED25519 signature verification cost",
		},
		{
			name: "set invalid sig verify cost Secp256k1",
			req: &types.MsgUpdateParams{
				Authority: s.accountKeeper.GetAuthority(),
				Params: types.Params{
					MaxMemoCharacters:      140,
					TxSigLimit:             9,
					TxSizeCostPerByte:      5,
					SigVerifyCostED25519:   694,
					SigVerifyCostSecp256k1: 0,
				},
			},
			expectErr: true,
			expErrMsg: "invalid SECK256k1 signature verification cost",
		},
		{
			name: "valid transaction",
			req: &types.MsgUpdateParams{
				Authority: s.accountKeeper.GetAuthority(),
				Params: types.Params{
					MaxMemoCharacters:      140,
					TxSigLimit:             9,
					TxSizeCostPerByte:      5,
					SigVerifyCostED25519:   694,
					SigVerifyCostSecp256k1: 511,
				},
			},
			expectErr: false,
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			_, err := s.msgServer.UpdateParams(s.ctx, tc.req)
			if tc.expectErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}

func (s *KeeperTestSuite) TestNonAtomicExec() {
	_, _, addr := testdata.KeyTestPubAddr()

	msgUpdateParams := &types.MsgUpdateParams{}

	msgAny, err := codectypes.NewAnyWithValue(msgUpdateParams)
	s.Require().NoError(err)

	testCases := []struct {
		name      string
		req       *types.MsgNonAtomicExec
		expectErr bool
		expErrMsg string
	}{
		{
			name: "error: empty signer",
			req: &types.MsgNonAtomicExec{
				Signer: "",
				Msgs:   []*any.Any{},
			},
			expectErr: true,
			expErrMsg: "empty signer address string is not allowed",
		},
		{
			name: "error: invalid signer",
			req: &types.MsgNonAtomicExec{
				Signer: "invalid_signer",
				Msgs:   []*any.Any{},
			},
			expectErr: true,
			expErrMsg: "invalid signer address",
		},
		{
			name: "error: empty messages",
			req: &types.MsgNonAtomicExec{
				Signer: addr.String(),
				Msgs:   []*any.Any{},
			},
			expectErr: true,
			expErrMsg: "messages cannot be empty",
		},
		{
			name: "valid transaction",
			req: &types.MsgNonAtomicExec{
				Signer: addr.String(),
				Msgs:   []*any.Any{msgAny},
			},
			expectErr: false,
		},
	}

	s.acctsModKeeper.EXPECT().SendModuleMessage(gomock.Any(), gomock.Any(), gomock.Any()).
		DoAndReturn(func(_ context.Context, sender []byte, msg proto.Message) (protoiface.MessageV1, error) {
			return msg, nil
		}).AnyTimes()

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			_, err := s.msgServer.NonAtomicExec(s.ctx, tc.req)
			if tc.expectErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}
