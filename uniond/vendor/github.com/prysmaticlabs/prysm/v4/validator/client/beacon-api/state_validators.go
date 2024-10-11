package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"strconv"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

type StateValidatorsProvider interface {
	GetStateValidators(context.Context, []string, []primitives.ValidatorIndex, []string) (*beacon.GetValidatorsResponse, error)
	GetStateValidatorsForSlot(context.Context, primitives.Slot, []string, []primitives.ValidatorIndex, []string) (*beacon.GetValidatorsResponse, error)
	GetStateValidatorsForHead(context.Context, []string, []primitives.ValidatorIndex, []string) (*beacon.GetValidatorsResponse, error)
}

type beaconApiStateValidatorsProvider struct {
	jsonRestHandler JsonRestHandler
}

func (c beaconApiStateValidatorsProvider) GetStateValidators(
	ctx context.Context,
	stringPubkeys []string,
	indexes []primitives.ValidatorIndex,
	statuses []string,
) (*beacon.GetValidatorsResponse, error) {
	stringIndices := convertValidatorIndicesToStrings(indexes)
	return c.getStateValidatorsHelper(ctx, "/eth/v1/beacon/states/head/validators", append(stringIndices, stringPubkeys...), statuses)
}

func (c beaconApiStateValidatorsProvider) GetStateValidatorsForSlot(
	ctx context.Context,
	slot primitives.Slot,
	stringPubkeys []string,
	indices []primitives.ValidatorIndex,
	statuses []string,
) (*beacon.GetValidatorsResponse, error) {
	stringIndices := convertValidatorIndicesToStrings(indices)
	url := fmt.Sprintf("/eth/v1/beacon/states/%d/validators", slot)
	return c.getStateValidatorsHelper(ctx, url, append(stringIndices, stringPubkeys...), statuses)
}

func (c beaconApiStateValidatorsProvider) GetStateValidatorsForHead(
	ctx context.Context,
	stringPubkeys []string,
	indices []primitives.ValidatorIndex,
	statuses []string,
) (*beacon.GetValidatorsResponse, error) {
	stringIndices := convertValidatorIndicesToStrings(indices)
	return c.getStateValidatorsHelper(ctx, "/eth/v1/beacon/states/head/validators", append(stringIndices, stringPubkeys...), statuses)
}

func convertValidatorIndicesToStrings(indices []primitives.ValidatorIndex) []string {
	var result []string
	indicesSet := make(map[primitives.ValidatorIndex]struct{}, len(indices))
	for _, index := range indices {
		if _, ok := indicesSet[index]; !ok {
			indicesSet[index] = struct{}{}
			result = append(result, strconv.FormatUint(uint64(index), 10))
		}
	}
	return result
}

func (c beaconApiStateValidatorsProvider) getStateValidatorsHelper(
	ctx context.Context,
	endpoint string,
	vals []string,
	statuses []string,
) (*beacon.GetValidatorsResponse, error) {
	req := beacon.GetValidatorsRequest{
		Ids:      []string{},
		Statuses: []string{},
	}
	req.Statuses = append(req.Statuses, statuses...)

	valSet := make(map[string]struct{}, len(vals))
	for _, v := range vals {
		if _, ok := valSet[v]; !ok {
			valSet[v] = struct{}{}
			req.Ids = append(req.Ids, v)
		}
	}

	reqBytes, err := json.Marshal(req)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to marshal request into JSON")
	}
	stateValidatorsJson := &beacon.GetValidatorsResponse{}
	if err = c.jsonRestHandler.Post(ctx, endpoint, nil, bytes.NewBuffer(reqBytes), stateValidatorsJson); err != nil {
		return nil, err
	}

	if stateValidatorsJson.Data == nil {
		return nil, errors.New("stateValidatorsJson.Data is nil")
	}

	return stateValidatorsJson, nil
}
