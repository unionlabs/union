package beacon

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strconv"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/gorilla/mux"
	chainMock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/lookup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/testutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestGetValidators(t *testing.T) {
	const exitedValIndex = 3
	var st state.BeaconState
	st, _ = util.DeterministicGenesisState(t, 4)
	vals := st.Validators()
	vals[exitedValIndex].ExitEpoch = 0
	require.NoError(t, st.SetValidators(vals))

	t.Run("get all", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 4, len(resp.Data))
		val := resp.Data[0]
		assert.Equal(t, "0", val.Index)
		assert.Equal(t, "32000000000", val.Balance)
		assert.Equal(t, "active_ongoing", val.Status)
		require.NotNil(t, val.Validator)
		assert.Equal(t, "0xa99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c", val.Validator.Pubkey)
		assert.Equal(t, "0x00ec7ef7780c9d151597924036262dd28dc60e1228f4da6fecf9d402cb3f3594", val.Validator.WithdrawalCredentials)
		assert.Equal(t, "32000000000", val.Validator.EffectiveBalance)
		assert.Equal(t, false, val.Validator.Slashed)
		assert.Equal(t, "0", val.Validator.ActivationEligibilityEpoch)
		assert.Equal(t, "0", val.Validator.ActivationEpoch)
		assert.Equal(t, "18446744073709551615", val.Validator.ExitEpoch)
		assert.Equal(t, "18446744073709551615", val.Validator.WithdrawableEpoch)
	})
	t.Run("get by index", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(
			http.MethodGet,
			"http://example.com/eth/v1/beacon/states/{state_id}/validators?id=0&id=1",
			nil,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 2, len(resp.Data))
		assert.Equal(t, "0", resp.Data[0].Index)
		assert.Equal(t, "1", resp.Data[1].Index)
	})
	t.Run("get by pubkey", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		pubkey1 := st.PubkeyAtIndex(primitives.ValidatorIndex(0))
		pubkey2 := st.PubkeyAtIndex(primitives.ValidatorIndex(1))
		hexPubkey1 := hexutil.Encode(pubkey1[:])
		hexPubkey2 := hexutil.Encode(pubkey2[:])
		request := httptest.NewRequest(
			http.MethodGet,
			fmt.Sprintf("http://example.com/eth/v1/beacon/states/{state_id}/validators?id=%s&id=%s", hexPubkey1, hexPubkey2),
			nil,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 2, len(resp.Data))
		assert.Equal(t, "0", resp.Data[0].Index)
		assert.Equal(t, "1", resp.Data[1].Index)
	})
	t.Run("get by both index and pubkey", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		pubkey := st.PubkeyAtIndex(primitives.ValidatorIndex(0))
		hexPubkey := hexutil.Encode(pubkey[:])
		request := httptest.NewRequest(
			http.MethodGet,
			fmt.Sprintf("http://example.com/eth/v1/beacon/states/{state_id}/validators?id=%s&id=1", hexPubkey),
			nil,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 2, len(resp.Data))
		assert.Equal(t, "0", resp.Data[0].Index)
		assert.Equal(t, "1", resp.Data[1].Index)
	})
	t.Run("state ID required", func(t *testing.T) {
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher: &chainMock.ChainService{},
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidator(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "state_id is required in URL params", e.Message)
	})
	t.Run("unknown pubkey is ignored", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		pubkey := st.PubkeyAtIndex(primitives.ValidatorIndex(1))
		hexPubkey := hexutil.Encode(pubkey[:])
		request := httptest.NewRequest(
			http.MethodGet,
			fmt.Sprintf("http://example.com/eth/v1/beacon/states/{state_id}/validators?id=%s&id=%s", hexPubkey, hexutil.Encode([]byte(strings.Repeat("x", fieldparams.BLSPubkeyLength)))),
			nil,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 1, len(resp.Data))
		assert.Equal(t, "1", resp.Data[0].Index)
	})
	t.Run("unknown index is ignored", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators?id=1&id=99999", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 1, len(resp.Data))
		assert.Equal(t, "1", resp.Data[0].Index)
	})
	t.Run("execution optimistic", func(t *testing.T) {
		chainService := &chainMock.ChainService{Optimistic: true}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, true, resp.ExecutionOptimistic)
	})
	t.Run("finalized", func(t *testing.T) {
		headerRoot, err := st.LatestBlockHeader().HashTreeRoot()
		require.NoError(t, err)
		chainService := &chainMock.ChainService{
			FinalizedRoots: map[[32]byte]bool{
				headerRoot: true,
			},
		}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, true, resp.Finalized)
	})
	t.Run("POST", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		var body bytes.Buffer
		req := &GetValidatorsRequest{
			Ids:      []string{"0", strconv.Itoa(exitedValIndex)},
			Statuses: []string{"exited"},
		}
		b, err := json.Marshal(req)
		require.NoError(t, err)
		_, err = body.Write(b)
		require.NoError(t, err)
		request := httptest.NewRequest(
			http.MethodPost,
			"http://example.com/eth/v1/beacon/states/{state_id}/validators",
			&body,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 1, len(resp.Data))
		assert.Equal(t, "3", resp.Data[0].Index)
	})
	t.Run("POST nil values", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		var body bytes.Buffer
		req := &GetValidatorsRequest{
			Ids:      nil,
			Statuses: nil,
		}
		b, err := json.Marshal(req)
		require.NoError(t, err)
		_, err = body.Write(b)
		require.NoError(t, err)
		request := httptest.NewRequest(
			http.MethodPost,
			"http://example.com/eth/v1/beacon/states/{state_id}/validators",
			&body,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 4, len(resp.Data))
	})
	t.Run("POST empty", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(
			http.MethodPost,
			"http://example.com/eth/v1/beacon/states/{state_id}/validators",
			nil,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "No data submitted", e.Message)
	})
	t.Run("POST invalid", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		body := bytes.Buffer{}
		_, err := body.WriteString("foo")
		require.NoError(t, err)
		request := httptest.NewRequest(
			http.MethodPost,
			"http://example.com/eth/v1/beacon/states/{state_id}/validators",
			&body,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "Could not decode request body", e.Message)
	})
}

