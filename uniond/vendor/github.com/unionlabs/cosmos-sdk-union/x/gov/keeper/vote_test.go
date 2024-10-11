package keeper_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections"
	sdkmath "cosmossdk.io/math"
	v1 "cosmossdk.io/x/gov/types/v1"

	"github.com/cosmos/cosmos-sdk/codec/address"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var invalidOption v1.VoteOption = 0x10

func TestVotes(t *testing.T) {
	govKeeper, mocks, _, ctx := setupGovKeeper(t)
	authKeeper, bankKeeper, stakingKeeper := mocks.acctKeeper, mocks.bankKeeper, mocks.stakingKeeper
	addrs := simtestutil.AddTestAddrsIncremental(bankKeeper, stakingKeeper, ctx, 2, sdkmath.NewInt(10000000))
	authKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	addrs0Str, err := authKeeper.AddressCodec().BytesToString(addrs[0])
	require.NoError(t, err)
	addrs1Str, err := authKeeper.AddressCodec().BytesToString(addrs[1])
	require.NoError(t, err)

	tp := TestProposal
	proposal, err := govKeeper.SubmitProposal(ctx, tp, "", "title", "description", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"), v1.ProposalType_PROPOSAL_TYPE_STANDARD)
	require.NoError(t, err)
	proposalID := proposal.Id
	metadata := "metadata"

	require.Error(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionYes), metadata), "proposal not on voting period")
	require.Error(t, govKeeper.AddVote(ctx, 10, addrs[0], v1.NewNonSplitVoteOption(v1.OptionYes), ""), "invalid proposal ID")

	proposal.Status = v1.StatusVotingPeriod
	err = govKeeper.Proposals.Set(ctx, proposal.Id, proposal)
	require.NoError(t, err)

	require.Error(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(invalidOption), ""), "invalid option")

	// Test first vote
	require.NoError(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionAbstain), metadata))
	vote, err := govKeeper.Votes.Get(ctx, collections.Join(proposalID, addrs[0]))
	require.Nil(t, err)
	require.Equal(t, addrs0Str, vote.Voter)
	require.Equal(t, proposalID, vote.ProposalId)
	require.True(t, len(vote.Options) == 1)
	require.Equal(t, v1.OptionAbstain, vote.Options[0].Option)

	// Test change of vote
	require.NoError(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionYes), ""))
	vote, err = govKeeper.Votes.Get(ctx, collections.Join(proposalID, addrs[0]))
	require.Nil(t, err)
	require.Equal(t, addrs0Str, vote.Voter)
	require.Equal(t, proposalID, vote.ProposalId)
	require.True(t, len(vote.Options) == 1)
	require.Equal(t, v1.OptionYes, vote.Options[0].Option)

	// Test second vote
	require.NoError(t, govKeeper.AddVote(ctx, proposalID, addrs[1], v1.WeightedVoteOptions{
		v1.NewWeightedVoteOption(v1.OptionYes, sdkmath.LegacyNewDecWithPrec(60, 2)),
		v1.NewWeightedVoteOption(v1.OptionNo, sdkmath.LegacyNewDecWithPrec(30, 2)),
		v1.NewWeightedVoteOption(v1.OptionAbstain, sdkmath.LegacyNewDecWithPrec(5, 2)),
		v1.NewWeightedVoteOption(v1.OptionNoWithVeto, sdkmath.LegacyNewDecWithPrec(5, 2)),
	}, ""))
	vote, err = govKeeper.Votes.Get(ctx, collections.Join(proposalID, addrs[1]))
	require.Nil(t, err)
	require.Equal(t, addrs1Str, vote.Voter)
	require.Equal(t, proposalID, vote.ProposalId)
	require.True(t, len(vote.Options) == 4)
	require.Equal(t, v1.OptionYes, vote.Options[0].Option)
	require.Equal(t, v1.OptionNo, vote.Options[1].Option)
	require.Equal(t, v1.OptionAbstain, vote.Options[2].Option)
	require.Equal(t, v1.OptionNoWithVeto, vote.Options[3].Option)
	require.Equal(t, vote.Options[0].Weight, sdkmath.LegacyNewDecWithPrec(60, 2).String())
	require.Equal(t, vote.Options[1].Weight, sdkmath.LegacyNewDecWithPrec(30, 2).String())
	require.Equal(t, vote.Options[2].Weight, sdkmath.LegacyNewDecWithPrec(5, 2).String())
	require.Equal(t, vote.Options[3].Weight, sdkmath.LegacyNewDecWithPrec(5, 2).String())

	// Test vote iterator
	// NOTE order of deposits is determined by the addresses
	var votes v1.Votes
	require.NoError(t, govKeeper.Votes.Walk(ctx, nil, func(_ collections.Pair[uint64, sdk.AccAddress], value v1.Vote) (stop bool, err error) {
		votes = append(votes, &value)
		return false, nil
	}))
	require.Len(t, votes, 2)
	var propVotes v1.Votes
	require.NoError(t, govKeeper.Votes.Walk(ctx, collections.NewPrefixedPairRange[uint64, sdk.AccAddress](proposalID), func(_ collections.Pair[uint64, sdk.AccAddress], value v1.Vote) (stop bool, err error) {
		propVotes = append(propVotes, &value)
		return false, nil
	}))
	require.Equal(t, votes, propVotes)
	require.Equal(t, addrs0Str, votes[0].Voter)
	require.Equal(t, proposalID, votes[0].ProposalId)
	require.True(t, len(votes[0].Options) == 1)
	require.Equal(t, v1.OptionYes, votes[0].Options[0].Option)
	require.Equal(t, addrs1Str, votes[1].Voter)
	require.Equal(t, proposalID, votes[1].ProposalId)
	require.True(t, len(votes[1].Options) == 4)
	require.Equal(t, votes[1].Options[0].Weight, sdkmath.LegacyNewDecWithPrec(60, 2).String())
	require.Equal(t, votes[1].Options[1].Weight, sdkmath.LegacyNewDecWithPrec(30, 2).String())
	require.Equal(t, votes[1].Options[2].Weight, sdkmath.LegacyNewDecWithPrec(5, 2).String())
	require.Equal(t, votes[1].Options[3].Weight, sdkmath.LegacyNewDecWithPrec(5, 2).String())

	// non existent vote
	_, err = govKeeper.Votes.Get(ctx, collections.Join(proposalID+100, addrs[1]))
	require.ErrorIs(t, err, collections.ErrNotFound)
}

