package keeper_test

import (
	"testing"
	"time"

	gogoproto "github.com/cosmos/gogoproto/proto"
	gogoprotoany "github.com/cosmos/gogoproto/types/any"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/authz"
	authzkeeper "cosmossdk.io/x/authz/keeper"
	authzmodule "cosmossdk.io/x/authz/module"
	authztestutil "cosmossdk.io/x/authz/testutil"
	banktypes "cosmossdk.io/x/bank/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

var (
	bankSendAuthMsgType = banktypes.SendAuthorization{}.MsgTypeURL()
	coins10             = sdk.NewCoins(sdk.NewInt64Coin("stake", 10))
	coins100            = sdk.NewCoins(sdk.NewInt64Coin("stake", 100))
	coins1000           = sdk.NewCoins(sdk.NewInt64Coin("stake", 1000))
)

type TestSuite struct {
	suite.Suite

	ctx           sdk.Context
	addrs         []sdk.AccAddress
	authzKeeper   authzkeeper.Keeper
	accountKeeper *authztestutil.MockAccountKeeper
	bankKeeper    *authztestutil.MockBankKeeper
	baseApp       *baseapp.BaseApp
	encCfg        moduletestutil.TestEncodingConfig
	queryClient   authz.QueryClient
	msgSrvr       authz.MsgServer
}

func (s *TestSuite) SetupTest() {
	key := storetypes.NewKVStoreKey(authzkeeper.StoreKey)
	storeService := runtime.NewKVStoreService(key)
	testCtx := testutil.DefaultContextWithDB(s.T(), key, storetypes.NewTransientStoreKey("transient_test"))
	s.ctx = testCtx.Ctx.WithHeaderInfo(header.Info{Time: time.Now().Round(0).UTC()})
	s.encCfg = moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, authzmodule.AppModule{})

	s.baseApp = baseapp.NewBaseApp(
		"authz",
		log.NewNopLogger(),
		testCtx.DB,
		s.encCfg.TxConfig.TxDecoder(),
	)
	s.baseApp.SetCMS(testCtx.CMS)
	s.baseApp.SetInterfaceRegistry(s.encCfg.InterfaceRegistry)

	s.addrs = simtestutil.CreateIncrementalAccounts(7)

	// gomock initializations
	ctrl := gomock.NewController(s.T())
	s.accountKeeper = authztestutil.NewMockAccountKeeper(ctrl)

	s.accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	s.bankKeeper = authztestutil.NewMockBankKeeper(ctrl)
	banktypes.RegisterInterfaces(s.encCfg.InterfaceRegistry)
	banktypes.RegisterMsgServer(s.baseApp.MsgServiceRouter(), s.bankKeeper)

	env := runtime.NewEnvironment(storeService, coretesting.NewNopLogger(), runtime.EnvWithQueryRouterService(s.baseApp.GRPCQueryRouter()), runtime.EnvWithMsgRouterService(s.baseApp.MsgServiceRouter()))
	s.authzKeeper = authzkeeper.NewKeeper(env, s.encCfg.Codec, s.accountKeeper)

	queryHelper := baseapp.NewQueryServerTestHelper(s.ctx, s.encCfg.InterfaceRegistry)
	authz.RegisterQueryServer(queryHelper, s.authzKeeper)
	authz.RegisterMsgServer(s.baseApp.MsgServiceRouter(), s.authzKeeper)
	queryClient := authz.NewQueryClient(queryHelper)
	s.queryClient = queryClient

	s.msgSrvr = s.authzKeeper
}

