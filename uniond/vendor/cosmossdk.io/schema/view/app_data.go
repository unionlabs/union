package view

// AppData is an interface which indexer data targets can implement to allow their app data including
// state, blocks, transactions and events to be queried. An app's state and event store can also implement
// this interface to provide an authoritative source of data for comparing with indexed data.
type AppData interface {
	// BlockNum indicates the last block that was persisted. It should be 0 if the target has no data
	// stored and wants to start syncing state.
	// If an indexer starts indexing after a chain's genesis (returning 0), the indexer manager
	// will attempt to perform a catch-up sync of state. Historical events will not be replayed, but an accurate
	// representation of the current state at the height at which indexing began can be reproduced.
	BlockNum() (uint64, error)

	// AppState returns the app state. If the view doesn't persist app state, nil should be returned.
	AppState() AppState
}
