package debug

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/gorilla/mux"
	"github.com/prysmaticlabs/prysm/v4/api"
	blockchainmock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	dbtest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/testutil"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestGetBeaconStateSSZ(t *testing.T) {
	fakeState, err := util.NewBeaconState()
	require.NoError(t, err)
	sszState, err := fakeState.MarshalSSZ()
	require.NoError(t, err)

	s := &Server{
		Stater: &testutil.MockStater{
			BeaconState: fakeState,
		},
	}

	request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/debug/beacon/states/{state_id}", nil)
	request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetBeaconStateSSZ(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	assert.DeepEqual(t, sszState, writer.Body.Bytes())
}

func TestGetBeaconStateV2(t *testing.T) {
	ctx := context.Background()
	db := dbtest.SetupDB(t)

	t.Run("phase0", func(t *testing.T) {
		fakeState, err := util.NewBeaconState()
		require.NoError(t, err)
		require.NoError(t, fakeState.SetSlot(123))
		chainService := &blockchainmock.ChainService{}
		s := &Server{
			Stater: &testutil.MockStater{
				BeaconState: fakeState,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v2/debug/beacon/states/{state_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetBeaconStateV2(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetBeaconStateV2Response{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, version.String(version.Phase0), resp.Version)
		st := &shared.BeaconState{}
		require.NoError(t, json.Unmarshal(resp.Data, st))
		assert.Equal(t, "123", st.Slot)
	})
	t.Run("Altair", func(t *testing.T) {
		fakeState, err := util.NewBeaconStateAltair()
		require.NoError(t, err)
		require.NoError(t, fakeState.SetSlot(123))
		chainService := &blockchainmock.ChainService{}
		s := &Server{
			Stater: &testutil.MockStater{
				BeaconState: fakeState,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v2/debug/beacon/states/{state_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetBeaconStateV2(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetBeaconStateV2Response{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, version.String(version.Altair), resp.Version)
		st := &shared.BeaconStateAltair{}
		require.NoError(t, json.Unmarshal(resp.Data, st))
		assert.Equal(t, "123", st.Slot)
	})
	t.Run("Bellatrix", func(t *testing.T) {
		fakeState, err := util.NewBeaconStateBellatrix()
		require.NoError(t, err)
		require.NoError(t, fakeState.SetSlot(123))
		chainService := &blockchainmock.ChainService{}
		s := &Server{
			Stater: &testutil.MockStater{
				BeaconState: fakeState,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v2/debug/beacon/states/{state_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetBeaconStateV2(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetBeaconStateV2Response{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, version.String(version.Bellatrix), resp.Version)
		st := &shared.BeaconStateBellatrix{}
		require.NoError(t, json.Unmarshal(resp.Data, st))
		assert.Equal(t, "123", st.Slot)
	})
	t.Run("Capella", func(t *testing.T) {
		fakeState, err := util.NewBeaconStateCapella()
		require.NoError(t, err)
		require.NoError(t, fakeState.SetSlot(123))
		chainService := &blockchainmock.ChainService{}
		s := &Server{
			Stater: &testutil.MockStater{
				BeaconState: fakeState,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v2/debug/beacon/states/{state_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetBeaconStateV2(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetBeaconStateV2Response{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, version.String(version.Capella), resp.Version)
		st := &shared.BeaconStateCapella{}
		require.NoError(t, json.Unmarshal(resp.Data, st))
		assert.Equal(t, "123", st.Slot)
	})
	t.Run("Deneb", func(t *testing.T) {
		fakeState, err := util.NewBeaconStateDeneb()
		require.NoError(t, err)
		require.NoError(t, fakeState.SetSlot(123))
		chainService := &blockchainmock.ChainService{}
		s := &Server{
			Stater: &testutil.MockStater{
				BeaconState: fakeState,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v2/debug/beacon/states/{state_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetBeaconStateV2(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetBeaconStateV2Response{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, version.String(version.Deneb), resp.Version)
		st := &shared.BeaconStateDeneb{}
		require.NoError(t, json.Unmarshal(resp.Data, st))
		assert.Equal(t, "123", st.Slot)
	})
	t.Run("execution optimistic", func(t *testing.T) {
		parentRoot := [32]byte{'a'}
		blk := util.NewBeaconBlock()
		blk.Block.ParentRoot = parentRoot[:]
		root, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		util.SaveBlock(t, ctx, db, blk)
		require.NoError(t, db.SaveGenesisBlockRoot(ctx, root))

		fakeState, err := util.NewBeaconStateBellatrix()
		require.NoError(t, err)
		chainService := &blockchainmock.ChainService{Optimistic: true}
		s := &Server{
			Stater: &testutil.MockStater{
				BeaconState: fakeState,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
			BeaconDB:              db,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v2/debug/beacon/states/{state_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetBeaconStateV2(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetBeaconStateV2Response{}
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

		fakeState, err := util.NewBeaconStateBellatrix()
		require.NoError(t, err)
		headerRoot, err := fakeState.LatestBlockHeader().HashTreeRoot()
		require.NoError(t, err)
		chainService := &blockchainmock.ChainService{
			FinalizedRoots: map[[32]byte]bool{
				headerRoot: true,
			},
		}
		s := &Server{
			Stater: &testutil.MockStater{
				BeaconState: fakeState,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
			BeaconDB:              db,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v2/debug/beacon/states/{state_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetBeaconStateV2(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetBeaconStateV2Response{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, true, resp.Finalized)
	})
}

func TestGetBeaconStateSSZV2(t *testing.T) {
	t.Run("Phase 0", func(t *testing.T) {
		fakeState, err := util.NewBeaconState()
		require.NoError(t, err)
		require.NoError(t, fakeState.SetSlot(123))

		s := &Server{
			Stater: &testutil.MockStater{
				BeaconState: fakeState,
			},
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v2/debug/beacon/states/{state_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetBeaconStateV2(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		assert.Equal(t, version.String(version.Phase0), writer.Header().Get(api.VersionHeader))
		sszExpected, err := fakeState.MarshalSSZ()
		require.NoError(t, err)
		assert.DeepEqual(t, sszExpected, writer.Body.Bytes())
	})
	t.Run("Altair", func(t *testing.T) {
		fakeState, err := util.NewBeaconStateAltair()
		require.NoError(t, err)
		require.NoError(t, fakeState.SetSlot(123))

		s := &Server{
			Stater: &testutil.MockStater{
				BeaconState: fakeState,
			},
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v2/debug/beacon/states/{state_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetBeaconStateV2(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		assert.Equal(t, version.String(version.Altair), writer.Header().Get(api.VersionHeader))
		sszExpected, err := fakeState.MarshalSSZ()
		require.NoError(t, err)
		assert.DeepEqual(t, sszExpected, writer.Body.Bytes())
	})
	t.Run("Bellatrix", func(t *testing.T) {
		fakeState, err := util.NewBeaconStateBellatrix()
		require.NoError(t, err)
		require.NoError(t, fakeState.SetSlot(123))

		s := &Server{
			Stater: &testutil.MockStater{
				BeaconState: fakeState,
			},
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v2/debug/beacon/states/{state_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetBeaconStateV2(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		assert.Equal(t, version.String(version.Bellatrix), writer.Header().Get(api.VersionHeader))
		sszExpected, err := fakeState.MarshalSSZ()
		require.NoError(t, err)
		assert.DeepEqual(t, sszExpected, writer.Body.Bytes())
	})
	t.Run("Capella", func(t *testing.T) {
		fakeState, err := util.NewBeaconStateCapella()
		require.NoError(t, err)
		require.NoError(t, fakeState.SetSlot(123))

		s := &Server{
			Stater: &testutil.MockStater{
				BeaconState: fakeState,
			},
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v2/debug/beacon/states/{state_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetBeaconStateV2(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		assert.Equal(t, version.String(version.Capella), writer.Header().Get(api.VersionHeader))
		sszExpected, err := fakeState.MarshalSSZ()
		require.NoError(t, err)
		assert.DeepEqual(t, sszExpected, writer.Body.Bytes())
	})
	t.Run("Deneb", func(t *testing.T) {
		fakeState, err := util.NewBeaconStateDeneb()
		require.NoError(t, err)
		require.NoError(t, fakeState.SetSlot(123))

		s := &Server{
			Stater: &testutil.MockStater{
				BeaconState: fakeState,
			},
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v2/debug/beacon/states/{state_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetBeaconStateV2(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		assert.Equal(t, version.String(version.Deneb), writer.Header().Get(api.VersionHeader))
		sszExpected, err := fakeState.MarshalSSZ()
		require.NoError(t, err)
		assert.DeepEqual(t, sszExpected, writer.Body.Bytes())
	})
}

func TestGetForkChoiceHeadsV2(t *testing.T) {
	expectedSlotsAndRoots := []struct {
		Slot string
		Root string
	}{{
		Slot: "0",
		Root: hexutil.Encode(bytesutil.PadTo([]byte("foo"), 32)),
	}, {
		Slot: "1",
		Root: hexutil.Encode(bytesutil.PadTo([]byte("bar"), 32)),
	}}

	chainService := &blockchainmock.ChainService{}
	s := &Server{
		HeadFetcher:           chainService,
		OptimisticModeFetcher: chainService,
	}

	request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v2/debug/beacon/heads", nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetForkChoiceHeadsV2(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	resp := &GetForkChoiceHeadsV2Response{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	assert.Equal(t, 2, len(resp.Data))
	for _, sr := range expectedSlotsAndRoots {
		found := false
		for _, h := range resp.Data {
			if h.Slot == sr.Slot {
				found = true
				assert.Equal(t, sr.Root, h.Root)
			}
			assert.Equal(t, false, h.ExecutionOptimistic)
		}
		assert.Equal(t, true, found, "Expected head not found")
	}

	t.Run("optimistic head", func(t *testing.T) {
		chainService := &blockchainmock.ChainService{
			Optimistic:      true,
			OptimisticRoots: make(map[[32]byte]bool),
		}
		for _, sr := range expectedSlotsAndRoots {
			b, err := hexutil.Decode(sr.Root)
			require.NoError(t, err)
			chainService.OptimisticRoots[bytesutil.ToBytes32(b)] = true
		}
		s := &Server{
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v2/debug/beacon/heads", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetForkChoiceHeadsV2(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetForkChoiceHeadsV2Response{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, 2, len(resp.Data))
		for _, sr := range expectedSlotsAndRoots {
			found := false
			for _, h := range resp.Data {
				if h.Slot == sr.Slot {
					found = true
					assert.Equal(t, sr.Root, h.Root)
				}
				assert.Equal(t, true, h.ExecutionOptimistic)
			}
			assert.Equal(t, true, found, "Expected head not found")
		}
	})
}

func TestGetForkChoice(t *testing.T) {
	store := doublylinkedtree.New()
	fRoot := [32]byte{'a'}
	fc := &forkchoicetypes.Checkpoint{Epoch: 2, Root: fRoot}
	require.NoError(t, store.UpdateFinalizedCheckpoint(fc))
	s := &Server{ForkchoiceFetcher: &blockchainmock.ChainService{ForkChoiceStore: store}}

	request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/debug/fork_choice", nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetForkChoice(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	resp := &GetForkChoiceDumpResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	require.Equal(t, "2", resp.FinalizedCheckpoint.Epoch)
}
