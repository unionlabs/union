package client

import (
	"context"
	"encoding/hex"
	"fmt"

	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/slashings"
	"github.com/prysmaticlabs/prysm/v4/validator/db/kv"
	"go.opencensus.io/trace"
)

var failedAttLocalProtectionErr = "attempted to make slashable attestation, rejected by local slashing protection"

// Checks if an attestation is slashable by comparing it with the attesting
// history for the given public key in our DB. If it is not, we then update the history
// with new values and save it to the database.
func (v *validator) slashableAttestationCheck(
	ctx context.Context,
	indexedAtt *ethpb.IndexedAttestation,
	pubKey [fieldparams.BLSPubkeyLength]byte,
	signingRoot32 [32]byte,
) error {
	ctx, span := trace.StartSpan(ctx, "validator.postAttSignUpdate")
	defer span.End()

	signingRoot := signingRoot32[:]

	// Based on EIP3076, validator should refuse to sign any attestation with source epoch less
	// than the minimum source epoch present in that signer’s attestations.
	lowestSourceEpoch, exists, err := v.db.LowestSignedSourceEpoch(ctx, pubKey)
	if err != nil {
		return err
	}
	if exists && indexedAtt.Data.Source.Epoch < lowestSourceEpoch {
		return fmt.Errorf(
			"could not sign attestation lower than lowest source epoch in db, %d < %d",
			indexedAtt.Data.Source.Epoch,
			lowestSourceEpoch,
		)
	}
	existingSigningRoot, err := v.db.SigningRootAtTargetEpoch(ctx, pubKey, indexedAtt.Data.Target.Epoch)
	if err != nil {
		return err
	}
	signingRootsDiffer := slashings.SigningRootsDiffer(existingSigningRoot, signingRoot)

	// Based on EIP3076, validator should refuse to sign any attestation with target epoch less
	// than or equal to the minimum target epoch present in that signer’s attestations.
	lowestTargetEpoch, exists, err := v.db.LowestSignedTargetEpoch(ctx, pubKey)
	if err != nil {
		return err
	}
	if signingRootsDiffer && exists && indexedAtt.Data.Target.Epoch <= lowestTargetEpoch {
		return fmt.Errorf(
			"could not sign attestation lower than or equal to lowest target epoch in db, %d <= %d",
			indexedAtt.Data.Target.Epoch,
			lowestTargetEpoch,
		)
	}
	fmtKey := "0x" + hex.EncodeToString(pubKey[:])
	slashingKind, err := v.db.CheckSlashableAttestation(ctx, pubKey, signingRoot, indexedAtt)
	if err != nil {
		if v.emitAccountMetrics {
			ValidatorAttestFailVec.WithLabelValues(fmtKey).Inc()
		}
		switch slashingKind {
		case kv.DoubleVote:
			log.Warn("Attestation is slashable as it is a double vote")
		case kv.SurroundingVote:
			log.Warn("Attestation is slashable as it is surrounding a previous attestation")
		case kv.SurroundedVote:
			log.Warn("Attestation is slashable as it is surrounded by a previous attestation")
		}
		return errors.Wrap(err, failedAttLocalProtectionErr)
	}

	if err := v.db.SaveAttestationForPubKey(ctx, pubKey, signingRoot32, indexedAtt); err != nil {
		return errors.Wrap(err, "could not save attestation history for validator public key")
	}

	return nil
}
