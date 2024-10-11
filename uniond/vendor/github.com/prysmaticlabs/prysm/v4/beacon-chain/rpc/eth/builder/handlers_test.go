package builder

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strconv"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/gorilla/mux"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/testutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func TestExpectedWithdrawals_BadRequest(t *testing.T) {
	st, err := util.NewBeaconStateCapella()
	slotsAhead := 5000
	require.NoError(t, err)
	capellaSlot, err := slots.EpochStart(params.BeaconConfig().CapellaForkEpoch)
	require.NoError(t, err)
	currentSlot := capellaSlot + primitives.Slot(slotsAhead)
	require.NoError(t, st.SetSlot(currentSlot))
	mockChainService := &mock.ChainService{Optimistic: true}

	testCases := []struct {
		name         string
		path         string
		urlParams    map[string]string
		state        state.BeaconState
		errorMessage string
	}{
		{
			name: "no state_id url params",
			path: "/eth/v1/builder/states/{state_id}/expected_withdrawals?proposal_slot" +
				strconv.FormatUint(uint64(currentSlot), 10),
			urlParams:    map[string]string{},
			state:        nil,
			errorMessage: "state_id is required in URL params",
		},
		{
			name:         "invalid proposal slot value",
			path:         "/eth/v1/builder/states/{state_id}/expected_withdrawals?proposal_slot=aaa",
			urlParams:    map[string]string{"state_id": "head"},
			state:        st,
			errorMessage: "invalid proposal slot value",
		},
		{
			name: "proposal slot < Capella start slot",
			path: "/eth/v1/builder/states/{state_id}/expected_withdrawals?proposal_slot=" +
				strconv.FormatUint(uint64(capellaSlot)-1, 10),
			urlParams:    map[string]string{"state_id": "head"},
			state:        st,
			errorMessage: "expected withdrawals are not supported before Capella fork",
		},
		{
			name: "proposal slot == Capella start slot",
			path: "/eth/v1/builder/states/{state_id}/expected_withdrawals?proposal_slot=" +
				strconv.FormatUint(uint64(capellaSlot), 10),
			urlParams:    map[string]string{"state_id": "head"},
			state:        st,
			errorMessage: "proposal slot must be bigger than state slot",
		},
		{
			name: "Proposal slot >= 128 slots ahead of state slot",
			path: "/eth/v1/builder/states/{state_id}/expected_withdrawals?proposal_slot=" +
				strconv.FormatUint(uint64(currentSlot+128), 10),
			urlParams:    map[string]string{"state_id": "head"},
			state:        st,
			errorMessage: "proposal slot cannot be >= 128 slots ahead of state slot",
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			s := &Server{
				FinalizationFetcher:   mockChainService,
				OptimisticModeFetcher: mockChainService,
				Stater:                &testutil.MockStater{BeaconState: testCase.state},
			}
			request := httptest.NewRequest("GET", testCase.path, nil)
			request = mux.SetURLVars(request, testCase.urlParams)
			writer := httptest.NewRecorder()
			writer.Body = &bytes.Buffer{}

			s.ExpectedWithdrawals(writer, request)
			assert.Equal(t, http.StatusBadRequest, writer.Code)
			e := &httputil.DefaultJsonError{}
			require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
			assert.Equal(t, http.StatusBadRequest, e.Code)
			assert.StringContains(t, testCase.errorMessage, e.Message)
		})
	}
}

