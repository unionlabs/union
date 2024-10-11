package cmtservice

import (
	"context"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	gogogrpc "github.com/cosmos/gogoproto/grpc"
	gogoprotoany "github.com/cosmos/gogoproto/types/any"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"cosmossdk.io/core/address"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	qtypes "github.com/cosmos/cosmos-sdk/types/query"
	"github.com/cosmos/cosmos-sdk/version"
)

var (
	_ ServiceServer                        = queryServer{}
	_ gogoprotoany.UnpackInterfacesMessage = &GetLatestValidatorSetResponse{}
)

type (
	abciQueryFn = func(context.Context, *abci.QueryRequest) (*abci.QueryResponse, error)

	queryServer struct {
		rpc            CometRPC
		queryFn        abciQueryFn
		consensusCodec address.Codec
	}
)

// NewQueryServer creates a new CometBFT query server.
func NewQueryServer(
	clientCtx CometRPC,
	queryFn abciQueryFn,
	consensusAddressCodec address.Codec,
) ServiceServer {
	return queryServer{
		rpc:            clientCtx,
		queryFn:        queryFn,
		consensusCodec: consensusAddressCodec,
	}
}

// GetSyncing implements ServiceServer.GetSyncing
func (s queryServer) GetSyncing(ctx context.Context, _ *GetSyncingRequest) (*GetSyncingResponse, error) {
	status, err := GetNodeStatus(ctx, s.rpc)
	if err != nil {
		return nil, err
	}

	return &GetSyncingResponse{
		Syncing: status.SyncInfo.CatchingUp,
	}, nil
}

// GetLatestBlock implements ServiceServer.GetLatestBlock
func (s queryServer) GetLatestBlock(ctx context.Context, _ *GetLatestBlockRequest) (*GetLatestBlockResponse, error) {
	status, err := getBlock(ctx, s.rpc, nil)
	if err != nil {
		return nil, err
	}

	protoBlockID := status.BlockID.ToProto()
	protoBlock, err := status.Block.ToProto()
	if err != nil {
		return nil, err
	}

	sdkBlock, err := convertBlock(protoBlock, s.consensusCodec)
	if err != nil {
		return nil, err
	}

	return &GetLatestBlockResponse{
		BlockId:  &protoBlockID,
		Block:    protoBlock,
		SdkBlock: sdkBlock,
	}, nil
}

// GetBlockByHeight implements ServiceServer.GetBlockByHeight
func (s queryServer) GetBlockByHeight(ctx context.Context, req *GetBlockByHeightRequest) (*GetBlockByHeightResponse, error) {
	blockHeight, err := getBlockHeight(ctx, s.rpc)
	if err != nil {
		return nil, err
	}

	if req.Height > blockHeight {
		return nil, status.Error(codes.InvalidArgument, "requested block height is bigger then the chain length")
	}

	protoBlockID, protoBlock, err := GetProtoBlock(ctx, s.rpc, &req.Height)
	if err != nil {
		return nil, err
	}

	sdkBlock, err := convertBlock(protoBlock, s.consensusCodec)
	if err != nil {
		return nil, err
	}

	return &GetBlockByHeightResponse{
		BlockId:  &protoBlockID,
		Block:    protoBlock,
		SdkBlock: sdkBlock,
	}, nil
}

// GetLatestValidatorSet implements ServiceServer.GetLatestValidatorSet
func (s queryServer) GetLatestValidatorSet(ctx context.Context, req *GetLatestValidatorSetRequest) (*GetLatestValidatorSetResponse, error) {
	page, limit, err := qtypes.ParsePagination(req.Pagination)
	if err != nil {
		return nil, err
	}

	return ValidatorsOutput(ctx, s.rpc, s.consensusCodec, nil, page, limit)
}

func (m *GetLatestValidatorSetResponse) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	var pubKey cryptotypes.PubKey
	for _, val := range m.Validators {
		err := unpacker.UnpackAny(val.PubKey, &pubKey)
		if err != nil {
			return err
		}
	}

	return nil
}

// GetValidatorSetByHeight implements ServiceServer.GetValidatorSetByHeight
func (s queryServer) GetValidatorSetByHeight(ctx context.Context, req *GetValidatorSetByHeightRequest) (*GetValidatorSetByHeightResponse, error) {
	page, limit, err := qtypes.ParsePagination(req.Pagination)
	if err != nil {
		return nil, err
	}

	blockHeight, err := getBlockHeight(ctx, s.rpc)
	if err != nil {
		return nil, status.Error(codes.Internal, "failed to parse chain height")
	}

	if req.Height > blockHeight {
		return nil, status.Error(codes.InvalidArgument, "requested block height is bigger then the chain length")
	}

	r, err := ValidatorsOutput(ctx, s.rpc, s.consensusCodec, &req.Height, page, limit)
	if err != nil {
		return nil, err
	}

	return &GetValidatorSetByHeightResponse{
		BlockHeight: r.BlockHeight,
		Validators:  r.Validators,
		Pagination:  r.Pagination,
	}, nil
}

