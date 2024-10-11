package keeper_test

import (
	gocontext "context"
	"time"

	"cosmossdk.io/x/slashing/testutil"
	slashingtypes "cosmossdk.io/x/slashing/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
)

func (s *KeeperTestSuite) TestGRPCQueryParams() {
	queryClient := s.queryClient
	require := s.Require()

	paramsResp, err := queryClient.Params(gocontext.Background(), &slashingtypes.QueryParamsRequest{})

	require.NoError(err)
	require.Equal(testutil.TestParams(), paramsResp.Params)
}

func (s *KeeperTestSuite) TestGRPCSigningInfo() {
	queryClient, ctx, keeper := s.queryClient, s.ctx, s.slashingKeeper
	require := s.Require()

	infoResp, err := queryClient.SigningInfo(gocontext.Background(), &slashingtypes.QuerySigningInfoRequest{ConsAddress: ""})
	require.Error(err)
	require.ErrorContains(err, "invalid request")
	require.Nil(infoResp)

	consStr, err := s.stakingKeeper.ConsensusAddressCodec().BytesToString(consAddr)
	require.NoError(err)

	signingInfo := slashingtypes.NewValidatorSigningInfo(
		consStr,
		0,
		time.Unix(2, 0),
		false,
		int64(0),
	)

	require.NoError(keeper.ValidatorSigningInfo.Set(ctx, consAddr, signingInfo))
	info, err := keeper.ValidatorSigningInfo.Get(ctx, consAddr)
	require.NoError(err)

	infoResp, err = queryClient.SigningInfo(gocontext.Background(),
		&slashingtypes.QuerySigningInfoRequest{ConsAddress: consStr})
	require.NoError(err)
	require.Equal(info, infoResp.ValSigningInfo)
}

func (s *KeeperTestSuite) TestGRPCSigningInfos() {
	queryClient, ctx, keeper := s.queryClient, s.ctx, s.slashingKeeper
	require := s.Require()

	// set two validator signing information
	consAddr1 := sdk.ConsAddress("addr1_______________")
	consStr1, err := s.stakingKeeper.ConsensusAddressCodec().BytesToString(consAddr1)
	require.NoError(err)
	consAddr2 := sdk.ConsAddress("addr2_______________")
	signingInfo := slashingtypes.NewValidatorSigningInfo(
		consStr1,
		0,
		time.Unix(2, 0),
		false,
		int64(0),
	)

	require.NoError(keeper.ValidatorSigningInfo.Set(ctx, consAddr1, signingInfo))
	signingInfo.Address = string(consAddr2)
	require.NoError(keeper.ValidatorSigningInfo.Set(ctx, consAddr2, signingInfo))
	var signingInfos []slashingtypes.ValidatorSigningInfo

	err = keeper.ValidatorSigningInfo.Walk(ctx, nil, func(consAddr sdk.ConsAddress, info slashingtypes.ValidatorSigningInfo) (stop bool, err error) {
		signingInfos = append(signingInfos, info)
		return false, nil
	})
	require.NoError(err)
	// verify all values are returned without pagination
	infoResp, err := queryClient.SigningInfos(gocontext.Background(),
		&slashingtypes.QuerySigningInfosRequest{Pagination: nil})
	require.NoError(err)
	require.Equal(signingInfos, infoResp.Info)

	infoResp, err = queryClient.SigningInfos(gocontext.Background(),
		&slashingtypes.QuerySigningInfosRequest{Pagination: &query.PageRequest{Limit: 1, CountTotal: true}})
	require.NoError(err)
	require.Len(infoResp.Info, 1)
	require.Equal(signingInfos[0], infoResp.Info[0])
	require.NotNil(infoResp.Pagination.NextKey)
	require.Equal(uint64(2), infoResp.Pagination.Total)
}
