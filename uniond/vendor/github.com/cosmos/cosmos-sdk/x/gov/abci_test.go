package gov_test

import (
	"testing"
	"time"

	"cosmossdk.io/math"
	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"

	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/cosmos/cosmos-sdk/x/gov"
	"github.com/cosmos/cosmos-sdk/x/gov/keeper"
	"github.com/cosmos/cosmos-sdk/x/gov/types"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
	"github.com/cosmos/cosmos-sdk/x/staking"
	stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

func TestTickExpiredDepositPeriod(t *testing.T) {
	suite := createTestSuite(t)
	app := suite.App
	ctx := app.BaseApp.NewContext(false, tmproto.Header{})
	addrs := simtestutil.AddTestAddrs(suite.BankKeeper, suite.StakingKeeper, ctx, 10, valTokens)

	header := tmproto.Header{Height: app.LastBlockHeight() + 1}
	app.BeginBlock(abci.RequestBeginBlock{Header: header})

	govMsgSvr := keeper.NewMsgServerImpl(suite.GovKeeper)

	inactiveQueue := suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, inactiveQueue.Valid())
	inactiveQueue.Close()

	newProposalMsg, err := v1.NewMsgSubmitProposal(
		[]sdk.Msg{mkTestLegacyContent(t)},
		sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 5)},
		addrs[0].String(),
		"",
		"Proposal",
		"description of proposal",
	)
	require.NoError(t, err)

	res, err := govMsgSvr.SubmitProposal(sdk.WrapSDKContext(ctx), newProposalMsg)
	require.NoError(t, err)
	require.NotNil(t, res)

	inactiveQueue = suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, inactiveQueue.Valid())
	inactiveQueue.Close()

	newHeader := ctx.BlockHeader()
	newHeader.Time = ctx.BlockHeader().Time.Add(time.Duration(1) * time.Second)
	ctx = ctx.WithBlockHeader(newHeader)

	inactiveQueue = suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, inactiveQueue.Valid())
	inactiveQueue.Close()

	newHeader = ctx.BlockHeader()
	newHeader.Time = ctx.BlockHeader().Time.Add(*suite.GovKeeper.GetParams(ctx).MaxDepositPeriod)
	ctx = ctx.WithBlockHeader(newHeader)

	inactiveQueue = suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.True(t, inactiveQueue.Valid())
	inactiveQueue.Close()

	gov.EndBlocker(ctx, suite.GovKeeper)

	inactiveQueue = suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, inactiveQueue.Valid())
	inactiveQueue.Close()
}

func TestTickMultipleExpiredDepositPeriod(t *testing.T) {
	suite := createTestSuite(t)
	app := suite.App
	ctx := app.BaseApp.NewContext(false, tmproto.Header{})
	addrs := simtestutil.AddTestAddrs(suite.BankKeeper, suite.StakingKeeper, ctx, 10, valTokens)

	header := tmproto.Header{Height: app.LastBlockHeight() + 1}
	app.BeginBlock(abci.RequestBeginBlock{Header: header})

	govMsgSvr := keeper.NewMsgServerImpl(suite.GovKeeper)

	inactiveQueue := suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, inactiveQueue.Valid())
	inactiveQueue.Close()

	newProposalMsg, err := v1.NewMsgSubmitProposal(
		[]sdk.Msg{mkTestLegacyContent(t)},
		sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 5)},
		addrs[0].String(),
		"",
		"Proposal",
		"description of proposal",
	)
	require.NoError(t, err)

	res, err := govMsgSvr.SubmitProposal(sdk.WrapSDKContext(ctx), newProposalMsg)
	require.NoError(t, err)
	require.NotNil(t, res)

	inactiveQueue = suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, inactiveQueue.Valid())
	inactiveQueue.Close()

	newHeader := ctx.BlockHeader()
	newHeader.Time = ctx.BlockHeader().Time.Add(time.Duration(2) * time.Second)
	ctx = ctx.WithBlockHeader(newHeader)

	inactiveQueue = suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, inactiveQueue.Valid())
	inactiveQueue.Close()

	newProposalMsg2, err := v1.NewMsgSubmitProposal(
		[]sdk.Msg{mkTestLegacyContent(t)},
		sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 5)},
		addrs[0].String(),
		"",
		"Proposal",
		"description of proposal",
	)
	require.NoError(t, err)

	res, err = govMsgSvr.SubmitProposal(sdk.WrapSDKContext(ctx), newProposalMsg2)
	require.NoError(t, err)
	require.NotNil(t, res)

	newHeader = ctx.BlockHeader()
	newHeader.Time = ctx.BlockHeader().Time.Add(*suite.GovKeeper.GetParams(ctx).MaxDepositPeriod).Add(time.Duration(-1) * time.Second)
	ctx = ctx.WithBlockHeader(newHeader)

	inactiveQueue = suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.True(t, inactiveQueue.Valid())
	inactiveQueue.Close()

	gov.EndBlocker(ctx, suite.GovKeeper)

	inactiveQueue = suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, inactiveQueue.Valid())
	inactiveQueue.Close()

	newHeader = ctx.BlockHeader()
	newHeader.Time = ctx.BlockHeader().Time.Add(time.Duration(5) * time.Second)
	ctx = ctx.WithBlockHeader(newHeader)

	inactiveQueue = suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.True(t, inactiveQueue.Valid())
	inactiveQueue.Close()

	gov.EndBlocker(ctx, suite.GovKeeper)

	inactiveQueue = suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, inactiveQueue.Valid())
	inactiveQueue.Close()
}