func (s *TestSuite) TestKeeper() {
	ctx, addrs := s.ctx, s.addrs
	now := ctx.HeaderInfo().Time
	require := s.Require()

	granterAddr := addrs[0]
	grantee1Addr := addrs[1]
	grantee2Addr := addrs[2]
	grantee3Addr := addrs[3]
	grantees := []sdk.AccAddress{grantee1Addr, grantee2Addr, grantee3Addr}

	s.T().Log("verify that no authorization returns nil")
	authorizations, err := s.authzKeeper.GetAuthorizations(ctx, granteeAddr, granterAddr)
	require.NoError(err)
	require.Len(authorizations, 0)

	s.T().Log("verify save, get and delete work for grants")
	sendAutz := &banktypes.SendAuthorization{SpendLimit: coins100}
	expire := now.AddDate(1, 0, 0)
	for _, grantee := range grantees {
		err = s.authzKeeper.SaveGrant(ctx, grantee, granterAddr, sendAutz, &expire)
		require.NoError(err)
	}

	for _, grantee := range grantees {
		authorizations, err = s.authzKeeper.GetAuthorizations(ctx, grantee, granterAddr)
		require.NoError(err)
		require.Len(authorizations, 1)
	}

	// Delete a single grant
	err = s.authzKeeper.DeleteGrant(ctx, grantee1Addr, granterAddr, sendAutz.MsgTypeURL())
	require.NoError(err)

	authorizations, err = s.authzKeeper.GetAuthorizations(ctx, grantee1Addr, granterAddr)
	require.NoError(err)
	require.Len(authorizations, 0)
	authorizations, err = s.authzKeeper.GetAuthorizations(ctx, grantee2Addr, granterAddr)
	require.NoError(err)
	require.Len(authorizations, 1)
	authorizations, err = s.authzKeeper.GetAuthorizations(ctx, grantee3Addr, granterAddr)
	require.NoError(err)
	require.Len(authorizations, 1)

	// Delete all grants for a granter
	err = s.authzKeeper.DeleteAllGrants(ctx, granterAddr)
	require.NoError(err)

	authorizations, err = s.authzKeeper.GetAuthorizations(ctx, grantee1Addr, granterAddr)
	require.NoError(err)
	require.Len(authorizations, 0)
	authorizations, err = s.authzKeeper.GetAuthorizations(ctx, grantee2Addr, granterAddr)
	require.NoError(err)
	require.Len(authorizations, 0)
	authorizations, err = s.authzKeeper.GetAuthorizations(ctx, grantee3Addr, granterAddr)
	require.NoError(err)
	require.Len(authorizations, 0)

	// Delete all grants for a granter with no grants, should error
	err = s.authzKeeper.DeleteAllGrants(ctx, granterAddr)
	require.Error(err)

	s.T().Log("verify granting same authorization overwrite existing authorization")
	err = s.authzKeeper.SaveGrant(ctx, granteeAddr, granterAddr, sendAutz, &expire)
	require.NoError(err)

	authorizations, err = s.authzKeeper.GetAuthorizations(ctx, granteeAddr, granterAddr)
	require.NoError(err)
	require.Len(authorizations, 1)

	sendAutz = &banktypes.SendAuthorization{SpendLimit: coins1000}
	err = s.authzKeeper.SaveGrant(ctx, granteeAddr, granterAddr, sendAutz, &expire)
	require.NoError(err)
	authorizations, err = s.authzKeeper.GetAuthorizations(ctx, granteeAddr, granterAddr)
	require.NoError(err)
	require.Len(authorizations, 1)
	authorization := authorizations[0]
	sendAuth := authorization.(*banktypes.SendAuthorization)
	require.Equal(sendAuth.SpendLimit, sendAutz.SpendLimit)
	require.Equal(sendAuth.MsgTypeURL(), sendAutz.MsgTypeURL())

	s.T().Log("verify removing non existing authorization returns error")
	err = s.authzKeeper.DeleteGrant(ctx, granterAddr, granteeAddr, "abcd")
	s.Require().Error(err)
}

func (s *TestSuite) TestKeeperIter() {
	ctx, addrs := s.ctx, s.addrs

	granterAddr := addrs[0]
	granteeAddr := addrs[1]
	granter2Addr := addrs[2]
	e := ctx.HeaderInfo().Time.AddDate(1, 0, 0)
	sendAuthz := banktypes.NewSendAuthorization(coins100, nil, s.accountKeeper.AddressCodec())

	err := s.authzKeeper.SaveGrant(ctx, granteeAddr, granterAddr, sendAuthz, &e)
	s.Require().NoError(err)

	err = s.authzKeeper.SaveGrant(ctx, granteeAddr, granter2Addr, sendAuthz, &e)
	s.Require().NoError(err)

	_ = s.authzKeeper.IterateGrants(ctx, func(granter, grantee sdk.AccAddress, grant authz.Grant) (bool, error) {
		s.Require().Equal(granteeAddr, grantee)
		s.Require().Contains([]sdk.AccAddress{granterAddr, granter2Addr}, granter)
		return true, nil
	})
}

