package baseapp

import (
	gocontext "context"
	"fmt"

	abci "github.com/cometbft/cometbft/abci/types"
	gogogrpc "github.com/cosmos/gogoproto/grpc"
	"google.golang.org/grpc"

	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// QueryServiceTestHelper provides a helper for making grpc query service
// rpc calls in unit tests. It implements both the grpc Server and ClientConn
// interfaces needed to register a query service server and create a query
// service client.
type QueryServiceTestHelper struct {
	*GRPCQueryRouter
	Ctx sdk.Context
}

var (
	_ gogogrpc.Server     = &QueryServiceTestHelper{}
	_ gogogrpc.ClientConn = &QueryServiceTestHelper{}
)

// NewQueryServerTestHelper creates a new QueryServiceTestHelper that wraps
// the provided sdk.Context
func NewQueryServerTestHelper(ctx sdk.Context, interfaceRegistry types.InterfaceRegistry) *QueryServiceTestHelper {
	qrt := NewGRPCQueryRouter()
	qrt.SetInterfaceRegistry(interfaceRegistry)
	return &QueryServiceTestHelper{GRPCQueryRouter: qrt, Ctx: ctx}
}

// Invoke implements the grpc ClientConn.Invoke method
func (q *QueryServiceTestHelper) Invoke(_ gocontext.Context, method string, args, reply interface{}, _ ...grpc.CallOption) error {
	querier := q.Route(method)
	if querier == nil {
		return fmt.Errorf("handler not found for %s", method)
	}
	reqBz, err := q.cdc.Marshal(args)
	if err != nil {
		return err
	}

	res, err := querier(q.Ctx, &abci.RequestQuery{Data: reqBz})
	if err != nil {
		return err
	}

	err = q.cdc.Unmarshal(res.Value, reply)
	if err != nil {
		return err
	}

	return nil
}

// NewStream implements the grpc ClientConn.NewStream method
func (q *QueryServiceTestHelper) NewStream(gocontext.Context, *grpc.StreamDesc, string, ...grpc.CallOption) (grpc.ClientStream, error) {
	return nil, fmt.Errorf("not supported")
}
