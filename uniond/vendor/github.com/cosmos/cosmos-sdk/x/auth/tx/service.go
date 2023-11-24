package tx

import (
	"context"
	"fmt"
	"strings"

	gogogrpc "github.com/cosmos/gogoproto/grpc"
	"github.com/golang/protobuf/proto" //nolint:staticcheck // keep legacy for now
	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/grpc/cmtservice"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/query"
	txtypes "github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/x/auth/migrations/legacytx"
)

// baseAppSimulateFn is the signature of the Baseapp#Simulate function.
type baseAppSimulateFn func(txBytes []byte) (sdk.GasInfo, *sdk.Result, error)

// txServer is the server for the protobuf Tx service.
type txServer struct {
	clientCtx         client.Context
	simulate          baseAppSimulateFn
	interfaceRegistry codectypes.InterfaceRegistry
}

// NewTxServer creates a new Tx service server.
func NewTxServer(clientCtx client.Context, simulate baseAppSimulateFn, interfaceRegistry codectypes.InterfaceRegistry) txtypes.ServiceServer {
	return txServer{
		clientCtx:         clientCtx,
		simulate:          simulate,
		interfaceRegistry: interfaceRegistry,
	}
}

var _ txtypes.ServiceServer = txServer{}

// GetTxsEvent implements the ServiceServer.TxsByEvents RPC method.
func (s txServer) GetTxsEvent(ctx context.Context, req *txtypes.GetTxsEventRequest) (*txtypes.GetTxsEventResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "request cannot be nil")
	}

	orderBy := parseOrderBy(req.OrderBy)

	result, err := QueryTxsByEvents(s.clientCtx, int(req.Page), int(req.Limit), req.Query, orderBy)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	txsList := make([]*txtypes.Tx, len(result.Txs))
	for i, tx := range result.Txs {
		protoTx, ok := tx.Tx.GetCachedValue().(*txtypes.Tx)
		if !ok {
			return nil, status.Errorf(codes.Internal, "expected %T, got %T", txtypes.Tx{}, tx.Tx.GetCachedValue())
		}

		txsList[i] = protoTx
	}

	return &txtypes.GetTxsEventResponse{
		Txs:         txsList,
		TxResponses: result.Txs,
		Total:       result.TotalCount,
	}, nil
}

// Simulate implements the ServiceServer.Simulate RPC method.
func (s txServer) Simulate(ctx context.Context, req *txtypes.SimulateRequest) (*txtypes.SimulateResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid empty tx")
	}

	txBytes := req.TxBytes
	if txBytes == nil && req.Tx != nil {
		// This block is for backwards-compatibility.
		// We used to support passing a `Tx` in req. But if we do that, sig
		// verification might not pass, because the .Marshal() below might not
		// be the same marshaling done by the client.
		var err error
		txBytes, err = proto.Marshal(req.Tx)
		if err != nil {
			return nil, status.Errorf(codes.InvalidArgument, "invalid tx; %v", err)
		}
	}

	if txBytes == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty txBytes is not allowed")
	}

	gasInfo, result, err := s.simulate(txBytes)
	if err != nil {
		return nil, status.Errorf(codes.Unknown, "%v With gas wanted: '%d' and gas used: '%d' ", err, gasInfo.GasWanted, gasInfo.GasUsed)
	}

	return &txtypes.SimulateResponse{
		GasInfo: &gasInfo,
		Result:  result,
	}, nil
}

