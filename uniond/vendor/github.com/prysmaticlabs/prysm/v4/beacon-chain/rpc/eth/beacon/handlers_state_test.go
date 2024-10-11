package beacon

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strconv"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/gorilla/mux"
	chainMock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	dbTest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/testutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	ethpbalpha "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestGetStateRoot(t *testing.T) {
	ctx := context.Background()
	fakeState, err := util.NewBeaconState()
	require.NoError(t, err)
	stateRoot, err := fakeState.HashTreeRoot(ctx)
	require.NoError(t, err)
	db := dbTest.SetupDB(t)
	parentRoot := [32]byte{'a'}
	blk := util.NewBeaconBlock()
	blk.Block.ParentRoot = parentRoot[:]
	root, err := blk.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, ctx, db, blk)
	require.NoError(t, db.SaveGenesisBlockRoot(ctx, root))

	chainService := &chainMock.ChainService{}
	s := &Server{
		Stater: &testutil.MockStater{
			BeaconStateRoot: stateRoot[:],
			BeaconState:     fakeState,
		},
		HeadFetcher:           chainService,
		OptimisticModeFetcher: chainService,
		FinalizationFetcher:   chainService,
		BeaconDB:              db,
	}

	request := httptest.NewRequest(http.MethodGet, "http://example.com//eth/v1/beacon/states/{state_id}/root", nil)
	request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetStateRoot(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	resp := &GetStateRootResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	assert.Equal(t, hexutil.Encode(stateRoot[:]), resp.Data.Root)

	t.Run("execution optimistic", func(t *testing.T) {
		chainService := &chainMock.ChainService{Optimistic: true}
		s := &Server{
			Stater: &testutil.MockStater{
				BeaconStateRoot: stateRoot[:],
				BeaconState:     fakeState,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
			BeaconDB:              db,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com//eth/v1/beacon/states/{state_id}/root", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetStateRoot(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetStateRootResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.DeepEqual(t, true, resp.ExecutionOptimistic)
	})

	t.Run("finalized", func(t *testing.T) {
		headerRoot, err := fakeState.LatestBlockHeader().HashTreeRoot()
		require.NoError(t, err)
		chainService := &chainMock.ChainService{
			FinalizedRoots: map[[32]byte]bool{
				headerRoot: true,
			},
		}
		s := &Server{
			Stater: &testutil.MockStater{
				BeaconStateRoot: stateRoot[:],
				BeaconState:     fakeState,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
			BeaconDB:              db,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com//eth/v1/beacon/states/{state_id}/root", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetStateRoot(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetStateRootResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.DeepEqual(t, true, resp.Finalized)
	})
}

func TestGetRandao(t *testing.T) {
	mixCurrent := bytesutil.ToBytes32([]byte("current"))
	mixOld := bytesutil.ToBytes32([]byte("old"))
	epochCurrent := primitives.Epoch(100000)
	epochOld := 100000 - params.BeaconConfig().EpochsPerHistoricalVector + 1

	ctx := context.Background()
	st, err := util.NewBeaconState()
	require.NoError(t, err)
	// Set slot to epoch 100000
	require.NoError(t, st.SetSlot(params.BeaconConfig().SlotsPerEpoch*100000))
	require.NoError(t, st.UpdateRandaoMixesAtIndex(uint64(epochCurrent%params.BeaconConfig().EpochsPerHistoricalVector), mixCurrent))
	require.NoError(t, st.UpdateRandaoMixesAtIndex(uint64(epochOld%params.BeaconConfig().EpochsPerHistoricalVector), mixOld))

	headEpoch := primitives.Epoch(1)
	headSt, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, headSt.SetSlot(params.BeaconConfig().SlotsPerEpoch))
	headRandao := bytesutil.ToBytes32([]byte("head"))
	require.NoError(t, headSt.UpdateRandaoMixesAtIndex(uint64(headEpoch), headRandao))

	db := dbTest.SetupDB(t)
	chainService := &chainMock.ChainService{}
	s := &Server{
		Stater: &testutil.MockStater{
			BeaconState: st,
		},
		HeadFetcher:           chainService,
		OptimisticModeFetcher: chainService,
		FinalizationFetcher:   chainService,
		BeaconDB:              db,
	}

	t.Run("no epoch requested", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, "http://example.com//eth/v1/beacon/states/{state_id}/randao", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetRandao(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetRandaoResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, hexutil.Encode(mixCurrent[:]), resp.Data.Randao)
	})
	t.Run("current epoch requested", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://example.com//eth/v1/beacon/states/{state_id}/randao?epoch=%d", epochCurrent), nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetRandao(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetRandaoResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, hexutil.Encode(mixCurrent[:]), resp.Data.Randao)
	})
	t.Run("old epoch requested", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://example.com//eth/v1/beacon/states/{state_id}/randao?epoch=%d", epochOld), nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetRandao(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetRandaoResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, hexutil.Encode(mixOld[:]), resp.Data.Randao)
	})
	t.Run("head state below `EpochsPerHistoricalVector`", func(t *testing.T) {
		s.Stater = &testutil.MockStater{
			BeaconState: headSt,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com//eth/v1/beacon/states/{state_id}/randao", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetRandao(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetRandaoResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, hexutil.Encode(headRandao[:]), resp.Data.Randao)
	})
	t.Run("epoch too old", func(t *testing.T) {
		epochTooOld := primitives.Epoch(100000 - st.RandaoMixesLength())
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://example.com//eth/v1/beacon/states/{state_id}/randao?epoch=%d", epochTooOld), nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetRandao(writer, request)
		require.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		require.StringContains(t, "Epoch is out of range for the randao mixes of the state", e.Message)
	})
	t.Run("epoch in the future", func(t *testing.T) {
		futureEpoch := primitives.Epoch(100000 + 1)
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://example.com//eth/v1/beacon/states/{state_id}/randao?epoch=%d", futureEpoch), nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetRandao(writer, request)
		require.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		require.StringContains(t, "Epoch is out of range for the randao mixes of the state", e.Message)
	})
	t.Run("execution optimistic", func(t *testing.T) {
		parentRoot := [32]byte{'a'}
		blk := util.NewBeaconBlock()
		blk.Block.ParentRoot = parentRoot[:]
		root, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		util.SaveBlock(t, ctx, db, blk)
		require.NoError(t, db.SaveGenesisBlockRoot(ctx, root))

		chainService := &chainMock.ChainService{Optimistic: true}
		s := &Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
			BeaconDB:              db,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com//eth/v1/beacon/states/{state_id}/randao", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetRandao(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetRandaoResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.DeepEqual(t, true, resp.ExecutionOptimistic)
	})
	t.Run("finalized", func(t *testing.T) {
		parentRoot := [32]byte{'a'}
		blk := util.NewBeaconBlock()
		blk.Block.ParentRoot = parentRoot[:]
		root, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		util.SaveBlock(t, ctx, db, blk)
		require.NoError(t, db.SaveGenesisBlockRoot(ctx, root))

		headerRoot, err := headSt.LatestBlockHeader().HashTreeRoot()
		require.NoError(t, err)
		chainService := &chainMock.ChainService{
			FinalizedRoots: map[[32]byte]bool{
				headerRoot: true,
			},
		}
		s := &Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
			BeaconDB:              db,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com//eth/v1/beacon/states/{state_id}/randao", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetRandao(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetRandaoResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.DeepEqual(t, true, resp.Finalized)
	})
}

func Test_currentCommitteeIndicesFromState(t *testing.T) {
	st, _ := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().SyncCommitteeSize)
	vals := st.Validators()
	wantedCommittee := make([][]byte, params.BeaconConfig().SyncCommitteeSize)
	wantedIndices := make([]string, len(wantedCommittee))
	for i := 0; i < len(wantedCommittee); i++ {
		wantedIndices[i] = strconv.FormatUint(uint64(i), 10)
		wantedCommittee[i] = vals[i].PublicKey
	}
	require.NoError(t, st.SetCurrentSyncCommittee(&ethpbalpha.SyncCommittee{
		Pubkeys:         wantedCommittee,
		AggregatePubkey: bytesutil.PadTo([]byte{}, params.BeaconConfig().BLSPubkeyLength),
	}))

	t.Run("OK", func(t *testing.T) {
		indices, committee, err := currentCommitteeIndicesFromState(st)
		require.NoError(t, err)
		require.DeepEqual(t, wantedIndices, indices)
		require.DeepEqual(t, wantedCommittee, committee.Pubkeys)
	})
	t.Run("validator in committee not found in state", func(t *testing.T) {
		wantedCommittee[0] = bytesutil.PadTo([]byte("fakepubkey"), 48)
		require.NoError(t, st.SetCurrentSyncCommittee(&ethpbalpha.SyncCommittee{
			Pubkeys:         wantedCommittee,
			AggregatePubkey: bytesutil.PadTo([]byte{}, params.BeaconConfig().BLSPubkeyLength),
		}))
		_, _, err := currentCommitteeIndicesFromState(st)
		require.ErrorContains(t, "index not found for pubkey", err)
	})
}

