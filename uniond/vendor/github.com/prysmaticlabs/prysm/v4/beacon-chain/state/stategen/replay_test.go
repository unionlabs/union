package stategen

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	testDB "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	consensusblocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"google.golang.org/protobuf/proto"
)

func TestReplayBlocks_AllSkipSlots(t *testing.T) {
	beaconDB := testDB.SetupDB(t)

	beaconState, _ := util.DeterministicGenesisState(t, 32)
	genesisBlock := blocks.NewGenesisBlock([]byte{})
	bodyRoot, err := genesisBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	err = beaconState.SetLatestBlockHeader(&ethpb.BeaconBlockHeader{
		Slot:       genesisBlock.Block.Slot,
		ParentRoot: genesisBlock.Block.ParentRoot,
		StateRoot:  params.BeaconConfig().ZeroHash[:],
		BodyRoot:   bodyRoot[:],
	})
	require.NoError(t, err)
	require.NoError(t, beaconState.SetSlashings(make([]uint64, params.BeaconConfig().EpochsPerSlashingsVector)))
	cp := beaconState.CurrentJustifiedCheckpoint()
	var mockRoot [32]byte
	copy(mockRoot[:], "hello-world")
	cp.Root = mockRoot[:]
	require.NoError(t, beaconState.SetCurrentJustifiedCheckpoint(cp))
	require.NoError(t, beaconState.AppendCurrentEpochAttestations(&ethpb.PendingAttestation{}))

	service := New(beaconDB, doublylinkedtree.New())
	targetSlot := params.BeaconConfig().SlotsPerEpoch - 1
	newState, err := service.replayBlocks(context.Background(), beaconState, []interfaces.ReadOnlySignedBeaconBlock{}, targetSlot)
	require.NoError(t, err)
	assert.Equal(t, targetSlot, newState.Slot(), "Did not advance slots")
}

func TestReplayBlocks_SameSlot(t *testing.T) {
	beaconDB := testDB.SetupDB(t)

	beaconState, _ := util.DeterministicGenesisState(t, 32)
	genesisBlock := blocks.NewGenesisBlock([]byte{})
	bodyRoot, err := genesisBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	err = beaconState.SetLatestBlockHeader(&ethpb.BeaconBlockHeader{
		Slot:       genesisBlock.Block.Slot,
		ParentRoot: genesisBlock.Block.ParentRoot,
		StateRoot:  params.BeaconConfig().ZeroHash[:],
		BodyRoot:   bodyRoot[:],
	})
	require.NoError(t, err)
	require.NoError(t, beaconState.SetSlashings(make([]uint64, params.BeaconConfig().EpochsPerSlashingsVector)))
	cp := beaconState.CurrentJustifiedCheckpoint()
	var mockRoot [32]byte
	copy(mockRoot[:], "hello-world")
	cp.Root = mockRoot[:]
	require.NoError(t, beaconState.SetCurrentJustifiedCheckpoint(cp))
	require.NoError(t, beaconState.AppendCurrentEpochAttestations(&ethpb.PendingAttestation{}))

	service := New(beaconDB, doublylinkedtree.New())
	targetSlot := beaconState.Slot()
	newState, err := service.replayBlocks(context.Background(), beaconState, []interfaces.ReadOnlySignedBeaconBlock{}, targetSlot)
	require.NoError(t, err)
	assert.Equal(t, targetSlot, newState.Slot(), "Did not advance slots")
}

