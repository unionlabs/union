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
func IndexerFromConfig(cfg *config.Config, dbProvider config.DBProvider, chainID string) (
	txIdx txindex.TxIndexer, blockIdx indexer.BlockIndexer, allIndexersDisabled bool, err error,
) {
	switch cfg.TxIndex.Indexer {
	case "kv":
		store, err := dbProvider(&config.DBContext{ID: "tx_index", Config: cfg})
		if err != nil {
			return nil, nil, false, err
		}

		return kv.NewTxIndex(store),
			blockidxkv.New(dbm.NewPrefixDB(store, []byte("block_events")),
				blockidxkv.WithCompaction(cfg.Storage.Compact, cfg.Storage.CompactionInterval)),
			false,
			nil

	case "psql":
		conn := cfg.TxIndex.PsqlConn
		if conn == "" {
			return nil, nil, false, errors.New("the psql connection settings cannot be empty")
		}
		opts := []psql.EventSinkOption{}

		txIndexCfg := cfg.TxIndex
		if txIndexCfg.TableBlocks != "" {
			opts = append(opts, psql.WithTableBlocks(txIndexCfg.TableBlocks))
		}

		if txIndexCfg.TableTxResults != "" {
			opts = append(opts, psql.WithTableTxResults(txIndexCfg.TableTxResults))
		}

		if txIndexCfg.TableEvents != "" {
			opts = append(opts, psql.WithTableEvents(txIndexCfg.TableEvents))
		}

		if txIndexCfg.TableAttributes != "" {
			opts = append(opts, psql.WithTableAttributes(txIndexCfg.TableAttributes))
		}

		es, err := psql.NewEventSink(cfg.TxIndex.PsqlConn, chainID, opts...)
		if err != nil {
			return nil, nil, false, fmt.Errorf("creating psql indexer: %w", err)
		}
		return es.TxIndexer(), es.BlockIndexer(), false, nil

	default:
		return &null.TxIndex{}, &blockidxnull.BlockerIndexer{}, true, nil
	}
}