func TestVotes_Optimisic(t *testing.T) {
	govKeeper, mocks, _, ctx := setupGovKeeper(t)
	authKeeper, bankKeeper, stakingKeeper := mocks.acctKeeper, mocks.bankKeeper, mocks.stakingKeeper
	addrs := simtestutil.AddTestAddrsIncremental(bankKeeper, stakingKeeper, ctx, 2, sdkmath.NewInt(10000000))
	authKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	proposal, err := govKeeper.SubmitProposal(ctx, nil, "", "title", "description", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"), v1.ProposalType_PROPOSAL_TYPE_OPTIMISTIC)
	require.NoError(t, err)

	proposal.Status = v1.StatusVotingPeriod
	require.NoError(t, govKeeper.Proposals.Set(ctx, proposal.Id, proposal))

	proposalID := proposal.Id

	// invalid options
	require.Error(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(invalidOption), ""), "invalid option")
	require.Error(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionYes), ""), "invalid option")
	require.Error(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionAbstain), "invalid option"))
	require.Error(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionNoWithVeto), ""), "invalid option")
	require.Error(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionSpam), ""), "invalid option")

	// valid options
	require.NoError(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionNo), ""))
}

func TestVotes_MultipleChoiceProposal(t *testing.T) {
	govKeeper, mocks, _, ctx := setupGovKeeper(t)
	authKeeper, bankKeeper, stakingKeeper := mocks.acctKeeper, mocks.bankKeeper, mocks.stakingKeeper
	addrs := simtestutil.AddTestAddrsIncremental(bankKeeper, stakingKeeper, ctx, 2, sdkmath.NewInt(10000000))
	authKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	proposal, err := govKeeper.SubmitProposal(ctx, nil, "", "title", "description", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"), v1.ProposalType_PROPOSAL_TYPE_MULTIPLE_CHOICE)
	require.NoError(t, err)
	err = govKeeper.ProposalVoteOptions.Set(ctx, proposal.Id, v1.ProposalVoteOptions{
		OptionOne:   "Vote for @tac0turle",
		OptionTwo:   "Vote for @facudomedica",
		OptionThree: "Vote for @alexanderbez",
	})
	require.NoError(t, err)

	proposal.Status = v1.StatusVotingPeriod
	require.NoError(t, govKeeper.Proposals.Set(ctx, proposal.Id, proposal))

	proposalID := proposal.Id

	// invalid options
	require.Error(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(invalidOption), ""), "invalid option")
	require.Error(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionFour), ""), "invalid option") // option four is not defined.

	// valid options
	require.NoError(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionOne), ""))
	require.NoError(t, govKeeper.AddVote(ctx, proposalID, addrs[1], v1.NewNonSplitVoteOption(v1.OptionTwo), ""))
	require.NoError(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionThree), ""))
}