func TestReplayBlocks_LowerSlotBlock(t *testing.T) {
	beaconDB := testDB.SetupDB(t)

	beaconState, _ := util.DeterministicGenesisState(t, 32)
	require.NoError(t, beaconState.SetSlot(1))
	genesisBlock := blocks.NewGenesisBlock([]byte{})
	bodyRoot, err := genesisBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	err = beaconState.SetLatestBlockHeader(&ethpb.BeaconBlockHeader{
		Slot:       genesisBlock.Block.Slot,
		ParentRoot: genesisBlock.Block.ParentRoot,
		StateRoot:  params.BeaconConfig().ZeroHash[:],
		BodyRoot:   bodyRoot[:],
	})
	require.NoError(t, err)
	require.NoError(t, beaconState.SetSlashings(make([]uint64, params.BeaconConfig().EpochsPerSlashingsVector)))
	cp := beaconState.CurrentJustifiedCheckpoint()
	var mockRoot [32]byte
	copy(mockRoot[:], "hello-world")
	cp.Root = mockRoot[:]
	require.NoError(t, beaconState.SetCurrentJustifiedCheckpoint(cp))
	require.NoError(t, beaconState.AppendCurrentEpochAttestations(&ethpb.PendingAttestation{}))

	service := New(beaconDB, doublylinkedtree.New())
	targetSlot := beaconState.Slot()
	b := util.NewBeaconBlock()
	b.Block.Slot = beaconState.Slot() - 1
	wsb, err := consensusblocks.NewSignedBeaconBlock(b)
	require.NoError(t, err)
	newState, err := service.replayBlocks(context.Background(), beaconState, []interfaces.ReadOnlySignedBeaconBlock{wsb}, targetSlot)
	require.NoError(t, err)
	assert.Equal(t, targetSlot, newState.Slot(), "Did not advance slots")
}

func TestReplayBlocks_ThroughForkBoundary(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	bCfg := params.BeaconConfig().Copy()
	bCfg.AltairForkEpoch = 1
	bCfg.ForkVersionSchedule[bytesutil.ToBytes4(bCfg.AltairForkVersion)] = 1
	params.OverrideBeaconConfig(bCfg)

	beaconState, _ := util.DeterministicGenesisState(t, 32)
	genesisBlock := blocks.NewGenesisBlock([]byte{})
	bodyRoot, err := genesisBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	err = beaconState.SetLatestBlockHeader(&ethpb.BeaconBlockHeader{
		Slot:       genesisBlock.Block.Slot,
		ParentRoot: genesisBlock.Block.ParentRoot,
		StateRoot:  params.BeaconConfig().ZeroHash[:],
		BodyRoot:   bodyRoot[:],
	})
	require.NoError(t, err)

	service := New(testDB.SetupDB(t), doublylinkedtree.New())
	targetSlot := params.BeaconConfig().SlotsPerEpoch
	newState, err := service.replayBlocks(context.Background(), beaconState, []interfaces.ReadOnlySignedBeaconBlock{}, targetSlot)
	require.NoError(t, err)

	// Verify state is version Altair.
	assert.Equal(t, version.Altair, newState.Version())
}

func TestReplayBlocks_ThroughCapellaForkBoundary(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	bCfg := params.BeaconConfig().Copy()
	bCfg.AltairForkEpoch = 1
	bCfg.ForkVersionSchedule[bytesutil.ToBytes4(bCfg.AltairForkVersion)] = 1
	bCfg.BellatrixForkEpoch = 2
	bCfg.ForkVersionSchedule[bytesutil.ToBytes4(bCfg.BellatrixForkVersion)] = 2
	bCfg.CapellaForkEpoch = 3
	bCfg.ForkVersionSchedule[bytesutil.ToBytes4(bCfg.CapellaForkVersion)] = 3
	params.OverrideBeaconConfig(bCfg)

	beaconState, _ := util.DeterministicGenesisState(t, 32)
	genesisBlock := blocks.NewGenesisBlock([]byte{})
	bodyRoot, err := genesisBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	err = beaconState.SetLatestBlockHeader(&ethpb.BeaconBlockHeader{
		Slot:       genesisBlock.Block.Slot,
		ParentRoot: genesisBlock.Block.ParentRoot,
		StateRoot:  params.BeaconConfig().ZeroHash[:],
		BodyRoot:   bodyRoot[:],
	})
	require.NoError(t, err)

	service := New(testDB.SetupDB(t), doublylinkedtree.New())
	targetSlot := params.BeaconConfig().SlotsPerEpoch * 2
	newState, err := service.replayBlocks(context.Background(), beaconState, []interfaces.ReadOnlySignedBeaconBlock{}, targetSlot)
	require.NoError(t, err)

	// Verify state is version Bellatrix.
	assert.Equal(t, version.Bellatrix, newState.Version())

	targetSlot = params.BeaconConfig().SlotsPerEpoch * 3
	newState, err = service.replayBlocks(context.Background(), newState, []interfaces.ReadOnlySignedBeaconBlock{}, targetSlot)
	require.NoError(t, err)

	// Verify state is version Capella.
	assert.Equal(t, version.Capella, newState.Version())
}

