package types

import "context"

//go:generate ../../scripts/mockery_generate.sh Application

// Application is an interface that enables any finite, deterministic state machine
// to be driven by a blockchain-based replication engine via the ABCI.
type Application interface {
	// Info/Query Connection
	Info(ctx context.Context, req *InfoRequest) (*InfoResponse, error)    // Return application info
	Query(ctx context.Context, req *QueryRequest) (*QueryResponse, error) // Query for state

	// Mempool Connection
	CheckTx(ctx context.Context, req *CheckTxRequest) (*CheckTxResponse, error) // Validate a tx for the mempool

	// Consensus Connection
	InitChain(ctx context.Context, req *InitChainRequest) (*InitChainResponse, error) // Initialize blockchain w validators/other info from CometBFT
	PrepareProposal(ctx context.Context, req *PrepareProposalRequest) (*PrepareProposalResponse, error)
	ProcessProposal(ctx context.Context, req *ProcessProposalRequest) (*ProcessProposalResponse, error)
	// Deliver the decided block with its txs to the Application
	FinalizeBlock(ctx context.Context, req *FinalizeBlockRequest) (*FinalizeBlockResponse, error)
	// Create application specific vote extension
	ExtendVote(ctx context.Context, req *ExtendVoteRequest) (*ExtendVoteResponse, error)
	// Verify application's vote extension data
	VerifyVoteExtension(ctx context.Context, req *VerifyVoteExtensionRequest) (*VerifyVoteExtensionResponse, error)
	// Commit the state and return the application Merkle root hash
	Commit(ctx context.Context, req *CommitRequest) (*CommitResponse, error)

	// State Sync Connection
	ListSnapshots(ctx context.Context, req *ListSnapshotsRequest) (*ListSnapshotsResponse, error)                // List available snapshots
	OfferSnapshot(ctx context.Context, req *OfferSnapshotRequest) (*OfferSnapshotResponse, error)                // Offer a snapshot to the application
	LoadSnapshotChunk(ctx context.Context, req *LoadSnapshotChunkRequest) (*LoadSnapshotChunkResponse, error)    // Load a snapshot chunk
	ApplySnapshotChunk(ctx context.Context, req *ApplySnapshotChunkRequest) (*ApplySnapshotChunkResponse, error) // Apply a snapshot chunk
}

// -------------------------------------------------------
// BaseApplication is a base form of Application

var _ Application = (*BaseApplication)(nil)

type BaseApplication struct{}

func NewBaseApplication() *BaseApplication {
	return &BaseApplication{}
}

func (BaseApplication) Info(context.Context, *InfoRequest) (*InfoResponse, error) {
	return &InfoResponse{}, nil
}

func (BaseApplication) CheckTx(context.Context, *CheckTxRequest) (*CheckTxResponse, error) {
	return &CheckTxResponse{Code: CodeTypeOK}, nil
}

func (BaseApplication) Commit(context.Context, *CommitRequest) (*CommitResponse, error) {
	return &CommitResponse{}, nil
}

func (BaseApplication) Query(context.Context, *QueryRequest) (*QueryResponse, error) {
	return &QueryResponse{Code: CodeTypeOK}, nil
}

func (BaseApplication) InitChain(context.Context, *InitChainRequest) (*InitChainResponse, error) {
	return &InitChainResponse{}, nil
}

func (BaseApplication) ListSnapshots(context.Context, *ListSnapshotsRequest) (*ListSnapshotsResponse, error) {
	return &ListSnapshotsResponse{}, nil
}

func (BaseApplication) OfferSnapshot(context.Context, *OfferSnapshotRequest) (*OfferSnapshotResponse, error) {
	return &OfferSnapshotResponse{}, nil
}

func (BaseApplication) LoadSnapshotChunk(context.Context, *LoadSnapshotChunkRequest) (*LoadSnapshotChunkResponse, error) {
	return &LoadSnapshotChunkResponse{}, nil
}

func (BaseApplication) ApplySnapshotChunk(context.Context, *ApplySnapshotChunkRequest) (*ApplySnapshotChunkResponse, error) {
	return &ApplySnapshotChunkResponse{}, nil
}

func (BaseApplication) PrepareProposal(_ context.Context, req *PrepareProposalRequest) (*PrepareProposalResponse, error) {
	txs := make([][]byte, 0, len(req.Txs))
	var totalBytes int64
	for _, tx := range req.Txs {
		totalBytes += int64(len(tx))
		if totalBytes > req.MaxTxBytes {
			break
		}
		txs = append(txs, tx)
	}
	return &PrepareProposalResponse{Txs: txs}, nil
}

func (BaseApplication) ProcessProposal(context.Context, *ProcessProposalRequest) (*ProcessProposalResponse, error) {
	return &ProcessProposalResponse{Status: PROCESS_PROPOSAL_STATUS_ACCEPT}, nil
}

func (BaseApplication) ExtendVote(context.Context, *ExtendVoteRequest) (*ExtendVoteResponse, error) {
	return &ExtendVoteResponse{}, nil
}

func (BaseApplication) VerifyVoteExtension(context.Context, *VerifyVoteExtensionRequest) (*VerifyVoteExtensionResponse, error) {
	return &VerifyVoteExtensionResponse{
		Status: VERIFY_VOTE_EXTENSION_STATUS_ACCEPT,
	}, nil
}

func (BaseApplication) FinalizeBlock(_ context.Context, req *FinalizeBlockRequest) (*FinalizeBlockResponse, error) {
	txs := make([]*ExecTxResult, len(req.Txs))
	for i := range req.Txs {
		txs[i] = &ExecTxResult{Code: CodeTypeOK}
	}
	return &FinalizeBlockResponse{
		TxResults: txs,
	}, nil
}
