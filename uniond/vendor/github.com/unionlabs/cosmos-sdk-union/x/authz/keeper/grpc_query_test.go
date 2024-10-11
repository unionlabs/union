package keeper_test

import (
	gocontext "context"
	"fmt"
	"time"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/x/authz"
	banktypes "cosmossdk.io/x/bank/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
)

func (suite *TestSuite) TestGRPCQueryAuthorization() {
	queryClient, addrs := suite.queryClient, suite.addrs
	var (
		req              *authz.QueryGrantsRequest
		expAuthorization authz.Authorization
	)

	addr0, err := suite.accountKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)
	addr1, err := suite.accountKeeper.AddressCodec().BytesToString(addrs[1])
	suite.Require().NoError(err)

	testCases := []struct {
		msg      string
		malleate func(require *require.Assertions)
		expError string
		postTest func(require *require.Assertions, res *authz.QueryGrantsResponse)
	}{
		{
			"fail invalid granter addr",
			func(require *require.Assertions) {
				req = &authz.QueryGrantsRequest{}
			},
			"empty address string is not allowed",
			func(require *require.Assertions, res *authz.QueryGrantsResponse) {},
		},
		{
			"fail invalid grantee addr",
			func(require *require.Assertions) {
				req = &authz.QueryGrantsRequest{
					Granter: addr0,
				}
			},
			"empty address string is not allowed",
			func(require *require.Assertions, res *authz.QueryGrantsResponse) {},
		},
		{
			"fail invalid msg-type",
			func(require *require.Assertions) {
				req = &authz.QueryGrantsRequest{
					Granter:    addr0,
					Grantee:    addr1,
					MsgTypeUrl: "unknown",
				}
			},
			"authorization not found for unknown type",
			func(require *require.Assertions, res *authz.QueryGrantsResponse) {},
		},
		{
			"authorization not found",
			func(require *require.Assertions) {
				req = &authz.QueryGrantsRequest{
					Granter:    addr1,
					Grantee:    addr0,
					MsgTypeUrl: banktypes.SendAuthorization{}.MsgTypeURL(),
				}
			},
			"authorization not found for /cosmos.bank.v1beta1.MsgSend",
			func(require *require.Assertions, res *authz.QueryGrantsResponse) {},
		},
		{
			"success",
			func(require *require.Assertions) {
				expAuthorization = suite.createSendAuthorization(addrs[0], addrs[1])
				req = &authz.QueryGrantsRequest{
					Granter:    addr1,
					Grantee:    addr0,
					MsgTypeUrl: expAuthorization.MsgTypeURL(),
				}
			},
			"",
			func(require *require.Assertions, res *authz.QueryGrantsResponse) {
				var auth authz.Authorization
				require.Equal(1, len(res.Grants))
				err := suite.encCfg.InterfaceRegistry.UnpackAny(res.Grants[0].Authorization, &auth)
				require.NoError(err)
				require.NotNil(auth)
				require.Equal(auth.String(), expAuthorization.String())
			},
		},
		{
			"success with allow list",
			func(require *require.Assertions) {
				expAuthorization = suite.createSendAuthorizationWithAllowList(addrs[0], addrs[1])
				require.Len(expAuthorization.(*banktypes.SendAuthorization).GetAllowList(), 1)
				req = &authz.QueryGrantsRequest{
					Granter:    addr1,
					Grantee:    addr0,
					MsgTypeUrl: expAuthorization.MsgTypeURL(),
				}
			},
			"",
			func(require *require.Assertions, res *authz.QueryGrantsResponse) {
				var auth authz.Authorization
				require.Equal(1, len(res.Grants))
				err := suite.encCfg.InterfaceRegistry.UnpackAny(res.Grants[0].Authorization, &auth)
				require.NoError(err)
				require.NotNil(auth)
				require.Equal(auth.String(), expAuthorization.String())
				require.Equal(auth.(*banktypes.SendAuthorization).GetAllowList(), expAuthorization.(*banktypes.SendAuthorization).GetAllowList())
			},
		},
	}
	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			require := suite.Require()
			tc.malleate(require)
			result, err := queryClient.Grants(gocontext.Background(), req)
			if tc.expError == "" {
				require.NoError(err)
			} else {
				require.Error(err)
				require.Contains(err.Error(), tc.expError)
			}
			tc.postTest(require, result)
		})
	}
}