func TestLoadBlocks_FirstBranch(t *testing.T) {
	beaconDB := testDB.SetupDB(t)
	ctx := context.Background()
	s := &State{
		beaconDB: beaconDB,
	}

	roots, savedBlocks, err := tree1(t, beaconDB, bytesutil.PadTo([]byte{'A'}, 32))
	require.NoError(t, err)

	filteredBlocks, err := s.loadBlocks(ctx, 0, 8, roots[len(roots)-1])
	require.NoError(t, err)

	wanted := []*ethpb.SignedBeaconBlock{
		savedBlocks[8],
		savedBlocks[6],
		savedBlocks[4],
		savedBlocks[2],
		savedBlocks[1],
		savedBlocks[0],
	}

	for i, block := range wanted {
		filteredBlocksPb, err := filteredBlocks[i].Proto()
		require.NoError(t, err)
		if !proto.Equal(block, filteredBlocksPb) {
			t.Error("Did not get wanted blocks")
		}
	}
}

func TestLoadBlocks_SecondBranch(t *testing.T) {
	beaconDB := testDB.SetupDB(t)
	ctx := context.Background()
	s := &State{
		beaconDB: beaconDB,
	}

	roots, savedBlocks, err := tree1(t, beaconDB, bytesutil.PadTo([]byte{'A'}, 32))
	require.NoError(t, err)

	filteredBlocks, err := s.loadBlocks(ctx, 0, 5, roots[5])
	require.NoError(t, err)

	wanted := []*ethpb.SignedBeaconBlock{
		savedBlocks[5],
		savedBlocks[3],
		savedBlocks[1],
		savedBlocks[0],
	}

	for i, block := range wanted {
		filteredBlocksPb, err := filteredBlocks[i].Proto()
		require.NoError(t, err)
		if !proto.Equal(block, filteredBlocksPb) {
			t.Error("Did not get wanted blocks")
		}
	}
}

func TestLoadBlocks_ThirdBranch(t *testing.T) {
	beaconDB := testDB.SetupDB(t)
	ctx := context.Background()
	s := &State{
		beaconDB: beaconDB,
	}

	roots, savedBlocks, err := tree1(t, beaconDB, bytesutil.PadTo([]byte{'A'}, 32))
	require.NoError(t, err)

	filteredBlocks, err := s.loadBlocks(ctx, 0, 7, roots[7])
	require.NoError(t, err)

	wanted := []*ethpb.SignedBeaconBlock{
		savedBlocks[7],
		savedBlocks[6],
		savedBlocks[4],
		savedBlocks[2],
		savedBlocks[1],
		savedBlocks[0],
	}

	for i, block := range wanted {
		filteredBlocksPb, err := filteredBlocks[i].Proto()
		require.NoError(t, err)
		if !proto.Equal(block, filteredBlocksPb) {
			t.Error("Did not get wanted blocks")
		}
	}
}

