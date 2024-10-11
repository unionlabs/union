package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"reflect"
	"strconv"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/protobuf/ptypes/empty"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/validator"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/prysmaticlabs/prysm/v4/validator/client/iface"
)

type beaconApiBeaconChainClient struct {
	fallbackClient          iface.BeaconChainClient
	jsonRestHandler         JsonRestHandler
	stateValidatorsProvider StateValidatorsProvider
}

const getValidatorPerformanceEndpoint = "/prysm/validators/performance"

func (c beaconApiBeaconChainClient) getHeadBlockHeaders(ctx context.Context) (*beacon.GetBlockHeaderResponse, error) {
	blockHeader := beacon.GetBlockHeaderResponse{}
	err := c.jsonRestHandler.Get(ctx, "/eth/v1/beacon/headers/head", &blockHeader)
	if err != nil {
		return nil, err
	}

	if blockHeader.Data == nil || blockHeader.Data.Header == nil {
		return nil, errors.New("block header data is nil")
	}

	if blockHeader.Data.Header.Message == nil {
		return nil, errors.New("block header message is nil")
	}

	return &blockHeader, nil
}

func (c beaconApiBeaconChainClient) GetChainHead(ctx context.Context, _ *empty.Empty) (*ethpb.ChainHead, error) {
	const endpoint = "/eth/v1/beacon/states/head/finality_checkpoints"

	finalityCheckpoints := beacon.GetFinalityCheckpointsResponse{}
	if err := c.jsonRestHandler.Get(ctx, endpoint, &finalityCheckpoints); err != nil {
		return nil, err
	}

	if finalityCheckpoints.Data == nil {
		return nil, errors.New("finality checkpoints data is nil")
	}

	if finalityCheckpoints.Data.Finalized == nil {
		return nil, errors.New("finalized checkpoint is nil")
	}

	finalizedEpoch, err := strconv.ParseUint(finalityCheckpoints.Data.Finalized.Epoch, 10, 64)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to parse finalized epoch `%s`", finalityCheckpoints.Data.Finalized.Epoch)
	}

	finalizedSlot, err := slots.EpochStart(primitives.Epoch(finalizedEpoch))
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get first slot for epoch `%d`", finalizedEpoch)
	}

	finalizedRoot, err := hexutil.Decode(finalityCheckpoints.Data.Finalized.Root)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to decode finalized checkpoint root `%s`", finalityCheckpoints.Data.Finalized.Root)
	}

	if finalityCheckpoints.Data.CurrentJustified == nil {
		return nil, errors.New("current justified checkpoint is nil")
	}

	justifiedEpoch, err := strconv.ParseUint(finalityCheckpoints.Data.CurrentJustified.Epoch, 10, 64)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to parse current justified checkpoint epoch `%s`", finalityCheckpoints.Data.CurrentJustified.Epoch)
	}

	justifiedSlot, err := slots.EpochStart(primitives.Epoch(justifiedEpoch))
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get first slot for epoch `%d`", justifiedEpoch)
	}

	justifiedRoot, err := hexutil.Decode(finalityCheckpoints.Data.CurrentJustified.Root)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to decode current justified checkpoint root `%s`", finalityCheckpoints.Data.CurrentJustified.Root)
	}

	if finalityCheckpoints.Data.PreviousJustified == nil {
		return nil, errors.New("previous justified checkpoint is nil")
	}

	previousJustifiedEpoch, err := strconv.ParseUint(finalityCheckpoints.Data.PreviousJustified.Epoch, 10, 64)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to parse previous justified checkpoint epoch `%s`", finalityCheckpoints.Data.PreviousJustified.Epoch)
	}

	previousJustifiedSlot, err := slots.EpochStart(primitives.Epoch(previousJustifiedEpoch))
	if err != nil {
		return nil, errors.Wrapf(err, "failed to get first slot for epoch `%d`", previousJustifiedEpoch)
	}

	previousJustifiedRoot, err := hexutil.Decode(finalityCheckpoints.Data.PreviousJustified.Root)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to decode previous justified checkpoint root `%s`", finalityCheckpoints.Data.PreviousJustified.Root)
	}

	blockHeader, err := c.getHeadBlockHeaders(ctx)
	if err != nil {
		return nil, errors.Wrap(err, "failed to get head block headers")
	}

	headSlot, err := strconv.ParseUint(blockHeader.Data.Header.Message.Slot, 10, 64)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to parse head block slot `%s`", blockHeader.Data.Header.Message.Slot)
	}

	headEpoch := slots.ToEpoch(primitives.Slot(headSlot))

	headBlockRoot, err := hexutil.Decode(blockHeader.Data.Root)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to decode head block root `%s`", blockHeader.Data.Root)
	}

	return &ethpb.ChainHead{
		HeadSlot:                   primitives.Slot(headSlot),
		HeadEpoch:                  headEpoch,
		HeadBlockRoot:              headBlockRoot,
		FinalizedSlot:              finalizedSlot,
		FinalizedEpoch:             primitives.Epoch(finalizedEpoch),
		FinalizedBlockRoot:         finalizedRoot,
		JustifiedSlot:              justifiedSlot,
		JustifiedEpoch:             primitives.Epoch(justifiedEpoch),
		JustifiedBlockRoot:         justifiedRoot,
		PreviousJustifiedSlot:      previousJustifiedSlot,
		PreviousJustifiedEpoch:     primitives.Epoch(previousJustifiedEpoch),
		PreviousJustifiedBlockRoot: previousJustifiedRoot,
		OptimisticStatus:           blockHeader.ExecutionOptimistic,
	}, nil
}

