package keeper_test

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/x/gov/keeper"
	"cosmossdk.io/x/gov/types"
	v1 "cosmossdk.io/x/gov/types/v1"

	"github.com/cosmos/cosmos-sdk/codec/address"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
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

func (h *MockGovHooksReceiver) AfterProposalSubmission(ctx context.Context, proposalID uint64) error {
	h.AfterProposalSubmissionValid = true
	return nil
}

func (h *MockGovHooksReceiver) AfterProposalDeposit(ctx context.Context, proposalID uint64, depositorAddr sdk.AccAddress) error {
	h.AfterProposalDepositValid = true
	return nil
}

func (h *MockGovHooksReceiver) AfterProposalVote(ctx context.Context, proposalID uint64, voterAddr sdk.AccAddress) error {
	h.AfterProposalVoteValid = true
	return nil
}

func (h *MockGovHooksReceiver) AfterProposalFailedMinDeposit(ctx context.Context, proposalID uint64) error {
	h.AfterProposalFailedMinDepositValid = true
	return nil
}

func (h *MockGovHooksReceiver) AfterProposalVotingPeriodEnded(ctx context.Context, proposalID uint64) error {
	h.AfterProposalVotingPeriodEndedValid = true
	return nil
}

func TestHooks(t *testing.T) {
	minDeposit := v1.DefaultParams().MinDeposit
	govKeeper, mocks, _, ctx := setupGovKeeper(t)
	authKeeper, bankKeeper, stakingKeeper := mocks.acctKeeper, mocks.bankKeeper, mocks.stakingKeeper
	addrs := simtestutil.AddTestAddrs(bankKeeper, stakingKeeper, ctx, 1, minDeposit[0].Amount)

	authKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()
	stakingKeeper.EXPECT().ValidatorAddressCodec().Return(address.NewBech32Codec("cosmosvaloper")).AnyTimes()

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
	_, err := govKeeper.SubmitProposal(ctx, tp, "", "test", "summary", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"), v1.ProposalType_PROPOSAL_TYPE_STANDARD)
	require.NoError(t, err)
	require.True(t, govHooksReceiver.AfterProposalSubmissionValid)

	params, _ := govKeeper.Params.Get(ctx)
	newHeader := ctx.HeaderInfo()
	newHeader.Time = ctx.HeaderInfo().Time.Add(*params.MaxDepositPeriod).Add(time.Duration(1) * time.Second)
	ctx = ctx.WithHeaderInfo(newHeader)
	err = govKeeper.EndBlocker(ctx)
	require.NoError(t, err)

	require.True(t, govHooksReceiver.AfterProposalFailedMinDepositValid)

	p2, err := govKeeper.SubmitProposal(ctx, tp, "", "test", "summary", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"), v1.ProposalType_PROPOSAL_TYPE_STANDARD)
	require.NoError(t, err)

	activated, err := govKeeper.AddDeposit(ctx, p2.Id, addrs[0], minDeposit)
	require.True(t, activated)
	require.NoError(t, err)
	require.True(t, govHooksReceiver.AfterProposalDepositValid)

	err = govKeeper.AddVote(ctx, p2.Id, addrs[0], v1.NewNonSplitVoteOption(v1.OptionYes), "")
	require.NoError(t, err)
	require.True(t, govHooksReceiver.AfterProposalVoteValid)

	newHeader = ctx.HeaderInfo()
	newHeader.Time = ctx.HeaderInfo().Time.Add(*params.VotingPeriod).Add(time.Duration(1) * time.Second)
	ctx = ctx.WithHeaderInfo(newHeader)
	err = govKeeper.EndBlocker(ctx)
	require.NoError(t, err)
	require.True(t, govHooksReceiver.AfterProposalVotingPeriodEndedValid)
}