func TestLoadBlocks_SameSlots(t *testing.T) {
	beaconDB := testDB.SetupDB(t)
	ctx := context.Background()
	s := &State{
		beaconDB: beaconDB,
	}

	roots, savedBlocks, err := tree2(t, beaconDB, bytesutil.PadTo([]byte{'A'}, 32))
	require.NoError(t, err)

	filteredBlocks, err := s.loadBlocks(ctx, 0, 3, roots[6])
	require.NoError(t, err)

	wanted := []*ethpb.SignedBeaconBlock{
		savedBlocks[6],
		savedBlocks[5],
		savedBlocks[1],
		savedBlocks[0],
	}

	for i, block := range wanted {
		filteredBlocksPb, err := filteredBlocks[i].Proto()
		require.NoError(t, err)
		if !proto.Equal(block, filteredBlocksPb) {
			t.Error("Did not get wanted blocks")
		}
	}
}

func TestLoadBlocks_SameEndSlots(t *testing.T) {
	beaconDB := testDB.SetupDB(t)
	ctx := context.Background()
	s := &State{
		beaconDB: beaconDB,
	}

	roots, savedBlocks, err := tree3(t, beaconDB, bytesutil.PadTo([]byte{'A'}, 32))
	require.NoError(t, err)

	filteredBlocks, err := s.loadBlocks(ctx, 0, 2, roots[2])
	require.NoError(t, err)

	wanted := []*ethpb.SignedBeaconBlock{
		savedBlocks[2],
		savedBlocks[1],
		savedBlocks[0],
	}

	for i, block := range wanted {
		filteredBlocksPb, err := filteredBlocks[i].Proto()
		require.NoError(t, err)
		if !proto.Equal(block, filteredBlocksPb) {
			t.Error("Did not get wanted blocks")
		}
	}
}

func TestLoadBlocks_SameEndSlotsWith2blocks(t *testing.T) {
	beaconDB := testDB.SetupDB(t)
	ctx := context.Background()
	s := &State{
		beaconDB: beaconDB,
	}

	roots, savedBlocks, err := tree4(t, beaconDB, bytesutil.PadTo([]byte{'A'}, 32))
	require.NoError(t, err)

	filteredBlocks, err := s.loadBlocks(ctx, 0, 2, roots[1])
	require.NoError(t, err)

	wanted := []*ethpb.SignedBeaconBlock{
		savedBlocks[1],
		savedBlocks[0],
	}

	for i, block := range wanted {
		filteredBlocksPb, err := filteredBlocks[i].Proto()
		require.NoError(t, err)
		if !proto.Equal(block, filteredBlocksPb) {
			t.Error("Did not get wanted blocks")
		}
	}
}

func TestLoadBlocks_BadStart(t *testing.T) {
	beaconDB := testDB.SetupDB(t)
	ctx := context.Background()
	s := &State{
		beaconDB: beaconDB,
	}

	roots, _, err := tree1(t, beaconDB, bytesutil.PadTo([]byte{'A'}, 32))
	require.NoError(t, err)
	_, err = s.loadBlocks(ctx, 0, 5, roots[8])
	assert.ErrorContains(t, "end block roots don't match", err)
}

