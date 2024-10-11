package verify

import (
	"fmt"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestBlobAlignsWithBlock(t *testing.T) {
	tests := []struct {
		name         string
		blockAndBlob func(t *testing.T) (blocks.ROBlock, []blocks.ROBlob)
		err          error
	}{
		{
			name: "happy path",
			blockAndBlob: func(t *testing.T) (blocks.ROBlock, []blocks.ROBlob) {
				return util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 0, 1)
			},
		},
		{
			name: "mismatched roots",
			blockAndBlob: func(t *testing.T) (blocks.ROBlock, []blocks.ROBlob) {
				blk, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 0, 1)
				tweaked := blobs[0].BlobSidecar
				tweaked.SignedBlockHeader.Header.Slot = tweaked.SignedBlockHeader.Header.Slot + 1
				tampered, err := blocks.NewROBlob(tweaked)
				require.NoError(t, err)
				return blk, []blocks.ROBlob{tampered}
			},
			err: ErrBlobBlockMisaligned,
		},
		{
			name: "mismatched roots - fake",
			blockAndBlob: func(t *testing.T) (blocks.ROBlock, []blocks.ROBlob) {
				blk, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 0, 1)
				copied := blobs[0].BlobSidecar
				// exact same header, mess with the root
				fake, err := blocks.NewROBlobWithRoot(copied, bytesutil.ToBytes32([]byte("derp")))
				require.NoError(t, err)
				return blk, []blocks.ROBlob{fake}
			},
			err: ErrBlobBlockMisaligned,
		},
		{
			name: "before deneb",
			blockAndBlob: func(t *testing.T) (blocks.ROBlock, []blocks.ROBlob) {
				cb := util.NewBeaconBlockCapella()
				blk, err := blocks.NewSignedBeaconBlock(cb)
				require.NoError(t, err)
				rob, err := blocks.NewROBlock(blk)
				require.NoError(t, err)
				return rob, []blocks.ROBlob{blocks.ROBlob{}}
			},
		},
	}

	for _, tt := range tests {
		block, blobs := tt.blockAndBlob(t)
		for i := range blobs {
			t.Run(tt.name+fmt.Sprintf(" blob %d", i), func(t *testing.T) {
				err := BlobAlignsWithBlock(blobs[i], block)
				if tt.err == nil {
					require.NoError(t, err)
				} else {
					require.ErrorIs(t, err, tt.err)
				}
			})
		}
	}
}
