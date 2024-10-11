package doublylinkedtree

import (
	"context"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// NewSlot mimics the implementation of `on_tick` in fork choice consensus spec.
// It resets the proposer boost root in fork choice, and it updates store's justified checkpoint
// if a better checkpoint on the store's finalized checkpoint chain.
// This should only be called at the start of every slot interval.
//
// Spec pseudocode definition:
//
//	# Reset store.proposer_boost_root if this is a new slot
//	if current_slot > previous_slot:
//	    store.proposer_boost_root = Root()
//
//	# Not a new epoch, return
//	if not (current_slot > previous_slot and compute_slots_since_epoch_start(current_slot) == 0):
//	    return
//
//	# Update store.justified_checkpoint if a better checkpoint on the store.finalized_checkpoint chain
//	if store.best_justified_checkpoint.epoch > store.justified_checkpoint.epoch:
//	    finalized_slot = compute_start_slot_at_epoch(store.finalized_checkpoint.epoch)
//	    ancestor_at_finalized_slot = get_ancestor(store, store.best_justified_checkpoint.root, finalized_slot)
//	    if ancestor_at_finalized_slot == store.finalized_checkpoint.root:
//	        store.justified_checkpoint = store.best_justified_checkpoint
func (f *ForkChoice) NewSlot(ctx context.Context, slot primitives.Slot) error {
	// Reset proposer boost root
	f.store.proposerBoostRoot = [32]byte{}

	// Return if it's not a new epoch.
	if !slots.IsEpochStart(slot) {
		return nil
	}

	// Update store.justified_checkpoint if a better checkpoint on the store.finalized_checkpoint chain
	if err := f.updateUnrealizedCheckpoints(ctx); err != nil {
		return errors.Wrap(err, "could not update unrealized checkpoints")
	}
	return f.store.prune(ctx)
}
