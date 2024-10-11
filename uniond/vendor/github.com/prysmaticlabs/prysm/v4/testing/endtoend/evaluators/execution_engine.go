package evaluators

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/policies"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"google.golang.org/grpc"
)

// OptimisticSyncEnabled checks that the node is in an optimistic state.
var OptimisticSyncEnabled = types.Evaluator{
	Name:       "optimistic_sync_at_epoch_%d",
	Policy:     policies.AllEpochs,
	Evaluation: optimisticSyncEnabled,
}

func optimisticSyncEnabled(_ *types.EvaluationContext, conns ...*grpc.ClientConn) error {
	for nodeIndex := range conns {
		path := fmt.Sprintf("http://localhost:%d/eth/v1/beacon/blinded_blocks/head", params.TestParams.Ports.PrysmBeaconNodeGatewayPort+nodeIndex)
		resp := beacon.GetBlockV2Response{}
		httpResp, err := http.Get(path) // #nosec G107 -- path can't be constant because it depends on port param and node index
		if err != nil {
			return err
		}
		if httpResp.StatusCode != http.StatusOK {
			e := httputil.DefaultJsonError{}
			if err = json.NewDecoder(httpResp.Body).Decode(&e); err != nil {
				return err
			}
			return fmt.Errorf("%s (status code %d)", e.Message, e.Code)
		}
		if err = json.NewDecoder(httpResp.Body).Decode(&resp); err != nil {
			return err
		}
		headSlot, err := retrieveHeadSlot(&resp)
		if err != nil {
			return err
		}
		currEpoch := slots.ToEpoch(primitives.Slot(headSlot))
		startSlot, err := slots.EpochStart(currEpoch)
		if err != nil {
			return err
		}
		for i := startSlot; i <= primitives.Slot(headSlot); i++ {
			path = fmt.Sprintf("http://localhost:%d/eth/v1/beacon/blinded_blocks/%d", params.TestParams.Ports.PrysmBeaconNodeGatewayPort+nodeIndex, i)
			resp = beacon.GetBlockV2Response{}
			httpResp, err = http.Get(path) // #nosec G107 -- path can't be constant because it depends on port param and node index
			if err != nil {
				return err
			}
			if httpResp.StatusCode == http.StatusNotFound {
				// Continue in the event of non-existent blocks.
				continue
			}
			if httpResp.StatusCode != http.StatusOK {
				e := httputil.DefaultJsonError{}
				if err = json.NewDecoder(httpResp.Body).Decode(&e); err != nil {
					return err
				}
				return fmt.Errorf("%s (status code %d)", e.Message, e.Code)
			}
			if err = json.NewDecoder(httpResp.Body).Decode(&resp); err != nil {
				return err
			}
			if !resp.ExecutionOptimistic {
				return errors.New("expected block to be optimistic, but it is not")
			}
		}
	}
	return nil
}

func retrieveHeadSlot(resp *beacon.GetBlockV2Response) (uint64, error) {
	var headSlot uint64
	var err error
	switch resp.Version {
	case version.String(version.Phase0):
		b := &shared.BeaconBlock{}
		if err := json.Unmarshal(resp.Data.Message, b); err != nil {
			return 0, err
		}
		headSlot, err = strconv.ParseUint(b.Slot, 10, 64)
		if err != nil {
			return 0, err
		}
	case version.String(version.Altair):
		b := &shared.BeaconBlockAltair{}
		if err := json.Unmarshal(resp.Data.Message, b); err != nil {
			return 0, err
		}
		headSlot, err = strconv.ParseUint(b.Slot, 10, 64)
		if err != nil {
			return 0, err
		}
	case version.String(version.Bellatrix):
		b := &shared.BeaconBlockBellatrix{}
		if err := json.Unmarshal(resp.Data.Message, b); err != nil {
			return 0, err
		}
		headSlot, err = strconv.ParseUint(b.Slot, 10, 64)
		if err != nil {
			return 0, err
		}
	case version.String(version.Capella):
		b := &shared.BeaconBlockCapella{}
		if err := json.Unmarshal(resp.Data.Message, b); err != nil {
			return 0, err
		}
		headSlot, err = strconv.ParseUint(b.Slot, 10, 64)
		if err != nil {
			return 0, err
		}
	case version.String(version.Deneb):
		b := &shared.BeaconBlockDeneb{}
		if err := json.Unmarshal(resp.Data.Message, b); err != nil {
			return 0, err
		}
		headSlot, err = strconv.ParseUint(b.Slot, 10, 64)
		if err != nil {
			return 0, err
		}
	default:
		return 0, errors.New("no valid block type retrieved")
	}
	return headSlot, nil
}