func TestTickPassedDepositPeriod(t *testing.T) {
	suite := createTestSuite(t)
	app := suite.App
	ctx := app.BaseApp.NewContext(false, tmproto.Header{})
	addrs := simtestutil.AddTestAddrs(suite.BankKeeper, suite.StakingKeeper, ctx, 10, valTokens)

	header := tmproto.Header{Height: app.LastBlockHeight() + 1}
	app.BeginBlock(abci.RequestBeginBlock{Header: header})

	govMsgSvr := keeper.NewMsgServerImpl(suite.GovKeeper)

	inactiveQueue := suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, inactiveQueue.Valid())
	inactiveQueue.Close()
	activeQueue := suite.GovKeeper.ActiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, activeQueue.Valid())
	activeQueue.Close()

	newProposalMsg, err := v1.NewMsgSubmitProposal(
		[]sdk.Msg{mkTestLegacyContent(t)},
		sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 5)},
		addrs[0].String(),
		"",
		"Proposal",
		"description of proposal",
	)
	require.NoError(t, err)

	res, err := govMsgSvr.SubmitProposal(sdk.WrapSDKContext(ctx), newProposalMsg)
	require.NoError(t, err)
	require.NotNil(t, res)

	proposalID := res.ProposalId

	inactiveQueue = suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, inactiveQueue.Valid())
	inactiveQueue.Close()

	newHeader := ctx.BlockHeader()
	newHeader.Time = ctx.BlockHeader().Time.Add(time.Duration(1) * time.Second)
	ctx = ctx.WithBlockHeader(newHeader)

	inactiveQueue = suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, inactiveQueue.Valid())
	inactiveQueue.Close()

	newDepositMsg := v1.NewMsgDeposit(addrs[1], proposalID, sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 5)})

	res1, err := govMsgSvr.Deposit(sdk.WrapSDKContext(ctx), newDepositMsg)
	require.NoError(t, err)
	require.NotNil(t, res1)

	activeQueue = suite.GovKeeper.ActiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, activeQueue.Valid())
	activeQueue.Close()
}

