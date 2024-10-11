package sync

import (
	"context"
	"time"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	"go.opencensus.io/trace"
)

const signatureVerificationInterval = 50 * time.Millisecond

const verifierLimit = 50

type signatureVerifier struct {
	set     *bls.SignatureBatch
	resChan chan error
}

// A routine that runs in the background to perform batch
// verifications of incoming messages from gossip.
func (s *Service) verifierRoutine() {
	verifierBatch := make([]*signatureVerifier, 0)
	ticker := time.NewTicker(signatureVerificationInterval)
	for {
		select {
		case <-s.ctx.Done():
			// Clean up currently utilised resources.
			ticker.Stop()
			for i := 0; i < len(verifierBatch); i++ {
				verifierBatch[i].resChan <- s.ctx.Err()
			}
			return
		case sig := <-s.signatureChan:
			verifierBatch = append(verifierBatch, sig)
			if len(verifierBatch) >= verifierLimit {
				verifyBatch(verifierBatch)
				verifierBatch = []*signatureVerifier{}
			}
		case <-ticker.C:
			if len(verifierBatch) > 0 {
				verifyBatch(verifierBatch)
				verifierBatch = []*signatureVerifier{}
			}
		}
	}
}

func (s *Service) validateWithBatchVerifier(ctx context.Context, message string, set *bls.SignatureBatch) (pubsub.ValidationResult, error) {
	_, span := trace.StartSpan(ctx, "sync.validateWithBatchVerifier")
	defer span.End()

	resChan := make(chan error)
	verificationSet := &signatureVerifier{set: set.Copy(), resChan: resChan}
	s.signatureChan <- verificationSet

	resErr := <-resChan
	close(resChan)
	// If verification fails we fallback to individual verification
	// of each signature set.
	if resErr != nil {
		log.WithError(resErr).Tracef("Could not perform batch verification of %s", message)
		verified, err := set.Verify()
		if err != nil {
			verErr := errors.Wrapf(err, "Could not verify %s", message)
			tracing.AnnotateError(span, verErr)
			return pubsub.ValidationReject, verErr
		}
		if !verified {
			verErr := errors.Errorf("Verification of %s failed", message)
			tracing.AnnotateError(span, verErr)
			return pubsub.ValidationReject, verErr
		}
	}
	return pubsub.ValidationAccept, nil
}

func verifyBatch(verifierBatch []*signatureVerifier) {
	if len(verifierBatch) == 0 {
		return
	}
	aggSet := verifierBatch[0].set

	for i := 1; i < len(verifierBatch); i++ {
		aggSet = aggSet.Join(verifierBatch[i].set)
	}
	var verificationErr error

	aggSet, verificationErr = performBatchAggregation(aggSet)
	if verificationErr == nil {
		verified, err := aggSet.Verify()
		switch {
		case err != nil:
			verificationErr = err
		case !verified:
			verificationErr = errors.New("batch signature verification failed")
		}
	}
	for i := 0; i < len(verifierBatch); i++ {
		verifierBatch[i].resChan <- verificationErr
	}
}

func performBatchAggregation(aggSet *bls.SignatureBatch) (*bls.SignatureBatch, error) {
	currLen := len(aggSet.Signatures)
	num, aggSet, err := aggSet.RemoveDuplicates()
	if err != nil {
		return nil, err
	}
	duplicatesRemovedCounter.Add(float64(num))
	// Aggregate batches in the provided signature batch.
	aggSet, err = aggSet.AggregateBatch()
	if err != nil {
		return nil, err
	}
	// Record number of signature sets successfully batched.
	if currLen > len(aggSet.Signatures) {
		numberOfSetsAggregated.Observe(float64(currLen - len(aggSet.Signatures)))
	}
	return aggSet, nil
}
