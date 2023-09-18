package keeper_test

import (
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"

	"cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/x/staking/testutil"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

// IsValSetSorted reports whether valset is sorted.
func IsValSetSorted(data []stakingtypes.Validator, powerReduction math.Int) bool {
	n := len(data)
	for i := n - 1; i > 0; i-- {
		if stakingtypes.ValidatorsByVotingPower(data).Less(i, i-1, powerReduction) {
			return false
		}
	}
	return true
}

func (s *KeeperTestSuite) TestHistoricalInfo() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	_, addrVals := createValAddrs(50)

	validators := make([]stakingtypes.Validator, len(addrVals))

	for i, valAddr := range addrVals {
		validators[i] = testutil.NewValidator(s.T(), valAddr, PKs[i])
	}

	hi := stakingtypes.NewHistoricalInfo(ctx.BlockHeader(), validators, keeper.PowerReduction(ctx))
	keeper.SetHistoricalInfo(ctx, 2, &hi)

	recv, found := keeper.GetHistoricalInfo(ctx, 2)
	require.True(found, "HistoricalInfo not found after set")
	require.Equal(hi, recv, "HistoricalInfo not equal")
	require.True(IsValSetSorted(recv.Valset, keeper.PowerReduction(ctx)), "HistoricalInfo validators is not sorted")

	keeper.DeleteHistoricalInfo(ctx, 2)

	recv, found = keeper.GetHistoricalInfo(ctx, 2)
	require.False(found, "HistoricalInfo found after delete")
	require.Equal(stakingtypes.HistoricalInfo{}, recv, "HistoricalInfo is not empty")
}

func (s *KeeperTestSuite) TestTrackHistoricalInfo() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	_, addrVals := createValAddrs(50)

	// set historical entries in params to 5
	params := stakingtypes.DefaultParams()
	params.HistoricalEntries = 5
	keeper.SetParams(ctx, params)

	// set historical info at 5, 4 which should be pruned
	// and check that it has been stored
	h4 := tmproto.Header{
		ChainID: "HelloChain",
		Height:  4,
	}
	h5 := tmproto.Header{
		ChainID: "HelloChain",
		Height:  5,
	}
	valSet := []stakingtypes.Validator{
		testutil.NewValidator(s.T(), addrVals[0], PKs[0]),
		testutil.NewValidator(s.T(), addrVals[1], PKs[1]),
	}
	hi4 := stakingtypes.NewHistoricalInfo(h4, valSet, keeper.PowerReduction(ctx))
	hi5 := stakingtypes.NewHistoricalInfo(h5, valSet, keeper.PowerReduction(ctx))
	keeper.SetHistoricalInfo(ctx, 4, &hi4)
	keeper.SetHistoricalInfo(ctx, 5, &hi5)
	recv, found := keeper.GetHistoricalInfo(ctx, 4)
	require.True(found)
	require.Equal(hi4, recv)
	recv, found = keeper.GetHistoricalInfo(ctx, 5)
	require.True(found)
	require.Equal(hi5, recv)

	// Set bonded validators in keeper
	val1 := testutil.NewValidator(s.T(), addrVals[2], PKs[2])
	val1.Status = stakingtypes.Bonded // when not bonded, consensus power is Zero
	val1.Tokens = keeper.TokensFromConsensusPower(ctx, 10)
	keeper.SetValidator(ctx, val1)
	keeper.SetLastValidatorPower(ctx, val1.GetOperator(), 10)
	val2 := testutil.NewValidator(s.T(), addrVals[3], PKs[3])
	val1.Status = stakingtypes.Bonded
	val2.Tokens = keeper.TokensFromConsensusPower(ctx, 80)
	keeper.SetValidator(ctx, val2)
	keeper.SetLastValidatorPower(ctx, val2.GetOperator(), 80)

	vals := []stakingtypes.Validator{val1, val2}
	require.True(IsValSetSorted(vals, keeper.PowerReduction(ctx)))

	// Set Header for BeginBlock context
	header := tmproto.Header{
		ChainID: "HelloChain",
		Height:  10,
	}
	ctx = ctx.WithBlockHeader(header)

	keeper.TrackHistoricalInfo(ctx)

	// Check HistoricalInfo at height 10 is persisted
	expected := stakingtypes.HistoricalInfo{
		Header: header,
		Valset: vals,
	}
	recv, found = keeper.GetHistoricalInfo(ctx, 10)
	require.True(found, "GetHistoricalInfo failed after BeginBlock")
	require.Equal(expected, recv, "GetHistoricalInfo returned unexpected result")

	// Check HistoricalInfo at height 5, 4 is pruned
	recv, found = keeper.GetHistoricalInfo(ctx, 4)
	require.False(found, "GetHistoricalInfo did not prune earlier height")
	require.Equal(stakingtypes.HistoricalInfo{}, recv, "GetHistoricalInfo at height 4 is not empty after prune")
	recv, found = keeper.GetHistoricalInfo(ctx, 5)
	require.False(found, "GetHistoricalInfo did not prune first prune height")
	require.Equal(stakingtypes.HistoricalInfo{}, recv, "GetHistoricalInfo at height 5 is not empty after prune")
}

func (s *KeeperTestSuite) TestGetAllHistoricalInfo() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	_, addrVals := createValAddrs(50)

	valSet := []stakingtypes.Validator{
		testutil.NewValidator(s.T(), addrVals[0], PKs[0]),
		testutil.NewValidator(s.T(), addrVals[1], PKs[1]),
	}

	header1 := tmproto.Header{ChainID: "HelloChain", Height: 10}
	header2 := tmproto.Header{ChainID: "HelloChain", Height: 11}
	header3 := tmproto.Header{ChainID: "HelloChain", Height: 12}

	hist1 := stakingtypes.HistoricalInfo{Header: header1, Valset: valSet}
	hist2 := stakingtypes.HistoricalInfo{Header: header2, Valset: valSet}
	hist3 := stakingtypes.HistoricalInfo{Header: header3, Valset: valSet}

	expHistInfos := []stakingtypes.HistoricalInfo{hist1, hist2, hist3}

	for i, hi := range expHistInfos {
		keeper.SetHistoricalInfo(ctx, int64(10+i), &hi)
	}

	infos := keeper.GetAllHistoricalInfo(ctx)
	require.Equal(expHistInfos, infos)
}