func (c beaconApiBeaconChainClient) ListValidatorBalances(ctx context.Context, in *ethpb.ListValidatorBalancesRequest) (*ethpb.ValidatorBalances, error) {
	if c.fallbackClient != nil {
		return c.fallbackClient.ListValidatorBalances(ctx, in)
	}

	// TODO: Implement me
	panic("beaconApiBeaconChainClient.ListValidatorBalances is not implemented. To use a fallback client, pass a fallback client as the last argument of NewBeaconApiBeaconChainClientWithFallback.")
}

func (c beaconApiBeaconChainClient) ListValidators(ctx context.Context, in *ethpb.ListValidatorsRequest) (*ethpb.Validators, error) {
	pageSize := in.PageSize

	// We follow the gRPC behavior here, which returns a maximum of 250 results when pageSize == 0
	if pageSize == 0 {
		pageSize = 250
	}

	var pageToken uint64
	var err error

	if in.PageToken != "" {
		if pageToken, err = strconv.ParseUint(in.PageToken, 10, 64); err != nil {
			return nil, errors.Wrapf(err, "failed to parse page token `%s`", in.PageToken)
		}
	}

	var statuses []string
	if in.Active {
		statuses = []string{"active"}
	}

	pubkeys := make([]string, len(in.PublicKeys))
	for idx, pubkey := range in.PublicKeys {
		pubkeys[idx] = hexutil.Encode(pubkey)
	}

	var stateValidators *beacon.GetValidatorsResponse
	var epoch primitives.Epoch

	switch queryFilter := in.QueryFilter.(type) {
	case *ethpb.ListValidatorsRequest_Epoch:
		slot, err := slots.EpochStart(queryFilter.Epoch)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to get first slot for epoch `%d`", queryFilter.Epoch)
		}
		if stateValidators, err = c.stateValidatorsProvider.GetStateValidatorsForSlot(ctx, slot, pubkeys, in.Indices, statuses); err != nil {
			return nil, errors.Wrapf(err, "failed to get state validators for slot `%d`", slot)
		}
		epoch = slots.ToEpoch(slot)
	case *ethpb.ListValidatorsRequest_Genesis:
		if stateValidators, err = c.stateValidatorsProvider.GetStateValidatorsForSlot(ctx, 0, pubkeys, in.Indices, statuses); err != nil {
			return nil, errors.Wrapf(err, "failed to get genesis state validators")
		}
		epoch = 0
	case nil:
		if stateValidators, err = c.stateValidatorsProvider.GetStateValidatorsForHead(ctx, pubkeys, in.Indices, statuses); err != nil {
			return nil, errors.Wrap(err, "failed to get head state validators")
		}

		blockHeader, err := c.getHeadBlockHeaders(ctx)
		if err != nil {
			return nil, errors.Wrap(err, "failed to get head block headers")
		}

		slot, err := strconv.ParseUint(blockHeader.Data.Header.Message.Slot, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse header slot `%s`", blockHeader.Data.Header.Message.Slot)
		}

		epoch = slots.ToEpoch(primitives.Slot(slot))
	default:
		return nil, errors.Errorf("unsupported query filter type `%v`", reflect.TypeOf(queryFilter))
	}

	if stateValidators.Data == nil {
		return nil, errors.New("state validators data is nil")
	}

	start := pageToken * uint64(pageSize)
	if start > uint64(len(stateValidators.Data)) {
		start = uint64(len(stateValidators.Data))
	}

	end := start + uint64(pageSize)
	if end > uint64(len(stateValidators.Data)) {
		end = uint64(len(stateValidators.Data))
	}

	validators := make([]*ethpb.Validators_ValidatorContainer, end-start)
	for idx := start; idx < end; idx++ {
		stateValidator := stateValidators.Data[idx]

		if stateValidator.Validator == nil {
			return nil, errors.Errorf("state validator at index `%d` is nil", idx)
		}

		pubkey, err := hexutil.Decode(stateValidator.Validator.Pubkey)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode validator pubkey `%s`", stateValidator.Validator.Pubkey)
		}

		withdrawalCredentials, err := hexutil.Decode(stateValidator.Validator.WithdrawalCredentials)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode validator withdrawal credentials `%s`", stateValidator.Validator.WithdrawalCredentials)
		}

		effectiveBalance, err := strconv.ParseUint(stateValidator.Validator.EffectiveBalance, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse validator effective balance `%s`", stateValidator.Validator.EffectiveBalance)
		}

		validatorIndex, err := strconv.ParseUint(stateValidator.Index, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse validator index `%s`", stateValidator.Index)
		}

		activationEligibilityEpoch, err := strconv.ParseUint(stateValidator.Validator.ActivationEligibilityEpoch, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse validator activation eligibility epoch `%s`", stateValidator.Validator.ActivationEligibilityEpoch)
		}

		activationEpoch, err := strconv.ParseUint(stateValidator.Validator.ActivationEpoch, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse validator activation epoch `%s`", stateValidator.Validator.ActivationEpoch)
		}

		exitEpoch, err := strconv.ParseUint(stateValidator.Validator.ExitEpoch, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse validator exit epoch `%s`", stateValidator.Validator.ExitEpoch)
		}

		withdrawableEpoch, err := strconv.ParseUint(stateValidator.Validator.WithdrawableEpoch, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse validator withdrawable epoch `%s`", stateValidator.Validator.WithdrawableEpoch)
		}

		validators[idx-start] = &ethpb.Validators_ValidatorContainer{
			Index: primitives.ValidatorIndex(validatorIndex),
			Validator: &ethpb.Validator{
				PublicKey:                  pubkey,
				WithdrawalCredentials:      withdrawalCredentials,
				EffectiveBalance:           effectiveBalance,
				Slashed:                    stateValidator.Validator.Slashed,
				ActivationEligibilityEpoch: primitives.Epoch(activationEligibilityEpoch),
				ActivationEpoch:            primitives.Epoch(activationEpoch),
				ExitEpoch:                  primitives.Epoch(exitEpoch),
				WithdrawableEpoch:          primitives.Epoch(withdrawableEpoch),
			},
		}
	}

	var nextPageToken string
	if end < uint64(len(stateValidators.Data)) {
		nextPageToken = strconv.FormatUint(pageToken+1, 10)
	}

	return &ethpb.Validators{
		TotalSize:     int32(len(stateValidators.Data)),
		Epoch:         epoch,
		ValidatorList: validators,
		NextPageToken: nextPageToken,
	}, nil
}