func (s *TestSuite) TestKeeperGranterGrantsIter() {
	ctx, addrs := s.ctx, s.addrs

	granterAddr := addrs[0]
	granter2Addr := addrs[1]
	granteeAddr := addrs[2]
	grantee2Addr := addrs[3]
	grantee3Addr := addrs[4]
	e := ctx.HeaderInfo().Time.AddDate(1, 0, 0)
	sendAuthz := banktypes.NewSendAuthorization(coins100, nil, s.accountKeeper.AddressCodec())

	err := s.authzKeeper.SaveGrant(ctx, granteeAddr, granterAddr, sendAuthz, &e)
	s.Require().NoError(err)

	err = s.authzKeeper.SaveGrant(ctx, grantee2Addr, granterAddr, sendAuthz, &e)
	s.Require().NoError(err)

	err = s.authzKeeper.SaveGrant(ctx, grantee3Addr, granter2Addr, sendAuthz, &e)
	s.Require().NoError(err)

	_ = s.authzKeeper.IterateGranterGrants(ctx, granterAddr, func(grantee sdk.AccAddress, msgType string) (bool, error) {
		s.Require().Contains([]sdk.AccAddress{granteeAddr, grantee2Addr}, grantee)
		s.Require().NotContains([]sdk.AccAddress{grantee3Addr}, grantee)
		return true, nil
	})

	_ = s.authzKeeper.IterateGranterGrants(ctx, granter2Addr, func(grantee sdk.AccAddress, msgType string) (bool, error) {
		s.Require().Equal(grantee3Addr, grantee)
		s.Require().NotContains([]sdk.AccAddress{granteeAddr, grantee2Addr}, grantee)
		return true, nil
	})
}

