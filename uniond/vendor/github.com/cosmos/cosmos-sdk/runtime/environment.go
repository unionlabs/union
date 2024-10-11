package runtime

import (
	"context"

	"google.golang.org/grpc"
	"google.golang.org/protobuf/runtime/protoiface"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/log"
	"cosmossdk.io/core/store"

	"github.com/cosmos/cosmos-sdk/baseapp"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// NewEnvironment creates a new environment for the application
// For setting custom services that aren't set by default, use the EnvOption
// Note: Depinject always provide an environment with all services (mandatory and optional)
func NewEnvironment(
	kvService store.KVStoreService,
	logger log.Logger,
	opts ...EnvOption,
) appmodule.Environment {
	env := appmodule.Environment{
		Logger:             logger,
		EventService:       EventService{},
		HeaderService:      HeaderService{},
		BranchService:      BranchService{},
		GasService:         GasService{},
		TransactionService: TransactionService{},
		KVStoreService:     kvService,
		MsgRouterService:   NewMsgRouterService(failingMsgRouter{}),
		QueryRouterService: NewQueryRouterService(failingQueryRouter{}),
		MemStoreService:    failingMemStore{},
	}

	for _, opt := range opts {
		opt(&env)
	}

	return env
}

type EnvOption func(*appmodule.Environment)

func EnvWithMsgRouterService(msgServiceRouter *baseapp.MsgServiceRouter) EnvOption {
	return func(env *appmodule.Environment) {
		env.MsgRouterService = NewMsgRouterService(msgServiceRouter)
	}
}

func EnvWithQueryRouterService(queryServiceRouter *baseapp.GRPCQueryRouter) EnvOption {
	return func(env *appmodule.Environment) {
		env.QueryRouterService = NewQueryRouterService(queryServiceRouter)
	}
}

func EnvWithMemStoreService(memStoreService store.MemoryStoreService) EnvOption {
	return func(env *appmodule.Environment) {
		env.MemStoreService = memStoreService
	}
}

// failingMsgRouter is a message router that panics when accessed
// this is to ensure all fields are set in environment
type failingMsgRouter struct {
	baseapp.MessageRouter
}

func (failingMsgRouter) Handler(msg sdk.Msg) baseapp.MsgServiceHandler {
	panic("message router not set")
}

func (failingMsgRouter) HandlerByTypeURL(typeURL string) baseapp.MsgServiceHandler {
	panic("message router not set")
}

func (failingMsgRouter) ResponseNameByMsgName(msgName string) string {
	panic("message router not set")
}

func (failingMsgRouter) HybridHandlerByMsgName(msgName string) func(ctx context.Context, req, resp protoiface.MessageV1) error {
	panic("message router not set")
}

// failingQueryRouter is a query router that panics when accessed
// this is to ensure all fields are set in environment
type failingQueryRouter struct {
	baseapp.QueryRouter
}

func (failingQueryRouter) HybridHandlerByRequestName(name string) []func(ctx context.Context, req, resp protoiface.MessageV1) error {
	panic("query router not set")
}

func (failingQueryRouter) RegisterService(sd *grpc.ServiceDesc, handler interface{}) {
	panic("query router not set")
}

func (failingQueryRouter) ResponseNameByRequestName(requestName string) string {
	panic("query router not set")
}

func (failingQueryRouter) Route(path string) baseapp.GRPCQueryHandler {
	panic("query router not set")
}

func (failingQueryRouter) SetInterfaceRegistry(interfaceRegistry codectypes.InterfaceRegistry) {
	panic("query router not set")
}

// failingMemStore is a memstore that panics when accessed
// this is to ensure all fields are set in environment
type failingMemStore struct {
	store.MemoryStoreService
}

func (failingMemStore) OpenMemoryStore(context.Context) store.KVStore {
	panic("memory store not set")
}
