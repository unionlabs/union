package state_native

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stateutil"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	consensus_types "github.com/prysmaticlabs/prysm/v4/consensus-types"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// SetLatestBlockHeader in the beacon state.
func (b *BeaconState) SetLatestBlockHeader(val *ethpb.BeaconBlockHeader) error {
	b.lock.Lock()
	defer b.lock.Unlock()

	b.latestBlockHeader = ethpb.CopyBeaconBlockHeader(val)
	b.markFieldAsDirty(types.LatestBlockHeader)
	return nil
}

// SetBlockRoots for the beacon state. Updates the entire
// list to a new value by overwriting the previous one.
func (b *BeaconState) SetBlockRoots(val [][]byte) error {
	b.lock.Lock()
	defer b.lock.Unlock()

	if features.Get().EnableExperimentalState {
		if b.blockRootsMultiValue != nil {
			b.blockRootsMultiValue.Detach(b)
		}
		b.blockRootsMultiValue = NewMultiValueBlockRoots(val)
	} else {
		b.sharedFieldReferences[types.BlockRoots].MinusRef()
		b.sharedFieldReferences[types.BlockRoots] = stateutil.NewRef(1)

		rootsArr := make([][32]byte, fieldparams.BlockRootsLength)
		for i := 0; i < len(rootsArr); i++ {
			copy(rootsArr[i][:], val[i])
		}
		b.blockRoots = rootsArr
	}

	b.markFieldAsDirty(types.BlockRoots)
	b.rebuildTrie[types.BlockRoots] = true
	return nil
}

// UpdateBlockRootAtIndex for the beacon state. Updates the block root
// at a specific index to a new value.
func (b *BeaconState) UpdateBlockRootAtIndex(idx uint64, blockRoot [32]byte) error {
	b.lock.Lock()
	defer b.lock.Unlock()

	if features.Get().EnableExperimentalState {
		if err := b.blockRootsMultiValue.UpdateAt(b, idx, blockRoot); err != nil {
			return errors.Wrap(err, "could not update block roots")
		}
	} else {
		if uint64(len(b.blockRoots)) <= idx {
			return errors.Wrapf(consensus_types.ErrOutOfBounds, "block root index %d does not exist", idx)
		}

		r := b.blockRoots
		if ref := b.sharedFieldReferences[types.BlockRoots]; ref.Refs() > 1 {
			// Copy elements in underlying array by reference.
			r = make([][32]byte, len(b.blockRoots))
			copy(r, b.blockRoots)
			ref.MinusRef()
			b.sharedFieldReferences[types.BlockRoots] = stateutil.NewRef(1)
		}
		r[idx] = blockRoot
		b.blockRoots = r
	}

	b.markFieldAsDirty(types.BlockRoots)
	b.addDirtyIndices(types.BlockRoots, []uint64{idx})
	return nil
}
