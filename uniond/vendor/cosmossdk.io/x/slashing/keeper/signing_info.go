package keeper

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/bits-and-blooms/bitset"

	"cosmossdk.io/collections"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/x/slashing/types"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// HasValidatorSigningInfo returns if a given validator has signing information
// persisted.
func (k Keeper) HasValidatorSigningInfo(ctx context.Context, consAddr sdk.ConsAddress) bool {
	has, err := k.ValidatorSigningInfo.Has(ctx, consAddr)
	return err == nil && has
}

// JailUntil attempts to set a validator's JailedUntil attribute in its signing info.
func (k Keeper) JailUntil(ctx context.Context, consAddr sdk.ConsAddress, jailTime time.Time) error {
	signInfo, err := k.ValidatorSigningInfo.Get(ctx, consAddr)
	if err != nil {
		addr, err := k.sk.ConsensusAddressCodec().BytesToString(consAddr)
		if err != nil {
			return types.ErrNoSigningInfoFound.Wrapf("could not convert consensus address to string. Error: %s", err.Error())
		}
		return types.ErrNoSigningInfoFound.Wrapf("cannot jail validator with consensus address %s that does not have any signing information", addr)
	}

	signInfo.JailedUntil = jailTime
	return k.ValidatorSigningInfo.Set(ctx, consAddr, signInfo)
}

// Tombstone attempts to tombstone a validator.
func (k Keeper) Tombstone(ctx context.Context, consAddr sdk.ConsAddress) error {
	signInfo, err := k.ValidatorSigningInfo.Get(ctx, consAddr)
	if err != nil {
		addr, err := k.sk.ConsensusAddressCodec().BytesToString(consAddr)
		if err != nil {
			return types.ErrNoSigningInfoFound.Wrapf("could not convert consensus address to string. Error: %s", err.Error())
		}
		return types.ErrNoSigningInfoFound.Wrap(fmt.Sprintf("cannot tombstone validator with consensus address %s that does not have any signing information", addr))
	}

	if signInfo.Tombstoned {
		return types.ErrValidatorTombstoned.Wrap("cannot tombstone validator that is already tombstoned")
	}

	signInfo.Tombstoned = true
	return k.ValidatorSigningInfo.Set(ctx, consAddr, signInfo)
}

// IsTombstoned returns if a given validator by consensus address is tombstoned.
func (k Keeper) IsTombstoned(ctx context.Context, consAddr sdk.ConsAddress) bool {
	signInfo, err := k.ValidatorSigningInfo.Get(ctx, consAddr)
	if err != nil {
		return false
	}

	return signInfo.Tombstoned
}

// getMissedBlockBitmapChunk gets the bitmap chunk at the given chunk index for
// a validator's missed block signing window.
func (k Keeper) getMissedBlockBitmapChunk(ctx context.Context, addr sdk.ConsAddress, chunkIndex int64) ([]byte, error) {
	chunk, err := k.ValidatorMissedBlockBitmap.Get(ctx, collections.Join(addr.Bytes(), uint64(chunkIndex)))
	if err != nil && !errors.Is(err, collections.ErrNotFound) {
		return nil, err
	}
	return chunk, nil
}

// SetMissedBlockBitmapChunk sets the bitmap chunk at the given chunk index for
// a validator's missed block signing window.
func (k Keeper) SetMissedBlockBitmapChunk(ctx context.Context, addr sdk.ConsAddress, chunkIndex int64, chunk []byte) error {
	return k.ValidatorMissedBlockBitmap.Set(ctx, collections.Join(addr.Bytes(), uint64(chunkIndex)), chunk)
}

// getPreviousConsKey returns the old consensus key if it has rotated,
// allowing retrieval of missed blocks associated with the old key.
func (k Keeper) getPreviousConsKey(ctx context.Context, addr sdk.ConsAddress) (sdk.ConsAddress, error) {
	oldPk, err := k.sk.ValidatorIdentifier(ctx, addr)
	if err != nil {
		return nil, err
	}

	if oldPk != nil {
		return oldPk, nil
	}

	return addr, nil
}

