package commands

import (
	"errors"
	"fmt"
	"strings"

	"github.com/spf13/cobra"

	dbm "github.com/cometbft/cometbft-db"

	abcitypes "github.com/cometbft/cometbft/abci/types"
	cmtcfg "github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/libs/progressbar"
	"github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/state/indexer"
	blockidxkv "github.com/cometbft/cometbft/state/indexer/block/kv"
	"github.com/cometbft/cometbft/state/indexer/sink/psql"
	"github.com/cometbft/cometbft/state/txindex"
	"github.com/cometbft/cometbft/state/txindex/kv"
	"github.com/cometbft/cometbft/types"
)

const (
	reindexFailed = "event re-index failed: "
)

var (
	ErrHeightNotAvailable = errors.New("height is not available")
	ErrInvalidRequest     = errors.New("invalid request")
)

// ReIndexEventCmd constructs a command to re-index events in a block height interval.
var ReIndexEventCmd = &cobra.Command{
	Use:     "reindex-event",
	Aliases: []string{"reindex_event"},
	Short:   "reindex events to the event store backends",
	Long: `
reindex-event is an offline tooling to re-index block and tx events to the eventsinks,
you can run this command when the event store backend dropped/disconnected or you want to 
replace the backend. The default start-height is 0, meaning the tooling will start 
reindex from the base block height(inclusive); and the default end-height is 0, meaning 
the tooling will reindex until the latest block height(inclusive). User can omit
either or both arguments.

Note: This operation requires ABCI Responses. Do not set DiscardABCIResponses to true if you
want to use this command.
	`,
	Example: `
	cometbft reindex-event
	cometbft reindex-event --start-height 2
	cometbft reindex-event --end-height 10
	cometbft reindex-event --start-height 2 --end-height 10
	`,
	Run: func(cmd *cobra.Command, args []string) {
		bs, ss, err := loadStateAndBlockStore(config)
		if err != nil {
			fmt.Println(reindexFailed, err)
			return
		}

		state, err := ss.Load()
		if err != nil {
			fmt.Println(reindexFailed, err)
			return
		}

		if err := checkValidHeight(bs); err != nil {
			fmt.Println(reindexFailed, err)
			return
		}

		bi, ti, err := loadEventSinks(config, state.ChainID)
		if err != nil {
			fmt.Println(reindexFailed, err)
			return
		}

		riArgs := eventReIndexArgs{
			startHeight:  startHeight,
			endHeight:    endHeight,
			blockIndexer: bi,
			txIndexer:    ti,
			blockStore:   bs,
			stateStore:   ss,
		}
		if err := eventReIndex(cmd, riArgs); err != nil {
			panic(fmt.Errorf("%s: %w", reindexFailed, err))
		}

		fmt.Println("event re-index finished")
	},
}

var (
	startHeight int64
	endHeight   int64
)

func init() {
	ReIndexEventCmd.Flags().Int64Var(&startHeight, "start-height", 0, "the block height would like to start for re-index")
	ReIndexEventCmd.Flags().Int64Var(&endHeight, "end-height", 0, "the block height would like to finish for re-index")
}

func loadEventSinks(cfg *cmtcfg.Config, chainID string) (indexer.BlockIndexer, txindex.TxIndexer, error) {
	switch strings.ToLower(cfg.TxIndex.Indexer) {
	case "null":
		return nil, nil, errors.New("found null event sink, please check the tx-index section in the config.toml")
	case "psql":
		conn := cfg.TxIndex.PsqlConn
		if conn == "" {
			return nil, nil, errors.New("the psql connection settings cannot be empty")
		}
		es, err := psql.NewEventSink(conn, chainID)
		if err != nil {
			return nil, nil, err
		}
		return es.BlockIndexer(), es.TxIndexer(), nil
	case "kv":
		store, err := dbm.NewDB("tx_index", dbm.BackendType(cfg.DBBackend), cfg.DBDir())
		if err != nil {
			return nil, nil, err
		}

		txIndexer := kv.NewTxIndex(store)
		blockIndexer := blockidxkv.New(dbm.NewPrefixDB(store, []byte("block_events")))
		return blockIndexer, txIndexer, nil
	default:
		return nil, nil, fmt.Errorf("unsupported event sink type: %s", cfg.TxIndex.Indexer)
	}
}

type eventReIndexArgs struct {
	startHeight  int64
	endHeight    int64
	blockIndexer indexer.BlockIndexer
	txIndexer    txindex.TxIndexer
	blockStore   state.BlockStore
	stateStore   state.Store
}

func eventReIndex(cmd *cobra.Command, args eventReIndexArgs) error {
	var bar progressbar.Bar
	bar.NewOption(args.startHeight-1, args.endHeight)

	fmt.Println("start re-indexing events:")
	defer bar.Finish()
	for height := args.startHeight; height <= args.endHeight; height++ {
		select {
		case <-cmd.Context().Done():
			return fmt.Errorf("event re-index terminated at height %d: %w", height, cmd.Context().Err())
		default:
			block := args.blockStore.LoadBlock(height)
			if block == nil {
				return fmt.Errorf("not able to load block at height %d from the blockstore", height)
			}

			resp, err := args.stateStore.LoadFinalizeBlockResponse(height)
			if err != nil {
				return fmt.Errorf("not able to load ABCI Response at height %d from the statestore", height)
			}

			e := types.EventDataNewBlockEvents{
				Height: height,
				Events: resp.Events,
			}

			numTxs := len(resp.TxResults)

			var batch *txindex.Batch
			if numTxs > 0 {
				batch = txindex.NewBatch(int64(numTxs))

				for idx, txResult := range resp.TxResults {
					tr := abcitypes.TxResult{
						Height: height,
						Index:  uint32(idx),
						Tx:     block.Txs[idx],
						Result: *txResult,
					}

					if err = batch.Add(&tr); err != nil {
						return fmt.Errorf("adding tx to batch: %w", err)
					}
				}

				if err := args.txIndexer.AddBatch(batch); err != nil {
					return fmt.Errorf("tx event re-index at height %d failed: %w", height, err)
				}
			}

			if err := args.blockIndexer.Index(e); err != nil {
				return fmt.Errorf("block event re-index at height %d failed: %w", height, err)
			}
		}

		bar.Play(height)
	}

	return nil
}

func checkValidHeight(bs state.BlockStore) error {
	base := bs.Base()

	if startHeight == 0 {
		startHeight = base
		fmt.Printf("set the start block height to the base height of the blockstore %d \n", base)
	}

	if startHeight < base {
		return fmt.Errorf("%s (requested start height: %d, base height: %d)",
			ErrHeightNotAvailable, startHeight, base)
	}

	height := bs.Height()

	if startHeight > height {
		return fmt.Errorf(
			"%s (requested start height: %d, store height: %d)", ErrHeightNotAvailable, startHeight, height)
	}

	if endHeight == 0 || endHeight > height {
		endHeight = height
		fmt.Printf("set the end block height to the latest height of the blockstore %d \n", height)
	}

	if endHeight < base {
		return fmt.Errorf(
			"%s (requested end height: %d, base height: %d)", ErrHeightNotAvailable, endHeight, base)
	}

	if endHeight < startHeight {
		return fmt.Errorf(
			"%s (requested the end height: %d is less than the start height: %d)",
			ErrInvalidRequest, startHeight, endHeight)
	}

	return nil
}
