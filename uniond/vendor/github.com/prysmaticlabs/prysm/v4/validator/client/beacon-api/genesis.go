package beacon_api

import (
	"context"
	"net/http"
	"strconv"
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

type GenesisProvider interface {
	GetGenesis(ctx context.Context) (*beacon.Genesis, error)
}

type beaconApiGenesisProvider struct {
	jsonRestHandler JsonRestHandler
}

func (c beaconApiValidatorClient) waitForChainStart(ctx context.Context) (*ethpb.ChainStartResponse, error) {
	genesis, err := c.genesisProvider.GetGenesis(ctx)

	for err != nil {
		jsonErr := &httputil.DefaultJsonError{}
		httpNotFound := errors.As(err, &jsonErr) && jsonErr.Code == http.StatusNotFound
		if !httpNotFound {
			return nil, errors.Wrap(err, "failed to get genesis data")
		}

		// Error 404 means that the chain genesis info is not yet known, so we query it every second until it's ready
		select {
		case <-time.After(time.Second):
			genesis, err = c.genesisProvider.GetGenesis(ctx)
		case <-ctx.Done():
			return nil, errors.New("context canceled")
		}
	}

	genesisTime, err := strconv.ParseUint(genesis.GenesisTime, 10, 64)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to parse genesis time: %s", genesis.GenesisTime)
	}

	if !validRoot(genesis.GenesisValidatorsRoot) {
		return nil, errors.Errorf("invalid genesis validators root: %s", genesis.GenesisValidatorsRoot)
	}

	genesisValidatorRoot, err := hexutil.Decode(genesis.GenesisValidatorsRoot)
	if err != nil {
		return nil, errors.Wrap(err, "failed to decode genesis validators root")
	}

	chainStartResponse := &ethpb.ChainStartResponse{
		Started:               true,
		GenesisTime:           genesisTime,
		GenesisValidatorsRoot: genesisValidatorRoot,
	}

	return chainStartResponse, nil
}

// GetGenesis gets the genesis information from the beacon node via the /eth/v1/beacon/genesis endpoint
func (c beaconApiGenesisProvider) GetGenesis(ctx context.Context) (*beacon.Genesis, error) {
	genesisJson := &beacon.GetGenesisResponse{}
	if err := c.jsonRestHandler.Get(ctx, "/eth/v1/beacon/genesis", genesisJson); err != nil {
		return nil, err
	}

	if genesisJson.Data == nil {
		return nil, errors.New("genesis data is nil")
	}

	return genesisJson.Data, nil
}
