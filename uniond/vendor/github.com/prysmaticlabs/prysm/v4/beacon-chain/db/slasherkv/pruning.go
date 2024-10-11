package slasherkv

import (
	"bytes"
	"context"
	"encoding/binary"

	fssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	bolt "go.etcd.io/bbolt"
)

// PruneAttestationsAtEpoch deletes all attestations from the slasher DB with target epoch
// less than or equal to the specified epoch.
func (s *Store) PruneAttestationsAtEpoch(
	_ context.Context, maxEpoch primitives.Epoch,
) (numPruned uint, err error) {
	// We can prune everything less than the current epoch - history length.
	encodedEndPruneEpoch := fssz.MarshalUint64([]byte{}, uint64(maxEpoch))

	// We retrieve the lowest stored epoch in the attestations bucket.
	var lowestEpoch primitives.Epoch
	var hasData bool
	if err = s.db.View(func(tx *bolt.Tx) error {
		bkt := tx.Bucket(attestationDataRootsBucket)
		c := bkt.Cursor()
		k, _ := c.First()
		if k == nil {
			return nil
		}
		hasData = true
		lowestEpoch = primitives.Epoch(binary.LittleEndian.Uint64(k))
		return nil
	}); err != nil {
		return
	}

	// If there is no data stored, just exit early.
	if !hasData {
		return
	}

	// If the lowest epoch is greater than the end pruning epoch,
	// there is nothing to prune, so we return early.
	if lowestEpoch > maxEpoch {
		log.Debugf("Lowest epoch %d is > pruning epoch %d, nothing to prune", lowestEpoch, maxEpoch)
		return
	}

	if err = s.db.Update(func(tx *bolt.Tx) error {
		signingRootsBkt := tx.Bucket(attestationDataRootsBucket)
		attRecordsBkt := tx.Bucket(attestationRecordsBucket)
		c := signingRootsBkt.Cursor()

		// We begin a pruning iteration starting from the first item in the bucket.
		for k, v := c.First(); k != nil; k, v = c.Next() {
			// We check the epoch from the current key in the database.
			// If we have hit an epoch that is greater than the end epoch of the pruning process,
			// we then completely exit the process as we are done.
			if uint64PrefixGreaterThan(k, encodedEndPruneEpoch) {
				return nil
			}

			// Attestation in the database look like this:
			//  (target_epoch ++ _) => encode(attestation)
			// so it is possible we have a few adjacent objects that have the same slot, such as
			//  (target_epoch = 3 ++ _) => encode(attestation)
			if err := signingRootsBkt.Delete(k); err != nil {
				return err
			}
			if err := attRecordsBkt.Delete(v); err != nil {
				return err
			}
			slasherAttestationsPrunedTotal.Inc()
			numPruned++
		}
		return nil
	}); err != nil {
		return
	}
	return
}

// PruneProposalsAtEpoch deletes all proposals from the slasher DB with epoch
// less than or equal to the specified epoch.
func (s *Store) PruneProposalsAtEpoch(
	ctx context.Context, maxEpoch primitives.Epoch,
) (numPruned uint, err error) {
	var endPruneSlot primitives.Slot
	endPruneSlot, err = slots.EpochEnd(maxEpoch)
	if err != nil {
		return
	}
	encodedEndPruneSlot := fssz.MarshalUint64([]byte{}, uint64(endPruneSlot))

	// We retrieve the lowest stored slot in the proposals bucket.
	var lowestSlot primitives.Slot
	var hasData bool
	if err = s.db.View(func(tx *bolt.Tx) error {
		proposalBkt := tx.Bucket(proposalRecordsBucket)
		c := proposalBkt.Cursor()
		k, _ := c.First()
		if k == nil {
			return nil
		}
		hasData = true
		lowestSlot = slotFromProposalKey(k)
		return nil
	}); err != nil {
		return
	}

	// If there is no data stored, just exit early.
	if !hasData {
		return
	}

	// If the lowest slot is greater than the end pruning slot,
	// there is nothing to prune, so we return early.
	if lowestSlot > endPruneSlot {
		log.Debugf("Lowest slot %d is > pruning slot %d, nothing to prune", lowestSlot, endPruneSlot)
		return
	}

	if err = s.db.Update(func(tx *bolt.Tx) error {
		proposalBkt := tx.Bucket(proposalRecordsBucket)
		c := proposalBkt.Cursor()
		// We begin a pruning iteration starting from the first item in the bucket.
		for k, _ := c.First(); k != nil; k, _ = c.Next() {
			if ctx.Err() != nil {
				return ctx.Err()
			}
			// We check the slot from the current key in the database.
			// If we have hit a slot that is greater than the end slot of the pruning process,
			// we then completely exit the process as we are done.
			if uint64PrefixGreaterThan(k, encodedEndPruneSlot) {
				return nil
			}
			// Proposals in the database look like this:
			//  (slot ++ validatorIndex) => encode(proposal)
			// so it is possible we have a few adjacent objects that have the same slot, such as
			//  (slot = 3 ++ validatorIndex = 0) => ...
			//  (slot = 3 ++ validatorIndex = 1) => ...
			//  (slot = 3 ++ validatorIndex = 2) => ...
			if err := proposalBkt.Delete(k); err != nil {
				return err
			}
			slasherProposalsPrunedTotal.Inc()
			numPruned++
		}
		return nil
	}); err != nil {
		return
	}
	return
}

func slotFromProposalKey(key []byte) primitives.Slot {
	return primitives.Slot(binary.LittleEndian.Uint64(key[:8]))
}

func uint64PrefixGreaterThan(key, lessThan []byte) bool {
	enc := key[:8]
	return bytes.Compare(enc, lessThan) > 0
}
