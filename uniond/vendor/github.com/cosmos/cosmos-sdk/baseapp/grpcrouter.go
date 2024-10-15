package baseapp

import (
	"context"
	"fmt"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	gogogrpc "github.com/cosmos/gogoproto/grpc"
	"google.golang.org/grpc"
	"google.golang.org/grpc/encoding"
	"google.golang.org/protobuf/runtime/protoiface"

	"github.com/cosmos/cosmos-sdk/baseapp/internal/protocompat"
	"github.com/cosmos/cosmos-sdk/client/grpc/reflection"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type QueryRouter interface {
	HybridHandlerByRequestName(name string) []func(ctx context.Context, req, resp protoiface.MessageV1) error
	RegisterService(sd *grpc.ServiceDesc, handler interface{})
	ResponseNameByRequestName(requestName string) string
	Route(path string) GRPCQueryHandler
	SetInterfaceRegistry(interfaceRegistry codectypes.InterfaceRegistry)
}

// GRPCQueryRouter routes ABCI Query requests to GRPC handlers
type GRPCQueryRouter struct {
	// routes maps query handlers used in ABCIQuery.
	routes map[string]GRPCQueryHandler
	// hybridHandlers maps the request name to the handler. It is a hybrid handler which seamlessly
	// handles both gogo and protov2 messages.
	hybridHandlers map[string][]func(ctx context.Context, req, resp protoiface.MessageV1) error
	// responseByRequestName maps the request name to the response name.
	responseByRequestName map[string]string
	// binaryCodec is used to encode/decode binary protobuf messages.
	binaryCodec codec.BinaryCodec
	// cdc is the gRPC codec used by the router to correctly unmarshal messages.
	cdc encoding.Codec
	// serviceData contains the gRPC services and their handlers.
	serviceData []serviceData
}

// serviceData represents a gRPC service, along with its handler.
type serviceData struct {
	serviceDesc *grpc.ServiceDesc
	handler     interface{}
}

var (
	_ gogogrpc.Server = &GRPCQueryRouter{}
	_ QueryRouter     = &GRPCQueryRouter{}
)

// NewGRPCQueryRouter creates a new GRPCQueryRouter
func NewGRPCQueryRouter() *GRPCQueryRouter {
	return &GRPCQueryRouter{
		routes:                map[string]GRPCQueryHandler{},
		hybridHandlers:        map[string][]func(ctx context.Context, req, resp protoiface.MessageV1) error{},
		responseByRequestName: map[string]string{},
	}
}

// GRPCQueryHandler defines a function type which handles ABCI Query requests
// using gRPC
type GRPCQueryHandler = func(ctx sdk.Context, req *abci.QueryRequest) (*abci.QueryResponse, error)

// Route returns the GRPCQueryHandler for a given query route path or nil
// if not found
func (qrt *GRPCQueryRouter) Route(path string) GRPCQueryHandler {
	handler, found := qrt.routes[path]
	if !found {
		return nil
	}
	return handler
}

// RegisterService implements the gRPC Server.RegisterService method. sd is a gRPC
// service description, handler is an object which implements that gRPC service/
//
// This functions PANICS:
// - if a protobuf service is registered twice.
func (qrt *GRPCQueryRouter) RegisterService(sd *grpc.ServiceDesc, handler interface{}) {
	// adds a top-level query handler based on the gRPC service name
	for _, method := range sd.Methods {
		err := qrt.registerABCIQueryHandler(sd, method, handler)
		if err != nil {
			panic(err)
		}
		err = qrt.registerHybridHandler(sd, method, handler)
		if err != nil {
			panic(err)
		}
	}

	qrt.serviceData = append(qrt.serviceData, serviceData{
		serviceDesc: sd,
		handler:     handler,
	})
}

func (qrt *GRPCQueryRouter) registerABCIQueryHandler(sd *grpc.ServiceDesc, method grpc.MethodDesc, handler interface{}) error {
	fqName := fmt.Sprintf("/%s/%s", sd.ServiceName, method.MethodName)
	methodHandler := method.Handler

	// Check that each service is only registered once. If a service is
	// registered more than once, then we should error. Since we can't
	// return an error (`Server.RegisterService` interface restriction) we
	// panic (at startup).
	_, found := qrt.routes[fqName]
	if found {
		return fmt.Errorf(
			"gRPC query service %s has already been registered. Please make sure to only register each service once. "+
				"This usually means that there are conflicting modules registering the same gRPC query service",
			fqName,
		)
	}

	qrt.routes[fqName] = func(ctx sdk.Context, req *abci.QueryRequest) (*abci.QueryResponse, error) {
		// call the method handler from the service description with the handler object,
		// a wrapped sdk.Context with proto-unmarshaled data from the ABCI request data
		res, err := methodHandler(handler, ctx, func(i interface{}) error {
			return qrt.cdc.Unmarshal(req.Data, i)
		}, nil)
		if err != nil {
			return nil, err
		}

		// proto marshal the result bytes
		var resBytes []byte
		resBytes, err = qrt.cdc.Marshal(res)
		if err != nil {
			return nil, err
		}

		// return the result bytes as the response value
		return &abci.QueryResponse{
			Height: req.Height,
			Value:  resBytes,
		}, nil
	}
	return nil
}

func (qrt *GRPCQueryRouter) HybridHandlerByRequestName(name string) []func(ctx context.Context, req, resp protoiface.MessageV1) error {
	return qrt.hybridHandlers[name]
}

func (qrt *GRPCQueryRouter) ResponseNameByRequestName(requestName string) string {
	return qrt.responseByRequestName[requestName]
}

func (qrt *GRPCQueryRouter) registerHybridHandler(sd *grpc.ServiceDesc, method grpc.MethodDesc, handler interface{}) error {
	// extract message name from method descriptor
	inputName, err := protocompat.RequestFullNameFromMethodDesc(sd, method)
	if err != nil {
		return err
	}
	outputName, err := protocompat.ResponseFullNameFromMethodDesc(sd, method)
	if err != nil {
		return err
	}
	methodHandler, err := protocompat.MakeHybridHandler(qrt.binaryCodec, sd, method, handler)
	if err != nil {
		return err
	}
	// map input name to output name
	qrt.responseByRequestName[string(inputName)] = string(outputName)
	qrt.hybridHandlers[string(inputName)] = append(qrt.hybridHandlers[string(inputName)], methodHandler)
	return nil
}

// SetInterfaceRegistry sets the interface registry for the router. This will
// also register the interface reflection gRPC service.
func (qrt *GRPCQueryRouter) SetInterfaceRegistry(interfaceRegistry codectypes.InterfaceRegistry) {
	qrt.binaryCodec = codec.NewProtoCodec(interfaceRegistry)
	// instantiate the codec
	qrt.cdc = codec.NewProtoCodec(interfaceRegistry).GRPCCodec()
	// Once we have an interface registry, we can register the interface
	// registry reflection gRPC service.
	reflection.RegisterReflectionServiceServer(qrt, reflection.NewReflectionServiceServer(interfaceRegistry))
}