// tree1 constructs the following tree:
// B0 - B1 - - B3 -- B5
//
//	\- B2 -- B4 -- B6 ----- B8
//	                 \- B7
func tree1(t *testing.T, beaconDB db.Database, genesisRoot []byte) ([][32]byte, []*ethpb.SignedBeaconBlock, error) {
	b0 := util.NewBeaconBlock()
	b0.Block.Slot = 0
	b0.Block.ParentRoot = genesisRoot
	r0, err := b0.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b1 := util.NewBeaconBlock()
	b1.Block.Slot = 1
	b1.Block.ParentRoot = r0[:]
	r1, err := b1.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b2 := util.NewBeaconBlock()
	b2.Block.Slot = 2
	b2.Block.ParentRoot = r1[:]
	r2, err := b2.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b3 := util.NewBeaconBlock()
	b3.Block.Slot = 3
	b3.Block.ParentRoot = r1[:]
	r3, err := b3.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b4 := util.NewBeaconBlock()
	b4.Block.Slot = 4
	b4.Block.ParentRoot = r2[:]
	r4, err := b4.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b5 := util.NewBeaconBlock()
	b5.Block.Slot = 5
	b5.Block.ParentRoot = r3[:]
	r5, err := b5.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b6 := util.NewBeaconBlock()
	b6.Block.Slot = 6
	b6.Block.ParentRoot = r4[:]
	r6, err := b6.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b7 := util.NewBeaconBlock()
	b7.Block.Slot = 7
	b7.Block.ParentRoot = r6[:]
	r7, err := b7.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b8 := util.NewBeaconBlock()
	b8.Block.Slot = 8
	b8.Block.ParentRoot = r6[:]
	r8, err := b8.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	st, err := util.NewBeaconState()
	require.NoError(t, err)

	returnedBlocks := make([]*ethpb.SignedBeaconBlock, 0)
	for _, b := range []*ethpb.SignedBeaconBlock{b0, b1, b2, b3, b4, b5, b6, b7, b8} {
		beaconBlock := util.NewBeaconBlock()
		beaconBlock.Block.Slot = b.Block.Slot
		beaconBlock.Block.ParentRoot = bytesutil.PadTo(b.Block.ParentRoot, 32)
		wsb, err := consensusblocks.NewSignedBeaconBlock(beaconBlock)
		require.NoError(t, err)
		if err := beaconDB.SaveBlock(context.Background(), wsb); err != nil {
			return nil, nil, err
		}
		if err := beaconDB.SaveState(context.Background(), st.Copy(), bytesutil.ToBytes32(beaconBlock.Block.ParentRoot)); err != nil {
			return nil, nil, err
		}
		returnedBlocks = append(returnedBlocks, beaconBlock)
	}
	return [][32]byte{r0, r1, r2, r3, r4, r5, r6, r7, r8}, returnedBlocks, nil
}

// tree2 constructs the following tree:
// B0 - B1
//
//	\- B2
//	\- B2
//	\- B2
//	\- B2 -- B3
func tree2(t *testing.T, beaconDB db.Database, genesisRoot []byte) ([][32]byte, []*ethpb.SignedBeaconBlock, error) {
	b0 := util.NewBeaconBlock()
	b0.Block.Slot = 0
	b0.Block.ParentRoot = genesisRoot
	r0, err := b0.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b1 := util.NewBeaconBlock()
	b1.Block.Slot = 1
	b1.Block.ParentRoot = r0[:]
	r1, err := b1.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b21 := util.NewBeaconBlock()
	b21.Block.Slot = 2
	b21.Block.ParentRoot = r1[:]
	b21.Block.StateRoot = bytesutil.PadTo([]byte{'A'}, 32)
	r21, err := b21.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b22 := util.NewBeaconBlock()
	b22.Block.Slot = 2
	b22.Block.ParentRoot = r1[:]
	b22.Block.StateRoot = bytesutil.PadTo([]byte{'B'}, 32)
	r22, err := b22.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b23 := util.NewBeaconBlock()
	b23.Block.Slot = 2
	b23.Block.ParentRoot = r1[:]
	b23.Block.StateRoot = bytesutil.PadTo([]byte{'C'}, 32)
	r23, err := b23.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b24 := util.NewBeaconBlock()
	b24.Block.Slot = 2
	b24.Block.ParentRoot = r1[:]
	b24.Block.StateRoot = bytesutil.PadTo([]byte{'D'}, 32)
	r24, err := b24.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b3 := util.NewBeaconBlock()
	b3.Block.Slot = 3
	b3.Block.ParentRoot = r24[:]
	r3, err := b3.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	st, err := util.NewBeaconState()
	require.NoError(t, err)

	returnedBlocks := make([]*ethpb.SignedBeaconBlock, 0)
	for _, b := range []*ethpb.SignedBeaconBlock{b0, b1, b21, b22, b23, b24, b3} {
		beaconBlock := util.NewBeaconBlock()
		beaconBlock.Block.Slot = b.Block.Slot
		beaconBlock.Block.ParentRoot = bytesutil.PadTo(b.Block.ParentRoot, 32)
		beaconBlock.Block.StateRoot = bytesutil.PadTo(b.Block.StateRoot, 32)
		wsb, err := consensusblocks.NewSignedBeaconBlock(beaconBlock)
		require.NoError(t, err)
		if err := beaconDB.SaveBlock(context.Background(), wsb); err != nil {
			return nil, nil, err
		}
		if err := beaconDB.SaveState(context.Background(), st.Copy(), bytesutil.ToBytes32(beaconBlock.Block.ParentRoot)); err != nil {
			return nil, nil, err
		}
		returnedBlocks = append(returnedBlocks, beaconBlock)
	}
	return [][32]byte{r0, r1, r21, r22, r23, r24, r3}, returnedBlocks, nil
}

