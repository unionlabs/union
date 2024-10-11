package validator

import (
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/lookup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
)

// Server defines a server implementation for HTTP endpoints, providing
// access data relevant to the Ethereum Beacon Chain.
type Server struct {
	GenesisTimeFetcher    blockchain.TimeFetcher
	SyncChecker           sync.Checker
	HeadFetcher           blockchain.HeadFetcher
	CoreService           *core.Service
	OptimisticModeFetcher blockchain.OptimisticModeFetcher
	Stater                lookup.Stater
	ChainInfoFetcher      blockchain.ChainInfoFetcher
	BeaconDB              db.ReadOnlyDatabase
	FinalizationFetcher   blockchain.FinalizationFetcher
}
