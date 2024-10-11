package validator

import (
	"errors"
	"sync"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
)

var bundleCache = &blobsBundleCache{}

// BlobsBundleCache holds the KZG commitments and other relevant sidecar data for a local beacon block.
type blobsBundleCache struct {
	sync.Mutex
	slot   primitives.Slot
	bundle *enginev1.BlobsBundle
}

// add adds a blobs bundle to the cache.
// same slot overwrites the previous bundle.
func (c *blobsBundleCache) add(slot primitives.Slot, bundle *enginev1.BlobsBundle) {
	c.Lock()
	defer c.Unlock()

	if slot >= c.slot {
		c.bundle = bundle
		c.slot = slot
	}
}

// get gets a blobs bundle from the cache.
func (c *blobsBundleCache) get(slot primitives.Slot) *enginev1.BlobsBundle {
	c.Lock()
	defer c.Unlock()

	if c.slot == slot {
		return c.bundle
	}

	return nil
}

// prune acquires the lock before pruning.
func (c *blobsBundleCache) prune(minSlot primitives.Slot) {
	c.Lock()
	defer c.Unlock()

	if minSlot > c.slot {
		c.slot = 0
		c.bundle = nil
	}
}

// buildBlobSidecars given a block, builds the blob sidecars for the block.
func buildBlobSidecars(blk interfaces.SignedBeaconBlock, blobs [][]byte, kzgProofs [][]byte) ([]*ethpb.BlobSidecar, error) {
	if blk.Version() < version.Deneb {
		return nil, nil // No blobs before deneb.
	}
	denebBlk, err := blk.PbDenebBlock()
	if err != nil {
		return nil, err
	}
	cLen := len(denebBlk.Block.Body.BlobKzgCommitments)
	if cLen != len(blobs) || cLen != len(kzgProofs) {
		return nil, errors.New("blob KZG commitments don't match number of blobs or KZG proofs")
	}
	blobSidecars := make([]*ethpb.BlobSidecar, cLen)
	header, err := blk.Header()
	if err != nil {
		return nil, err
	}
	body := blk.Block().Body()
	for i := range blobSidecars {
		proof, err := blocks.MerkleProofKZGCommitment(body, i)
		if err != nil {
			return nil, err
		}
		blobSidecars[i] = &ethpb.BlobSidecar{
			Index:                    uint64(i),
			Blob:                     blobs[i],
			KzgCommitment:            denebBlk.Block.Body.BlobKzgCommitments[i],
			KzgProof:                 kzgProofs[i],
			SignedBlockHeader:        header,
			CommitmentInclusionProof: proof,
		}
	}
	return blobSidecars, nil
}