func Test_nextCommitteeIndicesFromState(t *testing.T) {
	st, _ := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().SyncCommitteeSize)
	vals := st.Validators()
	wantedCommittee := make([][]byte, params.BeaconConfig().SyncCommitteeSize)
	wantedIndices := make([]string, len(wantedCommittee))
	for i := 0; i < len(wantedCommittee); i++ {
		wantedIndices[i] = strconv.FormatUint(uint64(i), 10)
		wantedCommittee[i] = vals[i].PublicKey
	}
	require.NoError(t, st.SetNextSyncCommittee(&ethpbalpha.SyncCommittee{
		Pubkeys:         wantedCommittee,
		AggregatePubkey: bytesutil.PadTo([]byte{}, params.BeaconConfig().BLSPubkeyLength),
	}))

	t.Run("OK", func(t *testing.T) {
		indices, committee, err := nextCommitteeIndicesFromState(st)
		require.NoError(t, err)
		require.DeepEqual(t, wantedIndices, indices)
		require.DeepEqual(t, wantedCommittee, committee.Pubkeys)
	})
	t.Run("validator in committee not found in state", func(t *testing.T) {
		wantedCommittee[0] = bytesutil.PadTo([]byte("fakepubkey"), 48)
		require.NoError(t, st.SetNextSyncCommittee(&ethpbalpha.SyncCommittee{
			Pubkeys:         wantedCommittee,
			AggregatePubkey: bytesutil.PadTo([]byte{}, params.BeaconConfig().BLSPubkeyLength),
		}))
		_, _, err := nextCommitteeIndicesFromState(st)
		require.ErrorContains(t, "index not found for pubkey", err)
	})
}

