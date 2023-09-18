package vesting_test

import (
	"testing"
	"time"

	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	tmtime "github.com/cometbft/cometbft/types/time"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/auth/vesting"
	vestingtestutil "github.com/cosmos/cosmos-sdk/x/auth/vesting/testutil"
	vestingtypes "github.com/cosmos/cosmos-sdk/x/auth/vesting/types"
)

var (
	fromAddr   = sdk.AccAddress([]byte("from1________________"))
	to1Addr    = sdk.AccAddress([]byte("to1__________________"))
	to2Addr    = sdk.AccAddress([]byte("to2__________________"))
	to3Addr    = sdk.AccAddress([]byte("to3__________________"))
	fooCoin    = sdk.NewInt64Coin("foo", 100)
	periodCoin = sdk.NewInt64Coin("foo", 20)
)

type VestingTestSuite struct {
	suite.Suite

	ctx           sdk.Context
	accountKeeper authkeeper.AccountKeeper
	bankKeeper    *vestingtestutil.MockBankKeeper
	msgServer     vestingtypes.MsgServer
}

func (s *VestingTestSuite) SetupTest() {
	key := sdk.NewKVStoreKey(authtypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(s.T(), key, sdk.NewTransientStoreKey("transient_test"))
	s.ctx = testCtx.Ctx.WithBlockHeader(tmproto.Header{Time: tmtime.Now()})
	encCfg := moduletestutil.MakeTestEncodingConfig()

	maccPerms := map[string][]string{}

	ctrl := gomock.NewController(s.T())
	s.bankKeeper = vestingtestutil.NewMockBankKeeper(ctrl)
	s.accountKeeper = authkeeper.NewAccountKeeper(
		encCfg.Codec,
		key,
		authtypes.ProtoBaseAccount,
		maccPerms,
		"cosmos",
		authtypes.NewModuleAddress("gov").String(),
	)

	vestingtypes.RegisterInterfaces(encCfg.InterfaceRegistry)
	authtypes.RegisterInterfaces(encCfg.InterfaceRegistry)
	s.msgServer = vesting.NewMsgServerImpl(s.accountKeeper, s.bankKeeper)
}

func (s *VestingTestSuite) TestCreateVestingAccount() {
	testCases := map[string]struct {
		preRun    func()
		input     *vestingtypes.MsgCreateVestingAccount
		expErr    bool
		expErrMsg string
	}{
		"create for existing account": {
			preRun: func() {
				toAcc := s.accountKeeper.NewAccountWithAddress(s.ctx, to1Addr)
				s.bankKeeper.EXPECT().IsSendEnabledCoins(gomock.Any(), fooCoin).Return(nil)
				s.accountKeeper.SetAccount(s.ctx, toAcc)
				s.bankKeeper.EXPECT().BlockedAddr(to1Addr).Return(false)
			},
			input: vestingtypes.NewMsgCreateVestingAccount(
				fromAddr,
				to1Addr,
				sdk.Coins{fooCoin},
				time.Now().Unix(),
				true,
			),
			expErr:    true,
			expErrMsg: "already exists",
		},
		"create a valid delayed vesting account": {
			preRun: func() {
				s.bankKeeper.EXPECT().IsSendEnabledCoins(gomock.Any(), fooCoin).Return(nil)
				s.bankKeeper.EXPECT().BlockedAddr(to2Addr).Return(false)
				s.bankKeeper.EXPECT().SendCoins(gomock.Any(), fromAddr, to2Addr, sdk.Coins{fooCoin}).Return(nil)
			},
			input: vestingtypes.NewMsgCreateVestingAccount(
				fromAddr,
				to2Addr,
				sdk.Coins{fooCoin},
				time.Now().Unix(),
				true,
			),
			expErr:    false,
			expErrMsg: "",
		},
		"create a valid continuous vesting account": {
			preRun: func() {
				s.bankKeeper.EXPECT().IsSendEnabledCoins(gomock.Any(), fooCoin).Return(nil)
				s.bankKeeper.EXPECT().BlockedAddr(to3Addr).Return(false)
				s.bankKeeper.EXPECT().SendCoins(gomock.Any(), fromAddr, to3Addr, sdk.Coins{fooCoin}).Return(nil)
			},
			input: vestingtypes.NewMsgCreateVestingAccount(
				fromAddr,
				to3Addr,
				sdk.Coins{fooCoin},
				time.Now().Unix(),
				false,
			),
			expErr:    false,
			expErrMsg: "",
		},
	}

	for name, tc := range testCases {
		s.Run(name, func() {
			tc.preRun()
			_, err := s.msgServer.CreateVestingAccount(s.ctx, tc.input)
			if tc.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}

func (s *VestingTestSuite) TestCreatePermanentLockedAccount() {
	testCases := map[string]struct {
		preRun    func()
		input     *vestingtypes.MsgCreatePermanentLockedAccount
		expErr    bool
		expErrMsg string
	}{
		"create for existing account": {
			preRun: func() {
				toAcc := s.accountKeeper.NewAccountWithAddress(s.ctx, to1Addr)
				s.bankKeeper.EXPECT().IsSendEnabledCoins(gomock.Any(), fooCoin).Return(nil)
				s.bankKeeper.EXPECT().BlockedAddr(to1Addr).Return(false)
				s.accountKeeper.SetAccount(s.ctx, toAcc)
			},
			input: vestingtypes.NewMsgCreatePermanentLockedAccount(
				fromAddr,
				to1Addr,
				sdk.Coins{fooCoin},
			),
			expErr:    true,
			expErrMsg: "already exists",
		},
		"create a valid permanent locked account": {
			preRun: func() {
				s.bankKeeper.EXPECT().IsSendEnabledCoins(gomock.Any(), fooCoin).Return(nil)
				s.bankKeeper.EXPECT().BlockedAddr(to2Addr).Return(false)
				s.bankKeeper.EXPECT().SendCoins(gomock.Any(), fromAddr, to2Addr, sdk.Coins{fooCoin}).Return(nil)
			},
			input: vestingtypes.NewMsgCreatePermanentLockedAccount(
				fromAddr,
				to2Addr,
				sdk.Coins{fooCoin},
			),
			expErr:    false,
			expErrMsg: "",
		},
	}

	for name, tc := range testCases {
		s.Run(name, func() {
			tc.preRun()
			_, err := s.msgServer.CreatePermanentLockedAccount(s.ctx, tc.input)
			if tc.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}

func (s *VestingTestSuite) TestCreatePeriodicVestingAccount() {
	testCases := []struct {
		name      string
		preRun    func()
		input     *vestingtypes.MsgCreatePeriodicVestingAccount
		expErr    bool
		expErrMsg string
	}{
		{
			name: "create for existing account",
			preRun: func() {
				toAcc := s.accountKeeper.NewAccountWithAddress(s.ctx, to1Addr)
				s.accountKeeper.SetAccount(s.ctx, toAcc)
			},
			input: vestingtypes.NewMsgCreatePeriodicVestingAccount(
				fromAddr,
				to1Addr,
				time.Now().Unix(),
				[]vestingtypes.Period{
					{
						Length: 10,
						Amount: sdk.NewCoins(periodCoin),
					},
				},
			),
			expErr:    true,
			expErrMsg: "already exists",
		},
		{
			name: "create a valid periodic vesting account",
			preRun: func() {
				s.bankKeeper.EXPECT().IsSendEnabledCoins(gomock.Any(), periodCoin.Add(fooCoin)).Return(nil)
				s.bankKeeper.EXPECT().SendCoins(gomock.Any(), fromAddr, to2Addr, gomock.Any()).Return(nil)
			},
			input: vestingtypes.NewMsgCreatePeriodicVestingAccount(
				fromAddr,
				to2Addr,
				time.Now().Unix(),
				[]vestingtypes.Period{
					{
						Length: 10,
						Amount: sdk.NewCoins(periodCoin),
					},
					{
						Length: 20,
						Amount: sdk.NewCoins(fooCoin),
					},
				},
			),
			expErr:    false,
			expErrMsg: "",
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			tc.preRun()
			_, err := s.msgServer.CreatePeriodicVestingAccount(s.ctx, tc.input)
			if tc.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}

func TestVestingTestSuite(t *testing.T) {
	suite.Run(t, new(VestingTestSuite))
}
