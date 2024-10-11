package lightclient

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/gorilla/mux"

	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/testutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestLightClientHandler_GetLightClientBootstrap(t *testing.T) {
	helpers.ClearCache()
	slot := primitives.Slot(params.BeaconConfig().AltairForkEpoch * primitives.Epoch(params.BeaconConfig().SlotsPerEpoch)).Add(1)

	b := util.NewBeaconBlockCapella()
	b.Block.StateRoot = bytesutil.PadTo([]byte("foo"), 32)
	b.Block.Slot = slot

	signedBlock, err := blocks.NewSignedBeaconBlock(b)

	require.NoError(t, err)
	header, err := signedBlock.Header()
	require.NoError(t, err)

	r, err := b.Block.HashTreeRoot()
	require.NoError(t, err)

	bs, err := util.NewBeaconStateCapella(func(state *ethpb.BeaconStateCapella) error {
		state.BlockRoots[0] = r[:]
		return nil
	})
	require.NoError(t, err)

	require.NoError(t, bs.SetSlot(slot))
	require.NoError(t, bs.SetLatestBlockHeader(header.Header))

	mockBlocker := &testutil.MockBlocker{BlockToReturn: signedBlock}
	mockChainService := &mock.ChainService{Optimistic: true, Slot: &slot}
	s := &Server{
		Stater: &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{
			slot: bs,
		}},
		Blocker:     mockBlocker,
		HeadFetcher: mockChainService,
	}
	muxVars := make(map[string]string)
	muxVars["block_root"] = hexutil.Encode(r[:])
	request := httptest.NewRequest("GET", "http://foo.com/", nil)
	request = mux.SetURLVars(request, muxVars)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetLightClientBootstrap(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	resp := &LightClientBootstrapResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	require.Equal(t, "capella", resp.Version)
	require.Equal(t, hexutil.Encode(header.Header.BodyRoot), resp.Data.Header.BodyRoot)
	require.NotNil(t, resp.Data)
}

func TestLightClientHandler_GetLightClientUpdatesByRange(t *testing.T) {
	helpers.ClearCache()
	ctx := context.Background()
	config := params.BeaconConfig()
	slot := primitives.Slot(config.AltairForkEpoch * primitives.Epoch(config.SlotsPerEpoch)).Add(1)

	attestedState, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = attestedState.SetSlot(slot.Sub(1))
	require.NoError(t, err)

	parent := util.NewBeaconBlockCapella()
	parent.Block.Slot = slot.Sub(1)

	signedParent, err := blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	parentHeader, err := signedParent.Header()
	require.NoError(t, err)
	attestedHeader := parentHeader.Header

	err = attestedState.SetLatestBlockHeader(attestedHeader)
	require.NoError(t, err)
	attestedStateRoot, err := attestedState.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	parent.Block.StateRoot = attestedStateRoot[:]
	signedParent, err = blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	st, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = st.SetSlot(slot)
	require.NoError(t, err)

	parentRoot, err := signedParent.Block().HashTreeRoot()
	require.NoError(t, err)

	block := util.NewBeaconBlockCapella()
	block.Block.Slot = slot
	block.Block.ParentRoot = parentRoot[:]

	for i := uint64(0); i < config.SyncCommitteeSize; i++ {
		block.Block.Body.SyncAggregate.SyncCommitteeBits.SetBitAt(i, true)
	}

	signedBlock, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	h, err := signedBlock.Header()
	require.NoError(t, err)

	err = st.SetLatestBlockHeader(h.Header)
	require.NoError(t, err)
	stateRoot, err := st.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	block.Block.StateRoot = stateRoot[:]
	signedBlock, err = blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	root, err := block.Block.HashTreeRoot()
	require.NoError(t, err)

	mockBlocker := &testutil.MockBlocker{
		RootBlockMap: map[[32]byte]interfaces.ReadOnlySignedBeaconBlock{
			parentRoot: signedParent,
			root:       signedBlock,
		},
		SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
			slot.Sub(1): signedParent,
			slot:        signedBlock,
		},
	}
	mockChainService := &mock.ChainService{Optimistic: true, Slot: &slot, State: st}
	s := &Server{
		Stater: &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{
			slot.Sub(1): attestedState,
			slot:        st,
		}},
		Blocker:     mockBlocker,
		HeadFetcher: mockChainService,
	}
	startPeriod := slot.Div(uint64(config.EpochsPerSyncCommitteePeriod)).Div(uint64(config.SlotsPerEpoch))
	url := fmt.Sprintf("http://foo.com/?count=1&start_period=%d", startPeriod)
	request := httptest.NewRequest("GET", url, nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetLightClientUpdatesByRange(writer, request)

	require.Equal(t, http.StatusOK, writer.Code)
	var resp []LightClientUpdateWithVersion
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), &resp))
	require.Equal(t, 1, len(resp))
	require.Equal(t, "capella", resp[0].Version)
	require.Equal(t, hexutil.Encode(attestedHeader.BodyRoot), resp[0].Data.AttestedHeader.BodyRoot)
	require.NotNil(t, resp)
}

