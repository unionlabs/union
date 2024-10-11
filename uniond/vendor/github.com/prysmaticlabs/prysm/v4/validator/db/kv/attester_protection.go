package kv

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/slashings"
	bolt "go.etcd.io/bbolt"
	"go.opencensus.io/trace"
)

// SlashingKind used for helpful information upon detection.
type SlashingKind int

// AttestationRecordSaveRequest includes the attestation record to save along
// with the appropriate call context.
type AttestationRecordSaveRequest struct {
	ctx    context.Context
	record *AttestationRecord
}

// AttestationRecord which can be represented by these simple values
// for manipulation by database methods.
type AttestationRecord struct {
	PubKey      [fieldparams.BLSPubkeyLength]byte
	Source      primitives.Epoch
	Target      primitives.Epoch
	SigningRoot []byte
}

// NewQueuedAttestationRecords constructor allocates the underlying slice and
// required attributes for managing pending attestation records.
func NewQueuedAttestationRecords() *QueuedAttestationRecords {
	return &QueuedAttestationRecords{
		records: make([]*AttestationRecord, 0, attestationBatchCapacity),
	}
}

// QueuedAttestationRecords is a thread-safe struct for managing a queue of
// attestation records to save to validator database.
type QueuedAttestationRecords struct {
	records []*AttestationRecord
	lock    sync.RWMutex
}

// Append a new attestation record to the queue.
func (p *QueuedAttestationRecords) Append(ar *AttestationRecord) {
	p.lock.Lock()
	defer p.lock.Unlock()
	p.records = append(p.records, ar)
}

// Flush all records. This method returns the current pending records and resets
// the pending records slice.
func (p *QueuedAttestationRecords) Flush() []*AttestationRecord {
	p.lock.Lock()
	defer p.lock.Unlock()
	recs := p.records
	p.records = make([]*AttestationRecord, 0, attestationBatchCapacity)
	return recs
}

// Len returns the current length of records.
func (p *QueuedAttestationRecords) Len() int {
	p.lock.RLock()
	defer p.lock.RUnlock()
	return len(p.records)
}

// A wrapper over an error received from a background routine
// saving batched attestations for slashing protection.
// This wrapper allows us to send this response over event feeds,
// as our event feed does not allow sending `nil` values to
// subscribers.
type saveAttestationsResponse struct {
	err error
}

// Enums representing the types of slashable events for attesters.
const (
	NotSlashable SlashingKind = iota
	DoubleVote
	SurroundingVote
	SurroundedVote
)

var (
	doubleVoteMessage      = "double vote found, existing attestation at target epoch %d with conflicting signing root %#x"
	surroundingVoteMessage = "attestation with (source %d, target %d) surrounds another with (source %d, target %d)"
	surroundedVoteMessage  = "attestation with (source %d, target %d) is surrounded by another with (source %d, target %d)"
)

// AttestationHistoryForPubKey retrieves a list of attestation records for data
// we have stored in the database for the given validator public key.
func (s *Store) AttestationHistoryForPubKey(ctx context.Context, pubKey [fieldparams.BLSPubkeyLength]byte) ([]*AttestationRecord, error) {
	records := make([]*AttestationRecord, 0)
	_, span := trace.StartSpan(ctx, "Validator.AttestationHistoryForPubKey")
	defer span.End()
	err := s.view(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(pubKeysBucket)
		pkBucket := bucket.Bucket(pubKey[:])
		if pkBucket == nil {
			return nil
		}
		signingRootsBucket := pkBucket.Bucket(attestationSigningRootsBucket)
		sourceEpochsBucket := pkBucket.Bucket(attestationSourceEpochsBucket)

		return sourceEpochsBucket.ForEach(func(sourceBytes, targetEpochsList []byte) error {
			targetEpochs := make([]primitives.Epoch, 0)
			for i := 0; i < len(targetEpochsList); i += 8 {
				epoch := bytesutil.BytesToEpochBigEndian(targetEpochsList[i : i+8])
				targetEpochs = append(targetEpochs, epoch)
			}
			sourceEpoch := bytesutil.BytesToEpochBigEndian(sourceBytes)
			for _, targetEpoch := range targetEpochs {
				record := &AttestationRecord{
					PubKey: pubKey,
					Source: sourceEpoch,
					Target: targetEpoch,
				}
				signingRoot := signingRootsBucket.Get(bytesutil.EpochToBytesBigEndian(targetEpoch))
				if len(signingRoot) != 0 {
					record.SigningRoot = make([]byte, fieldparams.RootLength)
					copy(record.SigningRoot, signingRoot)
				}
				records = append(records, record)
			}
			return nil
		})
	})
	return records, err
}

