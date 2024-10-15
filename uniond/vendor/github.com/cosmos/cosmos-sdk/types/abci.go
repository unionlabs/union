package types

import (
	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
)

// InitChainer initializes application state at genesis
type InitChainer func(ctx Context, req *abci.InitChainRequest) (*abci.InitChainResponse, error)

// PrepareCheckStater runs code during commit after the block has been committed, and the `checkState`
// has been branched for the new block.
type PrepareCheckStater func(ctx Context)

// Precommiter runs code during commit immediately before the `deliverState` is written to the `rootMultiStore`.
type Precommiter func(ctx Context)

// PeerFilter responds to p2p filtering queries from Tendermint
type PeerFilter func(info string) *abci.QueryResponse

// ProcessProposalHandler defines a function type alias for processing a proposer
type ProcessProposalHandler func(Context, *abci.ProcessProposalRequest) (*abci.ProcessProposalResponse, error)

// PrepareProposalHandler defines a function type alias for preparing a proposal
type PrepareProposalHandler func(Context, *abci.PrepareProposalRequest) (*abci.PrepareProposalResponse, error)

// CheckTxHandler defines a function type alias for executing logic before transactions are executed.
// `RunTx` is a function type alias for executing logic before transactions are executed.
// The passed in runtx does not override antehandlers, the execution mode is not passed into runtx to avoid overriding the execution mode.
type CheckTxHandler func(func(txBytes []byte, tx Tx) (gInfo GasInfo, result *Result, anteEvents []abci.Event, err error), *abci.CheckTxRequest) (*abci.CheckTxResponse, error)

// ExtendVoteHandler defines a function type alias for extending a pre-commit vote.
type ExtendVoteHandler func(Context, *abci.ExtendVoteRequest) (*abci.ExtendVoteResponse, error)

// VerifyVoteExtensionHandler defines a function type alias for verifying a
// pre-commit vote extension.
type VerifyVoteExtensionHandler func(Context, *abci.VerifyVoteExtensionRequest) (*abci.VerifyVoteExtensionResponse, error)

// PreBlocker runs code before the `BeginBlocker` and defines a function type alias for executing logic right
// before FinalizeBlock is called (but after its context has been set up). It is
// intended to allow applications to perform computation on vote extensions and
// persist their results in state.
//
// Note: returning an error will make FinalizeBlock fail.
type PreBlocker func(Context, *abci.FinalizeBlockRequest) error

// BeginBlocker defines a function type alias for executing application
// business logic before transactions are executed.
//
// Note: The BeginBlock ABCI method no longer exists in the ABCI specification
// as of CometBFT v0.38.0. This function type alias is provided for backwards
// compatibility with applications that still use the BeginBlock ABCI method
// and allows for existing BeginBlock functionality within applications.
type BeginBlocker func(Context) (BeginBlock, error)

// EndBlocker defines a function type alias for executing application
// business logic after transactions are executed but before committing.
//
// Note: The EndBlock ABCI method no longer exists in the ABCI specification
// as of CometBFT v0.38.0. This function type alias is provided for backwards
// compatibility with applications that still use the EndBlock ABCI method
// and allows for existing EndBlock functionality within applications.
type EndBlocker func(Context) (EndBlock, error)

// EndBlock defines a type which contains endblock events and validator set updates
type EndBlock struct {
	ValidatorUpdates []abci.ValidatorUpdate
	Events           []abci.Event
}

// BeginBlock defines a type which contains beginBlock events
type BeginBlock struct {
	Events []abci.Event
}
