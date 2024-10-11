package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/node"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/validator"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
	test_helpers "github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/test-helpers"
)

func TestSubmitAggregateSelectionProof(t *testing.T) {
	const (
		pubkeyStr                    = "0x8000091c2ae64ee414a54c1cc1fc67dec663408bc636cb86756e0200e41a75c8f86603f104f02c856983d2783116be13"
		syncingEndpoint              = "/eth/v1/node/syncing"
		attesterDutiesEndpoint       = "/eth/v1/validator/duties/attester"
		validatorsEndpoint           = "/eth/v1/beacon/states/head/validators"
		attestationDataEndpoint      = "/eth/v1/validator/attestation_data"
		aggregateAttestationEndpoint = "/eth/v1/validator/aggregate_attestation"
		validatorIndex               = "55293"
		slotSignature                = "0x8776a37d6802c4797d113169c5fcfda50e68a32058eb6356a6f00d06d7da64c841a00c7c38b9b94a204751eca53707bd03523ce4797827d9bacff116a6e776a20bbccff4b683bf5201b610797ed0502557a58a65c8395f8a1649b976c3112d15"
		slot                         = primitives.Slot(123)
		committeeIndex               = primitives.CommitteeIndex(1)
	)

	attesterDuties := []*validator.AttesterDuty{
		{
			Pubkey:          pubkeyStr,
			ValidatorIndex:  validatorIndex,
			Slot:            "123",
			CommitteeIndex:  "1",
			CommitteeLength: "3",
		},
	}

	attestationDataResponse := generateValidAttestation(uint64(slot), uint64(committeeIndex))
	attestationDataProto, err := attestationDataResponse.Data.ToConsensus()
	require.NoError(t, err)
	attestationDataRootBytes, err := attestationDataProto.HashTreeRoot()
	require.NoError(t, err)

	aggregateAttestation := &ethpb.Attestation{
		AggregationBits: test_helpers.FillByteSlice(4, 74),
		Data:            attestationDataProto,
		Signature:       test_helpers.FillByteSlice(96, 82),
	}

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	tests := []struct {
		name                       string
		isOptimistic               bool
		syncingErr                 error
		validatorsErr              error
		dutiesErr                  error
		attestationDataErr         error
		aggregateAttestationErr    error
		duties                     []*validator.AttesterDuty
		validatorsCalled           int
		attesterDutiesCalled       int
		attestationDataCalled      int
		aggregateAttestationCalled int
		expectedErrorMsg           string
	}{
		{
			name:                       "success",
			duties:                     attesterDuties,
			validatorsCalled:           1,
			attesterDutiesCalled:       1,
			attestationDataCalled:      1,
			aggregateAttestationCalled: 1,
		},
		{
			name:             "head is optimistic",
			isOptimistic:     true,
			expectedErrorMsg: "the node is currently optimistic and cannot serve validators",
		},
		{
			name:             "syncing error",
			syncingErr:       errors.New("bad request"),
			expectedErrorMsg: "failed to get syncing status",
		},
		{
			name:             "validator index error",
			validatorsCalled: 1,
			validatorsErr:    errors.New("bad request"),
			expectedErrorMsg: "failed to get validator index",
		},
		{
			name:                 "attester duties error",
			duties:               attesterDuties,
			validatorsCalled:     1,
			attesterDutiesCalled: 1,
			dutiesErr:            errors.New("bad request"),
			expectedErrorMsg:     "failed to get attester duties",
		},
		{
			name:                  "attestation data error",
			duties:                attesterDuties,
			validatorsCalled:      1,
			attesterDutiesCalled:  1,
			attestationDataCalled: 1,
			attestationDataErr:    errors.New("bad request"),
			expectedErrorMsg:      fmt.Sprintf("failed to get attestation data for slot=%d and committee_index=%d", slot, committeeIndex),
		},
		{
			name:                       "aggregate attestation error",
			duties:                     attesterDuties,
			validatorsCalled:           1,
			attesterDutiesCalled:       1,
			attestationDataCalled:      1,
			aggregateAttestationCalled: 1,
			aggregateAttestationErr:    errors.New("bad request"),
			expectedErrorMsg:           "bad request",
		},
		{
			name: "validator is not an aggregator",
			duties: []*validator.AttesterDuty{
				{
					Pubkey:          pubkeyStr,
					ValidatorIndex:  validatorIndex,
					Slot:            "123",
					CommitteeIndex:  "1",
					CommitteeLength: "64",
				},
			},
			validatorsCalled:     1,
			attesterDutiesCalled: 1,
			expectedErrorMsg:     "validator is not an aggregator",
		},
		{
			name:                 "no attester duties",
			duties:               []*validator.AttesterDuty{},
			validatorsCalled:     1,
			attesterDutiesCalled: 1,
			expectedErrorMsg:     fmt.Sprintf("no attester duty for the given slot %d", slot),
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			ctx := context.Background()
			jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

			// Call node syncing endpoint to check if head is optimistic.
			jsonRestHandler.EXPECT().Get(
				ctx,
				syncingEndpoint,
				&node.SyncStatusResponse{},
			).SetArg(
				2,
				node.SyncStatusResponse{
					Data: &node.SyncStatusResponseData{
						IsOptimistic: test.isOptimistic,
					},
				},
			).Return(
				test.syncingErr,
			).Times(1)

			valsReq := &beacon.GetValidatorsRequest{
				Ids:      []string{stringPubKey},
				Statuses: []string{},
			}
			valReqBytes, err := json.Marshal(valsReq)
			require.NoError(t, err)

			// Call validators endpoint to get validator index.
			jsonRestHandler.EXPECT().Post(
				ctx,
				validatorsEndpoint,
				nil,
				bytes.NewBuffer(valReqBytes),
				&beacon.GetValidatorsResponse{},
			).SetArg(
				4,
				beacon.GetValidatorsResponse{
					Data: []*beacon.ValidatorContainer{
						{
							Index:  validatorIndex,
							Status: "active_ongoing",
							Validator: &beacon.Validator{
								Pubkey: pubkeyStr,
							},
						},
					},
				},
			).Return(
				test.validatorsErr,
			).Times(test.validatorsCalled)

			// Call attester duties endpoint to get attester duties.
			validatorIndicesBytes, err := json.Marshal([]string{validatorIndex})
			require.NoError(t, err)
			jsonRestHandler.EXPECT().Post(
				ctx,
				fmt.Sprintf("%s/%d", attesterDutiesEndpoint, slots.ToEpoch(slot)),
				nil,
				bytes.NewBuffer(validatorIndicesBytes),
				&validator.GetAttesterDutiesResponse{},
			).SetArg(
				4,
				validator.GetAttesterDutiesResponse{
					Data: test.duties,
				},
			).Return(
				test.dutiesErr,
			).Times(test.attesterDutiesCalled)

			// Call attestation data to get attestation data root to query aggregate attestation.
			jsonRestHandler.EXPECT().Get(
				ctx,
				fmt.Sprintf("%s?committee_index=%d&slot=%d", attestationDataEndpoint, committeeIndex, slot),
				&validator.GetAttestationDataResponse{},
			).SetArg(
				2,
				attestationDataResponse,
			).Return(
				test.attestationDataErr,
			).Times(test.attestationDataCalled)

			// Call attestation data to get attestation data root to query aggregate attestation.
			jsonRestHandler.EXPECT().Get(
				ctx,
				fmt.Sprintf("%s?attestation_data_root=%s&slot=%d", aggregateAttestationEndpoint, hexutil.Encode(attestationDataRootBytes[:]), slot),
				&validator.AggregateAttestationResponse{},
			).SetArg(
				2,
				validator.AggregateAttestationResponse{
					Data: jsonifyAttestation(aggregateAttestation),
				},
			).Return(
				test.aggregateAttestationErr,
			).Times(test.aggregateAttestationCalled)

			pubkey, err := hexutil.Decode(pubkeyStr)
			require.NoError(t, err)

			slotSignatureBytes, err := hexutil.Decode(slotSignature)
			require.NoError(t, err)

			expectedResponse := &ethpb.AggregateSelectionResponse{
				AggregateAndProof: &ethpb.AggregateAttestationAndProof{
					AggregatorIndex: primitives.ValidatorIndex(55293),
					Aggregate:       aggregateAttestation,
					SelectionProof:  slotSignatureBytes,
				},
			}

			validatorClient := &beaconApiValidatorClient{
				jsonRestHandler: jsonRestHandler,
				stateValidatorsProvider: beaconApiStateValidatorsProvider{
					jsonRestHandler: jsonRestHandler,
				},
				dutiesProvider: beaconApiDutiesProvider{
					jsonRestHandler: jsonRestHandler,
				},
			}
			actualResponse, err := validatorClient.submitAggregateSelectionProof(ctx, &ethpb.AggregateSelectionRequest{
				Slot:           slot,
				CommitteeIndex: committeeIndex,
				PublicKey:      pubkey,
				SlotSignature:  slotSignatureBytes,
			})
			if test.expectedErrorMsg == "" {
				require.NoError(t, err)
				assert.DeepEqual(t, expectedResponse, actualResponse)
			} else {
				require.ErrorContains(t, test.expectedErrorMsg, err)
			}
		})
	}
}
