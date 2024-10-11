package validator

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	neturl "net/url"
	"strconv"
	"strings"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/lookup"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"

	"github.com/gorilla/mux"
	chainMock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/testutil"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestGetValidatorCountInvalidRequest(t *testing.T) {
	st, _ := util.DeterministicGenesisState(t, 10)
	stateIdCheckerStateFunc := func(_ context.Context, stateId []byte) (state.BeaconState, error) {
		stateIdString := strings.ToLower(string(stateId))
		switch stateIdString {
		case "head", "genesis", "finalized", "justified":
			return st, nil
		default:
			if len(stateId) == 32 {
				return nil, nil
			} else {
				_, parseErr := strconv.ParseUint(stateIdString, 10, 64)
				if parseErr != nil {
					// ID format does not match any valid options.
					e := lookup.NewStateIdParseError(parseErr)
					return nil, &e
				}
				return st, nil
			}
		}
	}

	tests := []struct {
		name                 string
		stater               lookup.Stater
		status               string
		stateID              string
		expectedErrorMessage string
		statusCode           int
	}{
		{
			name: "invalid status",
			stater: &testutil.MockStater{
				BeaconState: st,
			},
			status:               "helloworld",
			stateID:              "head",
			expectedErrorMessage: "invalid status query parameter",
			statusCode:           http.StatusBadRequest,
		},
		{
			name:                 "invalid state ID",
			stater:               &testutil.MockStater{StateProviderFunc: stateIdCheckerStateFunc},
			stateID:              "helloworld",
			expectedErrorMessage: "Invalid state ID",
			statusCode:           http.StatusBadRequest,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			chainService := &chainMock.ChainService{Optimistic: false, FinalizedRoots: make(map[[32]byte]bool)}

			server := &Server{
				OptimisticModeFetcher: chainService,
				FinalizationFetcher:   chainService,
				Stater:                test.stater,
			}

			testRouter := mux.NewRouter()
			testRouter.HandleFunc("/eth/v1/beacon/states/{state_id}/validator_count", server.GetValidatorCount)
			s := httptest.NewServer(testRouter)
			defer s.Close()

			queryParams := neturl.Values{}
			queryParams.Add("status", test.status)
			resp, err := http.Get(s.URL + fmt.Sprintf("/eth/v1/beacon/states/%s/validator_count?%s",
				test.stateID, queryParams.Encode()))
			require.NoError(t, err)
			require.Equal(t, http.StatusBadRequest, resp.StatusCode)

			body, err := io.ReadAll(resp.Body)
			require.NoError(t, err)

			var errJson httputil.DefaultJsonError
			err = json.Unmarshal(body, &errJson)
			require.NoError(t, err)
			require.Equal(t, test.statusCode, errJson.Code)
			require.StringContains(t, test.expectedErrorMessage, errJson.Message)
		})
	}
}