// GetMissedBlockBitmapValue returns true if a validator missed signing a block
// at the given index and false otherwise. The index provided is assumed to be
// the index in the range [0, SignedBlocksWindow), which represents the bitmap
// where each bit represents a height, and is determined by the validator's
// IndexOffset modulo SignedBlocksWindow. This index is used to fetch the chunk
// in the bitmap and the relative bit in that chunk.
func (k Keeper) GetMissedBlockBitmapValue(ctx context.Context, addr sdk.ConsAddress, index int64) (bool, error) {
	// get the old consensus key if it has rotated, allowing retrieval of missed blocks associated with the old key
	addr, err := k.getPreviousConsKey(ctx, addr)
	if err != nil {
		return false, err
	}

	// get the chunk or "word" in the logical bitmap
	chunkIndex := index / types.MissedBlockBitmapChunkSize

	bs := bitset.New(uint(types.MissedBlockBitmapChunkSize))
	chunk, err := k.getMissedBlockBitmapChunk(ctx, addr, chunkIndex)
	if err != nil {
		return false, errorsmod.Wrapf(err, "failed to get bitmap chunk; index: %d", index)
	}

	if chunk != nil {
		if err := bs.UnmarshalBinary(chunk); err != nil {
			return false, errorsmod.Wrapf(err, "failed to decode bitmap chunk; index: %d", index)
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
	// get the old consensus key if it has rotated, allowing retrieval of missed blocks associated with the old key
	addr, err := k.getPreviousConsKey(ctx, addr)
	if err != nil {
		return err
	}

	// get the chunk or "word" in the logical bitmap
	chunkIndex := index / types.MissedBlockBitmapChunkSize

	bs := bitset.New(uint(types.MissedBlockBitmapChunkSize))
	chunk, err := k.getMissedBlockBitmapChunk(ctx, addr, chunkIndex)
	if err != nil {
		return errorsmod.Wrapf(err, "failed to get bitmap chunk; index: %d", index)
	}

	if chunk != nil {
		if err := bs.UnmarshalBinary(chunk); err != nil {
			return errorsmod.Wrapf(err, "failed to decode bitmap chunk; index: %d", index)
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
		return errorsmod.Wrapf(err, "failed to encode bitmap chunk; index: %d", index)
	}

	return k.SetMissedBlockBitmapChunk(ctx, addr, chunkIndex, updatedChunk)
}

// DeleteMissedBlockBitmap removes a validator's missed block bitmap from state.
func (k Keeper) DeleteMissedBlockBitmap(ctx context.Context, addr sdk.ConsAddress) error {
	// get the old consensus key if it has rotated, allowing retrieval of missed blocks associated with the old key
	addr, err := k.getPreviousConsKey(ctx, addr)
	if err != nil {
		return err
	}

	rng := collections.NewPrefixedPairRange[[]byte, uint64](addr.Bytes())
	return k.ValidatorMissedBlockBitmap.Clear(ctx, rng)
}

// IterateMissedBlockBitmap iterates over a validator's signed blocks window
// bitmap and performs a callback function on each index, i.e. block height, in
// the range [0, SignedBlocksWindow).
//
// Note: A callback will only be executed over all bitmap chunks that exist in
// state.
func (k Keeper) IterateMissedBlockBitmap(ctx context.Context, addr sdk.ConsAddress, cb func(index int64, missed bool) (stop bool)) error {
	var index int64
	rng := collections.NewPrefixedPairRange[[]byte, uint64](addr.Bytes())
	return k.ValidatorMissedBlockBitmap.Walk(ctx, rng, func(key collections.Pair[[]byte, uint64], value []byte) (bool, error) {
		bs := bitset.New(uint(types.MissedBlockBitmapChunkSize))

		if err := bs.UnmarshalBinary(value); err != nil {
			return true, errorsmod.Wrapf(err, "failed to decode bitmap chunk; index: %v", key)
		}

		for i := uint(0); i < types.MissedBlockBitmapChunkSize; i++ {
			// execute the callback, where Test() returns true if the bit is set
			if cb(index, bs.Test(i)) {
				break
			}

			index++
		}
		return false, nil
	})
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

// performConsensusPubKeyUpdate updates the consensus address-pubkey relation
// and refreshes the signing info by replacing the old key with the new one.
func (k Keeper) performConsensusPubKeyUpdate(ctx context.Context, oldPubKey, newPubKey cryptotypes.PubKey) error {
	// Connect new consensus address with PubKey
	if err := k.AddrPubkeyRelation.Set(ctx, newPubKey.Address(), newPubKey); err != nil {
		return err
	}

	// Migrate ValidatorSigningInfo from oldPubKey to newPubKey
	signingInfo, err := k.ValidatorSigningInfo.Get(ctx, sdk.ConsAddress(oldPubKey.Address()))
	if err != nil {
		return types.ErrInvalidConsPubKey.Wrap("failed to get signing info for old public key")
	}

	consAddr, err := k.sk.ConsensusAddressCodec().BytesToString(newPubKey.Address())
	if err != nil {
		return err
	}

	signingInfo.Address = consAddr
	if err := k.ValidatorSigningInfo.Set(ctx, sdk.ConsAddress(newPubKey.Address()), signingInfo); err != nil {
		return err
	}

	return k.ValidatorSigningInfo.Remove(ctx, sdk.ConsAddress(oldPubKey.Address()))
}
