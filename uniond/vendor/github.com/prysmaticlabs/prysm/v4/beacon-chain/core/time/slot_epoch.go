package time

import (
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// CurrentEpoch returns the current epoch number calculated from
// the slot number stored in beacon state.
//
// Spec pseudocode definition:
//
//	def get_current_epoch(state: BeaconState) -> Epoch:
//	  """
//	  Return the current epoch.
//	  """
//	  return compute_epoch_at_slot(state.slot)
func CurrentEpoch(state state.ReadOnlyBeaconState) primitives.Epoch {
	return slots.ToEpoch(state.Slot())
}

// PrevEpoch returns the previous epoch number calculated from
// the slot number stored in beacon state. It also checks for
// underflow condition.
//
// Spec pseudocode definition:
//
//	def get_previous_epoch(state: BeaconState) -> Epoch:
//	  """`
//	  Return the previous epoch (unless the current epoch is ``GENESIS_EPOCH``).
//	  """
//	  current_epoch = get_current_epoch(state)
//	  return GENESIS_EPOCH if current_epoch == GENESIS_EPOCH else Epoch(current_epoch - 1)
func PrevEpoch(state state.ReadOnlyBeaconState) primitives.Epoch {
	currentEpoch := CurrentEpoch(state)
	if currentEpoch == 0 {
		return 0
	}
	return currentEpoch - 1
}

// NextEpoch returns the next epoch number calculated from
// the slot number stored in beacon state.
func NextEpoch(state state.ReadOnlyBeaconState) primitives.Epoch {
	return slots.ToEpoch(state.Slot()) + 1
}

// HigherEqualThanAltairVersionAndEpoch returns if the input state `s` has a higher version number than Altair state and input epoch `e` is higher equal than fork epoch.
func HigherEqualThanAltairVersionAndEpoch(s state.BeaconState, e primitives.Epoch) bool {
	return s.Version() >= version.Altair && e >= params.BeaconConfig().AltairForkEpoch
}

// CanUpgradeToAltair returns true if the input `slot` can upgrade to Altair.
// Spec code:
// If state.slot % SLOTS_PER_EPOCH == 0 and compute_epoch_at_slot(state.slot) == ALTAIR_FORK_EPOCH
func CanUpgradeToAltair(slot primitives.Slot) bool {
	epochStart := slots.IsEpochStart(slot)
	altairEpoch := slots.ToEpoch(slot) == params.BeaconConfig().AltairForkEpoch
	return epochStart && altairEpoch
}

// CanUpgradeToBellatrix returns true if the input `slot` can upgrade to Bellatrix fork.
//
// Spec code:
// If state.slot % SLOTS_PER_EPOCH == 0 and compute_epoch_at_slot(state.slot) == BELLATRIX_FORK_EPOCH
func CanUpgradeToBellatrix(slot primitives.Slot) bool {
	epochStart := slots.IsEpochStart(slot)
	bellatrixEpoch := slots.ToEpoch(slot) == params.BeaconConfig().BellatrixForkEpoch
	return epochStart && bellatrixEpoch
}

// CanUpgradeToCapella returns true if the input `slot` can upgrade to Capella.
// Spec code:
// If state.slot % SLOTS_PER_EPOCH == 0 and compute_epoch_at_slot(state.slot) == CAPELLA_FORK_EPOCH
func CanUpgradeToCapella(slot primitives.Slot) bool {
	epochStart := slots.IsEpochStart(slot)
	capellaEpoch := slots.ToEpoch(slot) == params.BeaconConfig().CapellaForkEpoch
	return epochStart && capellaEpoch
}

// CanUpgradeToDeneb returns true if the input `slot` can upgrade to Deneb.
// Spec code:
// If state.slot % SLOTS_PER_EPOCH == 0 and compute_epoch_at_slot(state.slot) == DENEB_FORK_EPOCH
func CanUpgradeToDeneb(slot primitives.Slot) bool {
	epochStart := slots.IsEpochStart(slot)
	DenebEpoch := slots.ToEpoch(slot) == params.BeaconConfig().DenebForkEpoch
	return epochStart && DenebEpoch
}

// CanProcessEpoch checks the eligibility to process epoch.
// The epoch can be processed at the end of the last slot of every epoch.
//
// Spec pseudocode definition:
//
//	If (state.slot + 1) % SLOTS_PER_EPOCH == 0:
func CanProcessEpoch(state state.ReadOnlyBeaconState) bool {
	return (state.Slot()+1)%params.BeaconConfig().SlotsPerEpoch == 0
}
