// Package psql implements an event sink backed by a PostgreSQL database.
package psql

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"strconv"
	"strings"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/lib/pq"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/internal/rand"
	"github.com/cometbft/cometbft/libs/pubsub/query"
	"github.com/cometbft/cometbft/types"
)

const (
	defaultTableBlocks     = "blocks"
	defaultTableTxResults  = "tx_results"
	defaultTableEvents     = "events"
	defaultTableAttributes = "attributes"
	driverName             = "postgres"
)

// EventSink is an indexer backend providing the tx/block index services.  This
// implementation stores records in a PostgreSQL database using the schema
// defined in state/indexer/sink/psql/schema.sql.
type EventSink struct {
	store           *sql.DB
	chainID         string
	tableBlocks     string
	tableTxResults  string
	tableEvents     string
	tableAttributes string
}

type EventSinkOption func(*EventSink)

// NewEventSink constructs an event sink associated with the PostgreSQL
// database specified by connStr. Events written to the sink are attributed to
// the specified chainID.
func NewEventSink(connStr, chainID string, opts ...EventSinkOption) (*EventSink, error) {
	es := &EventSink{
		chainID:         chainID,
		tableBlocks:     defaultTableBlocks,
		tableTxResults:  defaultTableTxResults,
		tableEvents:     defaultTableEvents,
		tableAttributes: defaultTableAttributes,
	}

	for _, opt := range opts {
		opt(es)
	}

	if es.store == nil && connStr != "" {
		db, err := sql.Open(driverName, connStr)
		if err != nil {
			return nil, err
		}
		es.store = db
	}

	return es, nil
}

func WithStore(store *sql.DB) EventSinkOption {
	return func(es *EventSink) {
		es.store = store
	}
}

func WithTableBlocks(tableBlocks string) EventSinkOption {
	return func(es *EventSink) {
		es.tableBlocks = tableBlocks
	}
}

func WithTableTxResults(tableTxResults string) EventSinkOption {
	return func(es *EventSink) {
		es.tableTxResults = tableTxResults
	}
}

func WithTableEvents(tableEvents string) EventSinkOption {
	return func(es *EventSink) {
		es.tableEvents = tableEvents
	}
}

