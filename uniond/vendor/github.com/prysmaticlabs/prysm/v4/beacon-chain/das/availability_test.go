package das

import (
	"bytes"
	"context"
	"testing"

	errors "github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filesystem"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/verification"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func Test_commitmentsToCheck(t *testing.T) {
	windowSlots, err := slots.EpochEnd(params.BeaconConfig().MinEpochsForBlobsSidecarsRequest)
	require.NoError(t, err)
	commits := [][]byte{
		bytesutil.PadTo([]byte("a"), 48),
		bytesutil.PadTo([]byte("b"), 48),
		bytesutil.PadTo([]byte("c"), 48),
		bytesutil.PadTo([]byte("d"), 48),
	}
	cases := []struct {
		name    string
		commits [][]byte
		block   func(*testing.T) blocks.ROBlock
		slot    primitives.Slot
		err     error
	}{
		{
			name: "pre deneb",
			block: func(t *testing.T) blocks.ROBlock {
				bb := util.NewBeaconBlockBellatrix()
				sb, err := blocks.NewSignedBeaconBlock(bb)
				require.NoError(t, err)
				rb, err := blocks.NewROBlock(sb)
				require.NoError(t, err)
				return rb
			},
		},
		{
			name: "commitments within da",
			block: func(t *testing.T) blocks.ROBlock {
				d := util.NewBeaconBlockDeneb()
				d.Block.Body.BlobKzgCommitments = commits
				d.Block.Slot = 100
				sb, err := blocks.NewSignedBeaconBlock(d)
				require.NoError(t, err)
				rb, err := blocks.NewROBlock(sb)
				require.NoError(t, err)
				return rb
			},
			commits: commits,
			slot:    100,
		},
		{
			name: "commitments outside da",
			block: func(t *testing.T) blocks.ROBlock {
				d := util.NewBeaconBlockDeneb()
				// block is from slot 0, "current slot" is window size +1 (so outside the window)
				d.Block.Body.BlobKzgCommitments = commits
				sb, err := blocks.NewSignedBeaconBlock(d)
				require.NoError(t, err)
				rb, err := blocks.NewROBlock(sb)
				require.NoError(t, err)
				return rb
			},
			slot: windowSlots + 1,
		},
		{
			name: "excessive commitments",
			block: func(t *testing.T) blocks.ROBlock {
				d := util.NewBeaconBlockDeneb()
				d.Block.Slot = 100
				// block is from slot 0, "current slot" is window size +1 (so outside the window)
				d.Block.Body.BlobKzgCommitments = commits
				// Double the number of commitments, assert that this is over the limit
				d.Block.Body.BlobKzgCommitments = append(commits, d.Block.Body.BlobKzgCommitments...)
				sb, err := blocks.NewSignedBeaconBlock(d)
				require.NoError(t, err)
				rb, err := blocks.NewROBlock(sb)
				require.NoError(t, err)
				c, err := rb.Block().Body().BlobKzgCommitments()
				require.NoError(t, err)
				require.Equal(t, true, len(c) > fieldparams.MaxBlobsPerBlock)
				return rb
			},
			slot: windowSlots + 1,
			err:  errIndexOutOfBounds,
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			b := c.block(t)
			co, err := commitmentsToCheck(b, c.slot)
			if c.err != nil {
				require.ErrorIs(t, err, c.err)
			} else {
				require.NoError(t, err)
			}
			require.Equal(t, len(c.commits), co.count())
			for i := 0; i < len(c.commits); i++ {
				require.Equal(t, true, bytes.Equal(c.commits[i], co[i]))
			}
		})
	}
}

func daAlwaysSucceeds(_ [][]byte, _ []*ethpb.BlobSidecar) error {
	return nil
}

type mockDA struct {
	t   *testing.T
	scs []blocks.ROBlob
	err error
}

func TestLazilyPersistent_Missing(t *testing.T) {
	ctx := context.Background()
	store := filesystem.NewEphemeralBlobStorage(t)

	blk, scs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 1, 3)

	mbv := &mockBlobBatchVerifier{t: t, scs: scs}
	as := NewLazilyPersistentStore(store, mbv)

	// Only one commitment persisted, should return error with other indices
	require.NoError(t, as.Persist(1, scs[2]))
	err := as.IsDataAvailable(ctx, 1, blk)
	require.ErrorIs(t, err, errMissingSidecar)

	// All but one persisted, return missing idx
	require.NoError(t, as.Persist(1, scs[0]))
	err = as.IsDataAvailable(ctx, 1, blk)
	require.ErrorIs(t, err, errMissingSidecar)

	// All persisted, return nil
	require.NoError(t, as.Persist(1, scs...))

	require.NoError(t, as.IsDataAvailable(ctx, 1, blk))
}

func TestLazilyPersistent_Mismatch(t *testing.T) {
	ctx := context.Background()
	store := filesystem.NewEphemeralBlobStorage(t)

	blk, scs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 1, 3)

	mbv := &mockBlobBatchVerifier{t: t, err: errors.New("kzg check should not run")}
	scs[0].KzgCommitment = bytesutil.PadTo([]byte("nope"), 48)
	as := NewLazilyPersistentStore(store, mbv)

	// Only one commitment persisted, should return error with other indices
	require.NoError(t, as.Persist(1, scs[0]))
	err := as.IsDataAvailable(ctx, 1, blk)
	require.NotNil(t, err)
	require.ErrorIs(t, err, errCommitmentMismatch)
}

func TestLazyPersistOnceCommitted(t *testing.T) {
	_, scs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 1, 6)
	as := NewLazilyPersistentStore(filesystem.NewEphemeralBlobStorage(t), &mockBlobBatchVerifier{})
	// stashes as expected
	require.NoError(t, as.Persist(1, scs...))
	// ignores duplicates
	require.ErrorIs(t, as.Persist(1, scs...), ErrDuplicateSidecar)

	// ignores index out of bound
	scs[0].Index = 6
	require.ErrorIs(t, as.Persist(1, scs[0]), errIndexOutOfBounds)

	_, more := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 1, 4)
	// ignores sidecars before the retention period
	slotOOB, err := slots.EpochStart(params.BeaconConfig().MinEpochsForBlobsSidecarsRequest)
	require.NoError(t, err)
	require.NoError(t, as.Persist(32+slotOOB, more[0]))

	// doesn't ignore new sidecars with a different block root
	require.NoError(t, as.Persist(1, more...))
}

type mockBlobBatchVerifier struct {
	t        *testing.T
	scs      []blocks.ROBlob
	err      error
	verified map[[32]byte]primitives.Slot
}

var _ BlobBatchVerifier = &mockBlobBatchVerifier{}

func (m *mockBlobBatchVerifier) VerifiedROBlobs(_ context.Context, _ blocks.ROBlock, scs []blocks.ROBlob) ([]blocks.VerifiedROBlob, error) {
	require.Equal(m.t, len(scs), len(m.scs))
	for i := range m.scs {
		require.Equal(m.t, m.scs[i], scs[i])
	}
	vscs := verification.FakeVerifySliceForTest(m.t, scs)
	return vscs, m.err
}

func (m *mockBlobBatchVerifier) MarkVerified(root [32]byte, slot primitives.Slot) {
	if m.verified == nil {
		m.verified = make(map[[32]byte]primitives.Slot)
	}
	m.verified[root] = slot
}
