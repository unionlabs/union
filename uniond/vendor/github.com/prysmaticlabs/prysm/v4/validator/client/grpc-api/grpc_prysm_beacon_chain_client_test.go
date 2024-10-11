package grpc_api

import (
	"context"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	mock "github.com/prysmaticlabs/prysm/v4/testing/validator-mock"
	"github.com/prysmaticlabs/prysm/v4/validator/client/iface"
)

func TestGetValidatorCount(t *testing.T) {
	st, _ := util.DeterministicGenesisState(t, 10)
	farFutureEpoch := params.BeaconConfig().FarFutureEpoch
	validators := []*ethpb.Validator{
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
		statuses         []string
		currentEpoch     int
		expectedResponse []iface.ValidatorCount
	}{
		{
			name:     "Head count active validators",
			statuses: []string{"active"},
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "active",
					Count:  13,
				},
			},
		},
		{
			name:     "Head count active ongoing validators",
			statuses: []string{"active_ongoing"},
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "active_ongoing",
					Count:  11,
				},
			},
		},
		{
			name:     "Head count active exiting validators",
			statuses: []string{"active_exiting"},
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "active_exiting",
					Count:  1,
				},
			},
		},
		{
			name:     "Head count active slashed validators",
			statuses: []string{"active_slashed"},
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "active_slashed",
					Count:  1,
				},
			},
		},
		{
			name:     "Head count pending validators",
			statuses: []string{"pending"},
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "pending",
					Count:  6,
				},
			},
		},
		{
			name:     "Head count pending initialized validators",
			statuses: []string{"pending_initialized"},
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "pending_initialized",
					Count:  1,
				},
			},
		},
		{
			name:     "Head count pending queued validators",
			statuses: []string{"pending_queued"},
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "pending_queued",
					Count:  5,
				},
			},
		},
		{
			name:         "Head count exited validators",
			statuses:     []string{"exited"},
			currentEpoch: 35,
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "exited",
					Count:  6,
				},
			},
		},
		{
			name:         "Head count exited slashed validators",
			statuses:     []string{"exited_slashed"},
			currentEpoch: 35,
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "exited_slashed",
					Count:  2,
				},
			},
		},
		{
			name:         "Head count exited unslashed validators",
			statuses:     []string{"exited_unslashed"},
			currentEpoch: 35,
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "exited_unslashed",
					Count:  4,
				},
			},
		},
		{
			name:         "Head count withdrawal validators",
			statuses:     []string{"withdrawal"},
			currentEpoch: 45,
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "withdrawal",
					Count:  2,
				},
			},
		},
		{
			name:         "Head count withdrawal possible validators",
			statuses:     []string{"withdrawal_possible"},
			currentEpoch: 45,
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "withdrawal_possible",
					Count:  1,
				},
			},
		},
		{
			name:         "Head count withdrawal done validators",
			statuses:     []string{"withdrawal_done"},
			currentEpoch: 45,
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "withdrawal_done",
					Count:  1,
				},
			},
		},
		{
			name:     "Head count active and pending validators",
			statuses: []string{"active", "pending"},
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "active",
					Count:  13,
				},
				{
					Status: "pending",
					Count:  6,
				},
			},
		},
		{
			name: "Head count of ALL validators",
			expectedResponse: []iface.ValidatorCount{
				{
					Status: "active",
					Count:  13,
				},
				{
					Status: "active_exiting",
					Count:  1,
				},
				{
					Status: "active_ongoing",
					Count:  11,
				},
				{
					Status: "active_slashed",
					Count:  1,
				},
				{
					Status: "pending",
					Count:  6,
				},
				{
					Status: "pending_initialized",
					Count:  1,
				},
				{
					Status: "pending_queued",
					Count:  5,
				},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			ctrl := gomock.NewController(t)
			defer ctrl.Finish()

			listValidatorResp := &ethpb.Validators{}
			for _, val := range st.Validators() {
				listValidatorResp.ValidatorList = append(listValidatorResp.ValidatorList, &ethpb.Validators_ValidatorContainer{
					Validator: val,
				})
			}

			beaconChainClient := mock.NewMockBeaconChainClient(ctrl)
			beaconChainClient.EXPECT().ListValidators(
				gomock.Any(),
				gomock.Any(),
			).Return(
				listValidatorResp,
				nil,
			)

			beaconChainClient.EXPECT().GetChainHead(
				gomock.Any(),
				gomock.Any(),
			).Return(
				&ethpb.ChainHead{HeadEpoch: primitives.Epoch(test.currentEpoch)},
				nil,
			)

			prysmBeaconChainClient := &grpcPrysmBeaconChainClient{
				beaconChainClient: beaconChainClient,
			}

			var statuses []validator.Status
			for _, status := range test.statuses {
				ok, valStatus := validator.StatusFromString(status)
				require.Equal(t, true, ok)
				statuses = append(statuses, valStatus)
			}
			vcCountResp, err := prysmBeaconChainClient.GetValidatorCount(context.Background(), "", statuses)
			require.NoError(t, err)
			require.DeepEqual(t, test.expectedResponse, vcCountResp)
		})
	}
}
