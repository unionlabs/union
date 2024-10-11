package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"testing"

	"github.com/golang/mock/gomock"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
	test_helpers "github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/test-helpers"
)

func TestProposeAttestation(t *testing.T) {
	attestation := &ethpb.Attestation{
		AggregationBits: test_helpers.FillByteSlice(4, 74),
		Data: &ethpb.AttestationData{
			Slot:            75,
			CommitteeIndex:  76,
			BeaconBlockRoot: test_helpers.FillByteSlice(32, 38),
			Source: &ethpb.Checkpoint{
				Epoch: 78,
				Root:  test_helpers.FillByteSlice(32, 79),
			},
			Target: &ethpb.Checkpoint{
				Epoch: 80,
				Root:  test_helpers.FillByteSlice(32, 81),
			},
		},
		Signature: test_helpers.FillByteSlice(96, 82),
	}

	tests := []struct {
		name                 string
		attestation          *ethpb.Attestation
		expectedErrorMessage string
		endpointError        error
		endpointCall         int
	}{
		{
			name:         "valid",
			attestation:  attestation,
			endpointCall: 1,
		},
		{
			name:                 "nil attestation",
			expectedErrorMessage: "attestation is nil",
		},
		{
			name: "nil attestation data",
			attestation: &ethpb.Attestation{
				AggregationBits: test_helpers.FillByteSlice(4, 74),
				Signature:       test_helpers.FillByteSlice(96, 82),
			},
			expectedErrorMessage: "attestation data is nil",
		},
		{
			name: "nil source checkpoint",
			attestation: &ethpb.Attestation{
				AggregationBits: test_helpers.FillByteSlice(4, 74),
				Data: &ethpb.AttestationData{
					Target: &ethpb.Checkpoint{},
				},
				Signature: test_helpers.FillByteSlice(96, 82),
			},
			expectedErrorMessage: "source/target in attestation data is nil",
		},
		{
			name: "nil target checkpoint",
			attestation: &ethpb.Attestation{
				AggregationBits: test_helpers.FillByteSlice(4, 74),
				Data: &ethpb.AttestationData{
					Source: &ethpb.Checkpoint{},
				},
				Signature: test_helpers.FillByteSlice(96, 82),
			},
			expectedErrorMessage: "source/target in attestation data is nil",
		},
		{
			name: "nil aggregation bits",
			attestation: &ethpb.Attestation{
				Data: &ethpb.AttestationData{
					Source: &ethpb.Checkpoint{},
					Target: &ethpb.Checkpoint{},
				},
				Signature: test_helpers.FillByteSlice(96, 82),
			},
			expectedErrorMessage: "attestation aggregation bits is empty",
		},
		{
			name: "nil signature",
			attestation: &ethpb.Attestation{
				AggregationBits: test_helpers.FillByteSlice(4, 74),
				Data: &ethpb.AttestationData{
					Source: &ethpb.Checkpoint{},
					Target: &ethpb.Checkpoint{},
				},
			},
			expectedErrorMessage: "attestation signature is empty",
		},
		{
			name:                 "bad request",
			attestation:          attestation,
			expectedErrorMessage: "bad request",
			endpointError:        errors.New("bad request"),
			endpointCall:         1,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			ctrl := gomock.NewController(t)
			jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

			var marshalledAttestations []byte
			if checkNilAttestation(test.attestation) == nil {
				b, err := json.Marshal(jsonifyAttestations([]*ethpb.Attestation{test.attestation}))
				require.NoError(t, err)
				marshalledAttestations = b
			}

			ctx := context.Background()

			jsonRestHandler.EXPECT().Post(
				ctx,
				"/eth/v1/beacon/pool/attestations",
				nil,
				bytes.NewBuffer(marshalledAttestations),
				nil,
			).Return(
				test.endpointError,
			).Times(test.endpointCall)

			validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
			proposeResponse, err := validatorClient.proposeAttestation(ctx, test.attestation)
			if test.expectedErrorMessage != "" {
				require.ErrorContains(t, test.expectedErrorMessage, err)
				return
			}

			require.NoError(t, err)
			require.NotNil(t, proposeResponse)

			expectedAttestationDataRoot, err := attestation.Data.HashTreeRoot()
			require.NoError(t, err)

			// Make sure that the attestation data root is set
			assert.DeepEqual(t, expectedAttestationDataRoot[:], proposeResponse.AttestationDataRoot)
		})
	}
}