func TestLightClientHandler_GetLightClientUpdatesByRange_TooBigInputCount(t *testing.T) {
	helpers.ClearCache()
	ctx := context.Background()
	config := params.BeaconConfig()
	slot := primitives.Slot(config.AltairForkEpoch * primitives.Epoch(config.SlotsPerEpoch)).Add(1)

	attestedState, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = attestedState.SetSlot(slot.Sub(1))
	require.NoError(t, err)

	parent := util.NewBeaconBlockCapella()
	parent.Block.Slot = slot.Sub(1)

	signedParent, err := blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	parentHeader, err := signedParent.Header()
	require.NoError(t, err)
	attestedHeader := parentHeader.Header

	err = attestedState.SetLatestBlockHeader(attestedHeader)
	require.NoError(t, err)
	attestedStateRoot, err := attestedState.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	parent.Block.StateRoot = attestedStateRoot[:]
	signedParent, err = blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	st, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = st.SetSlot(slot)
	require.NoError(t, err)

	parentRoot, err := signedParent.Block().HashTreeRoot()
	require.NoError(t, err)

	block := util.NewBeaconBlockCapella()
	block.Block.Slot = slot
	block.Block.ParentRoot = parentRoot[:]

	for i := uint64(0); i < config.SyncCommitteeSize; i++ {
		block.Block.Body.SyncAggregate.SyncCommitteeBits.SetBitAt(i, true)
	}

	signedBlock, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	h, err := signedBlock.Header()
	require.NoError(t, err)

	err = st.SetLatestBlockHeader(h.Header)
	require.NoError(t, err)
	stateRoot, err := st.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	block.Block.StateRoot = stateRoot[:]
	signedBlock, err = blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	root, err := block.Block.HashTreeRoot()
	require.NoError(t, err)

	mockBlocker := &testutil.MockBlocker{
		RootBlockMap: map[[32]byte]interfaces.ReadOnlySignedBeaconBlock{
			parentRoot: signedParent,
			root:       signedBlock,
		},
		SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
			slot.Sub(1): signedParent,
			slot:        signedBlock,
		},
	}
	mockChainService := &mock.ChainService{Optimistic: true, Slot: &slot, State: st}
	s := &Server{
		Stater: &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{
			slot.Sub(1): attestedState,
			slot:        st,
		}},
		Blocker:     mockBlocker,
		HeadFetcher: mockChainService,
	}
	startPeriod := slot.Div(uint64(config.EpochsPerSyncCommitteePeriod)).Div(uint64(config.SlotsPerEpoch))
	count := 129 // config.MaxRequestLightClientUpdates is 128
	url := fmt.Sprintf("http://foo.com/?count=%d&start_period=%d", count, startPeriod)
	request := httptest.NewRequest("GET", url, nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetLightClientUpdatesByRange(writer, request)

	require.Equal(t, http.StatusOK, writer.Code)
	var resp []LightClientUpdateWithVersion
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), &resp))
	require.Equal(t, 1, len(resp)) // Even with big count input, the response is still the max available period, which is 1 in test case.
	require.Equal(t, "capella", resp[0].Version)
	require.Equal(t, hexutil.Encode(attestedHeader.BodyRoot), resp[0].Data.AttestedHeader.BodyRoot)
	require.NotNil(t, resp)
}