func ValidatorsOutput(ctx context.Context, rpc CometRPC, consCodec address.Codec, height *int64, page, limit int) (*GetLatestValidatorSetResponse, error) {
	vs, err := getValidators(ctx, rpc, height, page, limit)
	if err != nil {
		return nil, err
	}

	resp := GetLatestValidatorSetResponse{
		BlockHeight: vs.BlockHeight,
		Validators:  make([]*Validator, len(vs.Validators)),
		Pagination: &qtypes.PageResponse{
			Total: uint64(vs.Total),
		},
	}

	for i, v := range vs.Validators {
		pk, err := cryptocodec.FromCmtPubKeyInterface(v.PubKey)
		if err != nil {
			return nil, err
		}
		anyPub, err := codectypes.NewAnyWithValue(pk)
		if err != nil {
			return nil, err
		}

		addr, err := consCodec.BytesToString(v.Address)
		if err != nil {
			return nil, err
		}

		resp.Validators[i] = &Validator{
			Address:          addr,
			ProposerPriority: v.ProposerPriority,
			PubKey:           anyPub,
			VotingPower:      v.VotingPower,
		}
	}

	return &resp, nil
}

// GetNodeInfo implements ServiceServer.GetNodeInfo
func (s queryServer) GetNodeInfo(ctx context.Context, _ *GetNodeInfoRequest) (*GetNodeInfoResponse, error) {
	status, err := GetNodeStatus(ctx, s.rpc)
	if err != nil {
		return nil, err
	}

	protoNodeInfo := status.NodeInfo.ToProto()
	nodeInfo := version.NewInfo()

	deps := make([]*Module, len(nodeInfo.BuildDeps))

	for i, dep := range nodeInfo.BuildDeps {
		deps[i] = &Module{
			Path:    dep.Path,
			Sum:     dep.Sum,
			Version: dep.Version,
		}
	}

	resp := GetNodeInfoResponse{
		DefaultNodeInfo: protoNodeInfo,
		ApplicationVersion: &VersionInfo{
			AppName:          nodeInfo.AppName,
			Name:             nodeInfo.Name,
			GitCommit:        nodeInfo.GitCommit,
			GoVersion:        nodeInfo.GoVersion,
			Version:          nodeInfo.Version,
			BuildTags:        nodeInfo.BuildTags,
			BuildDeps:        deps,
			CosmosSdkVersion: nodeInfo.CosmosSdkVersion,
		},
	}
	return &resp, nil
}

func (s queryServer) ABCIQuery(ctx context.Context, req *ABCIQueryRequest) (*ABCIQueryResponse, error) {
	if s.queryFn == nil {
		return nil, status.Error(codes.Internal, "ABCI Query handler undefined")
	}
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}
	if len(req.Path) == 0 {
		return nil, status.Error(codes.InvalidArgument, "empty query path")
	}

	if path := baseapp.SplitABCIQueryPath(req.Path); len(path) > 0 {
		switch path[0] {
		case baseapp.QueryPathApp, baseapp.QueryPathStore, baseapp.QueryPathP2P, baseapp.QueryPathCustom:
			// valid path

		default:
			// Otherwise, error as to prevent either valid gRPC service requests or
			// bogus ABCI queries.
			return nil, status.Errorf(codes.InvalidArgument, "unsupported ABCI query path: %s", req.Path)
		}
	}

	res, err := s.queryFn(ctx, req.ToABCIRequestQuery())
	if err != nil {
		return nil, err
	}
	return &ABCIQueryResponse{
		Code:      res.Code,
		Log:       res.Log,
		Info:      res.Info,
		Index:     res.Index,
		Key:       res.Key,
		Value:     res.Value,
		ProofOps:  res.ProofOps,
		Height:    res.Height,
		Codespace: res.Codespace,
	}, nil
}

// RegisterTendermintService registers the CometBFT queries on the gRPC router.
func RegisterTendermintService(
	clientCtx client.Context,
	server gogogrpc.Server,
	iRegistry codectypes.InterfaceRegistry,
	queryFn abciQueryFn,
) {
	node, err := clientCtx.GetNode()
	if err != nil {
		panic(err)
	}
	RegisterServiceServer(server, NewQueryServer(node, queryFn, clientCtx.ConsensusAddressCodec))
}

// RegisterGRPCGatewayRoutes mounts the CometBFT service's GRPC-gateway routes on the
// given Mux.
func RegisterGRPCGatewayRoutes(clientConn gogogrpc.ClientConn, mux *runtime.ServeMux) {
	_ = RegisterServiceHandlerClient(context.Background(), mux, NewServiceClient(clientConn))
}
