package lookup

import (
	"context"
	"fmt"
	"net/http"
	"reflect"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	mockChain "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filesystem"
	testDB "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/testutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/verification"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpbalpha "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestGetBlock(t *testing.T) {
	beaconDB := testDB.SetupDB(t)
	ctx := context.Background()

	genBlk, blkContainers := testutil.FillDBWithBlocks(ctx, t, beaconDB)
	canonicalRoots := make(map[[32]byte]bool)

	for _, bContr := range blkContainers {
		canonicalRoots[bytesutil.ToBytes32(bContr.BlockRoot)] = true
	}
	headBlock := blkContainers[len(blkContainers)-1]
	nextSlot := headBlock.GetPhase0Block().Block.Slot + 1

	b2 := util.NewBeaconBlock()
	b2.Block.Slot = 30
	b2.Block.ParentRoot = bytesutil.PadTo([]byte{1}, 32)
	util.SaveBlock(t, ctx, beaconDB, b2)
	b3 := util.NewBeaconBlock()
	b3.Block.Slot = 30
	b3.Block.ParentRoot = bytesutil.PadTo([]byte{4}, 32)
	util.SaveBlock(t, ctx, beaconDB, b3)
	b4 := util.NewBeaconBlock()
	b4.Block.Slot = nextSlot
	b4.Block.ParentRoot = bytesutil.PadTo([]byte{8}, 32)
	util.SaveBlock(t, ctx, beaconDB, b4)

	wsb, err := blocks.NewSignedBeaconBlock(headBlock.Block.(*ethpbalpha.BeaconBlockContainer_Phase0Block).Phase0Block)
	require.NoError(t, err)

	fetcher := &BeaconDbBlocker{
		BeaconDB: beaconDB,
		ChainInfoFetcher: &mockChain.ChainService{
			DB:                  beaconDB,
			Block:               wsb,
			Root:                headBlock.BlockRoot,
			FinalizedCheckPoint: &ethpbalpha.Checkpoint{Root: blkContainers[64].BlockRoot},
			CanonicalRoots:      canonicalRoots,
		},
	}

	root, err := genBlk.Block.HashTreeRoot()
	require.NoError(t, err)

	tests := []struct {
		name    string
		blockID []byte
		want    *ethpbalpha.SignedBeaconBlock
		wantErr bool
	}{
		{
			name:    "slot",
			blockID: []byte("30"),
			want:    blkContainers[30].Block.(*ethpbalpha.BeaconBlockContainer_Phase0Block).Phase0Block,
		},
		{
			name:    "bad formatting",
			blockID: []byte("3bad0"),
			wantErr: true,
		},
		{
			name:    "canonical",
			blockID: []byte("30"),
			want:    blkContainers[30].Block.(*ethpbalpha.BeaconBlockContainer_Phase0Block).Phase0Block,
		},
		{
			name:    "non canonical",
			blockID: []byte(fmt.Sprintf("%d", nextSlot)),
			want:    nil,
		},
		{
			name:    "head",
			blockID: []byte("head"),
			want:    headBlock.Block.(*ethpbalpha.BeaconBlockContainer_Phase0Block).Phase0Block,
		},
		{
			name:    "finalized",
			blockID: []byte("finalized"),
			want:    blkContainers[64].Block.(*ethpbalpha.BeaconBlockContainer_Phase0Block).Phase0Block,
		},
		{
			name:    "genesis",
			blockID: []byte("genesis"),
			want:    genBlk,
		},
		{
			name:    "genesis root",
			blockID: root[:],
			want:    genBlk,
		},
		{
			name:    "root",
			blockID: blkContainers[20].BlockRoot,
			want:    blkContainers[20].Block.(*ethpbalpha.BeaconBlockContainer_Phase0Block).Phase0Block,
		},
		{
			name:    "non-existent root",
			blockID: bytesutil.PadTo([]byte("hi there"), 32),
			want:    nil,
		},
		{
			name:    "hex",
			blockID: []byte(hexutil.Encode(blkContainers[20].BlockRoot)),
			want:    blkContainers[20].Block.(*ethpbalpha.BeaconBlockContainer_Phase0Block).Phase0Block,
		},
		{
			name:    "no block",
			blockID: []byte("105"),
			want:    nil,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, err := fetcher.Block(ctx, tt.blockID)
			if tt.wantErr {
				assert.NotEqual(t, err, nil, "no error has been returned")
				return
			}
			if tt.want == nil {
				assert.Equal(t, nil, result)
				return
			}
			require.NoError(t, err)
			pbBlock, err := result.PbPhase0Block()
			require.NoError(t, err)
			if !reflect.DeepEqual(pbBlock, tt.want) {
				t.Error("Expected blocks to equal")
			}
		})
	}
}

