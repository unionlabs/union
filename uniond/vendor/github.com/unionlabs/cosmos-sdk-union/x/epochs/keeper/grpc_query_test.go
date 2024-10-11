package keeper_test

import (
	"cosmossdk.io/x/epochs/types"
)

func (s *KeeperTestSuite) TestQueryEpochInfos() {
	s.SetupTest()
	queryClient := s.queryClient

	// Check that querying epoch infos on default genesis returns the default genesis epoch infos
	epochInfosResponse, err := queryClient.EpochInfos(s.Ctx, &types.QueryEpochsInfoRequest{})
	s.Require().NoError(err)
	s.Require().Len(epochInfosResponse.Epochs, 4)
	expectedEpochs := types.DefaultGenesis().Epochs
	for id := range expectedEpochs {
		expectedEpochs[id].StartTime = s.Ctx.BlockTime()
		expectedEpochs[id].CurrentEpochStartHeight = s.Ctx.BlockHeight()
	}

	s.Require().Equal(expectedEpochs, epochInfosResponse.Epochs)
}