func (suite *TestSuite) TestGRPCQueryGranterGrants() {
	require := suite.Require()
	queryClient, addrs := suite.queryClient, suite.addrs

	addr0, err := suite.accountKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)

	testCases := []struct {
		msg      string
		preRun   func()
		expError bool
		request  authz.QueryGranterGrantsRequest
		numItems int
	}{
		{
			"fail invalid granter addr",
			func() {},
			true,
			authz.QueryGranterGrantsRequest{},
			0,
		},
		{
			"valid case, single authorization",
			func() {
				suite.createSendAuthorization(addrs[1], addrs[0])
			},
			false,
			authz.QueryGranterGrantsRequest{
				Granter: addr0,
			},
			1,
		},
		{
			"valid case, multiple authorization",
			func() {
				suite.createSendAuthorization(addrs[2], addrs[0])
			},
			false,
			authz.QueryGranterGrantsRequest{
				Granter: addr0,
			},
			2,
		},
		{
			"valid case, pagination",
			func() {
			},
			false,
			authz.QueryGranterGrantsRequest{
				Granter: addr0,
				Pagination: &query.PageRequest{
					Limit: 1,
				},
			},
			1,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			tc.preRun()
			result, err := queryClient.GranterGrants(gocontext.Background(), &tc.request)
			if tc.expError {
				require.Error(err)
			} else {
				require.NoError(err)
				require.Len(result.Grants, tc.numItems)
			}
		})
	}
}

func (suite *TestSuite) TestGRPCQueryGranteeGrants() {
	require := suite.Require()
	queryClient, addrs := suite.queryClient, suite.addrs

	addr0, err := suite.accountKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)
	addr2, err := suite.accountKeeper.AddressCodec().BytesToString(addrs[2])
	suite.Require().NoError(err)

	testCases := []struct {
		msg      string
		preRun   func()
		expError bool
		request  authz.QueryGranteeGrantsRequest
		numItems int
	}{
		{
			"fail invalid granter addr",
			func() {},
			true,
			authz.QueryGranteeGrantsRequest{},
			0,
		},
		{
			"valid case, single authorization",
			func() {
				suite.createSendAuthorization(addrs[0], addrs[1])
			},
			false,
			authz.QueryGranteeGrantsRequest{
				Grantee: addr0,
			},
			1,
		},
		{
			"valid case, no authorization found",
			func() {},
			false,
			authz.QueryGranteeGrantsRequest{
				Grantee: addr2,
			},
			0,
		},
		{
			"valid case, multiple authorization",
			func() {
				suite.createSendAuthorization(addrs[0], addrs[2])
			},
			false,
			authz.QueryGranteeGrantsRequest{
				Grantee: addr0,
			},
			2,
		},
		{
			"valid case, pagination",
			func() {},
			false,
			authz.QueryGranteeGrantsRequest{
				Grantee: addr0,
				Pagination: &query.PageRequest{
					Limit: 1,
				},
			},
			1,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			tc.preRun()
			result, err := queryClient.GranteeGrants(gocontext.Background(), &tc.request)
			if tc.expError {
				require.Error(err)
			} else {
				require.NoError(err)
				require.Len(result.Grants, tc.numItems)
			}
		})
	}
}

func (suite *TestSuite) createSendAuthorization(grantee, granter sdk.AccAddress) authz.Authorization {
	exp := suite.ctx.HeaderInfo().Time.Add(time.Hour)
	newCoins := sdk.NewCoins(sdk.NewInt64Coin("steak", 100))
	authorization := &banktypes.SendAuthorization{SpendLimit: newCoins}
	err := suite.authzKeeper.SaveGrant(suite.ctx, grantee, granter, authorization, &exp)
	suite.Require().NoError(err)
	return authorization
}

func (suite *TestSuite) createSendAuthorizationWithAllowList(grantee, granter sdk.AccAddress) authz.Authorization {
	exp := suite.ctx.HeaderInfo().Time.Add(time.Hour)
	newCoins := sdk.NewCoins(sdk.NewInt64Coin("steak", 100))
	addr, err := suite.accountKeeper.AddressCodec().BytesToString(suite.addrs[5])
	suite.Require().NoError(err)
	authorization := &banktypes.SendAuthorization{SpendLimit: newCoins, AllowList: []string{addr}}
	err = suite.authzKeeper.SaveGrant(suite.ctx, grantee, granter, authorization, &exp)
	suite.Require().NoError(err)
	return authorization
}