// tree3 constructs the following tree:
// B0 - B1
//
//	\- B2
//	\- B2
//	\- B2
//	\- B2
func tree3(t *testing.T, beaconDB db.Database, genesisRoot []byte) ([][32]byte, []*ethpb.SignedBeaconBlock, error) {
	b0 := util.NewBeaconBlock()
	b0.Block.Slot = 0
	b0.Block.ParentRoot = genesisRoot
	r0, err := b0.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b1 := util.NewBeaconBlock()
	b1.Block.Slot = 1
	b1.Block.ParentRoot = r0[:]
	r1, err := b1.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b21 := util.NewBeaconBlock()
	b21.Block.Slot = 2
	b21.Block.ParentRoot = r1[:]
	b21.Block.StateRoot = bytesutil.PadTo([]byte{'A'}, 32)
	r21, err := b21.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b22 := util.NewBeaconBlock()
	b22.Block.Slot = 2
	b22.Block.ParentRoot = r1[:]
	b22.Block.StateRoot = bytesutil.PadTo([]byte{'B'}, 32)
	r22, err := b22.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b23 := util.NewBeaconBlock()
	b23.Block.Slot = 2
	b23.Block.ParentRoot = r1[:]
	b23.Block.StateRoot = bytesutil.PadTo([]byte{'C'}, 32)
	r23, err := b23.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b24 := util.NewBeaconBlock()
	b24.Block.Slot = 2
	b24.Block.ParentRoot = r1[:]
	b24.Block.StateRoot = bytesutil.PadTo([]byte{'D'}, 32)
	r24, err := b24.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	st, err := util.NewBeaconState()
	require.NoError(t, err)

	returnedBlocks := make([]*ethpb.SignedBeaconBlock, 0)
	for _, b := range []*ethpb.SignedBeaconBlock{b0, b1, b21, b22, b23, b24} {
		beaconBlock := util.NewBeaconBlock()
		beaconBlock.Block.Slot = b.Block.Slot
		beaconBlock.Block.ParentRoot = bytesutil.PadTo(b.Block.ParentRoot, 32)
		beaconBlock.Block.StateRoot = bytesutil.PadTo(b.Block.StateRoot, 32)
		wsb, err := consensusblocks.NewSignedBeaconBlock(beaconBlock)
		require.NoError(t, err)
		if err := beaconDB.SaveBlock(context.Background(), wsb); err != nil {
			return nil, nil, err
		}
		if err := beaconDB.SaveState(context.Background(), st.Copy(), bytesutil.ToBytes32(beaconBlock.Block.ParentRoot)); err != nil {
			return nil, nil, err
		}
		returnedBlocks = append(returnedBlocks, beaconBlock)
	}

	return [][32]byte{r0, r1, r21, r22, r23, r24}, returnedBlocks, nil
}