func (s *TestSuite) TestDispatchAction() {
	addrs := s.addrs
	require := s.Require()
	now := s.ctx.HeaderInfo().Time

	granterAddr := addrs[0]
	granteeAddr := addrs[1]
	granterStrAddr, err := s.accountKeeper.AddressCodec().BytesToString(addrs[0])
	s.Require().NoError(err)
	granteeStrAddr, err := s.accountKeeper.AddressCodec().BytesToString(addrs[1])
	s.Require().NoError(err)
	recipientStrAddr, err := s.accountKeeper.AddressCodec().BytesToString(addrs[2])
	s.Require().NoError(err)
	a := banktypes.NewSendAuthorization(coins100, nil, s.accountKeeper.AddressCodec())

	testCases := []struct {
		name       string
		req        authz.MsgExec
		expectErr  bool
		errMsg     string
		expectResp string
		preRun     func() sdk.Context
		postRun    func()
	}{
		{
			"expect error authorization not found",
			authz.NewMsgExec(granteeStrAddr, []sdk.Msg{
				&banktypes.MsgSend{
					Amount:      coins10,
					FromAddress: granterStrAddr,
					ToAddress:   recipientStrAddr,
				},
			}),
			true,
			"authorization not found",
			"",
			func() sdk.Context {
				// remove any existing authorizations
				err := s.authzKeeper.DeleteGrant(s.ctx, granteeAddr, granterAddr, bankSendAuthMsgType)
				require.Error(err)
				return s.ctx
			},
			func() {},
		},
		{
			"expect error expired authorization",
			authz.NewMsgExec(granteeStrAddr, []sdk.Msg{
				&banktypes.MsgSend{
					Amount:      coins10,
					FromAddress: granterStrAddr,
					ToAddress:   recipientStrAddr,
				},
			}),
			true,
			"authorization expired",
			"",
			func() sdk.Context {
				e := now.AddDate(0, 0, 1)
				err := s.authzKeeper.SaveGrant(s.ctx, granteeAddr, granterAddr, a, &e)
				require.NoError(err)
				return s.ctx.WithHeaderInfo(header.Info{Time: s.ctx.HeaderInfo().Time.AddDate(0, 0, 2)})
			},
			func() {},
		},
		{
			"expect error over spent limit",
			authz.NewMsgExec(granteeStrAddr, []sdk.Msg{
				&banktypes.MsgSend{
					Amount:      coins1000,
					FromAddress: granterStrAddr,
					ToAddress:   recipientStrAddr,
				},
			}),
			true,
			"requested amount is more than spend limit",
			"",
			func() sdk.Context {
				e := now.AddDate(0, 1, 0)
				err := s.authzKeeper.SaveGrant(s.ctx, granteeAddr, granterAddr, a, &e)
				require.NoError(err)
				return s.ctx
			},
			func() {},
		},
		{
			"valid test verify amount left",
			authz.NewMsgExec(granteeStrAddr, []sdk.Msg{
				&banktypes.MsgSend{
					Amount:      coins10,
					FromAddress: granterStrAddr,
					ToAddress:   recipientStrAddr,
				},
			}),
			false,
			"",
			"/cosmos.bank.v1beta1.MsgSendResponse",
			func() sdk.Context {
				e := now.AddDate(0, 1, 0)
				err := s.authzKeeper.SaveGrant(s.ctx, granteeAddr, granterAddr, a, &e)
				require.NoError(err)
				return s.ctx
			},
			func() {
				authzs, err := s.authzKeeper.GetAuthorizations(s.ctx, granteeAddr, granterAddr)
				require.NoError(err)
				require.Len(authzs, 1)
				authorization := authzs[0].(*banktypes.SendAuthorization)
				require.NotNil(authorization)
				require.Equal(authorization.SpendLimit, coins100.Sub(coins10...))
			},
		},
		{
			"valid test verify authorization is removed when it is used up",
			authz.NewMsgExec(granteeStrAddr, []sdk.Msg{
				&banktypes.MsgSend{
					Amount:      coins100,
					FromAddress: granterStrAddr,
					ToAddress:   recipientStrAddr,
				},
			}),
			false,
			"",
			"/cosmos.bank.v1beta1.MsgSendResponse",
			func() sdk.Context {
				e := now.AddDate(0, 1, 0)
				err := s.authzKeeper.SaveGrant(s.ctx, granteeAddr, granterAddr, a, &e)
				require.NoError(err)
				return s.ctx
			},
			func() {
				authzs, err := s.authzKeeper.GetAuthorizations(s.ctx, granteeAddr, granterAddr)
				require.NoError(err)
				require.Len(authzs, 0)
			},
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			ctx := tc.preRun()
			executeMsgs, err := tc.req.GetMessages()
			require.NoError(err)
			result, err := s.authzKeeper.DispatchActions(ctx, granteeAddr, executeMsgs)
			if tc.expectErr {
				require.Error(err)
				require.Nil(result)
				require.Contains(err.Error(), tc.errMsg)
			} else {
				require.NoError(err)
				require.NotEmpty(result)
				// unmarshal the result
				for _, res := range result {
					var msgRes gogoprotoany.Any
					err := gogoproto.Unmarshal(res, &msgRes)
					require.NoError(err)
					require.NotNil(msgRes)
					require.Equal(msgRes.TypeUrl, tc.expectResp)
				}
			}
			tc.postRun()
		})
	}
}