// CheckSlashableAttestation verifies an incoming attestation is
// not a double vote for a validator public key nor a surround vote.
func (s *Store) CheckSlashableAttestation(
	ctx context.Context, pubKey [fieldparams.BLSPubkeyLength]byte, signingRoot []byte, att *ethpb.IndexedAttestation,
) (SlashingKind, error) {
	ctx, span := trace.StartSpan(ctx, "Validator.CheckSlashableAttestation")
	defer span.End()
	var slashKind SlashingKind
	err := s.view(func(tx *bolt.Tx) error {
		if ctx.Err() != nil {
			return ctx.Err()
		}
		bucket := tx.Bucket(pubKeysBucket)
		pkBucket := bucket.Bucket(pubKey[:])
		if pkBucket == nil {
			return nil
		}

		// First we check for double votes.
		signingRootsBucket := pkBucket.Bucket(attestationSigningRootsBucket)
		if signingRootsBucket != nil {
			targetEpochBytes := bytesutil.EpochToBytesBigEndian(att.Data.Target.Epoch)
			existingSigningRoot := signingRootsBucket.Get(targetEpochBytes)

			// If a signing root exists in the database, and if this database signing root is empty => We consider the new attestation as a double vote.
			// If a signing root exists in the database, and if this database signing differs from the signing root of the new attestation => We consider the new attestation as a double vote.
			if existingSigningRoot != nil && (len(existingSigningRoot) == 0 || slashings.SigningRootsDiffer(existingSigningRoot, signingRoot)) {
				slashKind = DoubleVote
				return fmt.Errorf(doubleVoteMessage, att.Data.Target.Epoch, existingSigningRoot)
			}
		}

		sourceEpochsBucket := pkBucket.Bucket(attestationSourceEpochsBucket)
		targetEpochsBucket := pkBucket.Bucket(attestationTargetEpochsBucket)
		if sourceEpochsBucket == nil {
			return nil
		}

		// Is this attestation surrounding any other?
		var err error
		slashKind, err = s.checkSurroundingVote(sourceEpochsBucket, att)
		if err != nil {
			return err
		}
		if targetEpochsBucket == nil {
			return nil
		}

		// Is this attestation surrounded by any other?
		slashKind, err = s.checkSurroundedVote(targetEpochsBucket, att)
		if err != nil {
			return err
		}
		return nil
	})

	tracing.AnnotateError(span, err)
	return slashKind, err
}

// Iterate from the back of the bucket since we are looking for target_epoch > att.target_epoch
func (_ *Store) checkSurroundedVote(
	targetEpochsBucket *bolt.Bucket, att *ethpb.IndexedAttestation,
) (SlashingKind, error) {
	c := targetEpochsBucket.Cursor()
	for k, v := c.Last(); k != nil; k, v = c.Prev() {
		existingTargetEpoch := bytesutil.BytesToEpochBigEndian(k)
		if existingTargetEpoch <= att.Data.Target.Epoch {
			break
		}

		// There can be multiple source epochs attested per target epoch.
		attestedSourceEpochs := make([]primitives.Epoch, 0, len(v)/8)
		for i := 0; i < len(v); i += 8 {
			sourceEpoch := bytesutil.BytesToEpochBigEndian(v[i : i+8])
			attestedSourceEpochs = append(attestedSourceEpochs, sourceEpoch)
		}

		for _, existingSourceEpoch := range attestedSourceEpochs {
			existingAtt := &ethpb.IndexedAttestation{
				Data: &ethpb.AttestationData{
					Source: &ethpb.Checkpoint{Epoch: existingSourceEpoch},
					Target: &ethpb.Checkpoint{Epoch: existingTargetEpoch},
				},
			}
			surrounded := slashings.IsSurround(existingAtt, att)
			if surrounded {
				return SurroundedVote, fmt.Errorf(
					surroundedVoteMessage,
					att.Data.Source.Epoch,
					att.Data.Target.Epoch,
					existingSourceEpoch,
					existingTargetEpoch,
				)
			}
		}
	}
	return NotSlashable, nil
}

