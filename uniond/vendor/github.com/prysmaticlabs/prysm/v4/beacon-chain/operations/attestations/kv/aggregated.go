package kv

import (
	"context"
	"runtime"
	"sync"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	attaggregation "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation/aggregation/attestations"
	log "github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
)

// AggregateUnaggregatedAttestations aggregates the unaggregated attestations and saves the
// newly aggregated attestations in the pool.
// It tracks the unaggregated attestations that weren't able to aggregate to prevent
// the deletion of unaggregated attestations in the pool.
func (c *AttCaches) AggregateUnaggregatedAttestations(ctx context.Context) error {
	ctx, span := trace.StartSpan(ctx, "operations.attestations.kv.AggregateUnaggregatedAttestations")
	defer span.End()
	unaggregatedAtts, err := c.UnaggregatedAttestations()
	if err != nil {
		return err
	}
	return c.aggregateUnaggregatedAtts(ctx, unaggregatedAtts)
}

func (c *AttCaches) aggregateUnaggregatedAtts(ctx context.Context, unaggregatedAtts []*ethpb.Attestation) error {
	_, span := trace.StartSpan(ctx, "operations.attestations.kv.aggregateUnaggregatedAtts")
	defer span.End()

	attsByDataRoot := make(map[[32]byte][]*ethpb.Attestation, len(unaggregatedAtts))
	for _, att := range unaggregatedAtts {
		attDataRoot, err := att.Data.HashTreeRoot()
		if err != nil {
			return err
		}
		attsByDataRoot[attDataRoot] = append(attsByDataRoot[attDataRoot], att)
	}

	// Aggregate unaggregated attestations from the pool and save them in the pool.
	// Track the unaggregated attestations that aren't able to aggregate.
	leftOverUnaggregatedAtt := make(map[[32]byte]bool)

	leftOverUnaggregatedAtt = c.aggregateParallel(attsByDataRoot, leftOverUnaggregatedAtt)

	// Remove the unaggregated attestations from the pool that were successfully aggregated.
	for _, att := range unaggregatedAtts {
		h, err := hashFn(att)
		if err != nil {
			return err
		}
		if leftOverUnaggregatedAtt[h] {
			continue
		}
		if err := c.DeleteUnaggregatedAttestation(att); err != nil {
			return err
		}
	}
	return nil
}

// aggregateParallel aggregates attestations in parallel for `atts` and saves them in the pool,
// returns the unaggregated attestations that weren't able to aggregate.
// Given `n` CPU cores, it creates a channel of size `n` and spawns `n` goroutines to aggregate attestations
func (c *AttCaches) aggregateParallel(atts map[[32]byte][]*ethpb.Attestation, leftOver map[[32]byte]bool) map[[32]byte]bool {
	var leftoverLock sync.Mutex
	wg := sync.WaitGroup{}

	n := runtime.GOMAXPROCS(0) // defaults to the value of runtime.NumCPU
	ch := make(chan []*ethpb.Attestation, n)
	wg.Add(n)
	for i := 0; i < n; i++ {
		go func() {
			defer wg.Done()
			for as := range ch {
				aggregated, err := attaggregation.AggregateDisjointOneBitAtts(as)
				if err != nil {
					log.WithError(err).Error("could not aggregate unaggregated attestations")
					continue
				}
				if aggregated == nil {
					log.Error("nil aggregated attestation")
					continue
				}
				if helpers.IsAggregated(aggregated) {
					if err := c.SaveAggregatedAttestations([]*ethpb.Attestation{aggregated}); err != nil {
						log.WithError(err).Error("could not save aggregated attestation")
						continue
					}
				} else {
					h, err := hashFn(aggregated)
					if err != nil {
						log.WithError(err).Error("could not hash attestation")
						continue
					}
					leftoverLock.Lock()
					leftOver[h] = true
					leftoverLock.Unlock()
				}
			}
		}()
	}

	for _, as := range atts {
		ch <- as
	}

	close(ch)
	wg.Wait()

	return leftOver
}

// SaveAggregatedAttestation saves an aggregated attestation in cache.
func (c *AttCaches) SaveAggregatedAttestation(att *ethpb.Attestation) error {
	if err := helpers.ValidateNilAttestation(att); err != nil {
		return err
	}
	if !helpers.IsAggregated(att) {
		return errors.New("attestation is not aggregated")
	}
	has, err := c.HasAggregatedAttestation(att)
	if err != nil {
		return err
	}
	if has {
		return nil
	}

	seen, err := c.hasSeenBit(att)
	if err != nil {
		return err
	}
	if seen {
		return nil
	}

	r, err := hashFn(att.Data)
	if err != nil {
		return errors.Wrap(err, "could not tree hash attestation")
	}
	copiedAtt := ethpb.CopyAttestation(att)
	c.aggregatedAttLock.Lock()
	defer c.aggregatedAttLock.Unlock()
	atts, ok := c.aggregatedAtt[r]
	if !ok {
		atts := []*ethpb.Attestation{copiedAtt}
		c.aggregatedAtt[r] = atts
		return nil
	}

	atts, err = attaggregation.Aggregate(append(atts, copiedAtt))
	if err != nil {
		return err
	}
	c.aggregatedAtt[r] = atts

	return nil
}

