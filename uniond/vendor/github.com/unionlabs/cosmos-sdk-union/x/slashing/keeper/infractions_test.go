package keeper_test

import (
	"time"

	gogoany "github.com/cosmos/gogoproto/types/any"

	stakingv1beta1 "cosmossdk.io/api/cosmos/staking/v1beta1"
	"cosmossdk.io/core/comet"
	"cosmossdk.io/math"
	"cosmossdk.io/x/slashing/types"
	stakingtypes "cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (s *KeeperTestSuite) TestKeeper_HandleValidatorSignature() {
	_, edPubKey, valAddr := testdata.KeyTestPubAddrED25519()
	valStrAddr, err := s.stakingKeeper.ValidatorAddressCodec().BytesToString(valAddr)
	s.Require().NoError(err)
	consStrAddr, err := s.stakingKeeper.ConsensusAddressCodec().BytesToString(valAddr)
	s.Require().NoError(err)

	vpk, err := gogoany.NewAnyWithCacheWithValue(edPubKey)
	s.Require().NoError(err)

	_, pubKey, _ := testdata.KeyTestPubAddr()
	addr := pubKey.Address()
	tests := []struct {
		name        string
		height      int64
		validator   stakingtypes.Validator
		valSignInfo types.ValidatorSigningInfo
		flag        comet.BlockIDFlag
		wantErr     bool
		errMsg      string
	}{
		{
			name: "ok validator",
			validator: stakingtypes.Validator{
				OperatorAddress: valStrAddr,
				ConsensusPubkey: vpk,
				Jailed:          false,
				Status:          stakingtypes.Bonded,
				Tokens:          math.NewInt(100),
				DelegatorShares: math.LegacyNewDec(100),
			},
			valSignInfo: types.NewValidatorSigningInfo(consStrAddr, int64(0),
				time.Now().UTC().Add(100000000000), false, int64(10)),
			flag: comet.BlockIDFlagCommit,
		},
		{
			name: "jailed validator",
			validator: stakingtypes.Validator{
				Jailed: true,
			},
			flag: comet.BlockIDFlagCommit,
		},
		{
			name: "signingInfo startHeight > height",
			validator: stakingtypes.Validator{
				OperatorAddress: valStrAddr,
				ConsensusPubkey: vpk,
			},
			valSignInfo: types.NewValidatorSigningInfo(consStrAddr, int64(3),
				time.Now().UTC().Add(100000000000), false, int64(10)),
			flag:    comet.BlockIDFlagCommit,
			wantErr: true,
			errMsg:  "start height 3 , which is greater than the current height 0",
		},
		{
			name: "absent",
			validator: stakingtypes.Validator{
				OperatorAddress: valStrAddr,
				ConsensusPubkey: vpk,
				Jailed:          false,
				Status:          stakingtypes.Bonded,
				Tokens:          math.NewInt(100),
				DelegatorShares: math.LegacyNewDec(100),
			},
			valSignInfo: types.NewValidatorSigningInfo(consStrAddr, int64(0),
				time.Now().UTC().Add(100000000000), false, int64(10)),
			flag: comet.BlockIDFlagAbsent,
		},
		{
			name: "punish validator",
			validator: stakingtypes.Validator{
				OperatorAddress: valStrAddr,
				ConsensusPubkey: vpk,
				Jailed:          false,
				Status:          stakingtypes.Bonded,
				Tokens:          math.NewInt(100),
				DelegatorShares: math.LegacyNewDec(100),
			},
			valSignInfo: types.NewValidatorSigningInfo(consStrAddr, int64(0),
				time.Now().UTC().Add(100000000000), false, int64(501)),
			flag:   comet.BlockIDFlagAbsent,
			height: 2000,
		},
	}
	for _, tt := range tests {
		s.Run(tt.name, func() {
			headerInfo := s.ctx.HeaderInfo()
			headerInfo.Height = tt.height
			s.ctx = s.ctx.WithHeaderInfo(headerInfo)

			s.Require().NoError(s.slashingKeeper.ValidatorSigningInfo.Set(s.ctx, edPubKey.Address().Bytes(), tt.valSignInfo))

			s.stakingKeeper.EXPECT().ValidatorByConsAddr(s.ctx, sdk.ConsAddress(addr)).Return(tt.validator, nil)
			s.stakingKeeper.EXPECT().ValidatorIdentifier(s.ctx, sdk.ConsAddress(edPubKey.Address().Bytes())).Return(sdk.ConsAddress(edPubKey.Address().Bytes()), nil).AnyTimes()
			s.stakingKeeper.EXPECT().ValidatorByConsAddr(s.ctx, sdk.ConsAddress(edPubKey.Address().Bytes())).Return(tt.validator, nil).AnyTimes()
			downTime, err := math.LegacyNewDecFromStr("0.01")
			s.Require().NoError(err)
			s.stakingKeeper.EXPECT().SlashWithInfractionReason(s.ctx, sdk.ConsAddress(edPubKey.Address().Bytes()), int64(1998), int64(0), downTime, stakingv1beta1.Infraction_INFRACTION_DOWNTIME).Return(math.NewInt(19), nil).AnyTimes()
			s.stakingKeeper.EXPECT().Jail(s.ctx, sdk.ConsAddress(edPubKey.Address().Bytes())).Return(nil).AnyTimes()

			err = s.slashingKeeper.HandleValidatorSignature(s.ctx, addr, 0, tt.flag)
			if tt.wantErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tt.errMsg)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}
