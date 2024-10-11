package verification

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/interop"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func testSignedBlockBlobKeys(t *testing.T, valRoot []byte, slot primitives.Slot, nblobs int) (blocks.ROBlock, []blocks.ROBlob, bls.SecretKey, bls.PublicKey) {
	sks, pks, err := interop.DeterministicallyGenerateKeys(0, 1)
	require.NoError(t, err)
	block, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, slot, nblobs, util.WithProposerSigning(0, sks[0], valRoot))
	return block, blobs, sks[0], pks[0]
}

func TestVerifySignature(t *testing.T) {
	valRoot := [32]byte{}
	_, blobs, _, pk := testSignedBlockBlobKeys(t, valRoot[:], 0, 1)
	b := blobs[0]

	sc := newSigCache(valRoot[:], 1)
	cb := func(idx primitives.ValidatorIndex) (*eth.Validator, error) {
		return &eth.Validator{PublicKey: pk.Marshal()}, nil
	}
	mv := &mockValidatorAtIndexer{cb: cb}

	sd := blobToSignatureData(b)
	require.NoError(t, sc.VerifySignature(sd, mv))
}

func TestSignatureCacheMissThenHit(t *testing.T) {
	valRoot := [32]byte{}
	_, blobs, _, pk := testSignedBlockBlobKeys(t, valRoot[:], 0, 1)
	b := blobs[0]

	sc := newSigCache(valRoot[:], 1)
	cb := func(idx primitives.ValidatorIndex) (*eth.Validator, error) {
		return &eth.Validator{PublicKey: pk.Marshal()}, nil
	}

	sd := blobToSignatureData(b)
	cached, err := sc.SignatureVerified(sd)
	// Should not be cached yet.
	require.Equal(t, false, cached)
	require.NoError(t, err)

	mv := &mockValidatorAtIndexer{cb: cb}
	require.NoError(t, sc.VerifySignature(sd, mv))

	// Now it should be cached.
	cached, err = sc.SignatureVerified(sd)
	require.Equal(t, true, cached)
	require.NoError(t, err)

	// note the changed slot, which will give this blob a different cache key
	_, blobs, _, _ = testSignedBlockBlobKeys(t, valRoot[:], 1, 1)
	badSd := blobToSignatureData(blobs[0])

	// new value, should not be cached
	cached, err = sc.SignatureVerified(badSd)
	require.Equal(t, false, cached)
	require.NoError(t, err)

	// note that the first argument is incremented, so it will be a different deterministic key
	_, pks, err := interop.DeterministicallyGenerateKeys(1, 1)
	require.NoError(t, err)
	wrongKey := pks[0]
	cb = func(idx primitives.ValidatorIndex) (*eth.Validator, error) {
		return &eth.Validator{PublicKey: wrongKey.Marshal()}, nil
	}
	mv = &mockValidatorAtIndexer{cb: cb}
	require.ErrorIs(t, sc.VerifySignature(badSd, mv), signing.ErrSigFailedToVerify)

	// we should now get the failure error from the cache
	cached, err = sc.SignatureVerified(badSd)
	require.Equal(t, true, cached)
	require.ErrorIs(t, err, signing.ErrSigFailedToVerify)
}

type mockValidatorAtIndexer struct {
	cb func(idx primitives.ValidatorIndex) (*eth.Validator, error)
}

// ValidatorAtIndex implements ValidatorAtIndexer.
func (m *mockValidatorAtIndexer) ValidatorAtIndex(idx primitives.ValidatorIndex) (*eth.Validator, error) {
	return m.cb(idx)
}

var _ ValidatorAtIndexer = &mockValidatorAtIndexer{}

func TestProposerCache(t *testing.T) {
	ctx := context.Background()
	// 3 validators because that was the first number that produced a non-zero proposer index by default
	st, _ := util.DeterministicGenesisStateDeneb(t, 3)

	pc := newPropCache()
	_, cached := pc.Proposer(&forkchoicetypes.Checkpoint{}, 1)
	// should not be cached yet
	require.Equal(t, false, cached)

	// If this test breaks due to changes in the deterministic state gen, just replace '2' with whatever the right index is.
	expectedIdx := 2
	idx, err := pc.ComputeProposer(ctx, [32]byte{}, 1, st)
	require.NoError(t, err)
	require.Equal(t, primitives.ValidatorIndex(expectedIdx), idx)

	idx, cached = pc.Proposer(&forkchoicetypes.Checkpoint{}, 1)
	// TODO: update this test when we integrate a proposer id cache
	require.Equal(t, false, cached)
	require.Equal(t, primitives.ValidatorIndex(0), idx)
}
