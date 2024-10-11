package beacon_api

import (
	"context"
	"fmt"
	neturl "net/url"
	"strconv"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/validator"
	validator2 "github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
	"github.com/prysmaticlabs/prysm/v4/validator/client/iface"
)

// NewPrysmBeaconChainClient returns implementation of iface.PrysmBeaconChainClient.
func NewPrysmBeaconChainClient(jsonRestHandler JsonRestHandler, nodeClient iface.NodeClient) iface.PrysmBeaconChainClient {
	return prysmBeaconChainClient{
		jsonRestHandler: jsonRestHandler,
		nodeClient:      nodeClient,
	}
}

type prysmBeaconChainClient struct {
	jsonRestHandler JsonRestHandler
	nodeClient      iface.NodeClient
}

func (c prysmBeaconChainClient) GetValidatorCount(ctx context.Context, stateID string, statuses []validator2.Status) ([]iface.ValidatorCount, error) {
	// Check node version for prysm beacon node as it is a custom endpoint for prysm beacon node.
	nodeVersion, err := c.nodeClient.GetVersion(ctx, nil)
	if err != nil {
		return nil, errors.Wrap(err, "failed to get node version")
	}

	if !strings.Contains(strings.ToLower(nodeVersion.Version), "prysm") {
		return nil, iface.ErrNotSupported
	}

	queryParams := neturl.Values{}
	for _, status := range statuses {
		queryParams.Add("status", status.String())
	}

	queryUrl := buildURL(fmt.Sprintf("/eth/v1/beacon/states/%s/validator_count", stateID), queryParams)

	var validatorCountResponse validator.CountResponse
	if err = c.jsonRestHandler.Get(ctx, queryUrl, &validatorCountResponse); err != nil {
		return nil, err
	}

	if validatorCountResponse.Data == nil {
		return nil, errors.New("validator count data is nil")
	}

	if len(statuses) != 0 && len(statuses) != len(validatorCountResponse.Data) {
		return nil, errors.New("mismatch between validator count data and the number of statuses provided")
	}

	var resp []iface.ValidatorCount
	for _, vc := range validatorCountResponse.Data {
		count, err := strconv.ParseUint(vc.Count, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse validator count %s", vc.Count)
		}

		resp = append(resp, iface.ValidatorCount{
			Status: vc.Status,
			Count:  count,
		})
	}

	return resp, nil
}