func TestGetValidatorCount(t *testing.T) {
	st, _ := util.DeterministicGenesisState(t, 10)
	farFutureEpoch := params.BeaconConfig().FarFutureEpoch
	validators := []*eth.Validator{
		// Pending initialized.
		{
			ActivationEpoch:            farFutureEpoch,
			ActivationEligibilityEpoch: farFutureEpoch,
			ExitEpoch:                  farFutureEpoch,
			WithdrawableEpoch:          farFutureEpoch,
		},
		// Pending queued.
		{
			ActivationEpoch:            10,
			ActivationEligibilityEpoch: 4,
			ExitEpoch:                  farFutureEpoch,
			WithdrawableEpoch:          farFutureEpoch,
		},
		// Active ongoing.
		{
			ActivationEpoch: 0,
			ExitEpoch:       farFutureEpoch,
		},
		// Active slashed.
		{
			ActivationEpoch:   0,
			ExitEpoch:         30,
			Slashed:           true,
			WithdrawableEpoch: 50,
		},
		// Active exiting.
		{
			ActivationEpoch:   0,
			ExitEpoch:         30,
			Slashed:           false,
			WithdrawableEpoch: 50,
		},
		// Exit slashed (at epoch 35).
		{
			ActivationEpoch:   3,
			ExitEpoch:         30,
			WithdrawableEpoch: 50,
			Slashed:           true,
		},
		// Exit unslashed (at epoch 35).
		{
			ActivationEpoch:   3,
			ExitEpoch:         30,
			WithdrawableEpoch: 50,
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
	for _, validator := range validators {
		require.NoError(t, st.AppendValidator(validator))
		require.NoError(t, st.AppendBalance(params.BeaconConfig().MaxEffectiveBalance))
	}

	tests := []struct {
		name             string
		stateID          string
		statuses         []string
		currentEpoch     int
		expectedResponse CountResponse
	}{
		{
			name:     "Head count active validators",
			stateID:  "head",
			statuses: []string{"active"},
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "active",
						Count:  "13",
					},
				},
			},
		},
		{
			name:     "Head count active ongoing validators",
			stateID:  "head",
			statuses: []string{"active_ongoing"},
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "active_ongoing",
						Count:  "11",
					},
				},
			},
		},
		{
			name:     "Head count active exiting validators",
			stateID:  "head",
			statuses: []string{"active_exiting"},
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "active_exiting",
						Count:  "1",
					},
				},
			},
		},
		{
			name:     "Head count active slashed validators",
			stateID:  "head",
			statuses: []string{"active_slashed"},
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "active_slashed",
						Count:  "1",
					},
				},
			},
		},
		{
			name:     "Head count pending validators",
			stateID:  "head",
			statuses: []string{"pending"},
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "pending",
						Count:  "6",
					},
				},
			},
		},
		{
			name:     "Head count pending initialized validators",
			stateID:  "head",
			statuses: []string{"pending_initialized"},
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "pending_initialized",
						Count:  "1",
					},
				},
			},
		},
		{
			name:     "Head count pending queued validators",
			stateID:  "head",
			statuses: []string{"pending_queued"},
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "pending_queued",
						Count:  "5",
					},
				},
			},
		},
		{
			name:         "Head count exited validators",
			stateID:      "head",
			statuses:     []string{"exited"},
			currentEpoch: 35,
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "exited",
						Count:  "6",
					},
				},
			},
		},
		{
			name:         "Head count exited slashed validators",
			stateID:      "head",
			statuses:     []string{"exited_slashed"},
			currentEpoch: 35,
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "exited_slashed",
						Count:  "2",
					},
				},
			},
		},
		{
			name:         "Head count exited unslashed validators",
			stateID:      "head",
			statuses:     []string{"exited_unslashed"},
			currentEpoch: 35,
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "exited_unslashed",
						Count:  "4",
					},
				},
			},
		},
		{
			name:         "Head count withdrawal validators",
			stateID:      "head",
			statuses:     []string{"withdrawal"},
			currentEpoch: 45,
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "withdrawal",
						Count:  "2",
					},
				},
			},
		},
		{
			name:         "Head count withdrawal possible validators",
			stateID:      "head",
			statuses:     []string{"withdrawal_possible"},
			currentEpoch: 45,
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "withdrawal_possible",
						Count:  "1",
					},
				},
			},
		},
		{
			name:         "Head count withdrawal done validators",
			stateID:      "head",
			statuses:     []string{"withdrawal_done"},
			currentEpoch: 45,
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "withdrawal_done",
						Count:  "1",
					},
				},
			},
		},
		{
			name:     "Head count active and pending validators",
			stateID:  "head",
			statuses: []string{"active", "pending"},
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "active",
						Count:  "13",
					},
					{
						Status: "pending",
						Count:  "6",
					},
				},
			},
		},
		{
			name:    "Head count of ALL validators",
			stateID: "head",
			expectedResponse: CountResponse{
				ExecutionOptimistic: "false",
				Finalized:           "true",
				Data: []*Count{
					{
						Status: "active",
						Count:  "13",
					},
					{
						Status: "active_exiting",
						Count:  "1",
					},
					{
						Status: "active_ongoing",
						Count:  "11",
					},
					{
						Status: "active_slashed",
						Count:  "1",
					},
					{
						Status: "pending",
						Count:  "6",
					},
					{
						Status: "pending_initialized",
						Count:  "1",
					},
					{
						Status: "pending_queued",
						Count:  "5",
					},
				},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			chainService := &chainMock.ChainService{Optimistic: false, FinalizedRoots: make(map[[32]byte]bool)}
			blockRoot, err := st.LatestBlockHeader().HashTreeRoot()
			require.NoError(t, err)
			chainService.FinalizedRoots[blockRoot] = true
			require.NoError(t, st.SetSlot(params.BeaconConfig().SlotsPerEpoch*primitives.Slot(test.currentEpoch)))

			server := &Server{
				OptimisticModeFetcher: chainService,
				FinalizationFetcher:   chainService,
				Stater: &testutil.MockStater{
					BeaconState: st,
				},
			}

			testRouter := mux.NewRouter()
			testRouter.HandleFunc("/eth/v1/beacon/states/{state_id}/validator_count", server.GetValidatorCount)
			s := httptest.NewServer(testRouter)
			defer s.Close()

			queryParams := neturl.Values{}
			for _, status := range test.statuses {
				queryParams.Add("status", status)
			}
			resp, err := http.Get(s.URL + fmt.Sprintf("/eth/v1/beacon/states/%s/validator_count?%s",
				test.stateID, queryParams.Encode()))
			require.NoError(t, err)
			require.Equal(t, http.StatusOK, resp.StatusCode)
			body, err := io.ReadAll(resp.Body)
			require.NoError(t, err)

			var count CountResponse
			err = json.Unmarshal(body, &count)
			require.NoError(t, err)
			require.DeepEqual(t, test.expectedResponse, count)
		})
	}
}
