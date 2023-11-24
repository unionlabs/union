package keeper

import (
	"context"
	"time"

	"github.com/bits-and-blooms/bitset"

	"cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/slashing/types"
)

// GetValidatorSigningInfo retruns the ValidatorSigningInfo for a specific validator
// ConsAddress. If not found it returns ErrNoSigningInfoFound, but other errors
// may be returned if there is an error reading from the store.
func (k Keeper) GetValidatorSigningInfo(ctx context.Context, address sdk.ConsAddress) (types.ValidatorSigningInfo, error) {
	store := k.storeService.OpenKVStore(ctx)
	var info types.ValidatorSigningInfo
	bz, err := store.Get(types.ValidatorSigningInfoKey(address))
	if err != nil {
		return info, err
	}

	if bz == nil {
		return info, types.ErrNoSigningInfoFound
	}

	err = k.cdc.Unmarshal(bz, &info)
	return info, err
}

// HasValidatorSigningInfo returns if a given validator has signing information
// persisted.
func (k Keeper) HasValidatorSigningInfo(ctx context.Context, consAddr sdk.ConsAddress) bool {
	_, err := k.GetValidatorSigningInfo(ctx, consAddr)
	return err == nil
}

// SetValidatorSigningInfo sets the validator signing info to a consensus address key
func (k Keeper) SetValidatorSigningInfo(ctx context.Context, address sdk.ConsAddress, info types.ValidatorSigningInfo) error {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := k.cdc.Marshal(&info)
	if err != nil {
		return err
	}

	return store.Set(types.ValidatorSigningInfoKey(address), bz)
}

// IterateValidatorSigningInfos iterates over the stored ValidatorSigningInfo
func (k Keeper) IterateValidatorSigningInfos(ctx context.Context,
	handler func(address sdk.ConsAddress, info types.ValidatorSigningInfo) (stop bool),
) error {
	store := k.storeService.OpenKVStore(ctx)
	iter, err := store.Iterator(types.ValidatorSigningInfoKeyPrefix, storetypes.PrefixEndBytes(types.ValidatorSigningInfoKeyPrefix))
	if err != nil {
		return err
	}
	defer iter.Close()
	for ; iter.Valid(); iter.Next() {
		address := types.ValidatorSigningInfoAddress(iter.Key())
		var info types.ValidatorSigningInfo
		err = k.cdc.Unmarshal(iter.Value(), &info)
		if err != nil {
			return err
		}

		if handler(address, info) {
			break
		}
	}
	return nil
}

// JailUntil attempts to set a validator's JailedUntil attribute in its signing
// info. It will panic if the signing info does not exist for the validator.
func (k Keeper) JailUntil(ctx context.Context, consAddr sdk.ConsAddress, jailTime time.Time) error {
	signInfo, err := k.GetValidatorSigningInfo(ctx, consAddr)
	if err != nil {
		return errors.Wrap(err, "cannot jail validator that does not have any signing information")
	}

	signInfo.JailedUntil = jailTime
	return k.SetValidatorSigningInfo(ctx, consAddr, signInfo)
}

// Tombstone attempts to tombstone a validator. It will panic if signing info for
// the given validator does not exist.
func (k Keeper) Tombstone(ctx context.Context, consAddr sdk.ConsAddress) error {
	signInfo, err := k.GetValidatorSigningInfo(ctx, consAddr)
	if err != nil {
		return types.ErrNoSigningInfoFound.Wrap("cannot tombstone validator that does not have any signing information")
	}

	if signInfo.Tombstoned {
		return types.ErrValidatorTombstoned.Wrap("cannot tombstone validator that is already tombstoned")
	}

	signInfo.Tombstoned = true
	return k.SetValidatorSigningInfo(ctx, consAddr, signInfo)
}

// IsTombstoned returns if a given validator by consensus address is tombstoned.
func (k Keeper) IsTombstoned(ctx context.Context, consAddr sdk.ConsAddress) bool {
	signInfo, err := k.GetValidatorSigningInfo(ctx, consAddr)
	if err != nil {
		return false
	}

	return signInfo.Tombstoned
}

// getMissedBlockBitmapChunk gets the bitmap chunk at the given chunk index for
// a validator's missed block signing window.
func (k Keeper) getMissedBlockBitmapChunk(ctx context.Context, addr sdk.ConsAddress, chunkIndex int64) ([]byte, error) {
	store := k.storeService.OpenKVStore(ctx)
	chunk, err := store.Get(types.ValidatorMissedBlockBitmapKey(addr, chunkIndex))
	return chunk, err
}

// setMissedBlockBitmapChunk sets the bitmap chunk at the given chunk index for
// a validator's missed block signing window.
func (k Keeper) setMissedBlockBitmapChunk(ctx context.Context, addr sdk.ConsAddress, chunkIndex int64, chunk []byte) error {
	store := k.storeService.OpenKVStore(ctx)
	key := types.ValidatorMissedBlockBitmapKey(addr, chunkIndex)
	return store.Set(key, chunk)
}

