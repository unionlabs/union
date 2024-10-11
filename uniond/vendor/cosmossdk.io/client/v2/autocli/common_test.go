package autocli

import (
	"bytes"
	"context"
	"net"
	"testing"

	"github.com/spf13/cobra"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/protobuf/reflect/protoregistry"
	"gotest.tools/v3/assert"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	reflectionv2alpha1 "cosmossdk.io/api/cosmos/base/reflection/v2alpha1"
	"cosmossdk.io/client/v2/autocli/flag"
	"cosmossdk.io/client/v2/internal/testpb"
	"cosmossdk.io/x/bank"
	banktypes "cosmossdk.io/x/bank/types"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/codec/testutil"
	sdkkeyring "github.com/cosmos/cosmos-sdk/crypto/keyring"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

type fixture struct {
	conn      *testClientConn
	b         *Builder
	clientCtx client.Context
}

func initFixture(t *testing.T) *fixture {
	t.Helper()
	home := t.TempDir()
	server := grpc.NewServer()
	testpb.RegisterQueryServer(server, &testEchoServer{})
	reflectionv2alpha1.RegisterReflectionServiceServer(server, &testReflectionServer{})
	listener, err := net.Listen("tcp", "127.0.0.1:0")
	assert.NilError(t, err)
	go func() {
		err := server.Serve(listener)
		if err != nil {
			panic(err)
		}
	}()

	clientConn, err := grpc.NewClient(listener.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	assert.NilError(t, err)

	encodingConfig := moduletestutil.MakeTestEncodingConfig(testutil.CodecOptions{}, bank.AppModule{})
	kr, err := sdkkeyring.New(sdk.KeyringServiceName(), sdkkeyring.BackendMemory, home, nil, encodingConfig.Codec)
	assert.NilError(t, err)

	interfaceRegistry := encodingConfig.Codec.InterfaceRegistry()
	banktypes.RegisterInterfaces(interfaceRegistry)

	clientCtx := client.Context{}.
		WithAddressCodec(interfaceRegistry.SigningContext().AddressCodec()).
		WithValidatorAddressCodec(interfaceRegistry.SigningContext().ValidatorAddressCodec()).
		WithConsensusAddressCodec(addresscodec.NewBech32Codec("cosmosvalcons")).
		WithKeyring(kr).
		WithKeyringDir(home).
		WithHomeDir(home).
		WithViper("").
		WithInterfaceRegistry(interfaceRegistry).
		WithTxConfig(encodingConfig.TxConfig).
		WithAccountRetriever(client.MockAccountRetriever{}).
		WithChainID("autocli-test")

	conn := &testClientConn{ClientConn: clientConn}
	b := &Builder{
		Builder: flag.Builder{
			TypeResolver:          protoregistry.GlobalTypes,
			FileResolver:          protoregistry.GlobalFiles,
			AddressCodec:          clientCtx.AddressCodec,
			ValidatorAddressCodec: clientCtx.ValidatorAddressCodec,
			ConsensusAddressCodec: clientCtx.ConsensusAddressCodec,
		},
		GetClientConn: func(*cobra.Command) (grpc.ClientConnInterface, error) {
			return conn, nil
		},
		AddQueryConnFlags: flags.AddQueryFlagsToCmd,
		AddTxConnFlags:    flags.AddTxFlagsToCmd,
	}
	assert.NilError(t, b.ValidateAndComplete())

	return &fixture{
		conn:      conn,
		b:         b,
		clientCtx: clientCtx,
	}
}

func runCmd(fixture *fixture, command func(moduleName string, f *fixture) (*cobra.Command, error), args ...string) (*bytes.Buffer, error) {
	out := &bytes.Buffer{}
	cmd, err := command("test", fixture)
	if err != nil {
		return out, err
	}

	cmd.SetArgs(args)
	cmd.SetOut(out)
	return out, cmd.Execute()
}

type testReflectionServer struct {
	reflectionv2alpha1.UnimplementedReflectionServiceServer
}

func (t testReflectionServer) GetConfigurationDescriptor(_ context.Context, client *reflectionv2alpha1.GetConfigurationDescriptorRequest) (*reflectionv2alpha1.GetConfigurationDescriptorResponse, error) {
	return &reflectionv2alpha1.GetConfigurationDescriptorResponse{
		Config: &reflectionv2alpha1.ConfigurationDescriptor{
			Bech32AccountAddressPrefix: "cosmos",
		},
	}, nil
}

var _ reflectionv2alpha1.ReflectionServiceServer = testReflectionServer{}

type testClientConn struct {
	*grpc.ClientConn
	lastRequest  interface{}
	lastResponse interface{}
}

func (t *testClientConn) Invoke(ctx context.Context, method string, args, reply interface{}, opts ...grpc.CallOption) error {
	err := t.ClientConn.Invoke(ctx, method, args, reply, opts...)
	t.lastRequest = args
	t.lastResponse = reply
	return err
}

type testEchoServer struct {
	testpb.UnimplementedQueryServer
}

func (t testEchoServer) Echo(_ context.Context, request *testpb.EchoRequest) (*testpb.EchoResponse, error) {
	return &testpb.EchoResponse{Request: request}, nil
}

var _ testpb.QueryServer = testEchoServer{}

func TestEnhanceCommand(t *testing.T) {
	b := &Builder{}
	// Test that the command has a subcommand
	cmd := &cobra.Command{Use: "test"}
	cmd.AddCommand(&cobra.Command{Use: "test"})

	for i := 0; i < 2; i++ {
		cmdTp := cmdType(i)

		appOptions := AppOptions{
			ModuleOptions: map[string]*autocliv1.ModuleOptions{
				"test": {},
			},
		}

		err := b.enhanceCommandCommon(cmd, cmdTp, appOptions, map[string]*cobra.Command{})
		assert.NilError(t, err)

		cmd = &cobra.Command{Use: "test"}

		appOptions = AppOptions{
			ModuleOptions: map[string]*autocliv1.ModuleOptions{},
		}
		customCommands := map[string]*cobra.Command{
			"test2": {Use: "test"},
		}
		err = b.enhanceCommandCommon(cmd, cmdTp, appOptions, customCommands)
		assert.NilError(t, err)

		cmd = &cobra.Command{Use: "test"}
		appOptions = AppOptions{
			ModuleOptions: map[string]*autocliv1.ModuleOptions{
				"test": {Tx: nil},
			},
		}
		err = b.enhanceCommandCommon(cmd, cmdTp, appOptions, map[string]*cobra.Command{})
		assert.NilError(t, err)
	}
}

func TestErrorBuildCommand(t *testing.T) {
	fixture := initFixture(t)
	b := fixture.b
	b.AddQueryConnFlags = nil
	b.AddTxConnFlags = nil

	commandDescriptor := &autocliv1.ServiceCommandDescriptor{
		Service: testpb.Msg_ServiceDesc.ServiceName,
		RpcCommandOptions: []*autocliv1.RpcCommandOptions{
			{
				RpcMethod: "Send",
				PositionalArgs: []*autocliv1.PositionalArgDescriptor{
					{
						ProtoField: "un-existent-proto-field",
					},
				},
			},
		},
	}

	appOptions := AppOptions{
		ModuleOptions: map[string]*autocliv1.ModuleOptions{
			"test": {
				Query: commandDescriptor,
				Tx:    commandDescriptor,
			},
		},
	}

	_, err := b.BuildMsgCommand(context.Background(), appOptions, nil)
	assert.ErrorContains(t, err, "can't find field un-existent-proto-field")

	appOptions.ModuleOptions["test"].Tx = &autocliv1.ServiceCommandDescriptor{Service: "un-existent-service"}
	appOptions.ModuleOptions["test"].Query = &autocliv1.ServiceCommandDescriptor{Service: "un-existent-service"}
	_, err = b.BuildMsgCommand(context.Background(), appOptions, nil)
	assert.ErrorContains(t, err, "can't find service un-existent-service")
}
