package v2

import (
	"fmt"

	corestoretypes "cosmossdk.io/core/store"
	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
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
func migratePrefixProposalAddress(store corestoretypes.KVStore, prefixBz []byte) error {
	oldStore := prefix.NewStore(runtime.KVStoreAdapter(store), prefixBz)

	oldStoreIter := oldStore.Iterator(nil, nil)
	defer oldStoreIter.Close()

	for ; oldStoreIter.Valid(); oldStoreIter.Next() {
		proposalID := oldStoreIter.Key()[:proposalIDLen]
		addr := oldStoreIter.Key()[proposalIDLen:]
		newStoreKey := append(append(prefixBz, proposalID...), address.MustLengthPrefix(addr)...)

		// Set new key on store. Values don't change.
		err := store.Set(newStoreKey, oldStoreIter.Value())
		if err != nil {
			return err
		}
		oldStore.Delete(oldStoreIter.Key())
	}
	return nil
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
func migrateStoreWeightedVotes(store corestoretypes.KVStore, cdc codec.BinaryCodec) error {
	iterator := storetypes.KVStorePrefixIterator(runtime.KVStoreAdapter(store), types.VotesKeyPrefix)

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

		err = store.Set(iterator.Key(), bz)
		if err != nil {
			return err
		}
	}

	return nil
}

// MigrateStore performs in-place store migrations from v1 (v0.40) to v2 (v0.43). The
// migration includes:
//
// - Change addresses to be length-prefixed.
// - Change all legacy votes to ADR-037 weighted votes.
func MigrateStore(ctx sdk.Context, storeService corestoretypes.KVStoreService, cdc codec.BinaryCodec) error {
	store := storeService.OpenKVStore(ctx)
	err := migratePrefixProposalAddress(store, types.DepositsKeyPrefix)
	if err != nil {
		return err
	}

	err = migratePrefixProposalAddress(store, types.VotesKeyPrefix)
	if err != nil {
		return err
	}

	return migrateStoreWeightedVotes(store, cdc)
}