// GetMissedBlockBitmapValue returns true if a validator missed signing a block
// at the given index and false otherwise. The index provided is assumed to be
// the index in the range [0, SignedBlocksWindow), which represents the bitmap
// where each bit represents a height, and is determined by the validator's
// IndexOffset modulo SignedBlocksWindow. This index is used to fetch the chunk
// in the bitmap and the relative bit in that chunk.
func (k Keeper) GetMissedBlockBitmapValue(ctx context.Context, addr sdk.ConsAddress, index int64) (bool, error) {
	// get the chunk or "word" in the logical bitmap
	chunkIndex := index / types.MissedBlockBitmapChunkSize

	bs := bitset.New(uint(types.MissedBlockBitmapChunkSize))
	chunk, err := k.getMissedBlockBitmapChunk(ctx, addr, chunkIndex)
	if err != nil {
		return false, errors.Wrapf(err, "failed to get bitmap chunk; index: %d", index)
	}

	if chunk != nil {
		if err := bs.UnmarshalBinary(chunk); err != nil {
			return false, errors.Wrapf(err, "failed to decode bitmap chunk; index: %d", index)
		}
	}

	// get the bit position in the chunk of the logical bitmap, where Test()
	// checks if the bit is set.
	bitIndex := index % types.MissedBlockBitmapChunkSize
	return bs.Test(uint(bitIndex)), nil
}

// SetMissedBlockBitmapValue sets, i.e. flips, a bit in the validator's missed
// block bitmap. When missed=true, the bit is set, otherwise it set to zero. The
// index provided is assumed to be the index in the range [0, SignedBlocksWindow),
// which represents the bitmap where each bit represents a height, and is
// determined by the validator's IndexOffset modulo SignedBlocksWindow. This
// index is used to fetch the chunk in the bitmap and the relative bit in that
// chunk.
func (k Keeper) SetMissedBlockBitmapValue(ctx context.Context, addr sdk.ConsAddress, index int64, missed bool) error {
	// get the chunk or "word" in the logical bitmap
	chunkIndex := index / types.MissedBlockBitmapChunkSize

	bs := bitset.New(uint(types.MissedBlockBitmapChunkSize))
	chunk, err := k.getMissedBlockBitmapChunk(ctx, addr, chunkIndex)
	if err != nil {
		return errors.Wrapf(err, "failed to get bitmap chunk; index: %d", index)
	}

	if chunk != nil {
		if err := bs.UnmarshalBinary(chunk); err != nil {
			return errors.Wrapf(err, "failed to decode bitmap chunk; index: %d", index)
		}
	}

	// get the bit position in the chunk of the logical bitmap
	bitIndex := uint(index % types.MissedBlockBitmapChunkSize)
	if missed {
		bs.Set(bitIndex)
	} else {
		bs.Clear(bitIndex)
	}

	updatedChunk, err := bs.MarshalBinary()
	if err != nil {
		return errors.Wrapf(err, "failed to encode bitmap chunk; index: %d", index)
	}

	k.setMissedBlockBitmapChunk(ctx, addr, chunkIndex, updatedChunk)
	return nil
}

// DeleteMissedBlockBitmap removes a validator's missed block bitmap from state.
func (k Keeper) DeleteMissedBlockBitmap(ctx context.Context, addr sdk.ConsAddress) error {
	store := k.storeService.OpenKVStore(ctx)
	prefix := types.ValidatorMissedBlockBitmapPrefixKey(addr)
	iter, err := store.Iterator(prefix, storetypes.PrefixEndBytes(prefix))
	if err != nil {
		return err
	}
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		err = store.Delete(iter.Key())
		if err != nil {
			return err
		}
	}
	return nil
}

// IterateMissedBlockBitmap iterates over a validator's signed blocks window
// bitmap and performs a callback function on each index, i.e. block height, in
// the range [0, SignedBlocksWindow).
//
// Note: A callback will only be executed over all bitmap chunks that exist in
// state.
func (k Keeper) IterateMissedBlockBitmap(ctx context.Context, addr sdk.ConsAddress, cb func(index int64, missed bool) (stop bool)) error {
	store := k.storeService.OpenKVStore(ctx)
	prefix := types.ValidatorMissedBlockBitmapPrefixKey(addr)
	iter, err := store.Iterator(prefix, storetypes.PrefixEndBytes(prefix))
	if err != nil {
		return err
	}
	defer iter.Close()

	var index int64
	for ; iter.Valid(); iter.Next() {
		bs := bitset.New(uint(types.MissedBlockBitmapChunkSize))

		if err := bs.UnmarshalBinary(iter.Value()); err != nil {
			return errors.Wrapf(err, "failed to decode bitmap chunk; index: %v", string(iter.Key()))
		}

		for i := uint(0); i < types.MissedBlockBitmapChunkSize; i++ {
			// execute the callback, where Test() returns true if the bit is set
			if cb(index, bs.Test(i)) {
				break
			}

			index++
		}
	}
	return nil
}

// GetValidatorMissedBlocks returns array of missed blocks for given validator.
func (k Keeper) GetValidatorMissedBlocks(ctx context.Context, addr sdk.ConsAddress) ([]types.MissedBlock, error) {
	signedBlocksWindow, err := k.SignedBlocksWindow(ctx)
	if err != nil {
		return nil, err
	}

	missedBlocks := make([]types.MissedBlock, 0, signedBlocksWindow)
	err = k.IterateMissedBlockBitmap(ctx, addr, func(index int64, missed bool) (stop bool) {
		if missed {
			missedBlocks = append(missedBlocks, types.NewMissedBlock(index, missed))
		}

		return false
	})

	return missedBlocks, err
}