func TestGetValidators_FilterByStatus(t *testing.T) {
	var st state.BeaconState
	st, _ = util.DeterministicGenesisState(t, 1)

	farFutureEpoch := params.BeaconConfig().FarFutureEpoch
	validators := []*eth.Validator{
		// Pending initialized.
		{
			ActivationEpoch:            farFutureEpoch,
			ActivationEligibilityEpoch: farFutureEpoch,
		},
		// Pending queued.
		{
			ActivationEpoch:            10,
			ActivationEligibilityEpoch: 4,
		},
		// Active ongoing.
		{
			ActivationEpoch: 0,
			ExitEpoch:       farFutureEpoch,
		},
		// Active slashed.
		{
			ActivationEpoch: 0,
			ExitEpoch:       30,
			Slashed:         true,
		},
		// Active exiting.
		{
			ActivationEpoch: 3,
			ExitEpoch:       30,
			Slashed:         false,
		},
		// Exited slashed (at epoch 35).
		{
			ActivationEpoch:   3,
			ExitEpoch:         30,
			WithdrawableEpoch: 40,
			Slashed:           true,
		},
		// Exited unslashed (at epoch 35).
		{
			ActivationEpoch:   3,
			ExitEpoch:         30,
			WithdrawableEpoch: 40,
			Slashed:           false,
		},
		// Withdrawable (at epoch 45).
		{
			ActivationEpoch:   3,
			ExitEpoch:         30,
			WithdrawableEpoch: 40,
			EffectiveBalance:  params.BeaconConfig().MaxEffectiveBalance,
			Slashed:           false,
		},
		// Withdrawal done (at epoch 45).
		{
			ActivationEpoch:   3,
			ExitEpoch:         30,
			WithdrawableEpoch: 40,
			EffectiveBalance:  0,
			Slashed:           false,
		},
	}
	for _, val := range validators {
		require.NoError(t, st.AppendValidator(val))
		require.NoError(t, st.AppendBalance(params.BeaconConfig().MaxEffectiveBalance))
	}

	t.Run("active", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &lookup.BeaconDbStater{
				ChainInfoFetcher: &chainMock.ChainService{State: st},
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators?status=active", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, 3, len(resp.Data))
		for _, vc := range resp.Data {
			assert.Equal(
				t,
				true,
				vc.Status == "active_ongoing" ||
					vc.Status == "active_exiting" ||
					vc.Status == "active_slashed",
			)
		}
	})
	t.Run("active_ongoing", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &lookup.BeaconDbStater{
				ChainInfoFetcher: &chainMock.ChainService{State: st},
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators?status=active_ongoing", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, 2, len(resp.Data))
		for _, vc := range resp.Data {
			require.Equal(
				t,
				true,
				vc.Status == "active_ongoing",
			)
		}
	})
	require.NoError(t, st.SetSlot(params.BeaconConfig().SlotsPerEpoch*35))
	t.Run("exited", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &lookup.BeaconDbStater{
				ChainInfoFetcher: &chainMock.ChainService{State: st},
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators?status=exited", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, 4, len(resp.Data))
		for _, vc := range resp.Data {
			require.Equal(
				t,
				true,
				vc.Status == "exited_unslashed" || vc.Status == "exited_slashed",
			)
		}
	})
	t.Run("pending_initialized and exited_unslashed", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &lookup.BeaconDbStater{
				ChainInfoFetcher: &chainMock.ChainService{State: st},
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(
			http.MethodGet,
			"http://example.com/eth/v1/beacon/states/{state_id}/validators?status=pending_initialized&status=exited_unslashed",
			nil,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, 4, len(resp.Data))
		for _, vc := range resp.Data {
			require.Equal(
				t,
				true,
				vc.Status == "pending_initialized" || vc.Status == "exited_unslashed",
			)
		}
	})
	t.Run("pending and exited_slashed", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &lookup.BeaconDbStater{
				ChainInfoFetcher: &chainMock.ChainService{State: st},
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(
			http.MethodGet,
			"http://example.com/eth/v1/beacon/states/{state_id}/validators?status=pending&status=exited_slashed",
			nil,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, 2, len(resp.Data))
		for _, vc := range resp.Data {
			require.Equal(
				t,
				true,
				vc.Status == "pending_initialized" || vc.Status == "exited_slashed",
			)
		}
	})
}

