package state_native

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stateutil"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	consensus_types "github.com/prysmaticlabs/prysm/v4/consensus-types"
)

// SetStateRoots for the beacon state. Updates the state roots
// to a new value by overwriting the previous value.
func (b *BeaconState) SetStateRoots(val [][]byte) error {
	b.lock.Lock()
	defer b.lock.Unlock()

	if features.Get().EnableExperimentalState {
		if b.stateRootsMultiValue != nil {
			b.stateRootsMultiValue.Detach(b)
		}
		b.stateRootsMultiValue = NewMultiValueStateRoots(val)
	} else {
		b.sharedFieldReferences[types.StateRoots].MinusRef()
		b.sharedFieldReferences[types.StateRoots] = stateutil.NewRef(1)

		rootsArr := make([][32]byte, fieldparams.StateRootsLength)
		for i := 0; i < len(rootsArr); i++ {
			copy(rootsArr[i][:], val[i])
		}
		b.stateRoots = rootsArr
	}

	b.markFieldAsDirty(types.StateRoots)
	b.rebuildTrie[types.StateRoots] = true
	return nil
}

// UpdateStateRootAtIndex for the beacon state. Updates the state root
// at a specific index to a new value.
func (b *BeaconState) UpdateStateRootAtIndex(idx uint64, stateRoot [32]byte) error {
	if features.Get().EnableExperimentalState {
		if err := b.stateRootsMultiValue.UpdateAt(b, idx, stateRoot); err != nil {
			return errors.Wrap(err, "could not update state roots")
		}
	} else {
		if uint64(len(b.stateRoots)) <= idx {
			return errors.Wrapf(consensus_types.ErrOutOfBounds, "state root index %d does not exist", idx)
		}

		b.lock.Lock()

		r := b.stateRoots
		if ref := b.sharedFieldReferences[types.StateRoots]; ref.Refs() > 1 {
			// Copy elements in underlying array by reference.
			r = make([][32]byte, len(b.stateRoots))
			copy(r, b.stateRoots)
			ref.MinusRef()
			b.sharedFieldReferences[types.StateRoots] = stateutil.NewRef(1)
		}
		r[idx] = stateRoot
		b.stateRoots = r

		b.lock.Unlock()
	}

	b.lock.Lock()
	defer b.lock.Unlock()

	b.markFieldAsDirty(types.StateRoots)
	b.addDirtyIndices(types.StateRoots, []uint64{idx})
	return nil
}