func TestLightClientHandler_GetLightClientUpdatesByRange_TooEarlyPeriod(t *testing.T) {
	helpers.ClearCache()
	ctx := context.Background()
	config := params.BeaconConfig()
	slot := primitives.Slot(config.AltairForkEpoch * primitives.Epoch(config.SlotsPerEpoch)).Add(1)

	attestedState, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = attestedState.SetSlot(slot.Sub(1))
	require.NoError(t, err)

	parent := util.NewBeaconBlockCapella()
	parent.Block.Slot = slot.Sub(1)

	signedParent, err := blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	parentHeader, err := signedParent.Header()
	require.NoError(t, err)
	attestedHeader := parentHeader.Header

	err = attestedState.SetLatestBlockHeader(attestedHeader)
	require.NoError(t, err)
	attestedStateRoot, err := attestedState.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	parent.Block.StateRoot = attestedStateRoot[:]
	signedParent, err = blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	st, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = st.SetSlot(slot)
	require.NoError(t, err)

	parentRoot, err := signedParent.Block().HashTreeRoot()
	require.NoError(t, err)

	block := util.NewBeaconBlockCapella()
	block.Block.Slot = slot
	block.Block.ParentRoot = parentRoot[:]

	for i := uint64(0); i < config.SyncCommitteeSize; i++ {
		block.Block.Body.SyncAggregate.SyncCommitteeBits.SetBitAt(i, true)
	}

	signedBlock, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	h, err := signedBlock.Header()
	require.NoError(t, err)

	err = st.SetLatestBlockHeader(h.Header)
	require.NoError(t, err)
	stateRoot, err := st.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	block.Block.StateRoot = stateRoot[:]
	signedBlock, err = blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	root, err := block.Block.HashTreeRoot()
	require.NoError(t, err)

	mockBlocker := &testutil.MockBlocker{
		RootBlockMap: map[[32]byte]interfaces.ReadOnlySignedBeaconBlock{
			parentRoot: signedParent,
			root:       signedBlock,
		},
		SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
			slot.Sub(1): signedParent,
			slot:        signedBlock,
		},
	}
	mockChainService := &mock.ChainService{Optimistic: true, Slot: &slot, State: st}
	s := &Server{
		Stater: &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{
			slot.Sub(1): attestedState,
			slot:        st,
		}},
		Blocker:     mockBlocker,
		HeadFetcher: mockChainService,
	}
	startPeriod := 1 // very early period before Altair fork
	count := 1
	url := fmt.Sprintf("http://foo.com/?count=%d&start_period=%d", count, startPeriod)
	request := httptest.NewRequest("GET", url, nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetLightClientUpdatesByRange(writer, request)

	require.Equal(t, http.StatusOK, writer.Code)
	var resp []LightClientUpdateWithVersion
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), &resp))
	require.Equal(t, 1, len(resp))
	require.Equal(t, "capella", resp[0].Version)
	require.Equal(t, hexutil.Encode(attestedHeader.BodyRoot), resp[0].Data.AttestedHeader.BodyRoot)
	require.NotNil(t, resp)
}

func TestLightClientHandler_GetLightClientUpdatesByRange_TooBigCount(t *testing.T) {
	helpers.ClearCache()
	ctx := context.Background()
	config := params.BeaconConfig()
	slot := primitives.Slot(config.AltairForkEpoch * primitives.Epoch(config.SlotsPerEpoch)).Add(1)

	attestedState, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = attestedState.SetSlot(slot.Sub(1))
	require.NoError(t, err)

	parent := util.NewBeaconBlockCapella()
	parent.Block.Slot = slot.Sub(1)

	signedParent, err := blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	parentHeader, err := signedParent.Header()
	require.NoError(t, err)
	attestedHeader := parentHeader.Header

	err = attestedState.SetLatestBlockHeader(attestedHeader)
	require.NoError(t, err)
	attestedStateRoot, err := attestedState.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	parent.Block.StateRoot = attestedStateRoot[:]
	signedParent, err = blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	st, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = st.SetSlot(slot)
	require.NoError(t, err)

	parentRoot, err := signedParent.Block().HashTreeRoot()
	require.NoError(t, err)

	block := util.NewBeaconBlockCapella()
	block.Block.Slot = slot
	block.Block.ParentRoot = parentRoot[:]

	for i := uint64(0); i < config.SyncCommitteeSize; i++ {
		block.Block.Body.SyncAggregate.SyncCommitteeBits.SetBitAt(i, true)
	}

	signedBlock, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	h, err := signedBlock.Header()
	require.NoError(t, err)

	err = st.SetLatestBlockHeader(h.Header)
	require.NoError(t, err)
	stateRoot, err := st.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	block.Block.StateRoot = stateRoot[:]
	signedBlock, err = blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	root, err := block.Block.HashTreeRoot()
	require.NoError(t, err)

	mockBlocker := &testutil.MockBlocker{
		RootBlockMap: map[[32]byte]interfaces.ReadOnlySignedBeaconBlock{
			parentRoot: signedParent,
			root:       signedBlock,
		},
		SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
			slot.Sub(1): signedParent,
			slot:        signedBlock,
		},
	}
	mockChainService := &mock.ChainService{Optimistic: true, Slot: &slot, State: st}
	s := &Server{
		Stater: &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{
			slot.Sub(1): attestedState,
			slot:        st,
		}},
		Blocker:     mockBlocker,
		HeadFetcher: mockChainService,
	}
	startPeriod := 1 // very early period before Altair fork
	count := 10      // This is big count as we only have one period in test case.
	url := fmt.Sprintf("http://foo.com/?count=%d&start_period=%d", count, startPeriod)
	request := httptest.NewRequest("GET", url, nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetLightClientUpdatesByRange(writer, request)

	require.Equal(t, http.StatusOK, writer.Code)
	var resp []LightClientUpdateWithVersion
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), &resp))
	require.Equal(t, 1, len(resp))
	require.Equal(t, "capella", resp[0].Version)
	require.Equal(t, hexutil.Encode(attestedHeader.BodyRoot), resp[0].Data.AttestedHeader.BodyRoot)
	require.NotNil(t, resp)
}