func TestTickPassedVotingPeriod(t *testing.T) {
	suite := createTestSuite(t)
	app := suite.App
	ctx := app.BaseApp.NewContext(false, tmproto.Header{})
	addrs := simtestutil.AddTestAddrs(suite.BankKeeper, suite.StakingKeeper, ctx, 10, valTokens)

	SortAddresses(addrs)

	header := tmproto.Header{Height: app.LastBlockHeight() + 1}
	app.BeginBlock(abci.RequestBeginBlock{Header: header})

	govMsgSvr := keeper.NewMsgServerImpl(suite.GovKeeper)

	inactiveQueue := suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, inactiveQueue.Valid())
	inactiveQueue.Close()
	activeQueue := suite.GovKeeper.ActiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, activeQueue.Valid())
	activeQueue.Close()

	proposalCoins := sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, suite.StakingKeeper.TokensFromConsensusPower(ctx, 5))}
	newProposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{mkTestLegacyContent(t)}, proposalCoins, addrs[0].String(), "", "Proposal", "description of proposal")
	require.NoError(t, err)

	wrapCtx := sdk.WrapSDKContext(ctx)

	res, err := govMsgSvr.SubmitProposal(wrapCtx, newProposalMsg)
	require.NoError(t, err)
	require.NotNil(t, res)

	proposalID := res.ProposalId

	newHeader := ctx.BlockHeader()
	newHeader.Time = ctx.BlockHeader().Time.Add(time.Duration(1) * time.Second)
	ctx = ctx.WithBlockHeader(newHeader)

	newDepositMsg := v1.NewMsgDeposit(addrs[1], proposalID, proposalCoins)

	res1, err := govMsgSvr.Deposit(wrapCtx, newDepositMsg)
	require.NoError(t, err)
	require.NotNil(t, res1)

	newHeader = ctx.BlockHeader()
	newHeader.Time = ctx.BlockHeader().Time.Add(*suite.GovKeeper.GetParams(ctx).MaxDepositPeriod).Add(*suite.GovKeeper.GetParams(ctx).VotingPeriod)
	ctx = ctx.WithBlockHeader(newHeader)

	inactiveQueue = suite.GovKeeper.InactiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, inactiveQueue.Valid())
	inactiveQueue.Close()

	activeQueue = suite.GovKeeper.ActiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.True(t, activeQueue.Valid())

	activeProposalID := types.GetProposalIDFromBytes(activeQueue.Value())
	proposal, ok := suite.GovKeeper.GetProposal(ctx, activeProposalID)
	require.True(t, ok)
	require.Equal(t, v1.StatusVotingPeriod, proposal.Status)

	activeQueue.Close()

	gov.EndBlocker(ctx, suite.GovKeeper)

	activeQueue = suite.GovKeeper.ActiveProposalQueueIterator(ctx, ctx.BlockHeader().Time)
	require.False(t, activeQueue.Valid())
	activeQueue.Close()
}

func TestProposalPassedEndblocker(t *testing.T) {
	suite := createTestSuite(t)
	app := suite.App
	ctx := app.BaseApp.NewContext(false, tmproto.Header{})
	addrs := simtestutil.AddTestAddrs(suite.BankKeeper, suite.StakingKeeper, ctx, 10, valTokens)

	SortAddresses(addrs)

	govMsgSvr := keeper.NewMsgServerImpl(suite.GovKeeper)
	stakingMsgSvr := stakingkeeper.NewMsgServerImpl(suite.StakingKeeper)

	header := tmproto.Header{Height: app.LastBlockHeight() + 1}
	app.BeginBlock(abci.RequestBeginBlock{Header: header})

	valAddr := sdk.ValAddress(addrs[0])

	createValidators(t, stakingMsgSvr, ctx, []sdk.ValAddress{valAddr}, []int64{10})
	staking.EndBlocker(ctx, suite.StakingKeeper)

	macc := suite.GovKeeper.GetGovernanceAccount(ctx)
	require.NotNil(t, macc)
	initialModuleAccCoins := suite.BankKeeper.GetAllBalances(ctx, macc.GetAddress())

	proposal, err := suite.GovKeeper.SubmitProposal(ctx, []sdk.Msg{mkTestLegacyContent(t)}, "", "title", "summary", addrs[0])
	require.NoError(t, err)

	proposalCoins := sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, suite.StakingKeeper.TokensFromConsensusPower(ctx, 10))}
	newDepositMsg := v1.NewMsgDeposit(addrs[0], proposal.Id, proposalCoins)

	res, err := govMsgSvr.Deposit(sdk.WrapSDKContext(ctx), newDepositMsg)
	require.NoError(t, err)
	require.NotNil(t, res)

	macc = suite.GovKeeper.GetGovernanceAccount(ctx)
	require.NotNil(t, macc)
	moduleAccCoins := suite.BankKeeper.GetAllBalances(ctx, macc.GetAddress())

	deposits := initialModuleAccCoins.Add(proposal.TotalDeposit...).Add(proposalCoins...)
	require.True(t, moduleAccCoins.IsEqual(deposits))

	err = suite.GovKeeper.AddVote(ctx, proposal.Id, addrs[0], v1.NewNonSplitVoteOption(v1.OptionYes), "")
	require.NoError(t, err)

	newHeader := ctx.BlockHeader()
	newHeader.Time = ctx.BlockHeader().Time.Add(*suite.GovKeeper.GetParams(ctx).MaxDepositPeriod).Add(*suite.GovKeeper.GetParams(ctx).VotingPeriod)
	ctx = ctx.WithBlockHeader(newHeader)

	gov.EndBlocker(ctx, suite.GovKeeper)

	macc = suite.GovKeeper.GetGovernanceAccount(ctx)
	require.NotNil(t, macc)
	require.True(t, suite.BankKeeper.GetAllBalances(ctx, macc.GetAddress()).IsEqual(initialModuleAccCoins))
}