// tree4 constructs the following tree:
// B0
//
//	\- B2
//	\- B2
//	\- B2
//	\- B2
func tree4(t *testing.T, beaconDB db.Database, genesisRoot []byte) ([][32]byte, []*ethpb.SignedBeaconBlock, error) {
	b0 := util.NewBeaconBlock()
	b0.Block.Slot = 0
	b0.Block.ParentRoot = genesisRoot
	r0, err := b0.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b21 := util.NewBeaconBlock()
	b21.Block.Slot = 2
	b21.Block.ParentRoot = r0[:]
	b21.Block.StateRoot = bytesutil.PadTo([]byte{'A'}, 32)
	r21, err := b21.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b22 := util.NewBeaconBlock()
	b22.Block.Slot = 2
	b22.Block.ParentRoot = r0[:]
	b22.Block.StateRoot = bytesutil.PadTo([]byte{'B'}, 32)
	r22, err := b22.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b23 := util.NewBeaconBlock()
	b23.Block.Slot = 2
	b23.Block.ParentRoot = r0[:]
	b23.Block.StateRoot = bytesutil.PadTo([]byte{'C'}, 32)
	r23, err := b23.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	b24 := util.NewBeaconBlock()
	b24.Block.Slot = 2
	b24.Block.ParentRoot = r0[:]
	b24.Block.StateRoot = bytesutil.PadTo([]byte{'D'}, 32)
	r24, err := b24.Block.HashTreeRoot()
	if err != nil {
		return nil, nil, err
	}
	st, err := util.NewBeaconState()
	require.NoError(t, err)

	returnedBlocks := make([]*ethpb.SignedBeaconBlock, 0)
	for _, b := range []*ethpb.SignedBeaconBlock{b0, b21, b22, b23, b24} {
		beaconBlock := util.NewBeaconBlock()
		beaconBlock.Block.Slot = b.Block.Slot
		beaconBlock.Block.ParentRoot = bytesutil.PadTo(b.Block.ParentRoot, 32)
		beaconBlock.Block.StateRoot = bytesutil.PadTo(b.Block.StateRoot, 32)
		wsb, err := consensusblocks.NewSignedBeaconBlock(beaconBlock)
		require.NoError(t, err)
		if err := beaconDB.SaveBlock(context.Background(), wsb); err != nil {
			return nil, nil, err
		}
		if err := beaconDB.SaveState(context.Background(), st.Copy(), bytesutil.ToBytes32(beaconBlock.Block.ParentRoot)); err != nil {
			return nil, nil, err
		}
		returnedBlocks = append(returnedBlocks, beaconBlock)
	}

	return [][32]byte{r0, r21, r22, r23, r24}, returnedBlocks, nil
}

func TestLoadFinalizedBlocks(t *testing.T) {
	beaconDB := testDB.SetupDB(t)
	ctx := context.Background()
	s := &State{
		beaconDB: beaconDB,
	}
	gBlock := util.NewBeaconBlock()
	gRoot, err := gBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, ctx, beaconDB, gBlock)
	require.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, [32]byte{}))
	roots, _, err := tree1(t, beaconDB, gRoot[:])
	require.NoError(t, err)

	filteredBlocks, err := s.loadFinalizedBlocks(ctx, 0, 8)
	require.NoError(t, err)
	require.Equal(t, 0, len(filteredBlocks))
	require.NoError(t, beaconDB.SaveStateSummary(ctx, &ethpb.StateSummary{Root: roots[8][:]}))

	require.NoError(t, s.beaconDB.SaveFinalizedCheckpoint(ctx, &ethpb.Checkpoint{Root: roots[8][:]}))
	filteredBlocks, err = s.loadFinalizedBlocks(ctx, 0, 8)
	require.NoError(t, err)
	require.Equal(t, 10, len(filteredBlocks))
}
