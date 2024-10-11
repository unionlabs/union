package state_native

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stateutil"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	consensus_types "github.com/prysmaticlabs/prysm/v4/consensus-types"
)

// SetRandaoMixes for the beacon state. Updates the entire
// randao mixes to a new value by overwriting the previous one.
func (b *BeaconState) SetRandaoMixes(val [][]byte) error {
	b.lock.Lock()
	defer b.lock.Unlock()

	if features.Get().EnableExperimentalState {
		if b.randaoMixesMultiValue != nil {
			b.randaoMixesMultiValue.Detach(b)
		}
		b.randaoMixesMultiValue = NewMultiValueRandaoMixes(val)
	} else {
		b.sharedFieldReferences[types.RandaoMixes].MinusRef()
		b.sharedFieldReferences[types.RandaoMixes] = stateutil.NewRef(1)

		rootsArr := make([][32]byte, fieldparams.RandaoMixesLength)
		for i := 0; i < len(rootsArr); i++ {
			copy(rootsArr[i][:], val[i])
		}
		b.randaoMixes = rootsArr
	}

	b.markFieldAsDirty(types.RandaoMixes)
	b.rebuildTrie[types.RandaoMixes] = true
	return nil
}

// UpdateRandaoMixesAtIndex for the beacon state. Updates the randao mixes
// at a specific index to a new value.
func (b *BeaconState) UpdateRandaoMixesAtIndex(idx uint64, val [32]byte) error {
	if features.Get().EnableExperimentalState {
		if err := b.randaoMixesMultiValue.UpdateAt(b, idx, val); err != nil {
			return errors.Wrap(err, "could not update randao mixes")
		}
	} else {
		if uint64(len(b.randaoMixes)) <= idx {
			return errors.Wrapf(consensus_types.ErrOutOfBounds, "randao mix index %d does not exist", idx)
		}

		b.lock.Lock()

		m := b.randaoMixes
		if ref := b.sharedFieldReferences[types.RandaoMixes]; ref.Refs() > 1 {
			// Copy elements in underlying array by reference.
			m = make([][32]byte, len(b.randaoMixes))
			copy(m, b.randaoMixes)
			ref.MinusRef()
			b.sharedFieldReferences[types.RandaoMixes] = stateutil.NewRef(1)
		}
		m[idx] = val
		b.randaoMixes = m

		b.lock.Unlock()
	}

	b.lock.Lock()
	defer b.lock.Unlock()

	b.markFieldAsDirty(types.RandaoMixes)
	b.addDirtyIndices(types.RandaoMixes, []uint64{idx})
	return nil
}
