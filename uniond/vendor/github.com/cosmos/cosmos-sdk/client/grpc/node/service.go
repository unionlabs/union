package node

import (
	context "context"

	gogogrpc "github.com/cosmos/gogoproto/grpc"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/server/config"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// RegisterNodeService registers the node gRPC service on the provided gRPC router.
func RegisterNodeService(clientCtx client.Context, server gogogrpc.Server, cfg config.Config) {
	RegisterServiceServer(server, NewQueryServer(clientCtx, cfg))
}

// RegisterGRPCGatewayRoutes mounts the node gRPC service's GRPC-gateway routes
// on the given mux object.
func RegisterGRPCGatewayRoutes(clientConn gogogrpc.ClientConn, mux *runtime.ServeMux) {
	_ = RegisterServiceHandlerClient(context.Background(), mux, NewServiceClient(clientConn))
}

var _ ServiceServer = queryServer{}

type queryServer struct {
	clientCtx client.Context
	cfg       config.Config
}

func NewQueryServer(clientCtx client.Context, cfg config.Config) ServiceServer {
	return queryServer{
		clientCtx: clientCtx,
	}
}

func (s queryServer) Config(ctx context.Context, _ *ConfigRequest) (*ConfigResponse, error) {
	sdkCtx := sdk.UnwrapSDKContext(ctx)

	return &ConfigResponse{
		MinimumGasPrice:   sdkCtx.MinGasPrices().String(),
		PruningKeepRecent: s.cfg.PruningKeepRecent,
		PruningInterval:   s.cfg.PruningInterval,
	}, nil
}

func (s queryServer) Status(ctx context.Context, _ *StatusRequest) (*StatusResponse, error) {
	sdkCtx := sdk.UnwrapSDKContext(ctx)

	blockTime := sdkCtx.BlockTime()

	return &StatusResponse{
		// TODO: Get earliest version from store.
		//
		// Ref: ...
		// EarliestStoreHeight: sdkCtx.MultiStore(),
		Height:        uint64(sdkCtx.BlockHeight()),
		Timestamp:     &blockTime,
		AppHash:       sdkCtx.BlockHeader().AppHash,
		ValidatorHash: sdkCtx.BlockHeader().NextValidatorsHash,
	}, nil
}
