package v2

import (
	"fmt"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/store/prefix"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/x/gov/types"
	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
)

const proposalIDLen = 8

// migratePrefixProposalAddress is a helper function that migrates all keys of format:
// <prefix_bytes><proposal_id (8 bytes)><address_bytes>
// into format:
// <prefix_bytes><proposal_id (8 bytes)><address_len (1 byte)><address_bytes>
func migratePrefixProposalAddress(store sdk.KVStore, prefixBz []byte) {
	oldStore := prefix.NewStore(store, prefixBz)

	oldStoreIter := oldStore.Iterator(nil, nil)
	defer oldStoreIter.Close()

	for ; oldStoreIter.Valid(); oldStoreIter.Next() {
		proposalID := oldStoreIter.Key()[:proposalIDLen]
		addr := oldStoreIter.Key()[proposalIDLen:]
		newStoreKey := append(append(prefixBz, proposalID...), address.MustLengthPrefix(addr)...)

		// Set new key on store. Values don't change.
		store.Set(newStoreKey, oldStoreIter.Value())
		oldStore.Delete(oldStoreIter.Key())
	}
}

// migrateStoreWeightedVotes migrates a legacy vote to an ADR-037 weighted vote.
// Important: the `oldVote` has its `Option` field set, whereas the new weighted
// vote has its `Options` field set.
func migrateVote(oldVote v1beta1.Vote) v1beta1.Vote {
	return v1beta1.Vote{
		ProposalId: oldVote.ProposalId,
		Voter:      oldVote.Voter,
		Options:    v1beta1.NewNonSplitVoteOption(oldVote.Option),
	}
}

// migrateStoreWeightedVotes migrates in-place all legacy votes to ADR-037 weighted votes.
func migrateStoreWeightedVotes(store sdk.KVStore, cdc codec.BinaryCodec) error {
	iterator := sdk.KVStorePrefixIterator(store, types.VotesKeyPrefix)

	defer iterator.Close()
	for ; iterator.Valid(); iterator.Next() {
		var oldVote v1beta1.Vote
		err := cdc.Unmarshal(iterator.Value(), &oldVote)
		if err != nil {
			return err
		}

		newVote := migrateVote(oldVote)
		fmt.Println("migrateStoreWeightedVotes newVote=", newVote)
		bz, err := cdc.Marshal(&newVote)
		if err != nil {
			return err
		}

		store.Set(iterator.Key(), bz)
	}

	return nil
}

// MigrateStore performs in-place store migrations from v1 (v0.40) to v2 (v0.43). The
// migration includes:
//
// - Change addresses to be length-prefixed.
// - Change all legacy votes to ADR-037 weighted votes.
func MigrateStore(ctx sdk.Context, storeKey storetypes.StoreKey, cdc codec.BinaryCodec) error {
	store := ctx.KVStore(storeKey)
	migratePrefixProposalAddress(store, types.DepositsKeyPrefix)
	migratePrefixProposalAddress(store, types.VotesKeyPrefix)
	return migrateStoreWeightedVotes(store, cdc)
}