// Tests that all msg events included in an authz MsgExec tx
// Ref: https://github.com/cosmos/cosmos-sdk/issues/9501
func (s *TestSuite) TestDispatchedEvents() {
	require := s.Require()
	addrs := s.addrs
	granterAddr := addrs[0]
	granteeAddr := addrs[1]
	granterStrAddr, err := s.accountKeeper.AddressCodec().BytesToString(addrs[0])
	s.Require().NoError(err)
	granteeStrAddr, err := s.accountKeeper.AddressCodec().BytesToString(addrs[1])
	s.Require().NoError(err)
	recipientStrAddr, err := s.accountKeeper.AddressCodec().BytesToString(addrs[2])
	s.Require().NoError(err)
	expiration := s.ctx.HeaderInfo().Time.Add(1 * time.Second) // must be in the future

	msgs := authz.NewMsgExec(granteeStrAddr, []sdk.Msg{
		&banktypes.MsgSend{
			Amount:      coins10,
			FromAddress: granterStrAddr,
			ToAddress:   recipientStrAddr,
		},
	})

	// grant authorization
	err = s.authzKeeper.SaveGrant(s.ctx, granteeAddr, granterAddr, &banktypes.SendAuthorization{SpendLimit: coins10}, &expiration)
	require.NoError(err)
	authorizations, err := s.authzKeeper.GetAuthorizations(s.ctx, granteeAddr, granterAddr)
	require.NoError(err)
	require.Len(authorizations, 1)
	authorization := authorizations[0].(*banktypes.SendAuthorization)
	require.Equal(authorization.MsgTypeURL(), bankSendAuthMsgType)

	executeMsgs, err := msgs.GetMessages()
	require.NoError(err)

	result, err := s.authzKeeper.DispatchActions(s.ctx, granteeAddr, executeMsgs)
	require.NoError(err)
	require.NotNil(result)

	events := s.ctx.EventManager().Events()

	// get last 5 events (events that occur *after* the grant)
	events = events[len(events)-2:]

	requiredEvents := map[string]bool{
		"cosmos.authz.v1beta1.EventGrant":  true,
		"cosmos.authz.v1beta1.EventRevoke": true,
	}
	for _, e := range events {
		requiredEvents[e.Type] = true
	}
	for _, v := range requiredEvents {
		require.True(v)
	}
}

func (s *TestSuite) TestDequeueAllGrantsQueue() {
	require := s.Require()
	addrs := s.addrs
	granter := addrs[0]
	grantee := addrs[1]
	grantee1 := addrs[2]
	exp := s.ctx.HeaderInfo().Time.AddDate(0, 0, 1)
	a := banktypes.SendAuthorization{SpendLimit: coins100}

	// create few authorizations
	err := s.authzKeeper.SaveGrant(s.ctx, grantee, granter, &a, &exp)
	require.NoError(err)

	err = s.authzKeeper.SaveGrant(s.ctx, grantee1, granter, &a, &exp)
	require.NoError(err)

	exp2 := exp.AddDate(0, 1, 0)
	err = s.authzKeeper.SaveGrant(s.ctx, granter, grantee1, &a, &exp2)
	require.NoError(err)

	exp2 = exp.AddDate(2, 0, 0)
	err = s.authzKeeper.SaveGrant(s.ctx, granter, grantee, &a, &exp2)
	require.NoError(err)

	newCtx := s.ctx.WithHeaderInfo(header.Info{Time: exp.AddDate(1, 0, 0)})
	err = s.authzKeeper.DequeueAndDeleteExpiredGrants(newCtx, 200)
	require.NoError(err)

	s.T().Log("verify expired grants are pruned from the state")
	authzs, err := s.authzKeeper.GetAuthorizations(newCtx, grantee, granter)
	require.NoError(err)
	require.Len(authzs, 0)

	authzs, err = s.authzKeeper.GetAuthorizations(newCtx, granter, grantee1)
	require.NoError(err)
	require.Len(authzs, 0)

	authzs, err = s.authzKeeper.GetAuthorizations(newCtx, grantee1, granter)
	require.NoError(err)
	require.Len(authzs, 0)

	authzs, err = s.authzKeeper.GetAuthorizations(newCtx, granter, grantee)
	require.NoError(err)
	require.Len(authzs, 1)
}

