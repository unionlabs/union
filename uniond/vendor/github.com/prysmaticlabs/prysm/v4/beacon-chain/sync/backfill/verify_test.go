package backfill

import (
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	"github.com/prysmaticlabs/prysm/v4/runtime/interop"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestDomainCache(t *testing.T) {
	cfg := params.MainnetConfig()
	vRoot, err := hexutil.Decode("0x0011223344556677889900112233445566778899001122334455667788990011")
	dType := cfg.DomainBeaconProposer
	require.NoError(t, err)
	require.Equal(t, 32, len(vRoot))
	fsched := forks.NewOrderedSchedule(cfg)
	dc, err := newDomainCache(vRoot,
		dType, fsched)
	require.NoError(t, err)
	require.Equal(t, len(fsched), len(dc.forkDomains))
	for i := range fsched {
		e := fsched[i].Epoch
		ad, err := dc.forEpoch(e)
		require.NoError(t, err)
		ed, err := signing.ComputeDomain(dType, fsched[i].Version[:], vRoot)
		require.NoError(t, err)
		require.DeepEqual(t, ed, ad)
	}
}

func testBlocksWithKeys(t *testing.T, nBlocks uint64, nBlobs int, vr []byte) ([]blocks.ROBlock, [][]blocks.ROBlob, []bls.SecretKey, []bls.PublicKey) {
	blks := make([]blocks.ROBlock, nBlocks)
	blbs := make([][]blocks.ROBlob, nBlocks)
	sks, pks, err := interop.DeterministicallyGenerateKeys(0, nBlocks)
	require.NoError(t, err)
	prevRoot := [32]byte{}
	for i := uint64(0); i < nBlocks; i++ {
		block, blobs := util.GenerateTestDenebBlockWithSidecar(t, prevRoot, primitives.Slot(i), nBlobs, util.WithProposerSigning(primitives.ValidatorIndex(i), sks[i], vr))
		prevRoot = block.Root()
		blks[i] = block
		blbs[i] = blobs
	}
	return blks, blbs, sks, pks
}

func TestVerify(t *testing.T) {
	vr := make([]byte, 32)
	copy(vr, []byte("yooooo"))
	blks, _, _, pks := testBlocksWithKeys(t, 2, 0, vr)
	pubkeys := make([][fieldparams.BLSPubkeyLength]byte, len(pks))
	for i := range pks {
		pubkeys[i] = bytesutil.ToBytes48(pks[i].Marshal())
	}
	v, err := newBackfillVerifier(vr, pubkeys)
	require.NoError(t, err)
	notrob := make([]interfaces.ReadOnlySignedBeaconBlock, len(blks))
	// We have to unwrap the ROBlocks for this code because that's what it expects (for now).
	for i := range blks {
		notrob[i] = blks[i].ReadOnlySignedBeaconBlock
	}
	vbs, err := v.verify(notrob)
	require.NoError(t, err)
	require.Equal(t, len(blks), len(vbs))
}
