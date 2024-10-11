package validator

import (
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestConstructGenericBeaconBlock(t *testing.T) {
	vs := &Server{}

	// Test when sBlk or sBlk.Block() is nil
	t.Run("NilBlock", func(t *testing.T) {
		_, err := vs.constructGenericBeaconBlock(nil, nil)
		require.ErrorContains(t, "block cannot be nil", err)
	})

	// Test for Deneb version
	t.Run("deneb block", func(t *testing.T) {
		eb := util.NewBeaconBlockDeneb()
		b, err := blocks.NewSignedBeaconBlock(eb)
		require.NoError(t, err)
		commitment, err := hexutil.Decode("0x8dab030c51e16e84be9caab84ee3d0b8bbec1db4a0e4de76439da8424d9b957370a10a78851f97e4b54d2ce1ab0d686f")
		require.NoError(t, err)
		proof, err := hexutil.Decode("0xb4021b0de10f743893d4f71e1bf830c019e832958efd6795baf2f83b8699a9eccc5dc99015d8d4d8ec370d0cc333c06a")
		require.NoError(t, err)
		bundle := &enginev1.BlobsBundle{
			KzgCommitments: [][]byte{
				commitment,
			},
			Proofs: [][]byte{
				proof,
			},
			Blobs: [][]byte{
				make([]byte, fieldparams.BlobLength),
			},
		}

		contents := &eth.BeaconBlockContentsDeneb{Block: eb.Block, KzgProofs: bundle.Proofs, Blobs: bundle.Blobs}
		r1, err := contents.HashTreeRoot()
		require.NoError(t, err)
		result, err := vs.constructGenericBeaconBlock(b, bundle)
		require.NoError(t, err)
		r2, err := result.GetDeneb().HashTreeRoot()
		require.NoError(t, err)
		require.Equal(t, r1, r2)
		require.Equal(t, result.IsBlinded, false)
	})

	// Test for blind Deneb version
	t.Run("blind deneb block", func(t *testing.T) {
		b, err := blocks.NewSignedBeaconBlock(util.NewBlindedBeaconBlockDeneb())
		require.NoError(t, err)
		r1, err := b.Block().HashTreeRoot()
		require.NoError(t, err)
		scs := &enginev1.BlobsBundle{}
		result, err := vs.constructGenericBeaconBlock(b, scs)
		require.NoError(t, err)
		r2, err := result.GetBlindedDeneb().HashTreeRoot()
		require.NoError(t, err)
		require.Equal(t, r1, r2)
		// TODO: update with kzg commit check after updating generic beacon block
		require.Equal(t, result.IsBlinded, true)
	})

	// Test for Capella version
	t.Run("capella block", func(t *testing.T) {
		b, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockCapella())
		require.NoError(t, err)
		result, err := vs.constructGenericBeaconBlock(b, nil)
		require.NoError(t, err)
		r1, err := result.GetCapella().HashTreeRoot()
		require.NoError(t, err)
		r2, err := b.Block().HashTreeRoot()
		require.NoError(t, err)
		require.Equal(t, r1, r2)
		require.Equal(t, result.IsBlinded, false)
	})

	// Test for blind Capella version
	t.Run("blind capella block", func(t *testing.T) {
		b, err := blocks.NewSignedBeaconBlock(util.NewBlindedBeaconBlockCapella())
		require.NoError(t, err)
		result, err := vs.constructGenericBeaconBlock(b, nil)
		require.NoError(t, err)
		r1, err := result.GetBlindedCapella().HashTreeRoot()
		require.NoError(t, err)
		r2, err := b.Block().HashTreeRoot()
		require.NoError(t, err)
		require.Equal(t, r1, r2)
		require.Equal(t, result.IsBlinded, true)
	})

	// Test for Bellatrix version
	t.Run("bellatrix block", func(t *testing.T) {
		b, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockBellatrix())
		require.NoError(t, err)
		result, err := vs.constructGenericBeaconBlock(b, nil)
		require.NoError(t, err)
		r1, err := result.GetBellatrix().HashTreeRoot()
		require.NoError(t, err)
		r2, err := b.Block().HashTreeRoot()
		require.NoError(t, err)
		require.Equal(t, r1, r2)
		require.Equal(t, result.IsBlinded, false)
	})

	// Test for Altair version
	t.Run("altair block", func(t *testing.T) {
		b, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockAltair())
		require.NoError(t, err)
		result, err := vs.constructGenericBeaconBlock(b, nil)
		require.NoError(t, err)
		r1, err := result.GetAltair().HashTreeRoot()
		require.NoError(t, err)
		r2, err := b.Block().HashTreeRoot()
		require.NoError(t, err)
		require.Equal(t, r1, r2)
		require.Equal(t, result.IsBlinded, false)
	})

	// Test for phase0 version
	t.Run("phase0 block", func(t *testing.T) {
		b, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
		require.NoError(t, err)
		result, err := vs.constructGenericBeaconBlock(b, nil)
		require.NoError(t, err)
		r1, err := result.GetPhase0().HashTreeRoot()
		require.NoError(t, err)
		r2, err := b.Block().HashTreeRoot()
		require.NoError(t, err)
		require.Equal(t, r1, r2)
		require.Equal(t, result.IsBlinded, false)
	})
}