func TestGetBlob(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig().Copy()
	cfg.DenebForkEpoch = 1
	params.OverrideBeaconConfig(cfg)
	ctx := context.Background()
	db := testDB.SetupDB(t)
	denebBlock, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 123, 4)
	require.NoError(t, db.SaveBlock(context.Background(), denebBlock))
	_, bs, err := filesystem.NewEphemeralBlobStorageWithFs(t)
	require.NoError(t, err)
	testSidecars, err := verification.BlobSidecarSliceNoop(blobs)
	require.NoError(t, err)
	for i := range testSidecars {
		require.NoError(t, bs.Save(testSidecars[i]))
	}
	blockRoot := blobs[0].BlockRoot()
	t.Run("genesis", func(t *testing.T) {
		blocker := &BeaconDbBlocker{}
		_, rpcErr := blocker.Blobs(ctx, "genesis", nil)
		assert.Equal(t, http.StatusBadRequest, core.ErrorReasonToHTTP(rpcErr.Reason))
		assert.StringContains(t, "blobs are not supported for Phase 0 fork", rpcErr.Err.Error())
	})
	t.Run("head", func(t *testing.T) {
		blocker := &BeaconDbBlocker{
			ChainInfoFetcher: &mockChain.ChainService{Root: blockRoot[:]},
			BeaconDB:         db,
			BlobStorage:      bs,
		}
		verifiedBlobs, rpcErr := blocker.Blobs(ctx, "head", nil)
		assert.Equal(t, rpcErr == nil, true)
		require.Equal(t, 4, len(verifiedBlobs))
		sidecar := verifiedBlobs[0].BlobSidecar
		require.NotNil(t, sidecar)
		assert.Equal(t, uint64(0), sidecar.Index)
		assert.DeepEqual(t, blobs[0].Blob, sidecar.Blob)
		assert.DeepEqual(t, blobs[0].KzgCommitment, sidecar.KzgCommitment)
		assert.DeepEqual(t, blobs[0].KzgProof, sidecar.KzgProof)
		sidecar = verifiedBlobs[1].BlobSidecar
		require.NotNil(t, sidecar)
		assert.Equal(t, uint64(1), sidecar.Index)
		assert.DeepEqual(t, blobs[1].Blob, sidecar.Blob)
		assert.DeepEqual(t, blobs[1].KzgCommitment, sidecar.KzgCommitment)
		assert.DeepEqual(t, blobs[1].KzgProof, sidecar.KzgProof)
		sidecar = verifiedBlobs[2].BlobSidecar
		require.NotNil(t, sidecar)
		assert.Equal(t, uint64(2), sidecar.Index)
		assert.DeepEqual(t, blobs[2].Blob, sidecar.Blob)
		assert.DeepEqual(t, blobs[2].KzgCommitment, sidecar.KzgCommitment)
		assert.DeepEqual(t, blobs[2].KzgProof, sidecar.KzgProof)
		sidecar = verifiedBlobs[3].BlobSidecar
		require.NotNil(t, sidecar)
		assert.Equal(t, uint64(3), sidecar.Index)
		assert.DeepEqual(t, blobs[3].Blob, sidecar.Blob)
		assert.DeepEqual(t, blobs[3].KzgCommitment, sidecar.KzgCommitment)
		assert.DeepEqual(t, blobs[3].KzgProof, sidecar.KzgProof)
	})
	t.Run("finalized", func(t *testing.T) {
		blocker := &BeaconDbBlocker{
			ChainInfoFetcher: &mockChain.ChainService{FinalizedCheckPoint: &ethpbalpha.Checkpoint{Root: blockRoot[:]}},
			BeaconDB:         db,
			BlobStorage:      bs,
		}

		verifiedBlobs, rpcErr := blocker.Blobs(ctx, "finalized", nil)
		assert.Equal(t, rpcErr == nil, true)
		require.Equal(t, 4, len(verifiedBlobs))
	})
	t.Run("justified", func(t *testing.T) {
		blocker := &BeaconDbBlocker{
			ChainInfoFetcher: &mockChain.ChainService{CurrentJustifiedCheckPoint: &ethpbalpha.Checkpoint{Root: blockRoot[:]}},
			BeaconDB:         db,
			BlobStorage:      bs,
		}

		verifiedBlobs, rpcErr := blocker.Blobs(ctx, "justified", nil)
		assert.Equal(t, rpcErr == nil, true)
		require.Equal(t, 4, len(verifiedBlobs))
	})
	t.Run("root", func(t *testing.T) {
		blocker := &BeaconDbBlocker{
			BeaconDB:    db,
			BlobStorage: bs,
		}
		verifiedBlobs, rpcErr := blocker.Blobs(ctx, hexutil.Encode(blockRoot[:]), nil)
		assert.Equal(t, rpcErr == nil, true)
		require.Equal(t, 4, len(verifiedBlobs))
	})
	t.Run("slot", func(t *testing.T) {
		blocker := &BeaconDbBlocker{
			BeaconDB:    db,
			BlobStorage: bs,
		}
		verifiedBlobs, rpcErr := blocker.Blobs(ctx, "123", nil)
		assert.Equal(t, rpcErr == nil, true)
		require.Equal(t, 4, len(verifiedBlobs))
	})
	t.Run("one blob only", func(t *testing.T) {
		blocker := &BeaconDbBlocker{
			ChainInfoFetcher: &mockChain.ChainService{FinalizedCheckPoint: &ethpbalpha.Checkpoint{Root: blockRoot[:]}},
			BeaconDB:         db,
			BlobStorage:      bs,
		}
		verifiedBlobs, rpcErr := blocker.Blobs(ctx, "123", []uint64{2})
		assert.Equal(t, rpcErr == nil, true)
		require.Equal(t, 1, len(verifiedBlobs))
		sidecar := verifiedBlobs[0].BlobSidecar
		require.NotNil(t, sidecar)
		assert.Equal(t, uint64(2), sidecar.Index)
		assert.DeepEqual(t, blobs[2].Blob, sidecar.Blob)
		assert.DeepEqual(t, blobs[2].KzgCommitment, sidecar.KzgCommitment)
		assert.DeepEqual(t, blobs[2].KzgProof, sidecar.KzgProof)
	})
	t.Run("no blobs returns an empty array", func(t *testing.T) {
		blocker := &BeaconDbBlocker{
			ChainInfoFetcher: &mockChain.ChainService{FinalizedCheckPoint: &ethpbalpha.Checkpoint{Root: blockRoot[:]}},
			BeaconDB:         db,
			BlobStorage:      filesystem.NewEphemeralBlobStorage(t),
		}
		verifiedBlobs, rpcErr := blocker.Blobs(ctx, "123", nil)
		assert.Equal(t, rpcErr == nil, true)
		require.Equal(t, 0, len(verifiedBlobs))
	})
}
