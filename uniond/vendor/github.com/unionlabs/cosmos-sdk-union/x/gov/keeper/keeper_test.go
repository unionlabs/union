package keeper_test

import (
	"testing"

	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/collections"
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/gov/keeper"
	govtestutil "cosmossdk.io/x/gov/testutil"
	"cosmossdk.io/x/gov/types"
	v1 "cosmossdk.io/x/gov/types/v1"
	"cosmossdk.io/x/gov/types/v1beta1"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/address"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var address1 = "cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"

type KeeperTestSuite struct {
	suite.Suite

	cdc               codec.Codec
	ctx               sdk.Context
	govKeeper         *keeper.Keeper
	acctKeeper        *govtestutil.MockAccountKeeper
	bankKeeper        *govtestutil.MockBankKeeper
	stakingKeeper     *govtestutil.MockStakingKeeper
	queryClient       v1.QueryClient
	legacyQueryClient v1beta1.QueryClient
	addrs             []sdk.AccAddress
	msgSrvr           v1.MsgServer
	legacyMsgSrvr     v1beta1.MsgServer
}

func (suite *KeeperTestSuite) SetupSuite() {
	suite.reset()
}

func (suite *KeeperTestSuite) reset() {
	govKeeper, mocks, encCfg, ctx := setupGovKeeper(suite.T())
	acctKeeper, bankKeeper, stakingKeeper := mocks.acctKeeper, mocks.bankKeeper, mocks.stakingKeeper

	// Populate the gov account with some coins, as the TestProposal we have
	// is a MsgSend from the gov account.
	coins := sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(1000000)))
	err := bankKeeper.MintCoins(suite.ctx, mintModuleName, coins)
	suite.NoError(err)
	err = bankKeeper.SendCoinsFromModuleToModule(ctx, mintModuleName, types.ModuleName, coins)
	suite.NoError(err)

	queryHelper := baseapp.NewQueryServerTestHelper(ctx, encCfg.InterfaceRegistry)
	v1.RegisterQueryServer(queryHelper, keeper.NewQueryServer(govKeeper))
	legacyQueryHelper := baseapp.NewQueryServerTestHelper(ctx, encCfg.InterfaceRegistry)
	v1beta1.RegisterQueryServer(legacyQueryHelper, keeper.NewLegacyQueryServer(govKeeper))
	queryClient := v1.NewQueryClient(queryHelper)
	legacyQueryClient := v1beta1.NewQueryClient(legacyQueryHelper)

	suite.ctx = ctx
	suite.govKeeper = govKeeper
	suite.acctKeeper = acctKeeper
	suite.bankKeeper = bankKeeper
	suite.stakingKeeper = stakingKeeper
	suite.cdc = encCfg.Codec
	suite.queryClient = queryClient
	suite.legacyQueryClient = legacyQueryClient
	suite.msgSrvr = keeper.NewMsgServerImpl(suite.govKeeper)

	govStrAcct, err := suite.acctKeeper.AddressCodec().BytesToString(govAcct)
	suite.Require().NoError(err)

	suite.legacyMsgSrvr = keeper.NewLegacyMsgServerImpl(govStrAcct, suite.msgSrvr)
	suite.addrs = simtestutil.AddTestAddrsIncremental(bankKeeper, stakingKeeper, ctx, 3, sdkmath.NewInt(300000000))

	suite.acctKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()
}

func TestIncrementProposalNumber(t *testing.T) {
	govKeeper, mocks, _, ctx := setupGovKeeper(t)
	authKeeper := mocks.acctKeeper

	authKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	ac := address.NewBech32Codec("cosmos")
	addrBz, err := ac.StringToBytes(address1)
	require.NoError(t, err)

	tp := TestProposal
	_, err = govKeeper.SubmitProposal(ctx, tp, "", "test", "summary", addrBz, v1.ProposalType_PROPOSAL_TYPE_STANDARD)
	require.NoError(t, err)
	_, err = govKeeper.SubmitProposal(ctx, tp, "", "test", "summary", addrBz, v1.ProposalType_PROPOSAL_TYPE_STANDARD)
	require.NoError(t, err)
	_, err = govKeeper.SubmitProposal(ctx, tp, "", "test", "summary", addrBz, v1.ProposalType_PROPOSAL_TYPE_EXPEDITED)
	require.NoError(t, err)
	_, err = govKeeper.SubmitProposal(ctx, tp, "", "test", "summary", addrBz, v1.ProposalType_PROPOSAL_TYPE_EXPEDITED)
	require.NoError(t, err)
	_, err = govKeeper.SubmitProposal(ctx, tp, "", "test", "summary", addrBz, v1.ProposalType_PROPOSAL_TYPE_STANDARD)
	require.NoError(t, err)
	proposal6, err := govKeeper.SubmitProposal(ctx, tp, "", "test", "summary", addrBz, v1.ProposalType_PROPOSAL_TYPE_STANDARD)
	require.NoError(t, err)

	require.Equal(t, uint64(6), proposal6.Id)
}

func TestProposalQueues(t *testing.T) {
	govKeeper, mocks, _, ctx := setupGovKeeper(t)
	authKeeper := mocks.acctKeeper

	ac := address.NewBech32Codec("cosmos")
	addrBz, err := ac.StringToBytes(address1)
	require.NoError(t, err)
	authKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	// create test proposals
	tp := TestProposal
	proposal, err := govKeeper.SubmitProposal(ctx, tp, "", "test", "summary", addrBz, v1.ProposalType_PROPOSAL_TYPE_STANDARD)
	require.NoError(t, err)

	has, err := govKeeper.InactiveProposalsQueue.Has(ctx, collections.Join(*proposal.DepositEndTime, proposal.Id))
	require.NoError(t, err)
	require.True(t, has)

	require.NoError(t, govKeeper.ActivateVotingPeriod(ctx, proposal))

	proposal, err = govKeeper.Proposals.Get(ctx, proposal.Id)
	require.Nil(t, err)

	has, err = govKeeper.ActiveProposalsQueue.Has(ctx, collections.Join(*proposal.VotingEndTime, proposal.Id))
	require.NoError(t, err)
	require.True(t, has)
}

func TestSetHooks(t *testing.T) {
	govKeeper, _, _, _ := setupGovKeeper(t)
	require.Empty(t, govKeeper.Hooks())

	govHooksReceiver := MockGovHooksReceiver{}
	govKeeper.SetHooks(types.NewMultiGovHooks(&govHooksReceiver))
	require.NotNil(t, govKeeper.Hooks())
	require.Panics(t, func() {
		govKeeper.SetHooks(&govHooksReceiver)
	})
}

func TestGetGovGovernanceAndModuleAccountAddress(t *testing.T) {
	govKeeper, mocks, _, ctx := setupGovKeeper(t)
	authKeeper := mocks.acctKeeper
	mAcc := authKeeper.GetModuleAccount(ctx, "gov")
	require.Equal(t, mAcc, govKeeper.GetGovernanceAccount(ctx))
	mAddr := authKeeper.GetModuleAddress("gov")
	require.Equal(t, mAddr, govKeeper.ModuleAccountAddress())
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(KeeperTestSuite))
}
