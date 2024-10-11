package blockchain

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	v1 "github.com/prysmaticlabs/prysm/v4/proto/eth/v1"
	ethpbv2 "github.com/prysmaticlabs/prysm/v4/proto/eth/v2"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

type testlc struct {
	t              *testing.T
	ctx            context.Context
	state          state.BeaconState
	block          interfaces.ReadOnlySignedBeaconBlock
	attestedState  state.BeaconState
	attestedHeader *ethpb.BeaconBlockHeader
}

func newTestLc(t *testing.T) *testlc {
	return &testlc{t: t}
}

func (l *testlc) setupTest() *testlc {
	ctx := context.Background()

	slot := primitives.Slot(params.BeaconConfig().AltairForkEpoch * primitives.Epoch(params.BeaconConfig().SlotsPerEpoch)).Add(1)

	attestedState, err := util.NewBeaconStateCapella()
	require.NoError(l.t, err)
	err = attestedState.SetSlot(slot)
	require.NoError(l.t, err)

	parent := util.NewBeaconBlockCapella()
	parent.Block.Slot = slot

	signedParent, err := blocks.NewSignedBeaconBlock(parent)
	require.NoError(l.t, err)

	parentHeader, err := signedParent.Header()
	require.NoError(l.t, err)
	attestedHeader := parentHeader.Header

	err = attestedState.SetLatestBlockHeader(attestedHeader)
	require.NoError(l.t, err)
	attestedStateRoot, err := attestedState.HashTreeRoot(ctx)
	require.NoError(l.t, err)

	// get a new signed block so the root is updated with the new state root
	parent.Block.StateRoot = attestedStateRoot[:]
	signedParent, err = blocks.NewSignedBeaconBlock(parent)
	require.NoError(l.t, err)

	state, err := util.NewBeaconStateCapella()
	require.NoError(l.t, err)
	err = state.SetSlot(slot)
	require.NoError(l.t, err)

	parentRoot, err := signedParent.Block().HashTreeRoot()
	require.NoError(l.t, err)

	block := util.NewBeaconBlockCapella()
	block.Block.Slot = slot
	block.Block.ParentRoot = parentRoot[:]

	for i := uint64(0); i < params.BeaconConfig().MinSyncCommitteeParticipants; i++ {
		block.Block.Body.SyncAggregate.SyncCommitteeBits.SetBitAt(i, true)
	}

	signedBlock, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(l.t, err)

	h, err := signedBlock.Header()
	require.NoError(l.t, err)

	err = state.SetLatestBlockHeader(h.Header)
	require.NoError(l.t, err)
	stateRoot, err := state.HashTreeRoot(ctx)
	require.NoError(l.t, err)

	// get a new signed block so the root is updated with the new state root
	block.Block.StateRoot = stateRoot[:]
	signedBlock, err = blocks.NewSignedBeaconBlock(block)
	require.NoError(l.t, err)

	l.state = state
	l.attestedState = attestedState
	l.attestedHeader = attestedHeader
	l.block = signedBlock
	l.ctx = ctx

	return l
}

func (l *testlc) checkAttestedHeader(update *ethpbv2.LightClientUpdate) {
	require.Equal(l.t, l.attestedHeader.Slot, update.AttestedHeader.Slot, "Attested header slot is not equal")
	require.Equal(l.t, l.attestedHeader.ProposerIndex, update.AttestedHeader.ProposerIndex, "Attested header proposer index is not equal")
	require.DeepSSZEqual(l.t, l.attestedHeader.ParentRoot, update.AttestedHeader.ParentRoot, "Attested header parent root is not equal")
	require.DeepSSZEqual(l.t, l.attestedHeader.BodyRoot, update.AttestedHeader.BodyRoot, "Attested header body root is not equal")

	attestedStateRoot, err := l.attestedState.HashTreeRoot(l.ctx)
	require.NoError(l.t, err)
	require.DeepSSZEqual(l.t, attestedStateRoot[:], update.AttestedHeader.StateRoot, "Attested header state root is not equal")
}

func (l *testlc) checkSyncAggregate(update *ethpbv2.LightClientUpdate) {
	syncAggregate, err := l.block.Block().Body().SyncAggregate()
	require.NoError(l.t, err)
	require.DeepSSZEqual(l.t, syncAggregate.SyncCommitteeBits, update.SyncAggregate.SyncCommitteeBits, "SyncAggregate bits is not equal")
	require.DeepSSZEqual(l.t, syncAggregate.SyncCommitteeSignature, update.SyncAggregate.SyncCommitteeSignature, "SyncAggregate signature is not equal")
}

func TestLightClient_NewLightClientOptimisticUpdateFromBeaconState(t *testing.T) {
	l := newTestLc(t).setupTest()

	update, err := NewLightClientOptimisticUpdateFromBeaconState(l.ctx, l.state, l.block, l.attestedState)
	require.NoError(t, err)
	require.NotNil(t, update, "update is nil")

	require.Equal(t, l.block.Block().Slot(), update.SignatureSlot, "Signature slot is not equal")

	l.checkSyncAggregate(update)
	l.checkAttestedHeader(update)

	require.Equal(t, (*v1.BeaconBlockHeader)(nil), update.FinalizedHeader, "Finalized header is not nil")
	require.DeepSSZEqual(t, ([][]byte)(nil), update.FinalityBranch, "Finality branch is not nil")
}

func TestLightClient_NewLightClientFinalityUpdateFromBeaconState(t *testing.T) {
	l := newTestLc(t).setupTest()

	update, err := NewLightClientFinalityUpdateFromBeaconState(l.ctx, l.state, l.block, l.attestedState, nil)
	require.NoError(t, err)
	require.NotNil(t, update, "update is nil")

	require.Equal(t, l.block.Block().Slot(), update.SignatureSlot, "Signature slot is not equal")

	l.checkSyncAggregate(update)
	l.checkAttestedHeader(update)

	zeroHash := params.BeaconConfig().ZeroHash[:]
	require.NotNil(t, update.FinalizedHeader, "Finalized header is nil")
	require.Equal(t, primitives.Slot(0), update.FinalizedHeader.Slot, "Finalized header slot is not zero")
	require.Equal(t, primitives.ValidatorIndex(0), update.FinalizedHeader.ProposerIndex, "Finalized header proposer index is not zero")
	require.DeepSSZEqual(t, zeroHash, update.FinalizedHeader.ParentRoot, "Finalized header parent root is not zero")
	require.DeepSSZEqual(t, zeroHash, update.FinalizedHeader.StateRoot, "Finalized header state root is not zero")
	require.DeepSSZEqual(t, zeroHash, update.FinalizedHeader.BodyRoot, "Finalized header body root is not zero")
	require.Equal(t, finalityBranchNumOfLeaves, len(update.FinalityBranch), "Invalid finality branch leaves")
	for _, leaf := range update.FinalityBranch {
		require.DeepSSZEqual(t, zeroHash, leaf, "Leaf is not zero")
	}
}