// GetTx implements the ServiceServer.GetTx RPC method.
func (s txServer) GetTx(ctx context.Context, req *txtypes.GetTxRequest) (*txtypes.GetTxResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "request cannot be nil")
	}

	if len(req.Hash) == 0 {
		return nil, status.Error(codes.InvalidArgument, "tx hash cannot be empty")
	}

	// TODO We should also check the proof flag in gRPC header.
	// https://github.com/cosmos/cosmos-sdk/issues/7036.
	result, err := QueryTx(s.clientCtx, req.Hash)
	if err != nil {
		if strings.Contains(err.Error(), "not found") {
			return nil, status.Errorf(codes.NotFound, "tx not found: %s", req.Hash)
		}

		return nil, err
	}

	protoTx, ok := result.Tx.GetCachedValue().(*txtypes.Tx)
	if !ok {
		return nil, status.Errorf(codes.Internal, "expected %T, got %T", txtypes.Tx{}, result.Tx.GetCachedValue())
	}

	return &txtypes.GetTxResponse{
		Tx:         protoTx,
		TxResponse: result,
	}, nil
}

// protoTxProvider is a type which can provide a proto transaction. It is a
// workaround to get access to the wrapper TxBuilder's method GetProtoTx().
// ref: https://github.com/cosmos/cosmos-sdk/issues/10347
type protoTxProvider interface {
	GetProtoTx() *txtypes.Tx
}

// GetBlockWithTxs returns a block with decoded txs.
func (s txServer) GetBlockWithTxs(ctx context.Context, req *txtypes.GetBlockWithTxsRequest) (*txtypes.GetBlockWithTxsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "request cannot be nil")
	}

	sdkCtx := sdk.UnwrapSDKContext(ctx)
	currentHeight := sdkCtx.BlockHeight()

	if req.Height < 1 || req.Height > currentHeight {
		return nil, sdkerrors.ErrInvalidHeight.Wrapf("requested height %d but height must not be less than 1 "+
			"or greater than the current height %d", req.Height, currentHeight)
	}

	blockID, block, err := cmtservice.GetProtoBlock(ctx, s.clientCtx, &req.Height)
	if err != nil {
		return nil, err
	}

	var offset, limit uint64
	if req.Pagination != nil {
		offset = req.Pagination.Offset
		limit = req.Pagination.Limit
	} else {
		offset = 0
		limit = query.DefaultLimit
	}

	blockTxs := block.Data.Txs
	blockTxsLn := uint64(len(blockTxs))
	txs := make([]*txtypes.Tx, 0, limit)
	if offset >= blockTxsLn && blockTxsLn != 0 {
		return nil, sdkerrors.ErrInvalidRequest.Wrapf("out of range: cannot paginate %d txs with offset %d and limit %d", blockTxsLn, offset, limit)
	}
	decodeTxAt := func(i uint64) error {
		tx := blockTxs[i]
		txb, err := s.clientCtx.TxConfig.TxDecoder()(tx)
		if err != nil {
			return err
		}
		p, ok := txb.(protoTxProvider)
		if !ok {
			return sdkerrors.ErrTxDecode.Wrapf("could not cast %T to %T", txb, txtypes.Tx{})
		}
		txs = append(txs, p.GetProtoTx())
		return nil
	}
	if req.Pagination != nil && req.Pagination.Reverse {
		for i, count := offset, uint64(0); i > 0 && count != limit; i, count = i-1, count+1 {
			if err = decodeTxAt(i); err != nil {
				return nil, err
			}
		}
	} else {
		for i, count := offset, uint64(0); i < blockTxsLn && count != limit; i, count = i+1, count+1 {
			if err = decodeTxAt(i); err != nil {
				return nil, err
			}
		}
	}

	return &txtypes.GetBlockWithTxsResponse{
		Txs:     txs,
		BlockId: &blockID,
		Block:   block,
		Pagination: &query.PageResponse{
			Total: blockTxsLn,
		},
	}, nil
}

// BroadcastTx implements the ServiceServer.BroadcastTx RPC method.
func (s txServer) BroadcastTx(ctx context.Context, req *txtypes.BroadcastTxRequest) (*txtypes.BroadcastTxResponse, error) {
	return client.TxServiceBroadcast(ctx, s.clientCtx, req)
}