func Test_extractSyncSubcommittees(t *testing.T) {
	st, _ := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().SyncCommitteeSize)
	vals := st.Validators()
	syncCommittee := make([][]byte, params.BeaconConfig().SyncCommitteeSize)
	for i := 0; i < len(syncCommittee); i++ {
		syncCommittee[i] = vals[i].PublicKey
	}
	require.NoError(t, st.SetCurrentSyncCommittee(&ethpbalpha.SyncCommittee{
		Pubkeys:         syncCommittee,
		AggregatePubkey: bytesutil.PadTo([]byte{}, params.BeaconConfig().BLSPubkeyLength),
	}))

	commSize := params.BeaconConfig().SyncCommitteeSize
	subCommSize := params.BeaconConfig().SyncCommitteeSize / params.BeaconConfig().SyncCommitteeSubnetCount
	wantedSubcommitteeValidators := make([][]string, 0)

	for i := uint64(0); i < commSize; i += subCommSize {
		sub := make([]string, 0)
		start := i
		end := i + subCommSize
		if end > commSize {
			end = commSize
		}
		for j := start; j < end; j++ {
			sub = append(sub, strconv.FormatUint(j, 10))
		}
		wantedSubcommitteeValidators = append(wantedSubcommitteeValidators, sub)
	}

	t.Run("OK", func(t *testing.T) {
		committee, err := st.CurrentSyncCommittee()
		require.NoError(t, err)
		subcommittee, err := extractSyncSubcommittees(st, committee)
		require.NoError(t, err)
		for i, got := range subcommittee {
			want := wantedSubcommitteeValidators[i]
			require.DeepEqual(t, want, got)
		}
	})
	t.Run("validator in subcommittee not found in state", func(t *testing.T) {
		syncCommittee[0] = bytesutil.PadTo([]byte("fakepubkey"), 48)
		require.NoError(t, st.SetCurrentSyncCommittee(&ethpbalpha.SyncCommittee{
			Pubkeys:         syncCommittee,
			AggregatePubkey: bytesutil.PadTo([]byte{}, params.BeaconConfig().BLSPubkeyLength),
		}))
		committee, err := st.CurrentSyncCommittee()
		require.NoError(t, err)
		_, err = extractSyncSubcommittees(st, committee)
		require.ErrorContains(t, "index not found for pubkey", err)
	})
}

