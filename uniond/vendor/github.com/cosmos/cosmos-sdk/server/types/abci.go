package types

import (
	"context"

	abci "github.com/cometbft/cometbft/abci/types"
)

// ABCI is an interface that enables any finite, deterministic state machine
// to be driven by a blockchain-based replication engine via the ABCI.
type ABCI interface {
	// Info/Query Connection
	Info(*abci.RequestInfo) (*abci.ResponseInfo, error)                     // Return application info
	Query(context.Context, *abci.RequestQuery) (*abci.ResponseQuery, error) // Query for state

	// Mempool Connection
	CheckTx(*abci.RequestCheckTx) (*abci.ResponseCheckTx, error) // Validate a tx for the mempool

	// Consensus Connection
	InitChain(*abci.RequestInitChain) (*abci.ResponseInitChain, error) // Initialize blockchain w validators/other info from CometBFT
	PrepareProposal(*abci.RequestPrepareProposal) (*abci.ResponsePrepareProposal, error)
	ProcessProposal(*abci.RequestProcessProposal) (*abci.ResponseProcessProposal, error)
	// Deliver the decided block with its txs to the Application
	FinalizeBlock(*abci.RequestFinalizeBlock) (*abci.ResponseFinalizeBlock, error)
	// Create application specific vote extension
	ExtendVote(context.Context, *abci.RequestExtendVote) (*abci.ResponseExtendVote, error)
	// Verify application's vote extension data
	VerifyVoteExtension(*abci.RequestVerifyVoteExtension) (*abci.ResponseVerifyVoteExtension, error)
	// Commit the state and return the application Merkle root hash
	Commit() (*abci.ResponseCommit, error)

	// State Sync Connection
	ListSnapshots(*abci.RequestListSnapshots) (*abci.ResponseListSnapshots, error)                // List available snapshots
	OfferSnapshot(*abci.RequestOfferSnapshot) (*abci.ResponseOfferSnapshot, error)                // Offer a snapshot to the application
	LoadSnapshotChunk(*abci.RequestLoadSnapshotChunk) (*abci.ResponseLoadSnapshotChunk, error)    // Load a snapshot chunk
	ApplySnapshotChunk(*abci.RequestApplySnapshotChunk) (*abci.ResponseApplySnapshotChunk, error) // Apply a shapshot chunk
}
