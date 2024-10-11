package types

import (
	"context"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
)

// ABCIListener is the interface that we're exposing as a streaming service.
// It hooks into the ABCI message processing of the BaseApp.
// The error results are propagated to consensus state machine,
// if you don't want to affect consensus, handle the errors internally and always return `nil` in these APIs.
type ABCIListener interface {
	// ListenFinalizeBlock updates the streaming service with the latest FinalizeBlock messages
	ListenFinalizeBlock(ctx context.Context, req abci.FinalizeBlockRequest, res abci.FinalizeBlockResponse) error
	// ListenCommit updates the steaming service with the latest Commit messages and state changes
	ListenCommit(ctx context.Context, res abci.CommitResponse, changeSet []*StoreKVPair) error
}

// StreamingManager is the struct that maintains a list of ABCIListeners and configuration settings.
type StreamingManager struct {
	// ABCIListeners for hooking into the ABCI message processing of the BaseApp
	// and exposing the requests and responses to external consumers
	ABCIListeners []ABCIListener

	// StopNodeOnErr halts the node when ABCI streaming service listening results in an error.
	StopNodeOnErr bool
}