func TestGetSyncCommittees(t *testing.T) {
	ctx := context.Background()
	st, _ := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().SyncCommitteeSize)
	syncCommittee := make([][]byte, params.BeaconConfig().SyncCommitteeSize)
	vals := st.Validators()
	for i := 0; i < len(syncCommittee); i++ {
		syncCommittee[i] = vals[i].PublicKey
	}
	require.NoError(t, st.SetCurrentSyncCommittee(&ethpbalpha.SyncCommittee{
		Pubkeys:         syncCommittee,
		AggregatePubkey: bytesutil.PadTo([]byte{}, params.BeaconConfig().BLSPubkeyLength),
	}))
	stRoot, err := st.HashTreeRoot(ctx)
	require.NoError(t, err)
	db := dbTest.SetupDB(t)

	stSlot := st.Slot()
	chainService := &chainMock.ChainService{Slot: &stSlot}
	s := &Server{
		GenesisTimeFetcher: &testutil.MockGenesisTimeFetcher{
			Genesis: time.Now(),
		},
		Stater: &testutil.MockStater{
			BeaconState: st,
		},
		HeadFetcher:           chainService,
		OptimisticModeFetcher: chainService,
		FinalizationFetcher:   chainService,
		BeaconDB:              db,
		ChainInfoFetcher:      chainService,
	}

	request := httptest.NewRequest(http.MethodGet, "http://example.com//eth/v1/beacon/states/{state_id}/sync_committees", nil)
	request = mux.SetURLVars(request, map[string]string{"state_id": hexutil.Encode(stRoot[:])})
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetSyncCommittees(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	resp := &GetSyncCommitteeResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	committeeVals := resp.Data.Validators
	require.Equal(t, params.BeaconConfig().SyncCommitteeSize, uint64(len(committeeVals)))
	for i := uint64(0); i < params.BeaconConfig().SyncCommitteeSize; i++ {
		assert.Equal(t, strconv.FormatUint(i, 10), committeeVals[i])
	}
	require.Equal(t, params.BeaconConfig().SyncCommitteeSubnetCount, uint64(len(resp.Data.ValidatorAggregates)))
	for i := uint64(0); i < params.BeaconConfig().SyncCommitteeSubnetCount; i++ {
		vStartIndex := primitives.ValidatorIndex(params.BeaconConfig().SyncCommitteeSize / params.BeaconConfig().SyncCommitteeSubnetCount * i)
		vEndIndex := primitives.ValidatorIndex(params.BeaconConfig().SyncCommitteeSize/params.BeaconConfig().SyncCommitteeSubnetCount*(i+1) - 1)
		j := 0
		for vIndex := vStartIndex; vIndex <= vEndIndex; vIndex++ {
			assert.Equal(t, strconv.FormatUint(uint64(vIndex), 10), resp.Data.ValidatorAggregates[i][j])
			j++
		}
	}

	t.Run("execution optimistic", func(t *testing.T) {
		parentRoot := [32]byte{'a'}
		blk := util.NewBeaconBlock()
		blk.Block.ParentRoot = parentRoot[:]
		root, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		util.SaveBlock(t, ctx, db, blk)
		require.NoError(t, db.SaveGenesisBlockRoot(ctx, root))

		stSlot := st.Slot()
		chainService := &chainMock.ChainService{Optimistic: true, Slot: &stSlot}
		s := &Server{
			GenesisTimeFetcher: &testutil.MockGenesisTimeFetcher{
				Genesis: time.Now(),
			},
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
			BeaconDB:              db,
			ChainInfoFetcher:      chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com//eth/v1/beacon/states/{state_id}/sync_committees", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": hexutil.Encode(stRoot[:])})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommittees(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetSyncCommitteeResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, true, resp.ExecutionOptimistic)
	})

	t.Run("finalized", func(t *testing.T) {
		parentRoot := [32]byte{'a'}
		blk := util.NewBeaconBlock()
		blk.Block.ParentRoot = parentRoot[:]
		root, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		util.SaveBlock(t, ctx, db, blk)
		require.NoError(t, db.SaveGenesisBlockRoot(ctx, root))

		headerRoot, err := st.LatestBlockHeader().HashTreeRoot()
		require.NoError(t, err)
		stSlot := st.Slot()
		chainService := &chainMock.ChainService{
			FinalizedRoots: map[[32]byte]bool{
				headerRoot: true,
			},
			Slot: &stSlot,
		}
		s := &Server{
			GenesisTimeFetcher: &testutil.MockGenesisTimeFetcher{
				Genesis: time.Now(),
			},
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
			BeaconDB:              db,
			ChainInfoFetcher:      chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com//eth/v1/beacon/states/{state_id}/sync_committees", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": hexutil.Encode(stRoot[:])})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommittees(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetSyncCommitteeResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, true, resp.Finalized)
	})
}

type futureSyncMockFetcher struct {
	BeaconState     state.BeaconState
	BeaconStateRoot []byte
}

func (m *futureSyncMockFetcher) State(_ context.Context, stateId []byte) (state.BeaconState, error) {
	expectedRequest := []byte(strconv.FormatUint(uint64(0), 10))
	res := bytes.Compare(stateId, expectedRequest)
	if res != 0 {
		return nil, fmt.Errorf(
			"requested wrong epoch for next sync committee (expected %#x, received %#x)",
			expectedRequest,
			stateId,
		)
	}
	return m.BeaconState, nil
}
func (m *futureSyncMockFetcher) StateRoot(context.Context, []byte) ([]byte, error) {
	return m.BeaconStateRoot, nil
}

func (m *futureSyncMockFetcher) StateBySlot(context.Context, primitives.Slot) (state.BeaconState, error) {
	return m.BeaconState, nil
}

func TestGetSyncCommittees_Future(t *testing.T) {
	st, _ := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().SyncCommitteeSize)
	syncCommittee := make([][]byte, params.BeaconConfig().SyncCommitteeSize)
	vals := st.Validators()
	for i := 0; i < len(syncCommittee); i++ {
		syncCommittee[i] = vals[i].PublicKey
	}
	require.NoError(t, st.SetNextSyncCommittee(&ethpbalpha.SyncCommittee{
		Pubkeys:         syncCommittee,
		AggregatePubkey: bytesutil.PadTo([]byte{}, params.BeaconConfig().BLSPubkeyLength),
	}))
	db := dbTest.SetupDB(t)

	chainService := &chainMock.ChainService{}
	s := &Server{
		GenesisTimeFetcher: &testutil.MockGenesisTimeFetcher{
			Genesis: time.Now(),
		},
		Stater: &futureSyncMockFetcher{
			BeaconState: st,
		},
		HeadFetcher:           chainService,
		OptimisticModeFetcher: chainService,
		FinalizationFetcher:   chainService,
		BeaconDB:              db,
	}

	epoch := 2 * params.BeaconConfig().EpochsPerSyncCommitteePeriod
	request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://example.com//eth/v1/beacon/states/{state_id}/sync_committees?epoch=%d", epoch), nil)
	request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.GetSyncCommittees(writer, request)
	require.Equal(t, http.StatusBadRequest, writer.Code)
	e := &httputil.DefaultJsonError{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
	assert.Equal(t, http.StatusBadRequest, e.Code)
	assert.StringContains(t, "Could not fetch sync committee too far in the future", e.Message)

	epoch = 2*params.BeaconConfig().EpochsPerSyncCommitteePeriod - 1
	request = httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://example.com//eth/v1/beacon/states/{state_id}/sync_committees?epoch=%d", epoch), nil)
	request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
	writer = httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.GetSyncCommittees(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	resp := &GetSyncCommitteeResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	committeeVals := resp.Data.Validators
	require.Equal(t, params.BeaconConfig().SyncCommitteeSize, uint64(len(committeeVals)))
	for i := uint64(0); i < params.BeaconConfig().SyncCommitteeSize; i++ {
		assert.Equal(t, strconv.FormatUint(i, 10), committeeVals[i])
	}
	require.Equal(t, params.BeaconConfig().SyncCommitteeSubnetCount, uint64(len(resp.Data.ValidatorAggregates)))
	for i := uint64(0); i < params.BeaconConfig().SyncCommitteeSubnetCount; i++ {
		vStartIndex := primitives.ValidatorIndex(params.BeaconConfig().SyncCommitteeSize / params.BeaconConfig().SyncCommitteeSubnetCount * i)
		vEndIndex := primitives.ValidatorIndex(params.BeaconConfig().SyncCommitteeSize/params.BeaconConfig().SyncCommitteeSubnetCount*(i+1) - 1)
		j := 0
		for vIndex := vStartIndex; vIndex <= vEndIndex; vIndex++ {
			assert.Equal(t, strconv.FormatUint(uint64(vIndex), 10), resp.Data.ValidatorAggregates[i][j])
			j++
		}
	}
}
