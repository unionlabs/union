package verification

import (
	"context"
	"fmt"

	lru "github.com/hashicorp/golang-lru"
	lruwrpr "github.com/prysmaticlabs/prysm/v4/cache/lru"
	log "github.com/sirupsen/logrus"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

const (
	DefaultSignatureCacheSize = 256
)

// ValidatorAtIndexer defines the method needed to retrieve a validator by its index.
// This interface is satisfied by state.BeaconState, but can also be satisfied by a cache.
type ValidatorAtIndexer interface {
	ValidatorAtIndex(idx primitives.ValidatorIndex) (*ethpb.Validator, error)
}

// SignatureCache represents a type that can perform signature verification and cache the result so that it
// can be used when the same signature is seen in multiple places, like a SignedBeaconBlockHeader
// found in multiple BlobSidecars.
type SignatureCache interface {
	// VerifySignature perform signature verification and caches the result.
	VerifySignature(sig SignatureData, v ValidatorAtIndexer) (err error)
	// SignatureVerified accesses the result of a previous signature verification.
	SignatureVerified(sig SignatureData) (bool, error)
}

// SignatureData represents the set of parameters that together uniquely identify a signature observed on
// a beacon block. This is used as the key for the signature cache.
type SignatureData struct {
	Root      [32]byte
	Parent    [32]byte
	Signature [96]byte
	Proposer  primitives.ValidatorIndex
	Slot      primitives.Slot
}

func (d SignatureData) logFields() log.Fields {
	return log.Fields{
		"root":        fmt.Sprintf("%#x", d.Root),
		"parent_root": fmt.Sprintf("%#x", d.Parent),
		"signature":   fmt.Sprintf("%#x", d.Signature),
		"proposer":    d.Proposer,
		"slot":        d.Slot,
	}
}

func newSigCache(vr []byte, size int) *sigCache {
	return &sigCache{Cache: lruwrpr.New(size), valRoot: vr}
}

type sigCache struct {
	*lru.Cache
	valRoot []byte
}

// VerifySignature verifies the given signature data against the key obtained via ValidatorAtIndexer.
func (c *sigCache) VerifySignature(sig SignatureData, v ValidatorAtIndexer) (err error) {
	defer func() {
		if err == nil {
			c.Add(sig, true)
		} else {
			log.WithError(err).WithFields(sig.logFields()).Debug("caching failed signature verification result")
			c.Add(sig, false)
		}
	}()
	e := slots.ToEpoch(sig.Slot)
	fork, err := forks.Fork(e)
	if err != nil {
		return err
	}
	domain, err := signing.Domain(fork, e, params.BeaconConfig().DomainBeaconProposer, c.valRoot)
	if err != nil {
		return err
	}
	pv, err := v.ValidatorAtIndex(sig.Proposer)
	if err != nil {
		return err
	}
	pb, err := bls.PublicKeyFromBytes(pv.PublicKey)
	if err != nil {
		return err
	}
	s, err := bls.SignatureFromBytes(sig.Signature[:])
	if err != nil {
		return err
	}
	sr, err := signing.ComputeSigningRootForRoot(sig.Root, domain)
	if err != nil {
		return err
	}
	if !s.Verify(pb, sr[:]) {
		return signing.ErrSigFailedToVerify
	}

	return nil
}

// SignatureVerified checks the signature cache for the given key, and returns a boolean value of true
// if it has been seen before, and an error value indicating whether the signature verification succeeded.
// ie only a result of (true, nil) means a previous signature check passed.
func (c *sigCache) SignatureVerified(sig SignatureData) (bool, error) {
	val, seen := c.Get(sig)
	if !seen {
		return false, nil
	}
	verified, ok := val.(bool)
	if !ok {
		log.WithFields(sig.logFields()).Debug("ignoring invalid value found in signature cache")
		// This shouldn't happen, and if it does, the caller should treat it as a cache miss and run verification
		// again to correctly populate the cache key.
		return false, nil
	}
	if verified {
		return true, nil
	}
	return true, signing.ErrSigFailedToVerify
}

// ProposerCache represents a type that can compute the proposer for a given slot + parent root,
// and cache the result so that it can be reused when the same verification needs to be performed
// across multiple values.
type ProposerCache interface {
	ComputeProposer(ctx context.Context, root [32]byte, slot primitives.Slot, pst state.BeaconState) (primitives.ValidatorIndex, error)
	Proposer(c *forkchoicetypes.Checkpoint, slot primitives.Slot) (primitives.ValidatorIndex, bool)
}

func newPropCache() *propCache {
	return &propCache{}
}

type propCache struct {
}

// ComputeProposer takes the state for the given parent root and slot and computes the proposer index, updating the
// proposer index cache when successful.
func (*propCache) ComputeProposer(ctx context.Context, parent [32]byte, slot primitives.Slot, pst state.BeaconState) (primitives.ValidatorIndex, error) {
	pst, err := transition.ProcessSlotsUsingNextSlotCache(ctx, pst, parent[:], slot)
	if err != nil {
		return 0, err
	}
	idx, err := helpers.BeaconProposerIndex(ctx, pst)
	if err != nil {
		return 0, err
	}
	return idx, nil
}

// Proposer returns the validator index if it is found in the cache, along with a boolean indicating
// whether the value was present, similar to accessing an lru or go map.
func (*propCache) Proposer(c *forkchoicetypes.Checkpoint, slot primitives.Slot) (primitives.ValidatorIndex, bool) {
	id, err := helpers.ProposerIndexAtSlotFromCheckpoint(c, slot)
	if err != nil {
		return 0, false
	}
	return id, true
}
