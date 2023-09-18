package mock

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"path/filepath"

	db "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/types"
	"google.golang.org/grpc"

	bam "github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// NewApp creates a simple mock kvstore app for testing. It should work
// similar to a real app. Make sure rootDir is empty before running the test,
// in order to guarantee consistent results.
func NewApp(rootDir string, logger log.Logger) (abci.Application, error) {
	db, err := db.NewGoLevelDB("mock", filepath.Join(rootDir, "data"))
	if err != nil {
		return nil, err
	}

	capKeyMainStore := sdk.NewKVStoreKey("main")

	baseApp := bam.NewBaseApp("kvstore", logger, db, decodeTx)
	baseApp.MountStores(capKeyMainStore)
	baseApp.SetInitChainer(InitChainer(capKeyMainStore))

	interfaceRegistry := codectypes.NewInterfaceRegistry()
	interfaceRegistry.RegisterImplementations((*sdk.Msg)(nil), &kvstoreTx{})

	router := bam.NewMsgServiceRouter()
	router.SetInterfaceRegistry(interfaceRegistry)

	newDesc := &grpc.ServiceDesc{
		ServiceName: "test",
		Methods: []grpc.MethodDesc{
			{
				MethodName: "Test",
				Handler:    _Msg_Test_Handler,
			},
		},
	}

	router.RegisterService(newDesc, &MsgServerImpl{capKeyMainStore})
	baseApp.SetMsgServiceRouter(router)

	if err := baseApp.LoadLatestVersion(); err != nil {
		return nil, err
	}

	return baseApp, nil
}

// KVStoreHandler is a simple handler that takes kvstoreTx and writes
// them to the db.
func KVStoreHandler(storeKey storetypes.StoreKey) sdk.Handler {
	return func(ctx sdk.Context, msg sdk.Msg) (*sdk.Result, error) {
		dTx, ok := msg.(*kvstoreTx)
		if !ok {
			return nil, errors.New("KVStoreHandler should only receive kvstoreTx")
		}

		key := dTx.key
		value := dTx.value

		store := ctx.KVStore(storeKey)
		store.Set(key, value)

		return &sdk.Result{
			Log: fmt.Sprintf("set %s=%s", key, value),
		}, nil
	}
}

// basic KV structure
type KV struct {
	Key   string `json:"key"`
	Value string `json:"value"`
}

// What Genesis JSON is formatted as
type GenesisJSON struct {
	Values []KV `json:"values"`
}

// InitChainer returns a function that can initialize the chain
// with key/value pairs
func InitChainer(key storetypes.StoreKey) func(sdk.Context, abci.RequestInitChain) abci.ResponseInitChain {
	return func(ctx sdk.Context, req abci.RequestInitChain) abci.ResponseInitChain {
		stateJSON := req.AppStateBytes

		genesisState := new(GenesisJSON)
		err := json.Unmarshal(stateJSON, genesisState)
		if err != nil {
			panic(err) // TODO https://github.com/cosmos/cosmos-sdk/issues/468
			// return sdk.ErrGenesisParse("").TraceCause(err, "")
		}

		for _, val := range genesisState.Values {
			store := ctx.KVStore(key)
			store.Set([]byte(val.Key), []byte(val.Value))
		}
		return abci.ResponseInitChain{}
	}
}

// AppGenState can be passed into InitCmd, returns a static string of a few
// key-values that can be parsed by InitChainer
func AppGenState(_ *codec.LegacyAmino, _ types.GenesisDoc, _ []json.RawMessage) (appState json.RawMessage, err error) {
	appState = json.RawMessage(`{
  "values": [
    {
        "key": "hello",
        "value": "goodbye"
    },
    {
        "key": "foo",
        "value": "bar"
    }
  ]
}`)
	return
}

// AppGenStateEmpty returns an empty transaction state for mocking.
func AppGenStateEmpty(_ *codec.LegacyAmino, _ types.GenesisDoc, _ []json.RawMessage) (appState json.RawMessage, err error) {
	appState = json.RawMessage(``)
	return
}

// Manually write the handlers for this custom message
type MsgServer interface {
	Test(ctx context.Context, msg *kvstoreTx) (*sdk.Result, error)
}

type MsgServerImpl struct {
	capKeyMainStore *storetypes.KVStoreKey
}

func _Msg_Test_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(kvstoreTx)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(MsgServer).Test(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/kvstoreTx",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(MsgServer).Test(ctx, req.(*kvstoreTx))
	}
	return interceptor(ctx, in, info, handler)
}

func (m MsgServerImpl) Test(ctx context.Context, msg *kvstoreTx) (*sdk.Result, error) {
	return KVStoreHandler(m.capKeyMainStore)(sdk.UnwrapSDKContext(ctx), msg)
}