func TestGetValidator(t *testing.T) {
	var st state.BeaconState
	st, _ = util.DeterministicGenesisState(t, 2)

	t.Run("get by index", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators/{validator_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head", "validator_id": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidator(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, "0", resp.Data.Index)
		assert.Equal(t, "32000000000", resp.Data.Balance)
		assert.Equal(t, "active_ongoing", resp.Data.Status)
		require.NotNil(t, resp.Data.Validator)
		assert.Equal(t, "0xa99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c", resp.Data.Validator.Pubkey)
		assert.Equal(t, "0x00ec7ef7780c9d151597924036262dd28dc60e1228f4da6fecf9d402cb3f3594", resp.Data.Validator.WithdrawalCredentials)
		assert.Equal(t, "32000000000", resp.Data.Validator.EffectiveBalance)
		assert.Equal(t, false, resp.Data.Validator.Slashed)
		assert.Equal(t, "0", resp.Data.Validator.ActivationEligibilityEpoch)
		assert.Equal(t, "0", resp.Data.Validator.ActivationEpoch)
		assert.Equal(t, "18446744073709551615", resp.Data.Validator.ExitEpoch)
		assert.Equal(t, "18446744073709551615", resp.Data.Validator.WithdrawableEpoch)
	})
	t.Run("get by pubkey", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		pubKey := st.PubkeyAtIndex(primitives.ValidatorIndex(0))
		hexPubkey := hexutil.Encode(pubKey[:])
		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators/{validator_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head", "validator_id": hexPubkey})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidator(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, "0", resp.Data.Index)
	})
	t.Run("state ID required", func(t *testing.T) {
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher: &chainMock.ChainService{},
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators/{validator_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"validator_id": "1"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidator(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "state_id is required in URL params", e.Message)
	})
	t.Run("validator ID required", func(t *testing.T) {
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher: &chainMock.ChainService{},
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators/{validator_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidator(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "validator_id is required in URL params", e.Message)
	})
	t.Run("unknown index", func(t *testing.T) {
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher: &chainMock.ChainService{},
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators/{validator_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head", "validator_id": "99999"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidator(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "Invalid validator index", e.Message)
	})
	t.Run("unknown pubkey", func(t *testing.T) {
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher: &chainMock.ChainService{},
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators/{validator_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head", "validator_id": hexutil.Encode([]byte(strings.Repeat("x", fieldparams.BLSPubkeyLength)))})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidator(writer, request)
		assert.Equal(t, http.StatusNotFound, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusNotFound, e.Code)
		assert.StringContains(t, "Unknown validator", e.Message)
	})
	t.Run("execution optimistic", func(t *testing.T) {
		chainService := &chainMock.ChainService{Optimistic: true}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators/{validator_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head", "validator_id": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidator(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, true, resp.ExecutionOptimistic)
	})
	t.Run("finalized", func(t *testing.T) {
		headerRoot, err := st.LatestBlockHeader().HashTreeRoot()
		require.NoError(t, err)
		chainService := &chainMock.ChainService{
			FinalizedRoots: map[[32]byte]bool{
				headerRoot: true,
			},
		}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validators/{validator_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head", "validator_id": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidator(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, true, resp.Finalized)
	})
}

func TestGetValidatorBalances(t *testing.T) {
	var st state.BeaconState
	count := uint64(4)
	st, _ = util.DeterministicGenesisState(t, count)
	balances := make([]uint64, count)
	for i := uint64(0); i < count; i++ {
		balances[i] = i
	}
	require.NoError(t, st.SetBalances(balances))

	t.Run("get all", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validator_balances", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidatorBalances(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorBalancesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 4, len(resp.Data))
		val := resp.Data[3]
		assert.Equal(t, "3", val.Index)
		assert.Equal(t, "3", val.Balance)
	})
	t.Run("get by index", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(
			http.MethodGet,
			"http://example.com/eth/v1/beacon/states/{state_id}/validator_balances?id=0&id=1",
			nil,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidatorBalances(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorBalancesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 2, len(resp.Data))
		assert.Equal(t, "0", resp.Data[0].Index)
		assert.Equal(t, "1", resp.Data[1].Index)
	})
	t.Run("get by pubkey", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}
		pubkey1 := st.PubkeyAtIndex(primitives.ValidatorIndex(0))
		pubkey2 := st.PubkeyAtIndex(primitives.ValidatorIndex(1))
		hexPubkey1 := hexutil.Encode(pubkey1[:])
		hexPubkey2 := hexutil.Encode(pubkey2[:])

		request := httptest.NewRequest(
			http.MethodGet,
			fmt.Sprintf("http://example.com/eth/v1/beacon/states/{state_id}/validator_balances?id=%s&id=%s", hexPubkey1, hexPubkey2),
			nil,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidatorBalances(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorBalancesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 2, len(resp.Data))
		assert.Equal(t, "0", resp.Data[0].Index)
		assert.Equal(t, "1", resp.Data[1].Index)
	})
	t.Run("get by both index and pubkey", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		pubkey := st.PubkeyAtIndex(primitives.ValidatorIndex(0))
		hexPubkey := hexutil.Encode(pubkey[:])
		request := httptest.NewRequest(
			http.MethodGet,
			fmt.Sprintf("http://example.com/eth/v1/beacon/states/{state_id}/validators?id=%s&id=1", hexPubkey),
			nil,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidators(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 2, len(resp.Data))
		assert.Equal(t, "0", resp.Data[0].Index)
		assert.Equal(t, "1", resp.Data[1].Index)
	})
	t.Run("unknown pubkey is ignored", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		pubkey := st.PubkeyAtIndex(primitives.ValidatorIndex(1))
		hexPubkey := hexutil.Encode(pubkey[:])
		request := httptest.NewRequest(
			http.MethodGet,
			fmt.Sprintf("http://example.com/eth/v1/beacon/states/{state_id}/validator_balances?id=%s&id=%s", hexPubkey, hexutil.Encode([]byte(strings.Repeat("x", fieldparams.BLSPubkeyLength)))),
			nil,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidatorBalances(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorBalancesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 1, len(resp.Data))
		assert.Equal(t, "1", resp.Data[0].Index)
	})
	t.Run("unknown index is ignored", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validator_balances?id=1&id=99999", nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidatorBalances(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorBalancesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 1, len(resp.Data))
		assert.Equal(t, "1", resp.Data[0].Index)
	})
	t.Run("state ID required", func(t *testing.T) {
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher: &chainMock.ChainService{},
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/beacon/states/{state_id}/validator_balances", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidator(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "state_id is required in URL params", e.Message)
	})
	t.Run("execution optimistic", func(t *testing.T) {
		chainService := &chainMock.ChainService{Optimistic: true}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(
			http.MethodGet,
			"http://example.com/eth/v1/beacon/states/{state_id}/validator_balances?id=0",
			nil,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidatorBalances(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorBalancesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, true, resp.ExecutionOptimistic)
	})
	t.Run("finalized", func(t *testing.T) {
		headerRoot, err := st.LatestBlockHeader().HashTreeRoot()
		require.NoError(t, err)
		chainService := &chainMock.ChainService{
			FinalizedRoots: map[[32]byte]bool{
				headerRoot: true,
			},
		}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(
			http.MethodGet,
			"http://example.com/eth/v1/beacon/states/{state_id}/validator_balances?id=0",
			nil,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidatorBalances(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorBalancesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, true, resp.Finalized)
	})
	t.Run("POST", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		pubkey1 := st.PubkeyAtIndex(primitives.ValidatorIndex(0))
		pubkey2 := st.PubkeyAtIndex(primitives.ValidatorIndex(1))
		hexPubkey1 := hexutil.Encode(pubkey1[:])
		hexPubkey2 := hexutil.Encode(pubkey2[:])
		var body bytes.Buffer
		_, err := body.WriteString(fmt.Sprintf("[\"%s\",\"%s\"]", hexPubkey1, hexPubkey2))
		require.NoError(t, err)
		request := httptest.NewRequest(
			http.MethodPost,
			"http://example.com/eth/v1/beacon/states/{state_id}/validator_balances",
			&body,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidatorBalances(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetValidatorBalancesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 2, len(resp.Data))
		assert.Equal(t, "0", resp.Data[0].Index)
		assert.Equal(t, "1", resp.Data[1].Index)
	})
	t.Run("POST empty", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		request := httptest.NewRequest(
			http.MethodPost,
			"http://example.com/eth/v1/beacon/states/{state_id}/validator_balances",
			nil,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidatorBalances(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "No data submitted", e.Message)
	})
	t.Run("POST invalid", func(t *testing.T) {
		chainService := &chainMock.ChainService{}
		s := Server{
			Stater: &testutil.MockStater{
				BeaconState: st,
			},
			HeadFetcher:           chainService,
			OptimisticModeFetcher: chainService,
			FinalizationFetcher:   chainService,
		}

		body := bytes.Buffer{}
		_, err := body.WriteString("foo")
		require.NoError(t, err)
		request := httptest.NewRequest(
			http.MethodPost,
			"http://example.com/eth/v1/beacon/states/{state_id}/validator_balances",
			&body,
		)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetValidatorBalances(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "Could not decode request body", e.Message)
	})
}
