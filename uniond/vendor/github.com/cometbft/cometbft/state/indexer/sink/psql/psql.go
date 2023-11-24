// Package psql implements an event sink backed by a PostgreSQL database.
package psql

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/cosmos/gogoproto/proto"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/pubsub/query"
	"github.com/cometbft/cometbft/types"
)

const (
	tableBlocks     = "blocks"
	tableTxResults  = "tx_results"
	tableEvents     = "events"
	tableAttributes = "attributes"
	driverName      = "postgres"
)

// EventSink is an indexer backend providing the tx/block index services.  This
// implementation stores records in a PostgreSQL database using the schema
// defined in state/indexer/sink/psql/schema.sql.
type EventSink struct {
	store   *sql.DB
	chainID string
}

// NewEventSink constructs an event sink associated with the PostgreSQL
// database specified by connStr. Events written to the sink are attributed to
// the specified chainID.
func NewEventSink(connStr, chainID string) (*EventSink, error) {
	db, err := sql.Open(driverName, connStr)
	if err != nil {
		return nil, err
	}

	return &EventSink{
		store:   db,
		chainID: chainID,
	}, nil
}

// DB returns the underlying Postgres connection used by the sink.
// This is exported to support testing.
func (es *EventSink) DB() *sql.DB { return es.store }

// runInTransaction executes query in a fresh database transaction.
// If query reports an error, the transaction is rolled back and the
// error from query is reported to the caller.
// Otherwise, the result of committing the transaction is returned.
func runInTransaction(db *sql.DB, query func(*sql.Tx) error) error {
	dbtx, err := db.Begin()
	if err != nil {
		return err
	}
	if err := query(dbtx); err != nil {
		_ = dbtx.Rollback() // report the initial error, not the rollback
		return err
	}
	return dbtx.Commit()
}

// queryWithID executes the specified SQL query with the given arguments,
// expecting a single-row, single-column result containing an ID. If the query
// succeeds, the ID from the result is returned.
func queryWithID(tx *sql.Tx, query string, args ...interface{}) (uint32, error) {
	var id uint32
	if err := tx.QueryRow(query, args...).Scan(&id); err != nil {
		return 0, err
	}
	return id, nil
}

// insertEvents inserts a slice of events and any indexed attributes of those
// events into the database associated with dbtx.
//
// If txID > 0, the event is attributed to the transaction with that
// ID; otherwise it is recorded as a block event.
func insertEvents(dbtx *sql.Tx, blockID, txID uint32, evts []abci.Event) error {
	// Populate the transaction ID field iff one is defined (> 0).
	var txIDArg interface{}
	if txID > 0 {
		txIDArg = txID
	}

	const (
		insertEventQuery = `
			INSERT INTO ` + tableEvents + ` (block_id, tx_id, type)
			VALUES ($1, $2, $3)
			RETURNING rowid;
		`
		insertAttributeQuery = `
			INSERT INTO ` + tableAttributes + ` (event_id, key, composite_key, value)
			VALUES ($1, $2, $3, $4);
		`
	)

	// Add each event to the events table, and retrieve its row ID to use when
	// adding any attributes the event provides.
	for _, evt := range evts {
		// Skip events with an empty type.
		if evt.Type == "" {
			continue
		}

		eid, err := queryWithID(dbtx, insertEventQuery, blockID, txIDArg, evt.Type)
		if err != nil {
			return err
		}

		// Add any attributes flagged for indexing.
		for _, attr := range evt.Attributes {
			if !attr.Index {
				continue
			}
			compositeKey := evt.Type + "." + attr.Key
			if _, err := dbtx.Exec(insertAttributeQuery, eid, attr.Key, compositeKey, attr.Value); err != nil {
				return err
			}
		}
	}
	return nil
}

// makeIndexedEvent constructs an event from the specified composite key and
// value. If the key has the form "type.name", the event will have a single
// attribute with that name and the value; otherwise the event will have only
// a type and no attributes.
func makeIndexedEvent(compositeKey, value string) abci.Event {
	i := strings.Index(compositeKey, ".")
	if i < 0 {
		return abci.Event{Type: compositeKey}
	}
	return abci.Event{Type: compositeKey[:i], Attributes: []abci.EventAttribute{
		{Key: compositeKey[i+1:], Value: value, Index: true},
	}}
}

