package block

import (
	"errors"
	"fmt"

	dbm "github.com/cometbft/cometbft-db"

	"github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/state/indexer"
	blockidxkv "github.com/cometbft/cometbft/state/indexer/block/kv"
	blockidxnull "github.com/cometbft/cometbft/state/indexer/block/null"
	"github.com/cometbft/cometbft/state/indexer/sink/psql"
	"github.com/cometbft/cometbft/state/txindex"
	"github.com/cometbft/cometbft/state/txindex/kv"
	"github.com/cometbft/cometbft/state/txindex/null"
)

// EventSinksFromConfig constructs a slice of indexer.EventSink using the provided
// configuration.
//
//nolint:lll
func IndexerFromConfig(cfg *config.Config, dbProvider config.DBProvider, chainID string) (txindex.TxIndexer, indexer.BlockIndexer, error) {
	switch cfg.TxIndex.Indexer {
	case "kv":
		store, err := dbProvider(&config.DBContext{ID: "tx_index", Config: cfg})
		if err != nil {
			return nil, nil, err
		}

		return kv.NewTxIndex(store), blockidxkv.New(dbm.NewPrefixDB(store, []byte("block_events"))), nil

	case "psql":
		conn := cfg.TxIndex.PsqlConn
		if conn == "" {
			return nil, nil, errors.New("the psql connection settings cannot be empty")
		}
		es, err := psql.NewEventSink(cfg.TxIndex.PsqlConn, chainID)
		if err != nil {
			return nil, nil, fmt.Errorf("creating psql indexer: %w", err)
		}
		return es.TxIndexer(), es.BlockIndexer(), nil

	default:
		return &null.TxIndex{}, &blockidxnull.BlockerIndexer{}, nil
	}
}
