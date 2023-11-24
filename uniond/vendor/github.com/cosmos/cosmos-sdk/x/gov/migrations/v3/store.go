package v3

import (
	corestoretypes "cosmossdk.io/core/store"
	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/migrations/v1"
	govv1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
	govv1beta1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
)

// migrateProposals migrates all legacy proposals into MsgExecLegacyContent
// proposals.
func migrateProposals(store storetypes.KVStore, cdc codec.BinaryCodec) error {
	propStore := prefix.NewStore(store, v1.ProposalsKeyPrefix)

	iter := propStore.Iterator(nil, nil)
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		var oldProp govv1beta1.Proposal
		err := cdc.Unmarshal(iter.Value(), &oldProp)
		if err != nil {
			return err
		}

		newProp, err := convertToNewProposal(oldProp)
		if err != nil {
			return err
		}
		bz, err := cdc.Marshal(&newProp)
		if err != nil {
			return err
		}

		// Set new value on store.
		propStore.Set(iter.Key(), bz)
	}

	return nil
}

// migrateVotes migrates all v1beta1 weighted votes (with sdk.Dec as weight)
// to v1 weighted votes (with string as weight)
func migrateVotes(store storetypes.KVStore, cdc codec.BinaryCodec) error {
	votesStore := prefix.NewStore(store, v1.VotesKeyPrefix)

	iter := votesStore.Iterator(nil, nil)
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		var oldVote govv1beta1.Vote
		err := cdc.Unmarshal(iter.Value(), &oldVote)
		if err != nil {
			return err
		}

		newVote := govv1.Vote{
			ProposalId: oldVote.ProposalId,
			Voter:      oldVote.Voter,
		}
		newOptions := make([]*govv1.WeightedVoteOption, len(oldVote.Options))
		for i, o := range oldVote.Options {
			newOptions[i] = &govv1.WeightedVoteOption{
				Option: govv1.VoteOption(o.Option),
				Weight: o.Weight.String(), // Convert to decimal string
			}
		}
		newVote.Options = newOptions
		bz, err := cdc.Marshal(&newVote)
		if err != nil {
			return err
		}

		// Set new value on store.
		votesStore.Set(iter.Key(), bz)
	}

	return nil
}

// MigrateStore performs in-place store migrations from v2 (v0.43) to v3 (v0.46). The
// migration includes:
//
// - Migrate proposals to be Msg-based.
func MigrateStore(ctx sdk.Context, storeService corestoretypes.KVStoreService, cdc codec.BinaryCodec) error {
	store := storeService.OpenKVStore(ctx)

	if err := migrateVotes(runtime.KVStoreAdapter(store), cdc); err != nil {
		return err
	}

	return migrateProposals(runtime.KVStoreAdapter(store), cdc)
}
