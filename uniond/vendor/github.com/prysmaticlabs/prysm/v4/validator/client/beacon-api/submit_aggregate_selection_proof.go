package beacon_api

import (
	"context"
	"net/url"
	"strconv"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/validator"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func (c *beaconApiValidatorClient) submitAggregateSelectionProof(ctx context.Context, in *ethpb.AggregateSelectionRequest) (*ethpb.AggregateSelectionResponse, error) {
	isOptimistic, err := c.isOptimistic(ctx)
	if err != nil {
		return nil, err
	}

	// An optimistic validator MUST NOT participate in attestation. (i.e., sign across the DOMAIN_BEACON_ATTESTER, DOMAIN_SELECTION_PROOF or DOMAIN_AGGREGATE_AND_PROOF domains).
	if isOptimistic {
		return nil, errors.New("the node is currently optimistic and cannot serve validators")
	}

	validatorIndexResponse, err := c.validatorIndex(ctx, &ethpb.ValidatorIndexRequest{PublicKey: in.PublicKey})
	if err != nil {
		return nil, errors.Wrap(err, "failed to get validator index")
	}

	attesterDuties, err := c.dutiesProvider.GetAttesterDuties(ctx, slots.ToEpoch(in.Slot), []primitives.ValidatorIndex{validatorIndexResponse.Index})
	if err != nil {
		return nil, errors.Wrap(err, "failed to get attester duties")
	}

	if len(attesterDuties) == 0 {
		return nil, errors.Errorf("no attester duty for the given slot %d", in.Slot)
	}

	// First attester duty is required since we requested attester duties for one validator index.
	attesterDuty := attesterDuties[0]

	committeeLen, err := strconv.ParseUint(attesterDuty.CommitteeLength, 10, 64)
	if err != nil {
		return nil, errors.Wrap(err, "failed to parse committee length")
	}

	isAggregator, err := helpers.IsAggregator(committeeLen, in.SlotSignature)
	if err != nil {
		return nil, errors.Wrap(err, "failed to get aggregator status")
	}
	if !isAggregator {
		return nil, errors.New("validator is not an aggregator")
	}

	attestationData, err := c.getAttestationData(ctx, in.Slot, in.CommitteeIndex)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get attestation data for slot=%d and committee_index=%d", in.Slot, in.CommitteeIndex)
	}

	attestationDataRoot, err := attestationData.HashTreeRoot()
	if err != nil {
		return nil, errors.Wrap(err, "failed to calculate attestation data root")
	}

	aggregateAttestationResponse, err := c.getAggregateAttestation(ctx, in.Slot, attestationDataRoot[:])
	if err != nil {
		return nil, err
	}

	aggregatedAttestation, err := convertAttestationToProto(aggregateAttestationResponse.Data)
	if err != nil {
		return nil, errors.Wrap(err, "failed to convert aggregate attestation json to proto")
	}

	return &ethpb.AggregateSelectionResponse{
		AggregateAndProof: &ethpb.AggregateAttestationAndProof{
			AggregatorIndex: validatorIndexResponse.Index,
			Aggregate:       aggregatedAttestation,
			SelectionProof:  in.SlotSignature,
		},
	}, nil
}

func (c *beaconApiValidatorClient) getAggregateAttestation(
	ctx context.Context,
	slot primitives.Slot,
	attestationDataRoot []byte,
) (*validator.AggregateAttestationResponse, error) {
	params := url.Values{}
	params.Add("slot", strconv.FormatUint(uint64(slot), 10))
	params.Add("attestation_data_root", hexutil.Encode(attestationDataRoot))
	endpoint := buildURL("/eth/v1/validator/aggregate_attestation", params)

	var aggregateAttestationResponse validator.AggregateAttestationResponse
	if err := c.jsonRestHandler.Get(ctx, endpoint, &aggregateAttestationResponse); err != nil {
		return nil, err
	}

	return &aggregateAttestationResponse, nil
}