// TxEncode implements the ServiceServer.TxEncode RPC method.
func (s txServer) TxEncode(ctx context.Context, req *txtypes.TxEncodeRequest) (*txtypes.TxEncodeResponse, error) {
	if req.Tx == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid empty tx")
	}

	txBuilder := &wrapper{tx: req.Tx}

	encodedBytes, err := s.clientCtx.TxConfig.TxEncoder()(txBuilder)
	if err != nil {
		return nil, err
	}

	return &txtypes.TxEncodeResponse{
		TxBytes: encodedBytes,
	}, nil
}

// TxEncodeAmino implements the ServiceServer.TxEncodeAmino RPC method.
func (s txServer) TxEncodeAmino(ctx context.Context, req *txtypes.TxEncodeAminoRequest) (*txtypes.TxEncodeAminoResponse, error) {
	if req.AminoJson == "" {
		return nil, status.Error(codes.InvalidArgument, "invalid empty tx json")
	}

	var stdTx legacytx.StdTx
	err := s.clientCtx.LegacyAmino.UnmarshalJSON([]byte(req.AminoJson), &stdTx)
	if err != nil {
		return nil, err
	}

	encodedBytes, err := s.clientCtx.LegacyAmino.Marshal(stdTx)
	if err != nil {
		return nil, err
	}

	return &txtypes.TxEncodeAminoResponse{
		AminoBinary: encodedBytes,
	}, nil
}

// TxDecode implements the ServiceServer.TxDecode RPC method.
func (s txServer) TxDecode(ctx context.Context, req *txtypes.TxDecodeRequest) (*txtypes.TxDecodeResponse, error) {
	if req.TxBytes == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid empty tx bytes")
	}

	txb, err := s.clientCtx.TxConfig.TxDecoder()(req.TxBytes)
	if err != nil {
		return nil, err
	}

	txWrapper, ok := txb.(*wrapper)
	if ok {
		return &txtypes.TxDecodeResponse{
			Tx: txWrapper.tx,
		}, nil
	}

	return nil, fmt.Errorf("expected %T, got %T", &wrapper{}, txb)
}

// TxDecodeAmino implements the ServiceServer.TxDecodeAmino RPC method.
func (s txServer) TxDecodeAmino(ctx context.Context, req *txtypes.TxDecodeAminoRequest) (*txtypes.TxDecodeAminoResponse, error) {
	if req.AminoBinary == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid empty tx bytes")
	}

	var stdTx legacytx.StdTx
	err := s.clientCtx.LegacyAmino.Unmarshal(req.AminoBinary, &stdTx)
	if err != nil {
		return nil, err
	}

	res, err := s.clientCtx.LegacyAmino.MarshalJSON(stdTx)
	if err != nil {
		return nil, err
	}

	return &txtypes.TxDecodeAminoResponse{
		AminoJson: string(res),
	}, nil
}

// RegisterTxService registers the tx service on the gRPC router.
func RegisterTxService(
	qrt gogogrpc.Server,
	clientCtx client.Context,
	simulateFn baseAppSimulateFn,
	interfaceRegistry codectypes.InterfaceRegistry,
) {
	txtypes.RegisterServiceServer(
		qrt,
		NewTxServer(clientCtx, simulateFn, interfaceRegistry),
	)
}

// RegisterGRPCGatewayRoutes mounts the tx service's GRPC-gateway routes on the
// given Mux.
func RegisterGRPCGatewayRoutes(clientConn gogogrpc.ClientConn, mux *runtime.ServeMux) {
	txtypes.RegisterServiceHandlerClient(context.Background(), mux, txtypes.NewServiceClient(clientConn))
}

func parseOrderBy(orderBy txtypes.OrderBy) string {
	switch orderBy {
	case txtypes.OrderBy_ORDER_BY_ASC:
		return "asc"
	case txtypes.OrderBy_ORDER_BY_DESC:
		return "desc"
	default:
		return "" // Defaults to CometBFT's default, which is `asc` now.
	}
}