func TestLightClientHandler_GetLightClientUpdatesByRange_BeforeAltair(t *testing.T) {
	helpers.ClearCache()
	ctx := context.Background()
	config := params.BeaconConfig()
	slot := primitives.Slot(config.AltairForkEpoch * primitives.Epoch(config.SlotsPerEpoch)).Sub(1)

	attestedState, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = attestedState.SetSlot(slot.Sub(1))
	require.NoError(t, err)

	parent := util.NewBeaconBlockCapella()
	parent.Block.Slot = slot.Sub(1)

	signedParent, err := blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	parentHeader, err := signedParent.Header()
	require.NoError(t, err)
	attestedHeader := parentHeader.Header

	err = attestedState.SetLatestBlockHeader(attestedHeader)
	require.NoError(t, err)
	attestedStateRoot, err := attestedState.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	parent.Block.StateRoot = attestedStateRoot[:]
	signedParent, err = blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	st, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = st.SetSlot(slot)
	require.NoError(t, err)

	parentRoot, err := signedParent.Block().HashTreeRoot()
	require.NoError(t, err)

	block := util.NewBeaconBlockCapella()
	block.Block.Slot = slot
	block.Block.ParentRoot = parentRoot[:]

	for i := uint64(0); i < config.SyncCommitteeSize; i++ {
		block.Block.Body.SyncAggregate.SyncCommitteeBits.SetBitAt(i, true)
	}

	signedBlock, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	h, err := signedBlock.Header()
	require.NoError(t, err)

	err = st.SetLatestBlockHeader(h.Header)
	require.NoError(t, err)
	stateRoot, err := st.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	block.Block.StateRoot = stateRoot[:]
	signedBlock, err = blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	root, err := block.Block.HashTreeRoot()
	require.NoError(t, err)

	mockBlocker := &testutil.MockBlocker{
		RootBlockMap: map[[32]byte]interfaces.ReadOnlySignedBeaconBlock{
			parentRoot: signedParent,
			root:       signedBlock,
		},
		SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
			slot.Sub(1): signedParent,
			slot:        signedBlock,
		},
	}
	mockChainService := &mock.ChainService{Optimistic: true, Slot: &slot, State: st}
	s := &Server{
		Stater: &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{
			slot.Sub(1): attestedState,
			slot:        st,
		}},
		Blocker:     mockBlocker,
		HeadFetcher: mockChainService,
	}
	startPeriod := slot.Div(uint64(config.EpochsPerSyncCommitteePeriod)).Div(uint64(config.SlotsPerEpoch))
	count := 1
	url := fmt.Sprintf("http://foo.com/?count=%d&start_period=%d", count, startPeriod)
	request := httptest.NewRequest("GET", url, nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetLightClientUpdatesByRange(writer, request)

	require.Equal(t, http.StatusNotFound, writer.Code)
}

