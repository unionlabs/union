package types

import (
	"context"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
)

// ABCI is an interface that enables any finite, deterministic state machine
// to be driven by a blockchain-based replication engine via the ABCI.
type ABCI interface {
	// Info/Query Connection

	// Info returns application info
	Info(*abci.InfoRequest) (*abci.InfoResponse, error)
	// Query  returns application state
	Query(context.Context, *abci.QueryRequest) (*abci.QueryResponse, error)

	// Mempool Connection

	// CheckTx validate a tx for the mempool
	CheckTx(*abci.CheckTxRequest) (*abci.CheckTxResponse, error)

	// Consensus Connection

	// InitChain Initialize blockchain w validators/other info from CometBFT
	InitChain(*abci.InitChainRequest) (*abci.InitChainResponse, error)
	PrepareProposal(*abci.PrepareProposalRequest) (*abci.PrepareProposalResponse, error)
	ProcessProposal(*abci.ProcessProposalRequest) (*abci.ProcessProposalResponse, error)
	// FinalizeBlock deliver the decided block with its txs to the Application
	FinalizeBlock(*abci.FinalizeBlockRequest) (*abci.FinalizeBlockResponse, error)
	// ExtendVote create application specific vote extension
	ExtendVote(context.Context, *abci.ExtendVoteRequest) (*abci.ExtendVoteResponse, error)
	// VerifyVoteExtension verify application's vote extension data
	VerifyVoteExtension(*abci.VerifyVoteExtensionRequest) (*abci.VerifyVoteExtensionResponse, error)
	// Commit the state and return the application Merkle root hash
	Commit() (*abci.CommitResponse, error)

	// State Sync Connection

	// ListSnapshots list available snapshots
	ListSnapshots(*abci.ListSnapshotsRequest) (*abci.ListSnapshotsResponse, error)
	// OfferSnapshot offer a snapshot to the application
	OfferSnapshot(*abci.OfferSnapshotRequest) (*abci.OfferSnapshotResponse, error)
	// LoadSnapshotChunk load a snapshot chunk
	LoadSnapshotChunk(*abci.LoadSnapshotChunkRequest) (*abci.LoadSnapshotChunkResponse, error)
	// ApplySnapshotChunk apply a snapshot chunk
	ApplySnapshotChunk(*abci.ApplySnapshotChunkRequest) (*abci.ApplySnapshotChunkResponse, error)
}