func (c beaconApiBeaconChainClient) GetValidatorQueue(ctx context.Context, in *empty.Empty) (*ethpb.ValidatorQueue, error) {
	if c.fallbackClient != nil {
		return c.fallbackClient.GetValidatorQueue(ctx, in)
	}

	// TODO: Implement me
	panic("beaconApiBeaconChainClient.GetValidatorQueue is not implemented. To use a fallback client, pass a fallback client as the last argument of NewBeaconApiBeaconChainClientWithFallback.")
}

func (c beaconApiBeaconChainClient) GetValidatorPerformance(ctx context.Context, in *ethpb.ValidatorPerformanceRequest) (*ethpb.ValidatorPerformanceResponse, error) {
	request, err := json.Marshal(validator.PerformanceRequest{
		PublicKeys: in.PublicKeys,
		Indices:    in.Indices,
	})
	if err != nil {
		return nil, errors.Wrap(err, "failed to marshal request")
	}
	resp := &validator.PerformanceResponse{}
	if err = c.jsonRestHandler.Post(ctx, getValidatorPerformanceEndpoint, nil, bytes.NewBuffer(request), resp); err != nil {
		return nil, err
	}

	return &ethpb.ValidatorPerformanceResponse{
		CurrentEffectiveBalances:      resp.CurrentEffectiveBalances,
		CorrectlyVotedSource:          resp.CorrectlyVotedSource,
		CorrectlyVotedTarget:          resp.CorrectlyVotedTarget,
		CorrectlyVotedHead:            resp.CorrectlyVotedHead,
		BalancesBeforeEpochTransition: resp.BalancesBeforeEpochTransition,
		BalancesAfterEpochTransition:  resp.BalancesAfterEpochTransition,
		MissingValidators:             resp.MissingValidators,
		PublicKeys:                    resp.PublicKeys,
		InactivityScores:              resp.InactivityScores,
	}, nil
}

func (c beaconApiBeaconChainClient) GetValidatorParticipation(ctx context.Context, in *ethpb.GetValidatorParticipationRequest) (*ethpb.ValidatorParticipationResponse, error) {
	if c.fallbackClient != nil {
		return c.fallbackClient.GetValidatorParticipation(ctx, in)
	}

	// TODO: Implement me
	panic("beaconApiBeaconChainClient.GetValidatorParticipation is not implemented. To use a fallback client, pass a fallback client as the last argument of NewBeaconApiBeaconChainClientWithFallback.")
}

func NewBeaconApiBeaconChainClientWithFallback(jsonRestHandler JsonRestHandler, fallbackClient iface.BeaconChainClient) iface.BeaconChainClient {
	return &beaconApiBeaconChainClient{
		jsonRestHandler:         jsonRestHandler,
		fallbackClient:          fallbackClient,
		stateValidatorsProvider: beaconApiStateValidatorsProvider{jsonRestHandler: jsonRestHandler},
	}
}