func (s *TestSuite) TestGetAuthorization() {
	addr1 := s.addrs[3]
	addr2 := s.addrs[4]
	addr3 := s.addrs[5]
	addr4 := s.addrs[6]

	genAuthMulti := authz.NewGenericAuthorization(sdk.MsgTypeURL(&banktypes.MsgMultiSend{}))
	genAuthSend := authz.NewGenericAuthorization(sdk.MsgTypeURL(&banktypes.MsgSend{}))
	sendAuth := banktypes.NewSendAuthorization(coins10, nil, s.accountKeeper.AddressCodec())

	start := s.ctx.HeaderInfo().Time
	expired := start.Add(time.Duration(1) * time.Second)
	notExpired := start.Add(time.Duration(5) * time.Hour)

	s.Require().NoError(s.authzKeeper.SaveGrant(s.ctx, addr1, addr2, genAuthMulti, nil), "creating grant 1->2")
	s.Require().NoError(s.authzKeeper.SaveGrant(s.ctx, addr1, addr3, genAuthSend, &expired), "creating grant 1->3")
	s.Require().NoError(s.authzKeeper.SaveGrant(s.ctx, addr1, addr4, sendAuth, &notExpired), "creating grant 1->4")
	// Without access to private keeper methods, I don't know how to save a grant with an invalid authorization.
	newCtx := s.ctx.WithHeaderInfo(header.Info{Time: start.Add(time.Duration(1) * time.Minute)})

	tests := []struct {
		name    string
		grantee sdk.AccAddress
		granter sdk.AccAddress
		msgType string
		expAuth authz.Authorization
		expExp  *time.Time
	}{
		{
			name:    "grant has nil exp and is returned",
			grantee: addr1,
			granter: addr2,
			msgType: genAuthMulti.MsgTypeURL(),
			expAuth: genAuthMulti,
			expExp:  nil,
		},
		{
			name:    "grant is expired not returned",
			grantee: addr1,
			granter: addr3,
			msgType: genAuthSend.MsgTypeURL(),
			expAuth: nil,
			expExp:  nil,
		},
		{
			name:    "grant is not expired and is returned",
			grantee: addr1,
			granter: addr4,
			msgType: sendAuth.MsgTypeURL(),
			expAuth: sendAuth,
			expExp:  &notExpired,
		},
		{
			name:    "grant is not expired but wrong msg type returns nil",
			grantee: addr1,
			granter: addr4,
			msgType: genAuthMulti.MsgTypeURL(),
			expAuth: nil,
			expExp:  nil,
		},
		{
			name:    "no grant exists between the two",
			grantee: addr2,
			granter: addr3,
			msgType: genAuthSend.MsgTypeURL(),
			expAuth: nil,
			expExp:  nil,
		},
	}

	for _, tc := range tests {
		s.Run(tc.name, func() {
			actAuth, actExp := s.authzKeeper.GetAuthorization(newCtx, tc.grantee, tc.granter, tc.msgType)
			s.Assert().Equal(tc.expAuth, actAuth, "authorization")
			s.Assert().Equal(tc.expExp, actExp, "expiration")
		})
	}
}

func (s *TestSuite) TestGetAuthorizations() {
	require := s.Require()
	addr1 := s.addrs[1]
	addr2 := s.addrs[2]

	genAuthMulti := authz.NewGenericAuthorization(sdk.MsgTypeURL(&banktypes.MsgMultiSend{}))
	genAuthSend := authz.NewGenericAuthorization(sdk.MsgTypeURL(&banktypes.MsgSend{}))

	start := s.ctx.HeaderInfo().Time
	expired := start.Add(time.Duration(1) * time.Second)

	s.Require().NoError(s.authzKeeper.SaveGrant(s.ctx, addr1, addr2, genAuthMulti, &expired), "creating multi send grant 1->2")
	s.Require().NoError(s.authzKeeper.SaveGrant(s.ctx, addr1, addr2, genAuthSend, &expired), "creating send grant 1->2")

	authzs, err := s.authzKeeper.GetAuthorizations(s.ctx, addr1, addr2)
	require.NoError(err)
	require.Len(authzs, 2)
	require.Equal(sdk.MsgTypeURL(&banktypes.MsgMultiSend{}), authzs[0].MsgTypeURL())
	require.Equal(sdk.MsgTypeURL(&banktypes.MsgSend{}), authzs[1].MsgTypeURL())
}

func TestTestSuite(t *testing.T) {
	suite.Run(t, new(TestSuite))
}
