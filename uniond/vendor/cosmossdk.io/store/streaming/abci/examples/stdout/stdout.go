package main

import (
	"context"
	"fmt"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	"github.com/hashicorp/go-plugin"

	streamingabci "cosmossdk.io/store/streaming/abci"
	store "cosmossdk.io/store/types"
)

// StdoutPlugin is the implementation of the ABCIListener interface
// For Go plugins this is all that is required to process data sent over gRPC.
type StdoutPlugin struct {
	BlockHeight int64
}

func (a *StdoutPlugin) ListenFinalizeBlock(ctx context.Context, req abci.FinalizeBlockRequest, res abci.FinalizeBlockResponse) error {
	a.BlockHeight = req.Height
	// process tx messages (i.e: sent to external system)
	fmt.Printf("listen-finalize-block: block-height=%d req=%v res=%v", a.BlockHeight, req, res)
	return nil
}

func (a *StdoutPlugin) ListenCommit(ctx context.Context, res abci.CommitResponse, changeSet []*store.StoreKVPair) error {
	// process block commit messages (i.e: sent to external system)
	fmt.Printf("listen-commit: block_height=%d res=%v data=%v", a.BlockHeight, res, changeSet)
	return nil
}

func main() {
	plugin.Serve(&plugin.ServeConfig{
		HandshakeConfig: streamingabci.Handshake,
		Plugins: map[string]plugin.Plugin{
			"abci": &streamingabci.ListenerGRPCPlugin{Impl: &StdoutPlugin{}},
		},

		// A non-nil value here enables gRPC serving for this streaming...
		GRPCServer: plugin.DefaultGRPCServer,
	})
}