func TestExpectedWithdrawals(t *testing.T) {
	st, err := util.NewBeaconStateCapella()
	slotsAhead := 5000
	require.NoError(t, err)
	capellaSlot, err := slots.EpochStart(params.BeaconConfig().CapellaForkEpoch)
	require.NoError(t, err)
	currentSlot := capellaSlot + primitives.Slot(slotsAhead)
	require.NoError(t, st.SetSlot(currentSlot))
	mockChainService := &mock.ChainService{Optimistic: true}

	t.Run("get correct expected withdrawals", func(t *testing.T) {
		params.SetupTestConfigCleanup(t)
		cfg := params.BeaconConfig().Copy()
		cfg.MaxValidatorsPerWithdrawalsSweep = 16
		params.OverrideBeaconConfig(cfg)

		// Update state with updated validator fields
		valCount := 17
		validators := make([]*eth.Validator, 0, valCount)
		balances := make([]uint64, 0, valCount)
		for i := 0; i < valCount; i++ {
			blsKey, err := bls.RandKey()
			require.NoError(t, err)
			val := &eth.Validator{
				PublicKey:             blsKey.PublicKey().Marshal(),
				WithdrawalCredentials: make([]byte, 32),
				ExitEpoch:             params.BeaconConfig().FarFutureEpoch,
				WithdrawableEpoch:     params.BeaconConfig().FarFutureEpoch,
				EffectiveBalance:      params.BeaconConfig().MaxEffectiveBalance,
			}
			val.WithdrawalCredentials[0] = params.BeaconConfig().ETH1AddressWithdrawalPrefixByte
			validators = append(validators, val)
			balances = append(balances, params.BeaconConfig().MaxEffectiveBalance)
		}

		epoch := slots.ToEpoch(st.Slot())
		// Fully withdrawable now with more than 0 balance
		validators[5].WithdrawableEpoch = epoch
		// Fully withdrawable now but 0 balance
		validators[10].WithdrawableEpoch = epoch
		balances[10] = 0
		// Partially withdrawable now but fully withdrawable after 1 epoch
		validators[14].WithdrawableEpoch = epoch + 1
		balances[14] += params.BeaconConfig().MinDepositAmount
		// Partially withdrawable
		validators[15].WithdrawableEpoch = epoch + 2
		balances[15] += params.BeaconConfig().MinDepositAmount
		// Above sweep bound
		validators[16].WithdrawableEpoch = epoch + 1
		balances[16] += params.BeaconConfig().MinDepositAmount

		require.NoError(t, st.SetValidators(validators))
		require.NoError(t, st.SetBalances(balances))
		inactivityScores := make([]uint64, valCount)
		for i := range inactivityScores {
			inactivityScores[i] = 10
		}
		require.NoError(t, st.SetInactivityScores(inactivityScores))

		s := &Server{
			FinalizationFetcher:   mockChainService,
			OptimisticModeFetcher: mockChainService,
			Stater:                &testutil.MockStater{BeaconState: st},
		}
		request := httptest.NewRequest(
			"GET", "/eth/v1/builder/states/{state_id}/expected_withdrawals?proposal_slot="+
				strconv.FormatUint(uint64(currentSlot+params.BeaconConfig().SlotsPerEpoch), 10), nil)
		request = mux.SetURLVars(request, map[string]string{"state_id": "head"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.ExpectedWithdrawals(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &ExpectedWithdrawalsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, true, resp.ExecutionOptimistic)
		assert.Equal(t, false, resp.Finalized)
		assert.Equal(t, 3, len(resp.Data))
		expectedWithdrawal1 := &ExpectedWithdrawal{
			Index:          strconv.FormatUint(0, 10),
			ValidatorIndex: strconv.FormatUint(5, 10),
			Address:        hexutil.Encode(validators[5].WithdrawalCredentials[12:]),
			// Decreased due to epoch processing when state advanced forward
			Amount: strconv.FormatUint(31998257885, 10),
		}
		expectedWithdrawal2 := &ExpectedWithdrawal{
			Index:          strconv.FormatUint(1, 10),
			ValidatorIndex: strconv.FormatUint(14, 10),
			Address:        hexutil.Encode(validators[14].WithdrawalCredentials[12:]),
			// MaxEffectiveBalance + MinDepositAmount + decrease after epoch processing
			Amount: strconv.FormatUint(32998257885, 10),
		}
		expectedWithdrawal3 := &ExpectedWithdrawal{
			Index:          strconv.FormatUint(2, 10),
			ValidatorIndex: strconv.FormatUint(15, 10),
			Address:        hexutil.Encode(validators[15].WithdrawalCredentials[12:]),
			// MinDepositAmount + decrease after epoch processing
			Amount: strconv.FormatUint(998257885, 10),
		}
		require.DeepEqual(t, expectedWithdrawal1, resp.Data[0])
		require.DeepEqual(t, expectedWithdrawal2, resp.Data[1])
		require.DeepEqual(t, expectedWithdrawal3, resp.Data[2])
	})
}
