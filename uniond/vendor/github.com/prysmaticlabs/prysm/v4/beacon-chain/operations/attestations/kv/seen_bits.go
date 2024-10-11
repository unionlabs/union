package kv

import (
	"github.com/patrickmn/go-cache"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/go-bitfield"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

func (c *AttCaches) insertSeenBit(att *ethpb.Attestation) error {
	r, err := hashFn(att.Data)
	if err != nil {
		return err
	}

	v, ok := c.seenAtt.Get(string(r[:]))
	if ok {
		seenBits, ok := v.([]bitfield.Bitlist)
		if !ok {
			return errors.New("could not convert to bitlist type")
		}
		alreadyExists := false
		for _, bit := range seenBits {
			if c, err := bit.Contains(att.AggregationBits); err != nil {
				return err
			} else if c {
				alreadyExists = true
				break
			}
		}
		if !alreadyExists {
			seenBits = append(seenBits, att.AggregationBits)
		}
		c.seenAtt.Set(string(r[:]), seenBits, cache.DefaultExpiration /* one epoch */)
		return nil
	}

	c.seenAtt.Set(string(r[:]), []bitfield.Bitlist{att.AggregationBits}, cache.DefaultExpiration /* one epoch */)
	return nil
}

func (c *AttCaches) hasSeenBit(att *ethpb.Attestation) (bool, error) {
	r, err := hashFn(att.Data)
	if err != nil {
		return false, err
	}

	v, ok := c.seenAtt.Get(string(r[:]))
	if ok {
		seenBits, ok := v.([]bitfield.Bitlist)
		if !ok {
			return false, errors.New("could not convert to bitlist type")
		}
		for _, bit := range seenBits {
			if c, err := bit.Contains(att.AggregationBits); err != nil {
				return false, err
			} else if c {
				return true, nil
			}
		}
	}
	return false, nil
}
