package helpers

import (
	"math"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// BlockRootAtSlot returns the block root stored in the BeaconState for a recent slot.
// It returns an error if the requested block root is not within the slot range.
//
// Spec pseudocode definition:
//
//	def get_block_root_at_slot(state: BeaconState, slot: Slot) -> Root:
//	  """
//	  Return the block root at a recent ``slot``.
//	  """
//	  assert slot < state.slot <= slot + SLOTS_PER_HISTORICAL_ROOT
//	  return state.block_roots[slot % SLOTS_PER_HISTORICAL_ROOT]
func BlockRootAtSlot(state state.ReadOnlyBeaconState, slot primitives.Slot) ([]byte, error) {
	if math.MaxUint64-slot < params.BeaconConfig().SlotsPerHistoricalRoot {
		return []byte{}, errors.New("slot overflows uint64")
	}
	if slot >= state.Slot() || state.Slot() > slot+params.BeaconConfig().SlotsPerHistoricalRoot {
		return []byte{}, errors.Errorf("slot %d out of bounds", slot)
	}
	return state.BlockRootAtIndex(uint64(slot % params.BeaconConfig().SlotsPerHistoricalRoot))
}

// StateRootAtSlot returns the cached state root at that particular slot. If no state
// root has been cached it will return a zero-hash.
func StateRootAtSlot(state state.ReadOnlyBeaconState, slot primitives.Slot) ([]byte, error) {
	if slot >= state.Slot() || state.Slot() > slot+params.BeaconConfig().SlotsPerHistoricalRoot {
		return []byte{}, errors.Errorf("slot %d out of bounds", slot)
	}
	return state.StateRootAtIndex(uint64(slot % params.BeaconConfig().SlotsPerHistoricalRoot))
}

// BlockRoot returns the block root stored in the BeaconState for epoch start slot.
//
// Spec pseudocode definition:
//
//	def get_block_root(state: BeaconState, epoch: Epoch) -> Root:
//	  """
//	  Return the block root at the start of a recent ``epoch``.
//	  """
//	  return get_block_root_at_slot(state, compute_start_slot_at_epoch(epoch))
func BlockRoot(state state.ReadOnlyBeaconState, epoch primitives.Epoch) ([]byte, error) {
	s, err := slots.EpochStart(epoch)
	if err != nil {
		return nil, err
	}
	return BlockRootAtSlot(state, s)
}