func TestLightClientHandler_GetLightClientFinalityUpdate(t *testing.T) {
	helpers.ClearCache()
	ctx := context.Background()
	config := params.BeaconConfig()
	slot := primitives.Slot(config.AltairForkEpoch * primitives.Epoch(config.SlotsPerEpoch)).Add(1)

	attestedState, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = attestedState.SetSlot(slot.Sub(1))
	require.NoError(t, err)

	require.NoError(t, attestedState.SetFinalizedCheckpoint(&ethpb.Checkpoint{
		Epoch: config.AltairForkEpoch - 10,
		Root:  make([]byte, 32),
	}))

	parent := util.NewBeaconBlockCapella()
	parent.Block.Slot = slot.Sub(1)

	signedParent, err := blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	parentHeader, err := signedParent.Header()
	require.NoError(t, err)
	attestedHeader := parentHeader.Header

	err = attestedState.SetLatestBlockHeader(attestedHeader)
	require.NoError(t, err)
	attestedStateRoot, err := attestedState.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	parent.Block.StateRoot = attestedStateRoot[:]
	signedParent, err = blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	st, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = st.SetSlot(slot)
	require.NoError(t, err)

	parentRoot, err := signedParent.Block().HashTreeRoot()
	require.NoError(t, err)

	block := util.NewBeaconBlockCapella()
	block.Block.Slot = slot
	block.Block.ParentRoot = parentRoot[:]

	for i := uint64(0); i < config.SyncCommitteeSize; i++ {
		block.Block.Body.SyncAggregate.SyncCommitteeBits.SetBitAt(i, true)
	}

	signedBlock, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	h, err := signedBlock.Header()
	require.NoError(t, err)

	err = st.SetLatestBlockHeader(h.Header)
	require.NoError(t, err)
	stateRoot, err := st.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	block.Block.StateRoot = stateRoot[:]
	signedBlock, err = blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	root, err := block.Block.HashTreeRoot()
	require.NoError(t, err)

	mockBlocker := &testutil.MockBlocker{
		RootBlockMap: map[[32]byte]interfaces.ReadOnlySignedBeaconBlock{
			parentRoot: signedParent,
			root:       signedBlock,
		},
		SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
			slot.Sub(1): signedParent,
			slot:        signedBlock,
		},
	}
	mockChainService := &mock.ChainService{Optimistic: true, Slot: &slot, State: st, FinalizedRoots: map[[32]byte]bool{
		root: true,
	}}
	s := &Server{
		Stater: &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{
			slot.Sub(1): attestedState,
			slot:        st,
		}},
		Blocker:     mockBlocker,
		HeadFetcher: mockChainService,
	}
	request := httptest.NewRequest("GET", "http://foo.com", nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetLightClientFinalityUpdate(writer, request)

	require.Equal(t, http.StatusOK, writer.Code)
	resp := &LightClientUpdateWithVersion{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	require.Equal(t, "capella", resp.Version)
	require.Equal(t, hexutil.Encode(attestedHeader.BodyRoot), resp.Data.AttestedHeader.BodyRoot)
	require.NotNil(t, resp.Data)
}

func TestLightClientHandler_GetLightClientOptimisticUpdate(t *testing.T) {
	helpers.ClearCache()
	ctx := context.Background()
	config := params.BeaconConfig()
	slot := primitives.Slot(config.AltairForkEpoch * primitives.Epoch(config.SlotsPerEpoch)).Add(1)

	attestedState, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = attestedState.SetSlot(slot.Sub(1))
	require.NoError(t, err)

	require.NoError(t, attestedState.SetFinalizedCheckpoint(&ethpb.Checkpoint{
		Epoch: config.AltairForkEpoch - 10,
		Root:  make([]byte, 32),
	}))

	parent := util.NewBeaconBlockCapella()
	parent.Block.Slot = slot.Sub(1)

	signedParent, err := blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	parentHeader, err := signedParent.Header()
	require.NoError(t, err)
	attestedHeader := parentHeader.Header

	err = attestedState.SetLatestBlockHeader(attestedHeader)
	require.NoError(t, err)
	attestedStateRoot, err := attestedState.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	parent.Block.StateRoot = attestedStateRoot[:]
	signedParent, err = blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	st, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = st.SetSlot(slot)
	require.NoError(t, err)

	parentRoot, err := signedParent.Block().HashTreeRoot()
	require.NoError(t, err)

	block := util.NewBeaconBlockCapella()
	block.Block.Slot = slot
	block.Block.ParentRoot = parentRoot[:]

	for i := uint64(0); i < config.SyncCommitteeSize; i++ {
		block.Block.Body.SyncAggregate.SyncCommitteeBits.SetBitAt(i, true)
	}

	signedBlock, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	h, err := signedBlock.Header()
	require.NoError(t, err)

	err = st.SetLatestBlockHeader(h.Header)
	require.NoError(t, err)
	stateRoot, err := st.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	block.Block.StateRoot = stateRoot[:]
	signedBlock, err = blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	root, err := block.Block.HashTreeRoot()
	require.NoError(t, err)

	mockBlocker := &testutil.MockBlocker{
		RootBlockMap: map[[32]byte]interfaces.ReadOnlySignedBeaconBlock{
			parentRoot: signedParent,
			root:       signedBlock,
		},
		SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
			slot.Sub(1): signedParent,
			slot:        signedBlock,
		},
	}
	mockChainService := &mock.ChainService{Optimistic: true, Slot: &slot, State: st, FinalizedRoots: map[[32]byte]bool{
		root: true,
	}}
	s := &Server{
		Stater: &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{
			slot.Sub(1): attestedState,
			slot:        st,
		}},
		Blocker:     mockBlocker,
		HeadFetcher: mockChainService,
	}
	request := httptest.NewRequest("GET", "http://foo.com", nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetLightClientOptimisticUpdate(writer, request)

	require.Equal(t, http.StatusOK, writer.Code)
	resp := &LightClientUpdateWithVersion{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	require.Equal(t, "capella", resp.Version)
	require.Equal(t, hexutil.Encode(attestedHeader.BodyRoot), resp.Data.AttestedHeader.BodyRoot)
	require.NotNil(t, resp.Data)
}

func TestLightClientHandler_GetLightClientEventBlock(t *testing.T) {
	helpers.ClearCache()
	ctx := context.Background()
	config := params.BeaconConfig()
	slot := primitives.Slot(config.AltairForkEpoch * primitives.Epoch(config.SlotsPerEpoch)).Add(1)

	attestedState, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = attestedState.SetSlot(slot.Sub(1))
	require.NoError(t, err)

	require.NoError(t, attestedState.SetFinalizedCheckpoint(&ethpb.Checkpoint{
		Epoch: config.AltairForkEpoch - 10,
		Root:  make([]byte, 32),
	}))

	parent := util.NewBeaconBlockCapella()
	parent.Block.Slot = slot.Sub(1)

	signedParent, err := blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	parentHeader, err := signedParent.Header()
	require.NoError(t, err)
	attestedHeader := parentHeader.Header

	err = attestedState.SetLatestBlockHeader(attestedHeader)
	require.NoError(t, err)
	attestedStateRoot, err := attestedState.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	parent.Block.StateRoot = attestedStateRoot[:]
	signedParent, err = blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	st, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = st.SetSlot(slot)
	require.NoError(t, err)

	parentRoot, err := signedParent.Block().HashTreeRoot()
	require.NoError(t, err)

	block := util.NewBeaconBlockCapella()
	block.Block.Slot = slot
	block.Block.ParentRoot = parentRoot[:]

	for i := uint64(0); i < config.SyncCommitteeSize; i++ {
		block.Block.Body.SyncAggregate.SyncCommitteeBits.SetBitAt(i, true)
	}

	signedBlock, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	h, err := signedBlock.Header()
	require.NoError(t, err)

	err = st.SetLatestBlockHeader(h.Header)
	require.NoError(t, err)
	stateRoot, err := st.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	block.Block.StateRoot = stateRoot[:]
	signedBlock, err = blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	root, err := block.Block.HashTreeRoot()
	require.NoError(t, err)

	mockBlocker := &testutil.MockBlocker{
		RootBlockMap: map[[32]byte]interfaces.ReadOnlySignedBeaconBlock{
			parentRoot: signedParent,
			root:       signedBlock,
		},
		SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
			slot.Sub(1): signedParent,
			slot:        signedBlock,
		},
	}
	mockChainService := &mock.ChainService{Optimistic: true, Slot: &slot, State: st, FinalizedRoots: map[[32]byte]bool{
		root: true,
	}}
	s := &Server{
		Stater: &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{
			slot.Sub(1): attestedState,
			slot:        st,
		}},
		Blocker:     mockBlocker,
		HeadFetcher: mockChainService,
	}

	minSignaturesRequired := uint64(100)
	eventBlock, err := s.getLightClientEventBlock(ctx, minSignaturesRequired)

	require.NoError(t, err)
	require.NotNil(t, eventBlock)
	require.Equal(t, slot, eventBlock.Block().Slot())
	syncAggregate, err := eventBlock.Block().Body().SyncAggregate()
	require.NoError(t, err)
	require.Equal(t, true, syncAggregate.SyncCommitteeBits.Count() >= minSignaturesRequired)
}

