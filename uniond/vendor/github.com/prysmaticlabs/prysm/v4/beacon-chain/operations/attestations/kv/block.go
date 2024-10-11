package kv

import (
	"github.com/pkg/errors"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// SaveBlockAttestation saves an block attestation in cache.
func (c *AttCaches) SaveBlockAttestation(att *ethpb.Attestation) error {
	if att == nil {
		return nil
	}
	r, err := hashFn(att.Data)
	if err != nil {
		return errors.Wrap(err, "could not tree hash attestation")
	}

	c.blockAttLock.Lock()
	defer c.blockAttLock.Unlock()
	atts, ok := c.blockAtt[r]
	if !ok {
		atts = make([]*ethpb.Attestation, 0, 1)
	}

	// Ensure that this attestation is not already fully contained in an existing attestation.
	for _, a := range atts {
		if c, err := a.AggregationBits.Contains(att.AggregationBits); err != nil {
			return err
		} else if c {
			return nil
		}
	}

	c.blockAtt[r] = append(atts, ethpb.CopyAttestation(att))

	return nil
}

// BlockAttestations returns the block attestations in cache.
func (c *AttCaches) BlockAttestations() []*ethpb.Attestation {
	atts := make([]*ethpb.Attestation, 0)

	c.blockAttLock.RLock()
	defer c.blockAttLock.RUnlock()
	for _, att := range c.blockAtt {
		atts = append(atts, att...)
	}

	return atts
}

// DeleteBlockAttestation deletes a block attestation in cache.
func (c *AttCaches) DeleteBlockAttestation(att *ethpb.Attestation) error {
	if att == nil {
		return nil
	}
	r, err := hashFn(att.Data)
	if err != nil {
		return errors.Wrap(err, "could not tree hash attestation")
	}

	c.blockAttLock.Lock()
	defer c.blockAttLock.Unlock()
	delete(c.blockAtt, r)

	return nil
}
