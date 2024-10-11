package abci

import (
	"context"
	"os"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	"github.com/hashicorp/go-plugin"

	storetypes "cosmossdk.io/store/types"
)

var _ storetypes.ABCIListener = (*GRPCClient)(nil)

// GRPCClient is an implementation of the ABCIListener interface that talks over RPC.
type GRPCClient struct {
	client ABCIListenerServiceClient
}

// ListenFinalizeBlock listens to end block request and responses.
// In addition, it retrieves a types.Context from a context.Context instance.
// It panics if a types.Context was not properly attached.
// When the node is configured to stop on listening errors,
// it will terminate immediately and exit with a non-zero code.
func (m *GRPCClient) ListenFinalizeBlock(goCtx context.Context, req abci.FinalizeBlockRequest, res abci.FinalizeBlockResponse) error {
	ctx := goCtx.(storetypes.Context)
	sm := ctx.StreamingManager()
	request := &ListenFinalizeBlockRequest{Req: &req, Res: &res}
	_, err := m.client.ListenFinalizeBlock(goCtx, request)
	if err != nil && sm.StopNodeOnErr {
		ctx.Logger().Error("FinalizeBlock listening hook failed", "height", ctx.BlockHeight(), "err", err)
		cleanupAndExit()
	}
	return err
}

// ListenCommit listens to commit responses and state changes for the current block.
// In addition, it retrieves a types.Context from a context.Context instance.
// It panics if a types.Context was not properly attached.
// When the node is configured to stop on listening errors,
// it will terminate immediately and exit with a non-zero code.
func (m *GRPCClient) ListenCommit(goCtx context.Context, res abci.CommitResponse, changeSet []*storetypes.StoreKVPair) error {
	ctx := goCtx.(storetypes.Context)
	sm := ctx.StreamingManager()
	request := &ListenCommitRequest{BlockHeight: ctx.BlockHeight(), Res: &res, ChangeSet: changeSet}
	_, err := m.client.ListenCommit(goCtx, request)
	if err != nil && sm.StopNodeOnErr {
		ctx.Logger().Error("Commit listening hook failed", "height", ctx.BlockHeight(), "err", err)
		cleanupAndExit()
	}
	return err
}

func cleanupAndExit() {
	plugin.CleanupClients()
	os.Exit(1)
}

var _ ABCIListenerServiceServer = (*GRPCServer)(nil)

// GRPCServer is the gRPC server that GRPCClient talks to.
type GRPCServer struct {
	// This is the real implementation
	Impl storetypes.ABCIListener
}

func (m GRPCServer) ListenFinalizeBlock(ctx context.Context, request *ListenFinalizeBlockRequest) (*ListenFinalizeBlockResponse, error) {
	if err := m.Impl.ListenFinalizeBlock(ctx, *request.Req, *request.Res); err != nil {
		return nil, err
	}
	return &ListenFinalizeBlockResponse{}, nil
}

func (m GRPCServer) ListenCommit(ctx context.Context, request *ListenCommitRequest) (*ListenCommitResponse, error) {
	if err := m.Impl.ListenCommit(ctx, *request.Res, request.ChangeSet); err != nil {
		return nil, err
	}
	return &ListenCommitResponse{}, nil
}