// Iterate from the back of the bucket since we are looking for source_epoch > att.source_epoch
func (_ *Store) checkSurroundingVote(
	sourceEpochsBucket *bolt.Bucket, att *ethpb.IndexedAttestation,
) (SlashingKind, error) {
	c := sourceEpochsBucket.Cursor()
	for k, v := c.Last(); k != nil; k, v = c.Prev() {
		existingSourceEpoch := bytesutil.BytesToEpochBigEndian(k)
		if existingSourceEpoch <= att.Data.Source.Epoch {
			break
		}

		// There can be multiple target epochs attested per source epoch.
		attestedTargetEpochs := make([]primitives.Epoch, 0, len(v)/8)
		for i := 0; i < len(v); i += 8 {
			targetEpoch := bytesutil.BytesToEpochBigEndian(v[i : i+8])
			attestedTargetEpochs = append(attestedTargetEpochs, targetEpoch)
		}

		for _, existingTargetEpoch := range attestedTargetEpochs {
			existingAtt := &ethpb.IndexedAttestation{
				Data: &ethpb.AttestationData{
					Source: &ethpb.Checkpoint{Epoch: existingSourceEpoch},
					Target: &ethpb.Checkpoint{Epoch: existingTargetEpoch},
				},
			}
			surrounding := slashings.IsSurround(att, existingAtt)
			if surrounding {
				return SurroundingVote, fmt.Errorf(
					surroundingVoteMessage,
					att.Data.Source.Epoch,
					att.Data.Target.Epoch,
					existingSourceEpoch,
					existingTargetEpoch,
				)
			}
		}
	}
	return NotSlashable, nil
}

// SaveAttestationsForPubKey stores a batch of attestations all at once.
func (s *Store) SaveAttestationsForPubKey(
	ctx context.Context, pubKey [fieldparams.BLSPubkeyLength]byte, signingRoots [][]byte, atts []*ethpb.IndexedAttestation,
) error {
	ctx, span := trace.StartSpan(ctx, "Validator.SaveAttestationsForPubKey")
	defer span.End()
	if len(signingRoots) != len(atts) {
		return fmt.Errorf(
			"number of signing roots %d does not match number of attestations %d",
			len(signingRoots),
			len(atts),
		)
	}
	records := make([]*AttestationRecord, len(atts))
	for i, a := range atts {
		records[i] = &AttestationRecord{
			PubKey:      pubKey,
			Source:      a.Data.Source.Epoch,
			Target:      a.Data.Target.Epoch,
			SigningRoot: signingRoots[i],
		}
	}
	return s.saveAttestationRecords(ctx, records)
}

// SaveAttestationForPubKey saves an attestation for a validator public
// key for local validator slashing protection.
func (s *Store) SaveAttestationForPubKey(
	ctx context.Context, pubKey [fieldparams.BLSPubkeyLength]byte, signingRoot [32]byte, att *ethpb.IndexedAttestation,
) error {
	ctx, span := trace.StartSpan(ctx, "Validator.SaveAttestationForPubKey")
	defer span.End()
	s.batchedAttestationsChan <- &AttestationRecordSaveRequest{
		ctx: ctx,
		record: &AttestationRecord{
			PubKey:      pubKey,
			Source:      att.Data.Source.Epoch,
			Target:      att.Data.Target.Epoch,
			SigningRoot: signingRoot[:],
		},
	}

	// Subscribe to be notified when the attestation record queued
	// for saving to the DB is indeed saved. If an error occurred
	// during the process of saving the attestation record, the sender
	// will give us that error. We use a buffered channel
	// to prevent blocking the sender from notifying us of the result.
	_, innerSpan := trace.StartSpan(ctx, "Validator.SaveAttestationForPubKey.WaitForResponse")
	defer innerSpan.End()
	responseChan := make(chan saveAttestationsResponse, 1)
	defer close(responseChan)
	sub := s.batchAttestationsFlushedFeed.Subscribe(responseChan)
	defer sub.Unsubscribe()
	res := <-responseChan
	return res.err
}

