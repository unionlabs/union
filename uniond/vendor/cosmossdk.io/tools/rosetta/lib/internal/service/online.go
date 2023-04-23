package service

import (
	"context"
	"fmt"
	"os"
	"time"

	"github.com/coinbase/rosetta-sdk-go/types"

	crgerrs "cosmossdk.io/tools/rosetta/lib/errors"
	crgtypes "cosmossdk.io/tools/rosetta/lib/types"
	"github.com/cometbft/cometbft/libs/log"
)

// genesisBlockFetchTimeout defines a timeout to fetch the genesis block
const (
	genesisBlockFetchTimeout = 15 * time.Second
	genesisHashEnv           = "GENESIS_HASH"
)

// NewOnlineNetwork builds a single network adapter.
// It will get the Genesis block on the beginning to avoid calling it everytime.
func NewOnlineNetwork(network *types.NetworkIdentifier, client crgtypes.Client, logger log.Logger) (crgtypes.API, error) {
	ctx, cancel := context.WithTimeout(context.Background(), genesisBlockFetchTimeout)
	defer cancel()

	var genesisHeight int64 = -1 // to use initial_height in genesis.json
	block, err := client.BlockByHeight(ctx, &genesisHeight)
	if err != nil {
		return OnlineNetwork{}, err
	}

	// Get genesis hash from ENV. It should be set by an external script since is not possible to get
	// using tendermint API
	genesisHash := os.Getenv(genesisHashEnv)
	if genesisHash == "" {
		logger.Error(fmt.Sprintf("Genesis hash env '%s' is not properly set!", genesisHashEnv))
	}

	block.Block.Hash = genesisHash

	return OnlineNetwork{
		client:                 client,
		network:                network,
		networkOptions:         networkOptionsFromClient(client, block.Block),
		genesisBlockIdentifier: block.Block,
	}, nil
}

// OnlineNetwork groups together all the components required for the full rosetta implementation
type OnlineNetwork struct {
	client crgtypes.Client // used to query cosmos app + tendermint

	network        *types.NetworkIdentifier      // identifies the network, it's static
	networkOptions *types.NetworkOptionsResponse // identifies the network options, it's static

	genesisBlockIdentifier *types.BlockIdentifier // identifies genesis block, it's static
}

// AccountsCoins - relevant only for UTXO based chain
// see https://www.rosetta-api.org/docs/AccountApi.html#accountcoins
func (o OnlineNetwork) AccountCoins(_ context.Context, _ *types.AccountCoinsRequest) (*types.AccountCoinsResponse, *types.Error) {
	return nil, crgerrs.ToRosetta(crgerrs.ErrOffline)
}

// networkOptionsFromClient builds network options given the client
func networkOptionsFromClient(client crgtypes.Client, genesisBlock *types.BlockIdentifier) *types.NetworkOptionsResponse {
	var tsi *int64
	if genesisBlock != nil {
		tsi = &(genesisBlock.Index)
	}
	return &types.NetworkOptionsResponse{
		Version: &types.Version{
			RosettaVersion: crgtypes.SpecVersion,
			NodeVersion:    client.Version(),
		},
		Allow: &types.Allow{
			OperationStatuses:       client.OperationStatuses(),
			OperationTypes:          client.SupportedOperations(),
			Errors:                  crgerrs.SealAndListErrors(),
			HistoricalBalanceLookup: true,
			TimestampStartIndex:     tsi,
		},
	}
}
