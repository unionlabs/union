package types

import "context"

//go:generate ../../scripts/mockery_generate.sh Application

// Application is an interface that enables any finite, deterministic state machine
// to be driven by a blockchain-based replication engine via the ABCI.
type Application interface {
	// Info/Query Connection
	Info(context.Context, *RequestInfo) (*ResponseInfo, error)    // Return application info
	Query(context.Context, *RequestQuery) (*ResponseQuery, error) // Query for state

	// Mempool Connection
	CheckTx(context.Context, *RequestCheckTx) (*ResponseCheckTx, error) // Validate a tx for the mempool

	// Consensus Connection
	InitChain(context.Context, *RequestInitChain) (*ResponseInitChain, error) // Initialize blockchain w validators/other info from CometBFT
	PrepareProposal(context.Context, *RequestPrepareProposal) (*ResponsePrepareProposal, error)
	ProcessProposal(context.Context, *RequestProcessProposal) (*ResponseProcessProposal, error)
	// Deliver the decided block with its txs to the Application
	FinalizeBlock(context.Context, *RequestFinalizeBlock) (*ResponseFinalizeBlock, error)
	// Create application specific vote extension
	ExtendVote(context.Context, *RequestExtendVote) (*ResponseExtendVote, error)
	// Verify application's vote extension data
	VerifyVoteExtension(context.Context, *RequestVerifyVoteExtension) (*ResponseVerifyVoteExtension, error)
	// Commit the state and return the application Merkle root hash
	Commit(context.Context, *RequestCommit) (*ResponseCommit, error)

	// State Sync Connection
	ListSnapshots(context.Context, *RequestListSnapshots) (*ResponseListSnapshots, error)                // List available snapshots
	OfferSnapshot(context.Context, *RequestOfferSnapshot) (*ResponseOfferSnapshot, error)                // Offer a snapshot to the application
	LoadSnapshotChunk(context.Context, *RequestLoadSnapshotChunk) (*ResponseLoadSnapshotChunk, error)    // Load a snapshot chunk
	ApplySnapshotChunk(context.Context, *RequestApplySnapshotChunk) (*ResponseApplySnapshotChunk, error) // Apply a shapshot chunk
}

//-------------------------------------------------------
// BaseApplication is a base form of Application

var _ Application = (*BaseApplication)(nil)

type BaseApplication struct{}

func NewBaseApplication() *BaseApplication {
	return &BaseApplication{}
}

func (BaseApplication) Info(context.Context, *RequestInfo) (*ResponseInfo, error) {
	return &ResponseInfo{}, nil
}

func (BaseApplication) CheckTx(context.Context, *RequestCheckTx) (*ResponseCheckTx, error) {
	return &ResponseCheckTx{Code: CodeTypeOK}, nil
}

func (BaseApplication) Commit(context.Context, *RequestCommit) (*ResponseCommit, error) {
	return &ResponseCommit{}, nil
}

func (BaseApplication) Query(context.Context, *RequestQuery) (*ResponseQuery, error) {
	return &ResponseQuery{Code: CodeTypeOK}, nil
}

func (BaseApplication) InitChain(context.Context, *RequestInitChain) (*ResponseInitChain, error) {
	return &ResponseInitChain{}, nil
}

func (BaseApplication) ListSnapshots(context.Context, *RequestListSnapshots) (*ResponseListSnapshots, error) {
	return &ResponseListSnapshots{}, nil
}

func (BaseApplication) OfferSnapshot(context.Context, *RequestOfferSnapshot) (*ResponseOfferSnapshot, error) {
	return &ResponseOfferSnapshot{}, nil
}

func (BaseApplication) LoadSnapshotChunk(context.Context, *RequestLoadSnapshotChunk) (*ResponseLoadSnapshotChunk, error) {
	return &ResponseLoadSnapshotChunk{}, nil
}

func (BaseApplication) ApplySnapshotChunk(context.Context, *RequestApplySnapshotChunk) (*ResponseApplySnapshotChunk, error) {
	return &ResponseApplySnapshotChunk{}, nil
}

func (BaseApplication) PrepareProposal(_ context.Context, req *RequestPrepareProposal) (*ResponsePrepareProposal, error) {
	txs := make([][]byte, 0, len(req.Txs))
	var totalBytes int64
	for _, tx := range req.Txs {
		totalBytes += int64(len(tx))
		if totalBytes > req.MaxTxBytes {
			break
		}
		txs = append(txs, tx)
	}
	return &ResponsePrepareProposal{Txs: txs}, nil
}

func (BaseApplication) ProcessProposal(context.Context, *RequestProcessProposal) (*ResponseProcessProposal, error) {
	return &ResponseProcessProposal{Status: ResponseProcessProposal_ACCEPT}, nil
}

func (BaseApplication) ExtendVote(context.Context, *RequestExtendVote) (*ResponseExtendVote, error) {
	return &ResponseExtendVote{}, nil
}

func (BaseApplication) VerifyVoteExtension(context.Context, *RequestVerifyVoteExtension) (*ResponseVerifyVoteExtension, error) {
	return &ResponseVerifyVoteExtension{
		Status: ResponseVerifyVoteExtension_ACCEPT,
	}, nil
}

func (BaseApplication) FinalizeBlock(_ context.Context, req *RequestFinalizeBlock) (*ResponseFinalizeBlock, error) {
	txs := make([]*ExecTxResult, len(req.Txs))
	for i := range req.Txs {
		txs[i] = &ExecTxResult{Code: CodeTypeOK}
	}
	return &ResponseFinalizeBlock{
		TxResults: txs,
	}, nil
}
