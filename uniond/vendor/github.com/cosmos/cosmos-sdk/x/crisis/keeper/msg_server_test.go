package keeper_test

import (
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/crisis"
	"github.com/cosmos/cosmos-sdk/x/crisis/keeper"
	crisistestutil "github.com/cosmos/cosmos-sdk/x/crisis/testutil"
	"github.com/cosmos/cosmos-sdk/x/crisis/types"
)

type KeeperTestSuite struct {
	suite.Suite

	ctx        sdk.Context
	authKeeper *crisistestutil.MockSupplyKeeper
	keeper     *keeper.Keeper
}

func (s *KeeperTestSuite) SetupTest() {
	// gomock initializations
	ctrl := gomock.NewController(s.T())
	supplyKeeper := crisistestutil.NewMockSupplyKeeper(ctrl)

	key := sdk.NewKVStoreKey(types.StoreKey)
	testCtx := testutil.DefaultContextWithDB(s.T(), key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(crisis.AppModuleBasic{})
	keeper := keeper.NewKeeper(encCfg.Codec, key, 5, supplyKeeper, "", "")

	s.ctx = testCtx.Ctx
	s.keeper = keeper
	s.authKeeper = supplyKeeper
}

func (s *KeeperTestSuite) TestMsgVerifyInvariant() {
	// default params
	constantFee := sdk.NewCoin(sdk.DefaultBondDenom, sdk.NewInt(1000))
	err := s.keeper.SetConstantFee(s.ctx, constantFee)
	s.Require().NoError(err)

	sender := sdk.AccAddress([]byte("addr1_______________"))

	s.authKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil).Times(2)
	s.keeper.RegisterRoute("bank", "total-supply", func(sdk.Context) (string, bool) { return "", false })

	testCases := []struct {
		name      string
		input     *types.MsgVerifyInvariant
		expErr    bool
		expErrMsg string
	}{
		{
			name: "empty sender not allowed",
			input: &types.MsgVerifyInvariant{
				Sender:              "",
				InvariantModuleName: "bank",
				InvariantRoute:      "total-supply",
			},
			expErr:    true,
			expErrMsg: "empty address string is not allowed",
		},
		{
			name: "invalid sender address",
			input: &types.MsgVerifyInvariant{
				Sender:              "invalid address",
				InvariantModuleName: "bank",
				InvariantRoute:      "total-supply",
			},
			expErr:    true,
			expErrMsg: "decoding bech32 failed",
		},
		{
			name: "unregistered invariant route",
			input: &types.MsgVerifyInvariant{
				Sender:              sender.String(),
				InvariantModuleName: "module",
				InvariantRoute:      "invalidroute",
			},
			expErr:    true,
			expErrMsg: "unknown invariant",
		},
		{
			name: "valid invariant",
			input: &types.MsgVerifyInvariant{
				Sender:              sender.String(),
				InvariantModuleName: "bank",
				InvariantRoute:      "total-supply",
			},
			expErr: false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			_, err = s.keeper.VerifyInvariant(s.ctx, tc.input)
			if tc.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}

func (s *KeeperTestSuite) TestMsgUpdateParams() {
	// default params
	constantFee := sdk.NewCoin(sdk.DefaultBondDenom, sdk.NewInt(1000))

	testCases := []struct {
		name      string
		input     *types.MsgUpdateParams
		expErr    bool
		expErrMsg string
	}{
		{
			name: "invalid authority",
			input: &types.MsgUpdateParams{
				Authority:   "invalid",
				ConstantFee: constantFee,
			},
			expErr:    true,
			expErrMsg: "invalid authority",
		},
		{
			name: "invalid constant fee",
			input: &types.MsgUpdateParams{
				Authority:   s.keeper.GetAuthority(),
				ConstantFee: sdk.Coin{},
			},
			expErr: true,
		},
		{
			name: "negative constant fee",
			input: &types.MsgUpdateParams{
				Authority:   s.keeper.GetAuthority(),
				ConstantFee: sdk.Coin{Denom: sdk.DefaultBondDenom, Amount: sdk.NewInt(-1000)},
			},
			expErr: true,
		},
		{
			name: "all good",
			input: &types.MsgUpdateParams{
				Authority:   s.keeper.GetAuthority(),
				ConstantFee: constantFee,
			},
			expErr: false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			_, err := s.keeper.UpdateParams(s.ctx, tc.input)

			if tc.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(KeeperTestSuite))
}
