package state_native

import (
	"github.com/pkg/errors"
	customtypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native/custom-types"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	consensus_types "github.com/prysmaticlabs/prysm/v4/consensus-types"
)

// RandaoMixes of block proposers on the beacon chain.
func (b *BeaconState) RandaoMixes() [][]byte {
	b.lock.RLock()
	defer b.lock.RUnlock()

	mixes := b.randaoMixesVal()
	if mixes == nil {
		return nil
	}
	return mixes.Slice()
}

func (b *BeaconState) randaoMixesVal() customtypes.RandaoMixes {
	if features.Get().EnableExperimentalState {
		if b.randaoMixesMultiValue == nil {
			return nil
		}
		return b.randaoMixesMultiValue.Value(b)
	}
	return b.randaoMixes
}

// RandaoMixAtIndex retrieves a specific block root based on an
// input index value.
func (b *BeaconState) RandaoMixAtIndex(idx uint64) ([]byte, error) {
	b.lock.RLock()
	defer b.lock.RUnlock()

	if features.Get().EnableExperimentalState {
		if b.randaoMixesMultiValue == nil {
			return nil, nil
		}
		r, err := b.randaoMixesMultiValue.At(b, idx)
		if err != nil {
			return nil, err
		}
		return r[:], nil
	}

	if b.randaoMixes == nil {
		return nil, nil
	}
	m, err := b.randaoMixAtIndex(idx)
	if err != nil {
		return nil, err
	}
	return m[:], nil
}

// randaoMixAtIndex retrieves a specific block root based on an
// input index value.
// This assumes that a lock is already held on BeaconState.
func (b *BeaconState) randaoMixAtIndex(idx uint64) ([32]byte, error) {
	if uint64(len(b.randaoMixes)) <= idx {
		return [32]byte{}, errors.Wrapf(consensus_types.ErrOutOfBounds, "randao mix index %d does not exist", idx)
	}

	return b.randaoMixes[idx], nil
}

// RandaoMixesLength returns the length of the randao mixes slice.
func (b *BeaconState) RandaoMixesLength() int {
	b.lock.RLock()
	defer b.lock.RUnlock()

	if features.Get().EnableExperimentalState {
		if b.randaoMixesMultiValue == nil {
			return 0
		}
		return b.randaoMixesMultiValue.Len(b)
	}
	if b.randaoMixes == nil {
		return 0
	}
	return len(b.randaoMixes)
}
