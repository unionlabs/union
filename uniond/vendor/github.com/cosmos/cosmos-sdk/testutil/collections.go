package testutil

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"

	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// DiffCollectionsMigration is meant to aid in the migration from the previous store
// solution to collections. It takes a few steps to use it:
//  1. Write a test function that writes elements using the old store solution.
//  2. Run the test function and copy the hash that it outputs (run it a couple
//     of times to make sure no non-deterministic data is being used).
//  3. Change the code to use collections and run the test function again to make
//     sure the hash didn't change.
//
// First we write it as such:
//
//	func TestMigrateStore(t *testing.T) {
//		DiffCollectionsMigration(
//			ctx,
//			storeKey,
//			100,
//			func(i int64) {
//				err := SetPreviousProposerConsAddr(ctx, addrs[i])
//				require.NoError(t, err)
//			},
//			"abcdef0123456789",
//		)
//	}
//
// Then after we change the code to use collections, we modify the writeElem function:
//
//	func TestMigrateStore(t *testing.T) {
//		DiffCollectionsMigration(
//			ctx,
//			storeKey,
//			100,
//			func(i int64) {
//				err := keeper.PreviousProposer.Set(ctx, addrs[i])
//				require.NoError(t, err)
//			},
//			"abcdef0123456789",
//		)
//	}
func DiffCollectionsMigration(
	ctx sdk.Context,
	storeKey *storetypes.KVStoreKey,
	iterations int,
	writeElem func(int64),
	targetHash string,
) error {
	for i := int64(0); i < int64(iterations); i++ {
		writeElem(i)
	}

	h := sha256.New()
	it := ctx.KVStore(storeKey).Iterator(nil, nil)
	defer it.Close()
	for ; it.Valid(); it.Next() {
		h.Write(it.Key())
		h.Write(it.Value())
	}

	hash := h.Sum(nil)
	if hex.EncodeToString(hash) != targetHash {
		return fmt.Errorf("hashes don't match: %s != %s", hex.EncodeToString(hash), targetHash)
	}

	return nil
}