func TestVotes_CustomMaxVoteOptionsLen(t *testing.T) {
	maxVoteOptionsLen := 3
	govKeeper, mocks, _, ctx := setupGovKeeperWithMaxVoteOptionsLen(t, uint64(maxVoteOptionsLen))
	authKeeper, bankKeeper, stakingKeeper := mocks.acctKeeper, mocks.bankKeeper, mocks.stakingKeeper
	addrs := simtestutil.AddTestAddrsIncremental(bankKeeper, stakingKeeper, ctx, 2, sdkmath.NewInt(10000000))
	authKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	addrs1Str, err := authKeeper.AddressCodec().BytesToString(addrs[1])
	require.NoError(t, err)

	tp := TestProposal
	proposal, err := govKeeper.SubmitProposal(ctx, tp, "", "title", "description", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"), v1.ProposalType_PROPOSAL_TYPE_STANDARD)
	require.NoError(t, err)
	proposalID := proposal.Id
	metadata := "metadata"

	require.Error(t, govKeeper.AddVote(ctx, proposalID, addrs[0], v1.NewNonSplitVoteOption(v1.OptionYes), metadata), "proposal not on voting period")
	require.Error(t, govKeeper.AddVote(ctx, 10, addrs[0], v1.NewNonSplitVoteOption(v1.OptionYes), ""), "invalid proposal ID")

	proposal.Status = v1.StatusVotingPeriod
	err = govKeeper.Proposals.Set(ctx, proposal.Id, proposal)
	require.NoError(t, err)

	// not exceeding MaxVoteOptionsLen, no errors should be thrown
	require.NoError(t, govKeeper.AddVote(ctx, proposalID, addrs[1], v1.WeightedVoteOptions{
		v1.NewWeightedVoteOption(v1.OptionYes, sdkmath.LegacyNewDecWithPrec(60, 2)),
		v1.NewWeightedVoteOption(v1.OptionNo, sdkmath.LegacyNewDecWithPrec(30, 2)),
		v1.NewWeightedVoteOption(v1.OptionAbstain, sdkmath.LegacyNewDecWithPrec(10, 2)),
	}, ""))
	vote, err := govKeeper.Votes.Get(ctx, collections.Join(proposalID, addrs[1]))
	require.Nil(t, err)
	require.Equal(t, addrs1Str, vote.Voter)
	require.Equal(t, proposalID, vote.ProposalId)
	require.True(t, len(vote.Options) == 3)
	require.Equal(t, v1.OptionYes, vote.Options[0].Option)
	require.Equal(t, v1.OptionNo, vote.Options[1].Option)
	require.Equal(t, v1.OptionAbstain, vote.Options[2].Option)
	require.Equal(t, vote.Options[0].Weight, sdkmath.LegacyNewDecWithPrec(60, 2).String())
	require.Equal(t, vote.Options[1].Weight, sdkmath.LegacyNewDecWithPrec(30, 2).String())
	require.Equal(t, vote.Options[2].Weight, sdkmath.LegacyNewDecWithPrec(10, 2).String())

	// exceeding MaxVoteOptionsLen, an error should be thrown
	err = govKeeper.AddVote(ctx, proposalID, addrs[1], v1.WeightedVoteOptions{
		v1.NewWeightedVoteOption(v1.OptionYes, sdkmath.LegacyNewDecWithPrec(60, 2)),
		v1.NewWeightedVoteOption(v1.OptionNo, sdkmath.LegacyNewDecWithPrec(30, 2)),
		v1.NewWeightedVoteOption(v1.OptionAbstain, sdkmath.LegacyNewDecWithPrec(5, 2)),
		v1.NewWeightedVoteOption(v1.OptionNoWithVeto, sdkmath.LegacyNewDecWithPrec(5, 2)),
	}, "")
	require.Error(t, err)
	require.Contains(t, err.Error(), "too many weighted vote options")

	// only one vote should have gone through
	var votes v1.Votes
	require.NoError(t, govKeeper.Votes.Walk(ctx, nil, func(_ collections.Pair[uint64, sdk.AccAddress], value v1.Vote) (stop bool, err error) {
		votes = append(votes, &value)
		return false, nil
	}))
	require.Len(t, votes, 1)
}
