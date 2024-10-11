package client

import (
	"context"
	"fmt"

	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/sirupsen/logrus"
)

var failedBlockSignLocalErr = "attempted to sign a double proposal, block rejected by local protection"

// slashableProposalCheck checks if a block proposal is slashable by comparing it with the
// block proposals history for the given public key in our DB. If it is not, we then update the history
// with new values and save it to the database.
func (v *validator) slashableProposalCheck(
	ctx context.Context, pubKey [fieldparams.BLSPubkeyLength]byte, signedBlock interfaces.ReadOnlySignedBeaconBlock, signingRoot [32]byte,
) error {
	fmtKey := fmt.Sprintf("%#x", pubKey[:])

	blk := signedBlock.Block()
	prevSigningRoot, proposalAtSlotExists, prevSigningRootExists, err := v.db.ProposalHistoryForSlot(ctx, pubKey, blk.Slot())
	if err != nil {
		if v.emitAccountMetrics {
			ValidatorProposeFailVec.WithLabelValues(fmtKey).Inc()
		}
		return errors.Wrap(err, "failed to get proposal history")
	}

	lowestSignedProposalSlot, lowestProposalExists, err := v.db.LowestSignedProposal(ctx, pubKey)
	if err != nil {
		return err
	}

	// Based on EIP-3076 - Condition 2
	// -------------------------------
	if lowestProposalExists {
		// If the block slot is (strictly) less than the lowest signed proposal slot in the DB, we consider it slashable.
		if blk.Slot() < lowestSignedProposalSlot {
			return fmt.Errorf(
				"could not sign block with slot < lowest signed slot in db, block slot: %d < lowest signed slot: %d",
				blk.Slot(),
				lowestSignedProposalSlot,
			)
		}

		// If the block slot is equal to the lowest signed proposal slot and
		// - condition1: there is no signed proposal in the DB for this slot, or
		// - condition2: there is  a signed proposal in the DB for this slot, but with no associated signing root, or
		// - condition3: there is  a signed proposal in the DB for this slot, but the signing root differs,
		// ==> we consider it slashable.
		condition1 := !proposalAtSlotExists
		condition2 := proposalAtSlotExists && !prevSigningRootExists
		condition3 := proposalAtSlotExists && prevSigningRootExists && prevSigningRoot != signingRoot
		if blk.Slot() == lowestSignedProposalSlot && (condition1 || condition2 || condition3) {
			return fmt.Errorf(
				"could not sign block with slot == lowest signed slot in db if it is not a repeat signing, block slot: %d == slowest signed slot: %d",
				blk.Slot(),
				lowestSignedProposalSlot,
			)
		}
	}

	// Based on EIP-3076 - Condition 1
	// -------------------------------
	// If there is a signed proposal in the DB for this slot and
	// - there is no associated signing root, or
	// - the signing root differs,
	// ==> we consider it slashable.
	if proposalAtSlotExists && (!prevSigningRootExists || prevSigningRoot != signingRoot) {
		if v.emitAccountMetrics {
			ValidatorProposeFailVec.WithLabelValues(fmtKey).Inc()
		}
		return errors.New(failedBlockSignLocalErr)
	}

	// Save the proposal for this slot.
	if err := v.db.SaveProposalHistoryForSlot(ctx, pubKey, blk.Slot(), signingRoot[:]); err != nil {
		if v.emitAccountMetrics {
			ValidatorProposeFailVec.WithLabelValues(fmtKey).Inc()
		}
		return errors.Wrap(err, "failed to save updated proposal history")
	}

	return nil
}

func blockLogFields(pubKey [fieldparams.BLSPubkeyLength]byte, blk interfaces.ReadOnlyBeaconBlock, sig []byte) logrus.Fields {
	fields := logrus.Fields{
		"proposerPublicKey": fmt.Sprintf("%#x", pubKey),
		"proposerIndex":     blk.ProposerIndex(),
		"blockSlot":         blk.Slot(),
	}
	if sig != nil {
		fields["signature"] = fmt.Sprintf("%#x", sig)
	}
	return fields
}