func WithTableAttributes(tableAttributes string) EventSinkOption {
	return func(es *EventSink) {
		es.tableAttributes = tableAttributes
	}
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

func runBulkInsert(db *sql.DB, tableName string, columns []string, inserts [][]any) error {
	return runInTransaction(db, func(tx *sql.Tx) error {
		stmt, err := tx.Prepare(pq.CopyIn(tableName, columns...))
		if err != nil {
			return fmt.Errorf("preparing bulk insert statement: %w", err)
		}
		defer stmt.Close()
		for _, insert := range inserts {
			if _, err := stmt.Exec(insert...); err != nil {
				return fmt.Errorf("executing insert statement: %w", err)
			}
		}
		if _, err := stmt.Exec(); err != nil {
			return fmt.Errorf("flushing bulk insert: %w", err)
		}
		return nil
	})
}

func randomBigserial() int64 {
	return rand.Int63()
}

var (
	txrInsertColumns   = []string{"rowid", "block_id", "index", "created_at", "tx_hash", "tx_result"}
	eventInsertColumns = []string{"rowid", "block_id", "tx_id", "type"}
	attrInsertColumns  = []string{"event_id", "key", "composite_key", "value"}
)

func bulkInsertEvents(blockID, txID int64, events []abci.Event) (eventInserts, attrInserts [][]any) {
	// Populate the transaction ID field iff one is defined (> 0).
	var txIDArg any
	if txID > 0 {
		txIDArg = txID
	}
	for _, event := range events {
		// Skip events with an empty type.
		if event.Type == "" {
			continue
		}
		eventID := randomBigserial()
		eventInserts = append(eventInserts, []any{eventID, blockID, txIDArg, event.Type})
		for _, attr := range event.Attributes {
			if !attr.Index {
				continue
			}
			compositeKey := event.Type + "." + attr.Key
			attrInserts = append(attrInserts, []any{eventID, attr.Key, compositeKey, attr.Value})
		}
	}
	return eventInserts, attrInserts
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

	// Add the block to the blocks table and report back its row ID for use
	// in indexing the events for the block.
	var blockID int64
	//nolint:execinquery
	err := es.store.QueryRow(`
INSERT INTO `+es.tableBlocks+` (height, chain_id, created_at)
  VALUES ($1, $2, $3)
  ON CONFLICT DO NOTHING
  RETURNING rowid;
`, h.Height, es.chainID, ts).Scan(&blockID)
	if errors.Is(err, sql.ErrNoRows) {
		return nil // we already saw this block; quietly succeed
	} else if err != nil {
		return fmt.Errorf("indexing block header: %w", err)
	}

	// Insert the special block meta-event for height.
	events := append([]abci.Event{makeIndexedEvent(types.BlockHeightKey, strconv.FormatInt(h.Height, 10))}, h.Events...)
	// Insert all the block events. Order is important here,
	eventInserts, attrInserts := bulkInsertEvents(blockID, 0, events)
	if err := runBulkInsert(es.store, es.tableEvents, eventInsertColumns, eventInserts); err != nil {
		return fmt.Errorf("failed bulk insert of events: %w", err)
	}
	if err := runBulkInsert(es.store, es.tableAttributes, attrInsertColumns, attrInserts); err != nil {
		return fmt.Errorf("failed bulk insert of attributes: %w", err)
	}
	return nil
}

// getBlockIDs returns corresponding block ids for the provided heights.
func (es *EventSink) getBlockIDs(heights []int64) ([]int64, error) {
	var blockIDs pq.Int64Array
	if err := es.store.QueryRow(`
SELECT array_agg((
	SELECT rowid FROM `+es.tableBlocks+` WHERE height = txr.height AND chain_id = $1
)) FROM unnest($2::bigint[]) AS txr(height);`,
		es.chainID, pq.Array(heights)).Scan(&blockIDs); err != nil {
		return nil, fmt.Errorf("getting block ids for txs from sql: %w", err)
	}
	return blockIDs, nil
}

func prefetchTxrExistence(db *sql.DB, blockIDs []int64, indexes []uint32, txResultsTable string) ([]bool, error) {
	var existence []bool
	if err := db.QueryRow(`
SELECT array_agg((
	SELECT EXISTS(SELECT 1 FROM `+txResultsTable+` WHERE block_id = txr.block_id AND index = txr.index)
)) FROM UNNEST($1::bigint[], $2::integer[]) as txr(block_id, index);`,
		pq.Array(blockIDs), pq.Array(indexes)).Scan((*pq.BoolArray)(&existence)); err != nil {
		return nil, fmt.Errorf("fetching already indexed txrs: %w", err)
	}
	return existence, nil
}

func (es *EventSink) IndexTxEvents(txrs []*abci.TxResult) error {
	ts := time.Now().UTC()
	heights := make([]int64, len(txrs))
	indexes := make([]uint32, len(txrs))
	for i, txr := range txrs {
		heights[i] = txr.Height
		indexes[i] = txr.Index
	}
	// prefetch blockIDs for all txrs. Every block header must have been indexed
	// prior to the transactions belonging to it.
	blockIDs, err := es.getBlockIDs(heights)
	if err != nil {
		return fmt.Errorf("getting block ids for txs: %w", err)
	}
	alreadyIndexed, err := prefetchTxrExistence(es.store, blockIDs, indexes, es.tableTxResults)
	if err != nil {
		return fmt.Errorf("failed to prefetch which txrs were already indexed: %w", err)
	}
	txrInserts, attrInserts, eventInserts := make([][]any, 0, len(txrs)), make([][]any, 0, len(txrs)), make([][]any, 0, len(txrs))
	for i, txr := range txrs {
		if alreadyIndexed[i] {
			continue
		}
		// Encode the result message in protobuf wire format for indexing.
		resultData, err := proto.Marshal(txr)
		if err != nil {
			return fmt.Errorf("marshaling tx_result: %w", err)
		}
		// Index the hash of the underlying transaction as a hex string.
		txHash := fmt.Sprintf("%X", types.Tx(txr.Tx).Hash())
		// Generate random ID for this tx_result and insert a record for it
		txID := randomBigserial()
		txrInserts = append(txrInserts, []any{txID, blockIDs[i], txr.Index, ts, txHash, resultData})
		// Insert the special transaction meta-events for hash and height.
		events := append([]abci.Event{
			makeIndexedEvent(types.TxHashKey, txHash),
			makeIndexedEvent(types.TxHeightKey, strconv.FormatInt(txr.Height, 10)),
		},
			txr.Result.Events...,
		)
		newEventInserts, newAttrInserts := bulkInsertEvents(blockIDs[i], txID, events)
		eventInserts = append(eventInserts, newEventInserts...)
		attrInserts = append(attrInserts, newAttrInserts...)
	}
	if err := runBulkInsert(es.store, es.tableTxResults, txrInsertColumns, txrInserts); err != nil {
		return fmt.Errorf("bulk inserting txrs: %w", err)
	}
	if err := runBulkInsert(es.store, es.tableEvents, eventInsertColumns, eventInserts); err != nil {
		return fmt.Errorf("bulk inserting events: %w", err)
	}
	if err := runBulkInsert(es.store, es.tableAttributes, attrInsertColumns, attrInserts); err != nil {
		return fmt.Errorf("bulk inserting attributes: %w", err)
	}
	return nil
}

// SearchBlockEvents is not implemented by this sink, and reports an error for all queries.
func (*EventSink) SearchBlockEvents(_ context.Context, _ *query.Query) ([]int64, error) {
	return nil, errors.New("block search is not supported via the postgres event sink")
}

// SearchTxEvents is not implemented by this sink, and reports an error for all queries.
func (*EventSink) SearchTxEvents(_ context.Context, _ *query.Query) ([]*abci.TxResult, error) {
	return nil, errors.New("tx search is not supported via the postgres event sink")
}

// GetTxByHash is not implemented by this sink, and reports an error for all queries.
func (*EventSink) GetTxByHash(_ []byte) (*abci.TxResult, error) {
	return nil, errors.New("getTxByHash is not supported via the postgres event sink")
}

// HasBlock is not implemented by this sink, and reports an error for all queries.
func (*EventSink) HasBlock(_ int64) (bool, error) {
	return false, errors.New("hasBlock is not supported via the postgres event sink")
}

// Stop closes the underlying PostgreSQL database.
func (es *EventSink) Stop() error { return es.store.Close() }
