package debug

import (
	"context"
	"testing"
	"time"

	"github.com/prysmaticlabs/go-bitfield"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	dbTest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestServer_GetBlock(t *testing.T) {
	db := dbTest.SetupDB(t)
	ctx := context.Background()

	b := util.NewBeaconBlock()
	b.Block.Slot = 100
	util.SaveBlock(t, ctx, db, b)
	blockRoot, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	bs := &Server{
		BeaconDB: db,
	}
	res, err := bs.GetBlock(ctx, &ethpb.BlockRequestByRoot{
		BlockRoot: blockRoot[:],
	})
	require.NoError(t, err)
	wanted, err := b.MarshalSSZ()
	require.NoError(t, err)
	assert.DeepEqual(t, wanted, res.Encoded)

	// Checking for nil block.
	blockRoot = [32]byte{}
	res, err = bs.GetBlock(ctx, &ethpb.BlockRequestByRoot{
		BlockRoot: blockRoot[:],
	})
	require.NoError(t, err)
	assert.DeepEqual(t, []byte{}, res.Encoded)
}

func TestServer_GetAttestationInclusionSlot(t *testing.T) {
	db := dbTest.SetupDB(t)
	ctx := context.Background()
	offset := int64(2 * params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().SecondsPerSlot))
	bs := &Server{
		BeaconDB:           db,
		StateGen:           stategen.New(db, doublylinkedtree.New()),
		GenesisTimeFetcher: &mock.ChainService{Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second)},
	}

	s, _ := util.DeterministicGenesisState(t, 2048)
	tr := [32]byte{'a'}
	require.NoError(t, bs.StateGen.SaveState(ctx, tr, s))
	c, err := helpers.BeaconCommitteeFromState(context.Background(), s, 1, 0)
	require.NoError(t, err)

	a := &ethpb.Attestation{
		Data: &ethpb.AttestationData{
			Target:          &ethpb.Checkpoint{Root: tr[:]},
			Source:          &ethpb.Checkpoint{Root: make([]byte, 32)},
			BeaconBlockRoot: make([]byte, 32),
			Slot:            1,
		},
		AggregationBits: bitfield.Bitlist{0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01},
		Signature:       make([]byte, fieldparams.BLSSignatureLength),
	}
	b := util.NewBeaconBlock()
	b.Block.Slot = 2
	b.Block.Body.Attestations = []*ethpb.Attestation{a}
	util.SaveBlock(t, ctx, bs.BeaconDB, b)
	res, err := bs.GetInclusionSlot(ctx, &ethpb.InclusionSlotRequest{Slot: 1, Id: uint64(c[0])})
	require.NoError(t, err)
	require.Equal(t, b.Block.Slot, res.Slot)
	res, err = bs.GetInclusionSlot(ctx, &ethpb.InclusionSlotRequest{Slot: 1, Id: 9999999})
	require.NoError(t, err)
	require.Equal(t, params.BeaconConfig().FarFutureSlot, res.Slot)
}