func TestLightClientHandler_GetLightClientEventBlock_NeedFetchParent(t *testing.T) {
	helpers.ClearCache()
	ctx := context.Background()
	config := params.BeaconConfig()
	slot := primitives.Slot(config.AltairForkEpoch * primitives.Epoch(config.SlotsPerEpoch)).Add(1)

	attestedState, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = attestedState.SetSlot(slot.Sub(1))
	require.NoError(t, err)

	require.NoError(t, attestedState.SetFinalizedCheckpoint(&ethpb.Checkpoint{
		Epoch: config.AltairForkEpoch - 10,
		Root:  make([]byte, 32),
	}))

	parent := util.NewBeaconBlockCapella()
	parent.Block.Slot = slot.Sub(1)
	for i := uint64(0); i < config.SyncCommitteeSize; i++ {
		parent.Block.Body.SyncAggregate.SyncCommitteeBits.SetBitAt(i, true)
	}

	signedParent, err := blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	parentHeader, err := signedParent.Header()
	require.NoError(t, err)
	attestedHeader := parentHeader.Header

	err = attestedState.SetLatestBlockHeader(attestedHeader)
	require.NoError(t, err)
	attestedStateRoot, err := attestedState.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	parent.Block.StateRoot = attestedStateRoot[:]
	signedParent, err = blocks.NewSignedBeaconBlock(parent)
	require.NoError(t, err)

	st, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	err = st.SetSlot(slot)
	require.NoError(t, err)

	parentRoot, err := signedParent.Block().HashTreeRoot()
	require.NoError(t, err)

	block := util.NewBeaconBlockCapella()
	block.Block.Slot = slot
	block.Block.ParentRoot = parentRoot[:]

	for i := uint64(0); i < 10; i++ {
		block.Block.Body.SyncAggregate.SyncCommitteeBits.SetBitAt(i, true)
	}

	signedBlock, err := blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	h, err := signedBlock.Header()
	require.NoError(t, err)

	err = st.SetLatestBlockHeader(h.Header)
	require.NoError(t, err)
	stateRoot, err := st.HashTreeRoot(ctx)
	require.NoError(t, err)

	// get a new signed block so the root is updated with the new state root
	block.Block.StateRoot = stateRoot[:]
	signedBlock, err = blocks.NewSignedBeaconBlock(block)
	require.NoError(t, err)

	root, err := block.Block.HashTreeRoot()
	require.NoError(t, err)

	mockBlocker := &testutil.MockBlocker{
		RootBlockMap: map[[32]byte]interfaces.ReadOnlySignedBeaconBlock{
			parentRoot: signedParent,
			root:       signedBlock,
		},
		SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
			slot.Sub(1): signedParent,
			slot:        signedBlock,
		},
	}
	mockChainService := &mock.ChainService{Optimistic: true, Slot: &slot, State: st, FinalizedRoots: map[[32]byte]bool{
		root: true,
	}}
	s := &Server{
		Stater: &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{
			slot.Sub(1): attestedState,
			slot:        st,
		}},
		Blocker:     mockBlocker,
		HeadFetcher: mockChainService,
	}

	minSignaturesRequired := uint64(100)
	eventBlock, err := s.getLightClientEventBlock(ctx, minSignaturesRequired)

	require.NoError(t, err)
	require.NotNil(t, eventBlock)
	syncAggregate, err := eventBlock.Block().Body().SyncAggregate()
	require.NoError(t, err)
	require.Equal(t, true, syncAggregate.SyncCommitteeBits.Count() >= minSignaturesRequired)
	require.Equal(t, slot-1, eventBlock.Block().Slot())
}