// IndexBlockEvents indexes the specified block header, part of the
// indexer.EventSink interface.
func (es *EventSink) IndexBlockEvents(h types.EventDataNewBlockEvents) error {
	ts := time.Now().UTC()

	return runInTransaction(es.store, func(dbtx *sql.Tx) error {
		// Add the block to the blocks table and report back its row ID for use
		// in indexing the events for the block.
		blockID, err := queryWithID(dbtx, `
INSERT INTO `+tableBlocks+` (height, chain_id, created_at)
  VALUES ($1, $2, $3)
  ON CONFLICT DO NOTHING
  RETURNING rowid;
`, h.Height, es.chainID, ts)
		if err == sql.ErrNoRows {
			return nil // we already saw this block; quietly succeed
		} else if err != nil {
			return fmt.Errorf("indexing block header: %w", err)
		}

		// Insert the special block meta-event for height.
		if err := insertEvents(dbtx, blockID, 0, []abci.Event{
			makeIndexedEvent(types.BlockHeightKey, fmt.Sprint(h.Height)),
		}); err != nil {
			return fmt.Errorf("block meta-events: %w", err)
		}
		// Insert all the block events. Order is important here,
		if err := insertEvents(dbtx, blockID, 0, h.Events); err != nil {
			return fmt.Errorf("finalizeblock events: %w", err)
		}
		return nil
	})
}

func (es *EventSink) IndexTxEvents(txrs []*abci.TxResult) error {
	ts := time.Now().UTC()

	for _, txr := range txrs {
		// Encode the result message in protobuf wire format for indexing.
		resultData, err := proto.Marshal(txr)
		if err != nil {
			return fmt.Errorf("marshaling tx_result: %w", err)
		}

		// Index the hash of the underlying transaction as a hex string.
		txHash := fmt.Sprintf("%X", types.Tx(txr.Tx).Hash())

		if err := runInTransaction(es.store, func(dbtx *sql.Tx) error {
			// Find the block associated with this transaction. The block header
			// must have been indexed prior to the transactions belonging to it.
			blockID, err := queryWithID(dbtx, `
SELECT rowid FROM `+tableBlocks+` WHERE height = $1 AND chain_id = $2;
`, txr.Height, es.chainID)
			if err != nil {
				return fmt.Errorf("finding block ID: %w", err)
			}

			// Insert a record for this tx_result and capture its ID for indexing events.
			txID, err := queryWithID(dbtx, `
INSERT INTO `+tableTxResults+` (block_id, index, created_at, tx_hash, tx_result)
  VALUES ($1, $2, $3, $4, $5)
  ON CONFLICT DO NOTHING
  RETURNING rowid;
`, blockID, txr.Index, ts, txHash, resultData)
			if err == sql.ErrNoRows {
				return nil // we already saw this transaction; quietly succeed
			} else if err != nil {
				return fmt.Errorf("indexing tx_result: %w", err)
			}

			// Insert the special transaction meta-events for hash and height.
			if err := insertEvents(dbtx, blockID, txID, []abci.Event{
				makeIndexedEvent(types.TxHashKey, txHash),
				makeIndexedEvent(types.TxHeightKey, fmt.Sprint(txr.Height)),
			}); err != nil {
				return fmt.Errorf("indexing transaction meta-events: %w", err)
			}
			// Index any events packaged with the transaction.
			if err := insertEvents(dbtx, blockID, txID, txr.Result.Events); err != nil {
				return fmt.Errorf("indexing transaction events: %w", err)
			}
			return nil
		}); err != nil {
			return err
		}
	}
	return nil
}

// SearchBlockEvents is not implemented by this sink, and reports an error for all queries.
func (es *EventSink) SearchBlockEvents(_ context.Context, _ *query.Query) ([]int64, error) {
	return nil, errors.New("block search is not supported via the postgres event sink")
}

// SearchTxEvents is not implemented by this sink, and reports an error for all queries.
func (es *EventSink) SearchTxEvents(_ context.Context, _ *query.Query) ([]*abci.TxResult, error) {
	return nil, errors.New("tx search is not supported via the postgres event sink")
}

// GetTxByHash is not implemented by this sink, and reports an error for all queries.
func (es *EventSink) GetTxByHash(_ []byte) (*abci.TxResult, error) {
	return nil, errors.New("getTxByHash is not supported via the postgres event sink")
}

// HasBlock is not implemented by this sink, and reports an error for all queries.
func (es *EventSink) HasBlock(_ int64) (bool, error) {
	return false, errors.New("hasBlock is not supported via the postgres event sink")
}

// Stop closes the underlying PostgreSQL database.
func (es *EventSink) Stop() error { return es.store.Close() }
