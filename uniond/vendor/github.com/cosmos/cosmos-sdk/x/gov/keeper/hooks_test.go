package keeper_test

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/gov"
	"github.com/cosmos/cosmos-sdk/x/gov/keeper"
	"github.com/cosmos/cosmos-sdk/x/gov/types"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
)

var _ types.GovHooks = &MockGovHooksReceiver{}

// GovHooks event hooks for governance proposal object (noalias)
type MockGovHooksReceiver struct {
	AfterProposalSubmissionValid        bool
	AfterProposalDepositValid           bool
	AfterProposalVoteValid              bool
	AfterProposalFailedMinDepositValid  bool
	AfterProposalVotingPeriodEndedValid bool
}

func (h *MockGovHooksReceiver) AfterProposalSubmission(ctx sdk.Context, proposalID uint64) {
	h.AfterProposalSubmissionValid = true
}

func (h *MockGovHooksReceiver) AfterProposalDeposit(ctx sdk.Context, proposalID uint64, depositorAddr sdk.AccAddress) {
	h.AfterProposalDepositValid = true
}

func (h *MockGovHooksReceiver) AfterProposalVote(ctx sdk.Context, proposalID uint64, voterAddr sdk.AccAddress) {
	h.AfterProposalVoteValid = true
}

func (h *MockGovHooksReceiver) AfterProposalFailedMinDeposit(ctx sdk.Context, proposalID uint64) {
	h.AfterProposalFailedMinDepositValid = true
}

func (h *MockGovHooksReceiver) AfterProposalVotingPeriodEnded(ctx sdk.Context, proposalID uint64) {
	h.AfterProposalVotingPeriodEndedValid = true
}

func TestHooks(t *testing.T) {
	minDeposit := v1.DefaultParams().MinDeposit
	govKeeper, _, bankKeeper, stakingKeeper, _, ctx := setupGovKeeper(t)
	addrs := simtestutil.AddTestAddrs(bankKeeper, stakingKeeper, ctx, 1, minDeposit[0].Amount)

	govHooksReceiver := MockGovHooksReceiver{}

	keeper.UnsafeSetHooks(
		govKeeper, types.NewMultiGovHooks(&govHooksReceiver),
	)

	require.False(t, govHooksReceiver.AfterProposalSubmissionValid)
	require.False(t, govHooksReceiver.AfterProposalDepositValid)
	require.False(t, govHooksReceiver.AfterProposalVoteValid)
	require.False(t, govHooksReceiver.AfterProposalFailedMinDepositValid)
	require.False(t, govHooksReceiver.AfterProposalVotingPeriodEndedValid)

	tp := TestProposal
	_, err := govKeeper.SubmitProposal(ctx, tp, "", "test", "summary", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"))
	require.NoError(t, err)
	require.True(t, govHooksReceiver.AfterProposalSubmissionValid)

	newHeader := ctx.BlockHeader()
	newHeader.Time = ctx.BlockHeader().Time.Add(*govKeeper.GetParams(ctx).MaxDepositPeriod).Add(time.Duration(1) * time.Second)
	ctx = ctx.WithBlockHeader(newHeader)
	gov.EndBlocker(ctx, govKeeper)

	require.True(t, govHooksReceiver.AfterProposalFailedMinDepositValid)

	p2, err := govKeeper.SubmitProposal(ctx, tp, "", "test", "summary", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"))
	require.NoError(t, err)

	activated, err := govKeeper.AddDeposit(ctx, p2.Id, addrs[0], minDeposit)
	require.True(t, activated)
	require.NoError(t, err)
	require.True(t, govHooksReceiver.AfterProposalDepositValid)

	err = govKeeper.AddVote(ctx, p2.Id, addrs[0], v1.NewNonSplitVoteOption(v1.OptionYes), "")
	require.NoError(t, err)
	require.True(t, govHooksReceiver.AfterProposalVoteValid)

	newHeader = ctx.BlockHeader()
	newHeader.Time = ctx.BlockHeader().Time.Add(*govKeeper.GetParams(ctx).VotingPeriod).Add(time.Duration(1) * time.Second)
	ctx = ctx.WithBlockHeader(newHeader)
	gov.EndBlocker(ctx, govKeeper)
	require.True(t, govHooksReceiver.AfterProposalVotingPeriodEndedValid)
}