// Meant to run as a background routine, this function checks whether:
// (a) we have reached a max capacity of batched attestations in the Store or
// (b) attestationBatchWriteInterval has passed
// Based on whichever comes first, this function then proceeds
// to flush the attestations to the DB all at once in a single boltDB
// transaction for efficiency. Then, batched attestations slice is emptied out.
func (s *Store) batchAttestationWrites(ctx context.Context) {
	ticker := time.NewTicker(attestationBatchWriteInterval)
	defer ticker.Stop()
	for {
		select {
		case v := <-s.batchedAttestationsChan:
			_, span := trace.StartSpan(v.ctx, "batchAttestationWrites.handleBatchedAttestationSaveRequest")
			s.batchedAttestations.Append(v.record)

			span.AddAttributes(trace.Int64Attribute("num_records", int64(s.batchedAttestations.Len())))

			if numRecords := s.batchedAttestations.Len(); numRecords >= attestationBatchCapacity {
				log.WithField("numRecords", numRecords).Debug(
					"Reached max capacity of batched attestation records, flushing to DB",
				)
				if s.batchedAttestationsFlushInProgress.IsNotSet() {
					// Create a new context with the span information from the chan. This is to
					// prevent any context deadlines from the caller while maintaining the trace
					// relationships.
					ctx2 := trace.NewContext(ctx, span)
					s.flushAttestationRecords(ctx2, s.batchedAttestations.Flush())
				}
			}
			span.End()
		case <-ticker.C:
			if numRecords := s.batchedAttestations.Len(); numRecords > 0 {
				log.WithField("numRecords", numRecords).Debug(
					"Batched attestation records write interval reached, flushing to DB",
				)
				if s.batchedAttestationsFlushInProgress.IsNotSet() {
					s.flushAttestationRecords(ctx, s.batchedAttestations.Flush())
				}
			}
		case <-ctx.Done():
			return
		}
	}
}

// Flushes a list of batched attestations to the database
// and resets the list of batched attestations for future writes.
// This function notifies all subscribers for flushed attestations
// of the result of the save operation.
func (s *Store) flushAttestationRecords(ctx context.Context, records []*AttestationRecord) {
	ctx, span := trace.StartSpan(ctx, "validatorDB.flushAttestationRecords")
	defer span.End()

	if s.batchedAttestationsFlushInProgress.IsSet() {
		// This should never happen. This method should not be called when a flush is already in
		// progress. If you are seeing this log, check the atomic bool before calling this method.
		log.Error("Attempted to flush attestation records when already in progress")
		return
	}
	s.batchedAttestationsFlushInProgress.Set()
	defer s.batchedAttestationsFlushInProgress.UnSet()

	start := time.Now()
	err := s.saveAttestationRecords(ctx, records)
	// If there was any error, retry the records since the TX would have been reverted.
	if err == nil {
		log.WithField("duration", time.Since(start)).Debug("Successfully flushed batched attestations to DB")
	} else {
		// This should never happen.
		log.WithError(err).Error("Failed to batch save attestation records, retrying in queue")
		tracing.AnnotateError(span, err)
		for _, ar := range records {
			s.batchedAttestations.Append(ar)
		}
	}
	// Forward the error, if any, to all subscribers via an event feed.
	// We use a struct wrapper around the error as the event feed
	// cannot handle sending a raw `nil` in case there is no error.
	s.batchAttestationsFlushedFeed.Send(saveAttestationsResponse{
		err: err,
	})
}

