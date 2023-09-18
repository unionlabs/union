package keeper_test

import (
	"fmt"
	"strings"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
	"github.com/cosmos/cosmos-sdk/x/evidence/exported"
	"github.com/cosmos/cosmos-sdk/x/evidence/types"
)

func (suite *KeeperTestSuite) TestQueryEvidence() {
	var (
		req      *types.QueryEvidenceRequest
		evidence []exported.Evidence
	)

	testCases := []struct {
		msg       string
		malleate  func()
		expPass   bool
		expErrMsg string
		posttests func(res *types.QueryEvidenceResponse)
	}{
		{
			"invalid request with empty evidence hash",
			func() {
				req = &types.QueryEvidenceRequest{Hash: ""}
			},
			false,
			"invalid request; hash is empty",
			func(res *types.QueryEvidenceResponse) {},
		},
		{
			"evidence not found",
			func() {
				numEvidence := 1
				evidence = suite.populateEvidence(suite.ctx, numEvidence)
				evidenceHash := evidence[0].Hash().String()
				reqHash := strings.Repeat("a", len(evidenceHash))
				req = types.NewQueryEvidenceRequest(reqHash)
			},
			false,
			"not found",
			func(res *types.QueryEvidenceResponse) {
			},
		},
		{
			"success",
			func() {
				numEvidence := 100
				evidence = suite.populateEvidence(suite.ctx, numEvidence)
				req = types.NewQueryEvidenceRequest(evidence[0].Hash().String())
			},
			true,
			"",
			func(res *types.QueryEvidenceResponse) {
				var evi exported.Evidence
				err := suite.encCfg.InterfaceRegistry.UnpackAny(res.Evidence, &evi)
				suite.Require().NoError(err)
				suite.Require().NotNil(evi)
				suite.Require().Equal(evi, evidence[0])
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest()

			tc.malleate()
			ctx := sdk.WrapSDKContext(suite.ctx)

			res, err := suite.queryClient.Evidence(ctx, req)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
			} else {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
				suite.Require().Nil(res)
			}

			tc.posttests(res)
		})
	}
}

func (suite *KeeperTestSuite) TestQueryAllEvidence() {
	var req *types.QueryAllEvidenceRequest

	testCases := []struct {
		msg       string
		malleate  func()
		expPass   bool
		posttests func(res *types.QueryAllEvidenceResponse)
	}{
		{
			"success without evidence",
			func() {
				req = &types.QueryAllEvidenceRequest{}
			},
			true,
			func(res *types.QueryAllEvidenceResponse) {
				suite.Require().Empty(res.Evidence)
			},
		},
		{
			"success",
			func() {
				numEvidence := 100
				_ = suite.populateEvidence(suite.ctx, numEvidence)
				pageReq := &query.PageRequest{
					Key:        nil,
					Limit:      50,
					CountTotal: false,
				}
				req = types.NewQueryAllEvidenceRequest(pageReq)
			},
			true,
			func(res *types.QueryAllEvidenceResponse) {
				suite.Equal(len(res.Evidence), 50)
				suite.NotNil(res.Pagination.NextKey)
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest()

			tc.malleate()
			ctx := sdk.WrapSDKContext(suite.ctx)

			res, err := suite.queryClient.AllEvidence(ctx, req)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(res)
			}

			tc.posttests(res)
		})
	}
}