// SaveAggregatedAttestations saves a list of aggregated attestations in cache.
func (c *AttCaches) SaveAggregatedAttestations(atts []*ethpb.Attestation) error {
	for _, att := range atts {
		if err := c.SaveAggregatedAttestation(att); err != nil {
			log.WithError(err).Debug("Could not save aggregated attestation")
			if err := c.DeleteAggregatedAttestation(att); err != nil {
				log.WithError(err).Debug("Could not delete aggregated attestation")
			}
		}
	}
	return nil
}

// AggregatedAttestations returns the aggregated attestations in cache.
func (c *AttCaches) AggregatedAttestations() []*ethpb.Attestation {
	c.aggregatedAttLock.RLock()
	defer c.aggregatedAttLock.RUnlock()

	atts := make([]*ethpb.Attestation, 0)

	for _, a := range c.aggregatedAtt {
		atts = append(atts, a...)
	}

	return atts
}

// AggregatedAttestationsBySlotIndex returns the aggregated attestations in cache,
// filtered by committee index and slot.
func (c *AttCaches) AggregatedAttestationsBySlotIndex(ctx context.Context, slot primitives.Slot, committeeIndex primitives.CommitteeIndex) []*ethpb.Attestation {
	_, span := trace.StartSpan(ctx, "operations.attestations.kv.AggregatedAttestationsBySlotIndex")
	defer span.End()

	atts := make([]*ethpb.Attestation, 0)

	c.aggregatedAttLock.RLock()
	defer c.aggregatedAttLock.RUnlock()
	for _, a := range c.aggregatedAtt {
		if slot == a[0].Data.Slot && committeeIndex == a[0].Data.CommitteeIndex {
			atts = append(atts, a...)
		}
	}

	return atts
}

// DeleteAggregatedAttestation deletes the aggregated attestations in cache.
func (c *AttCaches) DeleteAggregatedAttestation(att *ethpb.Attestation) error {
	if err := helpers.ValidateNilAttestation(att); err != nil {
		return err
	}
	if !helpers.IsAggregated(att) {
		return errors.New("attestation is not aggregated")
	}
	r, err := hashFn(att.Data)
	if err != nil {
		return errors.Wrap(err, "could not tree hash attestation data")
	}

	if err := c.insertSeenBit(att); err != nil {
		return err
	}

	c.aggregatedAttLock.Lock()
	defer c.aggregatedAttLock.Unlock()
	attList, ok := c.aggregatedAtt[r]
	if !ok {
		return nil
	}

	filtered := make([]*ethpb.Attestation, 0)
	for _, a := range attList {
		if c, err := att.AggregationBits.Contains(a.AggregationBits); err != nil {
			return err
		} else if !c {
			filtered = append(filtered, a)
		}
	}
	if len(filtered) == 0 {
		delete(c.aggregatedAtt, r)
	} else {
		c.aggregatedAtt[r] = filtered
	}

	return nil
}

// HasAggregatedAttestation checks if the input attestations has already existed in cache.
func (c *AttCaches) HasAggregatedAttestation(att *ethpb.Attestation) (bool, error) {
	if err := helpers.ValidateNilAttestation(att); err != nil {
		return false, err
	}
	r, err := hashFn(att.Data)
	if err != nil {
		return false, errors.Wrap(err, "could not tree hash attestation")
	}

	c.aggregatedAttLock.RLock()
	defer c.aggregatedAttLock.RUnlock()
	if atts, ok := c.aggregatedAtt[r]; ok {
		for _, a := range atts {
			if c, err := a.AggregationBits.Contains(att.AggregationBits); err != nil {
				return false, err
			} else if c {
				return true, nil
			}
		}
	}

	c.blockAttLock.RLock()
	defer c.blockAttLock.RUnlock()
	if atts, ok := c.blockAtt[r]; ok {
		for _, a := range atts {
			if c, err := a.AggregationBits.Contains(att.AggregationBits); err != nil {
				return false, err
			} else if c {
				return true, nil
			}
		}
	}

	return false, nil
}

// AggregatedAttestationCount returns the number of aggregated attestations key in the pool.
func (c *AttCaches) AggregatedAttestationCount() int {
	c.aggregatedAttLock.RLock()
	defer c.aggregatedAttLock.RUnlock()
	return len(c.aggregatedAtt)
}