// Saves a list of attestation records to the database in a single boltDB
// transaction to minimize write lock contention compared to doing them
// all in individual, isolated boltDB transactions.
func (s *Store) saveAttestationRecords(ctx context.Context, atts []*AttestationRecord) error {
	_, span := trace.StartSpan(ctx, "Validator.saveAttestationRecords")
	defer span.End()
	return s.update(func(tx *bolt.Tx) error {
		// Initialize buckets for the lowest target and source epochs.
		lowestSourceBucket, err := tx.CreateBucketIfNotExists(lowestSignedSourceBucket)
		if err != nil {
			return err
		}
		lowestTargetBucket, err := tx.CreateBucketIfNotExists(lowestSignedTargetBucket)
		if err != nil {
			return err
		}
		bucket := tx.Bucket(pubKeysBucket)
		for _, att := range atts {
			pkBucket, err := bucket.CreateBucketIfNotExists(att.PubKey[:])
			if err != nil {
				return errors.Wrap(err, "could not create public key bucket")
			}
			sourceEpochBytes := bytesutil.EpochToBytesBigEndian(att.Source)
			targetEpochBytes := bytesutil.EpochToBytesBigEndian(att.Target)

			signingRootsBucket, err := pkBucket.CreateBucketIfNotExists(attestationSigningRootsBucket)
			if err != nil {
				return errors.Wrap(err, "could not create signing roots bucket")
			}
			if err := signingRootsBucket.Put(targetEpochBytes, att.SigningRoot); err != nil {
				return errors.Wrapf(err, "could not save signing root for epoch %d", att.Target)
			}
			sourceEpochsBucket, err := pkBucket.CreateBucketIfNotExists(attestationSourceEpochsBucket)
			if err != nil {
				return errors.Wrap(err, "could not create source epochs bucket")
			}

			// There can be multiple attested target epochs per source epoch.
			// If a previous list exists, we append to that list with the incoming target epoch.
			// Otherwise, we initialize it using the incoming target epoch.
			var existingAttestedTargetsBytes []byte
			if existing := sourceEpochsBucket.Get(sourceEpochBytes); existing != nil {
				existingAttestedTargetsBytes = append(existing, targetEpochBytes...)
			} else {
				existingAttestedTargetsBytes = targetEpochBytes
			}

			if err := sourceEpochsBucket.Put(sourceEpochBytes, existingAttestedTargetsBytes); err != nil {
				return errors.Wrapf(err, "could not save source epoch %d for epoch %d", att.Source, att.Target)
			}

			targetEpochsBucket, err := pkBucket.CreateBucketIfNotExists(attestationTargetEpochsBucket)
			if err != nil {
				return errors.Wrap(err, "could not create target epochs bucket")
			}
			var existingAttestedSourceBytes []byte
			if existing := targetEpochsBucket.Get(targetEpochBytes); existing != nil {
				existingAttestedSourceBytes = append(existing, sourceEpochBytes...)
			} else {
				existingAttestedSourceBytes = sourceEpochBytes
			}

			if err := targetEpochsBucket.Put(targetEpochBytes, existingAttestedSourceBytes); err != nil {
				return errors.Wrapf(err, "could not save target epoch %d for epoch %d", att.Target, att.Source)
			}

			// If the incoming source epoch is lower than the lowest signed source epoch, override.
			lowestSignedSourceBytes := lowestSourceBucket.Get(att.PubKey[:])
			var lowestSignedSourceEpoch primitives.Epoch
			if len(lowestSignedSourceBytes) >= 8 {
				lowestSignedSourceEpoch = bytesutil.BytesToEpochBigEndian(lowestSignedSourceBytes)
			}
			if len(lowestSignedSourceBytes) == 0 || att.Source < lowestSignedSourceEpoch {
				if err := lowestSourceBucket.Put(
					att.PubKey[:], bytesutil.EpochToBytesBigEndian(att.Source),
				); err != nil {
					return err
				}
			}

			// If the incoming target epoch is lower than the lowest signed target epoch, override.
			lowestSignedTargetBytes := lowestTargetBucket.Get(att.PubKey[:])
			var lowestSignedTargetEpoch primitives.Epoch
			if len(lowestSignedTargetBytes) >= 8 {
				lowestSignedTargetEpoch = bytesutil.BytesToEpochBigEndian(lowestSignedTargetBytes)
			}
			if len(lowestSignedTargetBytes) == 0 || att.Target < lowestSignedTargetEpoch {
				if err := lowestTargetBucket.Put(
					att.PubKey[:], bytesutil.EpochToBytesBigEndian(att.Target),
				); err != nil {
					return err
				}
			}
		}
		return nil
	})
}

