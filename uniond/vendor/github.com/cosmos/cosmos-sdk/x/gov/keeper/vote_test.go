package keeper_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
)

func TestVotes(t *testing.T) {
	govKeeper, _, bankKeeper, stakingKeeper, _, ctx := setupGovKeeper(t)
	addrs := simtestutil.AddTestAddrsIncremental(bankKeeper, stakingKeeper, ctx, 2, sdk.NewInt(10000000))

	tp := TestProposal
	proposal, err := govKeeper.SubmitProposal(ctx, tp, "", "title", "description", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"))
	require.NoError(t, err)
	proposalID := proposal.Id
	metadata := "metadata"

	var invalidOption v1.VoteOption = 0x10

	require.Error(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionYes), metadata), "proposal not on voting period")
	require.Error(t, govKeeper.AddVote(ctx, 10, addrs[0], v1.NewNonSplitVoteOption(v1.OptionYes), ""), "invalid proposal ID")

	proposal.Status = v1.StatusVotingPeriod
	govKeeper.SetProposal(ctx, proposal)

	require.Error(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(invalidOption), ""), "invalid option")

	// Test first vote
	require.NoError(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionAbstain), metadata))
	vote, found := govKeeper.GetVote(ctx, proposalID, addrs[0])
	require.True(t, found)
	require.Equal(t, addrs[0].String(), vote.Voter)
	require.Equal(t, proposalID, vote.ProposalId)
	require.True(t, len(vote.Options) == 1)
	require.Equal(t, v1.OptionAbstain, vote.Options[0].Option)

	// Test change of vote
	require.NoError(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionYes), ""))
	vote, found = govKeeper.GetVote(ctx, proposalID, addrs[0])
	require.True(t, found)
	require.Equal(t, addrs[0].String(), vote.Voter)
	require.Equal(t, proposalID, vote.ProposalId)
	require.True(t, len(vote.Options) == 1)
	require.Equal(t, v1.OptionYes, vote.Options[0].Option)

	// Test second vote
	require.NoError(t, govKeeper.AddVote(ctx, proposalID, addrs[1], v1.WeightedVoteOptions{
		v1.NewWeightedVoteOption(v1.OptionYes, sdk.NewDecWithPrec(60, 2)),
		v1.NewWeightedVoteOption(v1.OptionNo, sdk.NewDecWithPrec(30, 2)),
		v1.NewWeightedVoteOption(v1.OptionAbstain, sdk.NewDecWithPrec(5, 2)),
		v1.NewWeightedVoteOption(v1.OptionNoWithVeto, sdk.NewDecWithPrec(5, 2)),
	}, ""))
	vote, found = govKeeper.GetVote(ctx, proposalID, addrs[1])
	require.True(t, found)
	require.Equal(t, addrs[1].String(), vote.Voter)
	require.Equal(t, proposalID, vote.ProposalId)
	require.True(t, len(vote.Options) == 4)
	require.Equal(t, v1.OptionYes, vote.Options[0].Option)
	require.Equal(t, v1.OptionNo, vote.Options[1].Option)
	require.Equal(t, v1.OptionAbstain, vote.Options[2].Option)
	require.Equal(t, v1.OptionNoWithVeto, vote.Options[3].Option)
	require.Equal(t, vote.Options[0].Weight, sdk.NewDecWithPrec(60, 2).String())
	require.Equal(t, vote.Options[1].Weight, sdk.NewDecWithPrec(30, 2).String())
	require.Equal(t, vote.Options[2].Weight, sdk.NewDecWithPrec(5, 2).String())
	require.Equal(t, vote.Options[3].Weight, sdk.NewDecWithPrec(5, 2).String())

	// Test vote iterator
	// NOTE order of deposits is determined by the addresses
	votes := govKeeper.GetAllVotes(ctx)
	require.Len(t, votes, 2)
	require.Equal(t, votes, govKeeper.GetVotes(ctx, proposalID))
	require.Equal(t, addrs[0].String(), votes[0].Voter)
	require.Equal(t, proposalID, votes[0].ProposalId)
	require.True(t, len(votes[0].Options) == 1)
	require.Equal(t, v1.OptionYes, votes[0].Options[0].Option)
	require.Equal(t, addrs[1].String(), votes[1].Voter)
	require.Equal(t, proposalID, votes[1].ProposalId)
	require.True(t, len(votes[1].Options) == 4)
	require.Equal(t, votes[1].Options[0].Weight, sdk.NewDecWithPrec(60, 2).String())
	require.Equal(t, votes[1].Options[1].Weight, sdk.NewDecWithPrec(30, 2).String())
	require.Equal(t, votes[1].Options[2].Weight, sdk.NewDecWithPrec(5, 2).String())
	require.Equal(t, votes[1].Options[3].Weight, sdk.NewDecWithPrec(5, 2).String())
}
