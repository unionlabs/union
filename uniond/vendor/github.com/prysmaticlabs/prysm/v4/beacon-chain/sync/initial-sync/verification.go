package initialsync

import (
	"context"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/kzg"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/das"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/verification"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
)

var (
	// ErrBatchSignatureMismatch is returned by VerifiedROBlobs when any of the blobs in the batch have a signature
	// which does not match the signature for the block with a corresponding root.
	ErrBatchSignatureMismatch = errors.New("Sidecar block header signature does not match signed block")
	// ErrBlockRootMismatch is returned by VerifiedROBlobs in the scenario where the root of the given signed block
	// does not match the block header in one of the corresponding sidecars.
	ErrBatchBlockRootMismatch = errors.New("Sidecar block header root does not match signed block")
)

func newBlobVerifierFromInitializer(ini *verification.Initializer) verification.NewBlobVerifier {
	return func(b blocks.ROBlob, reqs []verification.Requirement) verification.BlobVerifier {
		return ini.NewBlobVerifier(b, reqs)
	}
}

func newBlobBatchVerifier(newVerifier verification.NewBlobVerifier) *BlobBatchVerifier {
	return &BlobBatchVerifier{
		verifyKzg:   kzg.Verify,
		newVerifier: newVerifier,
	}
}

type kzgVerifier func(b ...blocks.ROBlob) error

// BlobBatchVerifier solves problems that come from verifying batches of blobs from RPC.
// First: we only update forkchoice after the entire batch has completed, so the n+1 elements in the batch
// won't be in forkchoice yet.
// Second: it is more efficient to batch some verifications, like kzg commitment verification. Batch adds a
// method to BlobVerifier to verify the kzg commitments of all blob sidecars for a block together, then using the cached
// result of the batch verification when verifying the individual blobs.
type BlobBatchVerifier struct {
	verifyKzg   kzgVerifier
	newVerifier verification.NewBlobVerifier
}

var _ das.BlobBatchVerifier = &BlobBatchVerifier{}

func (batch *BlobBatchVerifier) VerifiedROBlobs(ctx context.Context, blk blocks.ROBlock, scs []blocks.ROBlob) ([]blocks.VerifiedROBlob, error) {
	if len(scs) == 0 {
		return nil, nil
	}
	// We assume the proposer was validated wrt the block in batch block processing before performing the DA check.

	// So at this stage we just need to make sure the value being signed and signature bytes match the block.
	for i := range scs {
		if blk.Signature() != bytesutil.ToBytes96(scs[i].SignedBlockHeader.Signature) {
			return nil, ErrBatchSignatureMismatch
		}
		// Extra defensive check to make sure the roots match. This should be unnecessary in practice since the root from
		// the block should be used as the lookup key into the cache of sidecars.
		if blk.Root() != scs[i].BlockRoot() {
			return nil, ErrBatchBlockRootMismatch
		}
	}
	// Verify commitments for all blobs at once. verifyOneBlob assumes it is only called once this check succeeds.
	if err := batch.verifyKzg(scs...); err != nil {
		return nil, err
	}
	vs := make([]blocks.VerifiedROBlob, len(scs))
	for i := range scs {
		vb, err := batch.verifyOneBlob(ctx, scs[i])
		if err != nil {
			return nil, err
		}
		vs[i] = vb
	}
	return vs, nil
}

func (batch *BlobBatchVerifier) verifyOneBlob(ctx context.Context, sc blocks.ROBlob) (blocks.VerifiedROBlob, error) {
	vb := blocks.VerifiedROBlob{}
	bv := batch.newVerifier(sc, verification.InitsyncSidecarRequirements)
	// We can satisfy the following 2 requirements immediately because VerifiedROBlobs always verifies commitments
	// and block signature for all blobs in the batch before calling verifyOneBlob.
	bv.SatisfyRequirement(verification.RequireSidecarKzgProofVerified)
	bv.SatisfyRequirement(verification.RequireValidProposerSignature)

	if err := bv.BlobIndexInBounds(); err != nil {
		return vb, err
	}
	if err := bv.SidecarInclusionProven(); err != nil {
		return vb, err
	}

	return bv.VerifiedROBlob()
}
