package keeper

import (
	"context"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/x/group"
	"cosmossdk.io/x/group/errors"
	"cosmossdk.io/x/group/internal/orm"

	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// Tally is a function that tallies a proposal by iterating through its votes,
// and returns the tally result without modifying the proposal or any state.
func (k Keeper) Tally(ctx context.Context, p group.Proposal, groupID uint64) (group.TallyResult, error) {
	// If proposal has already been tallied and updated, then its status is
	// accepted/rejected, in which case we just return the previously stored result.
	//
	// In all other cases (including withdrawn, aborted...) we do the tally
	// again.
	if p.Status == group.PROPOSAL_STATUS_ACCEPTED || p.Status == group.PROPOSAL_STATUS_REJECTED {
		return p.FinalTallyResult, nil
	}

	kvStore := k.KVStoreService.OpenKVStore(ctx)

	it, err := k.voteByProposalIndex.Get(kvStore, p.Id)
	if err != nil {
		return group.TallyResult{}, err
	}
	defer it.Close()

	tallyResult := group.DefaultTallyResult()

	for {
		var vote group.Vote
		_, err = it.LoadNext(&vote)
		if errors.ErrORMIteratorDone.Is(err) {
			break
		}
		if err != nil {
			return group.TallyResult{}, err
		}

		var member group.GroupMember
		err := k.groupMemberTable.GetOne(kvStore, orm.PrimaryKey(&group.GroupMember{
			GroupId: groupID,
			Member:  &group.Member{Address: vote.Voter},
		}, k.accKeeper.AddressCodec()), &member)

		switch {
		case sdkerrors.ErrNotFound.Is(err):
			// If the member left the group after voting, then we simply skip the
			// vote.
			continue
		case err != nil:
			// For any other errors, we stop and return the error.
			return group.TallyResult{}, err
		}

		if err := tallyResult.Add(vote, member.Member.Weight); err != nil {
			return group.TallyResult{}, errorsmod.Wrap(err, "add new vote")
		}
	}

	return tallyResult, nil
}
