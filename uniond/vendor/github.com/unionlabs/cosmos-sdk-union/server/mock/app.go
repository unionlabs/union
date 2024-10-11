package mock

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"path/filepath"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	dbm "github.com/cosmos/cosmos-db"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protodesc"
	"google.golang.org/protobuf/reflect/protoregistry"
	"google.golang.org/protobuf/types/descriptorpb"

	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"

	bam "github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/testutil"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
)

// NewApp creates a simple mock kvstore app for testing. It should work
// similar to a real app. Make sure rootDir is empty before running the test,
// in order to guarantee consistent results.
func NewApp(rootDir string, logger log.Logger) (servertypes.ABCI, error) {
	db, err := dbm.NewGoLevelDB("mock", filepath.Join(rootDir, "data"), nil)
	if err != nil {
		return nil, err
	}

	capKeyMainStore := storetypes.NewKVStoreKey("main")

	baseApp := bam.NewBaseApp("kvstore", logger, db, decodeTx)
	baseApp.MountStores(capKeyMainStore)
	baseApp.SetInitChainer(InitChainer(capKeyMainStore))

	interfaceRegistry := testutil.CodecOptions{}.NewInterfaceRegistry()
	interfaceRegistry.RegisterImplementations((*sdk.Msg)(nil), &KVStoreTx{})
	baseApp.SetInterfaceRegistry(interfaceRegistry)

	router := bam.NewMsgServiceRouter()
	router.SetInterfaceRegistry(interfaceRegistry)

	newDesc := &grpc.ServiceDesc{
		ServiceName: "Test",
		Methods: []grpc.MethodDesc{
			{
				MethodName: "Test",
				Handler:    MsgTestHandler,
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

// KVStoreHandler is a simple handler that takes KVStoreTx and writes
// them to the db.
func KVStoreHandler(storeKey storetypes.StoreKey) bam.MsgServiceHandler {
	return func(ctx sdk.Context, msg sdk.Msg) (*sdk.Result, error) {
		dTx, ok := msg.(*KVStoreTx)
		if !ok {
			return nil, errors.New("KVStoreHandler should only receive KVStoreTx")
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

// KV is a basic kv structure
type KV struct {
	Key   string `json:"key"`
	Value string `json:"value"`
}

// GenesisJSON what genesis JSON is formatted as
type GenesisJSON struct {
	Values []KV `json:"values"`
}

// InitChainer returns a function that can initialize the chain
// with key/value pairs
func InitChainer(key storetypes.StoreKey) func(sdk.Context, *abci.InitChainRequest) (*abci.InitChainResponse, error) {
	return func(ctx sdk.Context, req *abci.InitChainRequest) (*abci.InitChainResponse, error) {
		stateJSON := req.AppStateBytes

		genesisState := new(GenesisJSON)
		err := json.Unmarshal(stateJSON, genesisState)
		if err != nil {
			return &abci.InitChainResponse{}, err
		}

		for _, val := range genesisState.Values {
			store := ctx.KVStore(key)
			store.Set([]byte(val.Key), []byte(val.Value))
		}
		return &abci.InitChainResponse{}, nil
	}
}

// AppGenState can be passed into InitCmd, returns a static string of a few
// key-values that can be parsed by InitChainer
func AppGenState(_ *codec.LegacyAmino, _ genutiltypes.AppGenesis, _ []json.RawMessage) (appState json.RawMessage, err error) {
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
func AppGenStateEmpty(_ *codec.LegacyAmino, _ genutiltypes.AppGenesis, _ []json.RawMessage) (appState json.RawMessage, err error) {
	appState = json.RawMessage(``)
	return
}

// MsgServer manually write the handlers for this custom message
type MsgServer interface {
	Test(ctx context.Context, msg *KVStoreTx) (*sdk.Result, error)
}

type MsgServerImpl struct {
	capKeyMainStore *storetypes.KVStoreKey
}

func MsgTestHandler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(KVStoreTx)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(MsgServer).Test(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/KVStoreTx",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(MsgServer).Test(ctx, req.(*KVStoreTx))
	}
	return interceptor(ctx, in, info, handler)
}

func (m MsgServerImpl) Test(ctx context.Context, msg *KVStoreTx) (*sdk.Result, error) {
	return KVStoreHandler(m.capKeyMainStore)(sdk.UnwrapSDKContext(ctx), msg)
}

func init() {
	err := registerFauxDescriptor()
	if err != nil {
		panic(err)
	}
}

func registerFauxDescriptor() error {
	fauxDescriptor, err := protodesc.NewFile(&descriptorpb.FileDescriptorProto{
		Name:             proto.String("faux_proto/test.proto"),
		Dependency:       nil,
		PublicDependency: nil,
		WeakDependency:   nil,
		MessageType: []*descriptorpb.DescriptorProto{
			{
				Name: proto.String("KVStoreTx"),
			},
		},
		EnumType: nil,
		Service: []*descriptorpb.ServiceDescriptorProto{
			{
				Name: proto.String("Test"),
				Method: []*descriptorpb.MethodDescriptorProto{
					{
						Name:       proto.String("Test"),
						InputType:  proto.String("KVStoreTx"),
						OutputType: proto.String("KVStoreTx"),
					},
				},
			},
		},
		Extension:      nil,
		Options:        nil,
		SourceCodeInfo: nil,
		Syntax:         nil,
		Edition:        nil,
	}, nil)
	if err != nil {
		return err
	}

	return protoregistry.GlobalFiles.RegisterFile(fauxDescriptor)
}
