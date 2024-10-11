package keeper_test

import (
	banktypes "cosmossdk.io/x/bank/types"
	stakingtypes "cosmossdk.io/x/staking/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/keeper"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
)

func (suite *KeeperTestSuite) TestModuleQuerySafe() {
	var (
		msg          *types.MsgModuleQuerySafe
		expResponses [][]byte
	)
	testCases := []struct {
		name     string
		malleate func()
		expErr   error
	}{
		{
			"success",
			func() {
				balanceQueryBz, err := banktypes.NewQueryBalanceRequest(suite.chainA.SenderAccount.GetAddress().String(), sdk.DefaultBondDenom).Marshal()
				suite.Require().NoError(err)

				queryReq := types.QueryRequest{
					Path: "/cosmos.bank.v1beta1.Query/Balance",
					Data: balanceQueryBz,
				}

				msg = types.NewMsgModuleQuerySafe(suite.chainA.GetSimApp().ICAHostKeeper.GetAuthority(), []*types.QueryRequest{&queryReq})

				balance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress(), sdk.DefaultBondDenom)

				expResp := banktypes.QueryBalanceResponse{Balance: &balance}
				expRespBz, err := expResp.Marshal()
				suite.Require().NoError(err)

				expResponses = [][]byte{expRespBz}
			},
			nil,
		},
		{
			"success: multiple queries",
			func() {
				balanceQueryBz, err := banktypes.NewQueryBalanceRequest(suite.chainA.SenderAccount.GetAddress().String(), sdk.DefaultBondDenom).Marshal()
				suite.Require().NoError(err)

				queryReq := types.QueryRequest{
					Path: "/cosmos.bank.v1beta1.Query/Balance",
					Data: balanceQueryBz,
				}

				paramsQuery := stakingtypes.QueryParamsRequest{}
				paramsQueryBz, err := paramsQuery.Marshal()
				suite.Require().NoError(err)

				paramsQueryReq := types.QueryRequest{
					Path: "/cosmos.staking.v1beta1.Query/Params",
					Data: paramsQueryBz,
				}

				msg = types.NewMsgModuleQuerySafe(suite.chainA.GetSimApp().ICAHostKeeper.GetAuthority(), []*types.QueryRequest{&queryReq, &paramsQueryReq})

				balance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress(), sdk.DefaultBondDenom)

				expResp := banktypes.QueryBalanceResponse{Balance: &balance}
				expRespBz, err := expResp.Marshal()
				suite.Require().NoError(err)

				params, err := suite.chainA.GetSimApp().StakingKeeper.Params.Get(suite.chainA.GetContext())
				suite.Require().NoError(err)
				expParamsResp := stakingtypes.QueryParamsResponse{Params: params}
				expParamsRespBz, err := expParamsResp.Marshal()
				suite.Require().NoError(err)

				expResponses = [][]byte{expRespBz, expParamsRespBz}
			},
			nil,
		},
		{
			"failure: not module query safe",
			func() {
				balanceQueryBz, err := banktypes.NewQueryBalanceRequest(suite.chainA.SenderAccount.GetAddress().String(), sdk.DefaultBondDenom).Marshal()
				suite.Require().NoError(err)

				queryReq := types.QueryRequest{
					Path: "/cosmos.bank.v1beta1.Query/Balance",
					Data: balanceQueryBz,
				}

				paramsQuery := transfertypes.QueryParamsRequest{}
				paramsQueryBz, err := paramsQuery.Marshal()
				suite.Require().NoError(err)

				paramsQueryReq := types.QueryRequest{
					Path: "/ibc.applications.transfer.v1.Query/Params",
					Data: paramsQueryBz,
				}

				msg = types.NewMsgModuleQuerySafe(suite.chainA.GetSimApp().ICAHostKeeper.GetAuthority(), []*types.QueryRequest{&queryReq, &paramsQueryReq})
			},
			ibcerrors.ErrInvalidRequest,
		},
		{
			"failure: invalid query path",
			func() {
				balanceQueryBz, err := banktypes.NewQueryBalanceRequest(suite.chainA.SenderAccount.GetAddress().String(), sdk.DefaultBondDenom).Marshal()
				suite.Require().NoError(err)

				queryReq := types.QueryRequest{
					Path: "/cosmos.invalid.Query/Invalid",
					Data: balanceQueryBz,
				}

				msg = types.NewMsgModuleQuerySafe(suite.chainA.GetSimApp().ICAHostKeeper.GetAuthority(), []*types.QueryRequest{&queryReq})
			},
			ibcerrors.ErrInvalidRequest,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest()

			// reset
			msg = nil
			expResponses = nil

			tc.malleate()

			ctx := suite.chainA.GetContext()
			msgServer := keeper.NewMsgServerImpl(&suite.chainA.GetSimApp().ICAHostKeeper)
			res, err := msgServer.ModuleQuerySafe(ctx, msg)

			if tc.expErr == nil {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)

				suite.Require().ElementsMatch(expResponses, res.Responses)
			} else {
				suite.Require().ErrorIs(err, tc.expErr)
				suite.Require().Nil(res)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestUpdateParams() {
	testCases := []struct {
		name    string
		msg     *types.MsgUpdateParams
		expPass bool
	}{
		{
			"success",
			types.NewMsgUpdateParams(suite.chainA.GetSimApp().ICAHostKeeper.GetAuthority(), types.DefaultParams()),
			true,
		},
		{
			"invalid signer address",
			types.NewMsgUpdateParams("signer", types.DefaultParams()),
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest()

			ctx := suite.chainA.GetContext()
			msgServer := keeper.NewMsgServerImpl(&suite.chainA.GetSimApp().ICAHostKeeper)
			res, err := msgServer.UpdateParams(ctx, tc.msg)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(res)
			}
		})
	}
}