// AttestedPublicKeys retrieves all public keys that have attested.
func (s *Store) AttestedPublicKeys(ctx context.Context) ([][fieldparams.BLSPubkeyLength]byte, error) {
	_, span := trace.StartSpan(ctx, "Validator.AttestedPublicKeys")
	defer span.End()
	var err error
	attestedPublicKeys := make([][fieldparams.BLSPubkeyLength]byte, 0)
	err = s.view(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(pubKeysBucket)
		return bucket.ForEach(func(pubKey []byte, _ []byte) error {
			var pk [fieldparams.BLSPubkeyLength]byte
			copy(pk[:], pubKey)
			attestedPublicKeys = append(attestedPublicKeys, pk)
			return nil
		})
	})
	return attestedPublicKeys, err
}

// SigningRootAtTargetEpoch checks for an existing signing root at a specified
// target epoch for a given validator public key.
func (s *Store) SigningRootAtTargetEpoch(ctx context.Context, pubKey [fieldparams.BLSPubkeyLength]byte, target primitives.Epoch) ([]byte, error) {
	_, span := trace.StartSpan(ctx, "Validator.SigningRootAtTargetEpoch")
	defer span.End()

	var (
		signingRoot32     [32]byte
		signingRootExists bool
	)

	signingRoot := make([]byte, 0, 32)

	err := s.view(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(pubKeysBucket)

		// If no bucket exists for this public key, we return nil.
		pkBucket := bucket.Bucket(pubKey[:])
		if pkBucket == nil {
			return nil
		}

		// If no attestation bucket exists for this public key, we return nil.
		signingRootsBucket := pkBucket.Bucket(attestationSigningRootsBucket)
		if signingRootsBucket == nil {
			return nil
		}

		// If no signing root exists for this target epoch, we return nil.
		sr := signingRootsBucket.Get(bytesutil.EpochToBytesBigEndian(target))
		if len(sr) == 0 {
			return nil
		}

		signingRootExists = true
		copy(signingRoot32[:], sr)

		return nil
	})

	if signingRootExists {
		signingRoot = signingRoot32[:]
	}

	return signingRoot, err
}

// LowestSignedSourceEpoch returns the lowest signed source epoch for a validator public key.
// If no data exists, returning 0 is a sensible default.
func (s *Store) LowestSignedSourceEpoch(ctx context.Context, publicKey [fieldparams.BLSPubkeyLength]byte) (primitives.Epoch, bool, error) {
	_, span := trace.StartSpan(ctx, "Validator.LowestSignedSourceEpoch")
	defer span.End()

	var err error
	var lowestSignedSourceEpoch primitives.Epoch
	var exists bool
	err = s.view(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(lowestSignedSourceBucket)
		lowestSignedSourceBytes := bucket.Get(publicKey[:])
		// 8 because bytesutil.BytesToEpochBigEndian will return 0 if input is less than 8 bytes.
		if len(lowestSignedSourceBytes) < 8 {
			return nil
		}
		exists = true
		lowestSignedSourceEpoch = bytesutil.BytesToEpochBigEndian(lowestSignedSourceBytes)
		return nil
	})
	return lowestSignedSourceEpoch, exists, err
}

// LowestSignedTargetEpoch returns the lowest signed target epoch for a validator public key.
// If no data exists, returning 0 is a sensible default.
func (s *Store) LowestSignedTargetEpoch(ctx context.Context, publicKey [fieldparams.BLSPubkeyLength]byte) (primitives.Epoch, bool, error) {
	_, span := trace.StartSpan(ctx, "Validator.LowestSignedTargetEpoch")
	defer span.End()

	var err error
	var lowestSignedTargetEpoch primitives.Epoch
	var exists bool
	err = s.view(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(lowestSignedTargetBucket)
		lowestSignedTargetBytes := bucket.Get(publicKey[:])
		// 8 because bytesutil.BytesToEpochBigEndian will return 0 if input is less than 8 bytes.
		if len(lowestSignedTargetBytes) < 8 {
			return nil
		}
		exists = true
		lowestSignedTargetEpoch = bytesutil.BytesToEpochBigEndian(lowestSignedTargetBytes)
		return nil
	})
	return lowestSignedTargetEpoch, exists, err
}