func TestEndBlockerProposalHandlerFailed(t *testing.T) {
	suite := createTestSuite(t)
	app := suite.App
	ctx := app.BaseApp.NewContext(false, tmproto.Header{})
	addrs := simtestutil.AddTestAddrs(suite.BankKeeper, suite.StakingKeeper, ctx, 1, valTokens)

	SortAddresses(addrs)

	stakingMsgSvr := stakingkeeper.NewMsgServerImpl(suite.StakingKeeper)
	header := tmproto.Header{Height: app.LastBlockHeight() + 1}
	app.BeginBlock(abci.RequestBeginBlock{Header: header})

	valAddr := sdk.ValAddress(addrs[0])

	createValidators(t, stakingMsgSvr, ctx, []sdk.ValAddress{valAddr}, []int64{10})
	staking.EndBlocker(ctx, suite.StakingKeeper)

	msg := banktypes.NewMsgSend(authtypes.NewModuleAddress(types.ModuleName), addrs[0], sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdk.NewInt(100000))))
	proposal, err := suite.GovKeeper.SubmitProposal(ctx, []sdk.Msg{msg}, "", "Bank Msg Send", "send message", addrs[0])
	require.NoError(t, err)

	proposalCoins := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, suite.StakingKeeper.TokensFromConsensusPower(ctx, 10)))
	newDepositMsg := v1.NewMsgDeposit(addrs[0], proposal.Id, proposalCoins)

	govMsgSvr := keeper.NewMsgServerImpl(suite.GovKeeper)
	res, err := govMsgSvr.Deposit(sdk.WrapSDKContext(ctx), newDepositMsg)
	require.NoError(t, err)
	require.NotNil(t, res)

	err = suite.GovKeeper.AddVote(ctx, proposal.Id, addrs[0], v1.NewNonSplitVoteOption(v1.OptionYes), "")
	require.NoError(t, err)

	newHeader := ctx.BlockHeader()
	newHeader.Time = ctx.BlockHeader().Time.Add(*suite.GovKeeper.GetParams(ctx).MaxDepositPeriod).Add(*suite.GovKeeper.GetParams(ctx).VotingPeriod)
	ctx = ctx.WithBlockHeader(newHeader)

	// validate that the proposal fails/has been rejected
	gov.EndBlocker(ctx, suite.GovKeeper)

	proposal, ok := suite.GovKeeper.GetProposal(ctx, proposal.Id)
	require.True(t, ok)
	require.Equal(t, v1.StatusFailed, proposal.Status)
}

func createValidators(t *testing.T, stakingMsgSvr stakingtypes.MsgServer, ctx sdk.Context, addrs []sdk.ValAddress, powerAmt []int64) {
	require.True(t, len(addrs) <= len(pubkeys), "Not enough pubkeys specified at top of file.")

	for i := 0; i < len(addrs); i++ {
		valTokens := sdk.TokensFromConsensusPower(powerAmt[i], sdk.DefaultPowerReduction)
		valCreateMsg, err := stakingtypes.NewMsgCreateValidator(
			addrs[i], pubkeys[i], sdk.NewCoin(sdk.DefaultBondDenom, valTokens),
			TestDescription, TestCommissionRates, math.OneInt(),
		)
		require.NoError(t, err)
		res, err := stakingMsgSvr.CreateValidator(sdk.WrapSDKContext(ctx), valCreateMsg)
		require.NoError(t, err)
		require.NotNil(t, res)
	}
}
