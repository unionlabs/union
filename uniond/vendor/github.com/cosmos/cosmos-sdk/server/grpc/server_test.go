package grpc_test

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	"github.com/jhump/protoreflect/grpcreflect"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
	rpb "google.golang.org/grpc/reflection/grpc_reflection_v1alpha"

	"github.com/cosmos/cosmos-sdk/client"
	reflectionv1 "github.com/cosmos/cosmos-sdk/client/grpc/reflection"
	clienttx "github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/codec"
	reflectionv2 "github.com/cosmos/cosmos-sdk/server/grpc/reflection/v2alpha1"
	"github.com/cosmos/cosmos-sdk/testutil/network"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	grpctypes "github.com/cosmos/cosmos-sdk/types/grpc"
	txtypes "github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	authclient "github.com/cosmos/cosmos-sdk/x/auth/client"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

type IntegrationTestSuite struct {
	suite.Suite

	cfg     network.Config
	network *network.Network
	conn    *grpc.ClientConn
}

func (s *IntegrationTestSuite) SetupSuite() {
	var err error
	s.T().Log("setting up integration test suite")

	s.cfg, err = network.DefaultConfigWithAppConfig(network.MinimumAppConfig())
	s.NoError(err)
	s.cfg.NumValidators = 1

	s.network, err = network.New(s.T(), s.T().TempDir(), s.cfg)
	s.Require().NoError(err)

	_, err = s.network.WaitForHeight(2)
	s.Require().NoError(err)

	val0 := s.network.Validators[0]
	s.conn, err = grpc.Dial(
		val0.AppConfig.GRPC.Address,
		grpc.WithInsecure(), // Or else we get "no transport security set"
		grpc.WithDefaultCallOptions(grpc.ForceCodec(codec.NewProtoCodec(s.cfg.InterfaceRegistry).GRPCCodec())),
	)
	s.Require().NoError(err)
}

func (s *IntegrationTestSuite) TearDownSuite() {
	s.T().Log("tearing down integration test suite")
	s.conn.Close()
	s.network.Cleanup()
}

func (s *IntegrationTestSuite) TestGRPCServer_TestService() {
	// gRPC query to test service should work
	testClient := testdata.NewQueryClient(s.conn)
	testRes, err := testClient.Echo(context.Background(), &testdata.EchoRequest{Message: "hello"})
	s.Require().NoError(err)
	s.Require().Equal("hello", testRes.Message)
}

func (s *IntegrationTestSuite) TestGRPCServer_BankBalance() {
	val0 := s.network.Validators[0]

	// gRPC query to bank service should work
	denom := fmt.Sprintf("%stoken", val0.Moniker)
	bankClient := banktypes.NewQueryClient(s.conn)
	var header metadata.MD
	bankRes, err := bankClient.Balance(
		context.Background(),
		&banktypes.QueryBalanceRequest{Address: val0.Address.String(), Denom: denom},
		grpc.Header(&header), // Also fetch grpc header
	)
	s.Require().NoError(err)
	s.Require().Equal(
		sdk.NewCoin(denom, s.network.Config.AccountTokens),
		*bankRes.GetBalance(),
	)
	blockHeight := header.Get(grpctypes.GRPCBlockHeightHeader)
	s.Require().NotEmpty(blockHeight[0]) // Should contain the block height

	// Request metadata should work
	bankRes, err = bankClient.Balance(
		metadata.AppendToOutgoingContext(context.Background(), grpctypes.GRPCBlockHeightHeader, "1"), // Add metadata to request
		&banktypes.QueryBalanceRequest{Address: val0.Address.String(), Denom: denom},
		grpc.Header(&header),
	)
	s.Require().NoError(err)
	blockHeight = header.Get(grpctypes.GRPCBlockHeightHeader)
	s.Require().Equal([]string{"1"}, blockHeight)
}

func (s *IntegrationTestSuite) TestGRPCServer_Reflection() {
	// Test server reflection
	ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
	defer cancel()
	stub := rpb.NewServerReflectionClient(s.conn)
	// NOTE(fdymylja): we use grpcreflect because it solves imports too
	// so that we can always assert that given a reflection server it is
	// possible to fully query all the methods, without having any context
	// on the proto registry
	rc := grpcreflect.NewClient(ctx, stub)

	services, err := rc.ListServices()
	s.Require().NoError(err)
	s.Require().Greater(len(services), 0)

	for _, svc := range services {
		file, err := rc.FileContainingSymbol(svc)
		s.Require().NoError(err)
		sd := file.FindSymbol(svc)
		s.Require().NotNil(sd)
	}
}

func (s *IntegrationTestSuite) TestGRPCServer_InterfaceReflection() {
	// this tests the application reflection capabilities and compatibility between v1 and v2
	ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
	defer cancel()

	clientV2 := reflectionv2.NewReflectionServiceClient(s.conn)
	clientV1 := reflectionv1.NewReflectionServiceClient(s.conn)
	codecDesc, err := clientV2.GetCodecDescriptor(ctx, nil)
	s.Require().NoError(err)

	interfaces, err := clientV1.ListAllInterfaces(ctx, nil)
	s.Require().NoError(err)
	s.Require().Equal(len(codecDesc.Codec.Interfaces), len(interfaces.InterfaceNames))
	s.Require().Equal(len(s.cfg.InterfaceRegistry.ListAllInterfaces()), len(codecDesc.Codec.Interfaces))

	for _, iface := range interfaces.InterfaceNames {
		impls, err := clientV1.ListImplementations(ctx, &reflectionv1.ListImplementationsRequest{InterfaceName: iface})
		s.Require().NoError(err)

		s.Require().ElementsMatch(impls.ImplementationMessageNames, s.cfg.InterfaceRegistry.ListImplementations(iface))
	}
}

func (s *IntegrationTestSuite) TestGRPCServer_GetTxsEvent() {
	// Query the tx via gRPC without pagination. This used to panic, see
	// https://github.com/cosmos/cosmos-sdk/issues/8038.
	txServiceClient := txtypes.NewServiceClient(s.conn)
	_, err := txServiceClient.GetTxsEvent(
		context.Background(),
		&txtypes.GetTxsEventRequest{
			Events: []string{"message.action='send'"},
		},
	)
	s.Require().NoError(err)
}

func (s *IntegrationTestSuite) TestGRPCServer_BroadcastTx() {
	val0 := s.network.Validators[0]

	txBuilder := s.mkTxBuilder()

	txBytes, err := val0.ClientCtx.TxConfig.TxEncoder()(txBuilder.GetTx())
	s.Require().NoError(err)

	// Broadcast the tx via gRPC.
	queryClient := txtypes.NewServiceClient(s.conn)

	grpcRes, err := queryClient.BroadcastTx(
		context.Background(),
		&txtypes.BroadcastTxRequest{
			Mode:    txtypes.BroadcastMode_BROADCAST_MODE_SYNC,
			TxBytes: txBytes,
		},
	)
	s.Require().NoError(err)
	s.Require().Equal(uint32(0), grpcRes.TxResponse.Code)
}

// Test and enforce that we upfront reject any connections to baseapp containing
// invalid initial x-cosmos-block-height that aren't positive  and in the range [0, max(int64)]
// See issue https://github.com/cosmos/cosmos-sdk/issues/7662.
func (s *IntegrationTestSuite) TestGRPCServerInvalidHeaderHeights() {
	t := s.T()

	// We should reject connections with invalid block heights off the bat.
	invalidHeightStrs := []struct {
		value   string
		wantErr string
	}{
		{"-1", "height < 0"},
		{"9223372036854775808", "value out of range"}, // > max(int64) by 1
		{"-10", "height < 0"},
		{"18446744073709551615", "value out of range"}, // max uint64, which is  > max(int64)
		{"-9223372036854775809", "value out of range"}, // Out of the range of for negative int64
	}
	for _, tt := range invalidHeightStrs {
		t.Run(tt.value, func(t *testing.T) {
			testClient := testdata.NewQueryClient(s.conn)
			ctx := metadata.AppendToOutgoingContext(context.Background(), grpctypes.GRPCBlockHeightHeader, tt.value)
			testRes, err := testClient.Echo(ctx, &testdata.EchoRequest{Message: "hello"})
			require.Error(t, err)
			require.Nil(t, testRes)
			require.Contains(t, err.Error(), tt.wantErr)
		})
	}
}

// TestGRPCUnpacker - tests the grpc endpoint for Validator and using the interface registry unpack and extract the
// ConsAddr. (ref: https://github.com/cosmos/cosmos-sdk/issues/8045)
func (s *IntegrationTestSuite) TestGRPCUnpacker() {
	ir := s.cfg.InterfaceRegistry
	queryClient := stakingtypes.NewQueryClient(s.conn)
	validator, err := queryClient.Validator(context.Background(),
		&stakingtypes.QueryValidatorRequest{ValidatorAddr: s.network.Validators[0].ValAddress.String()})
	require.NoError(s.T(), err)

	// no unpacked interfaces yet, so ConsAddr will be nil
	nilAddr, err := validator.Validator.GetConsAddr()
	require.Error(s.T(), err)
	require.Nil(s.T(), nilAddr)

	// unpack the interfaces and now ConsAddr is not nil
	err = validator.Validator.UnpackInterfaces(ir)
	require.NoError(s.T(), err)
	addr, err := validator.Validator.GetConsAddr()
	require.NotNil(s.T(), addr)
	require.NoError(s.T(), err)
}

// mkTxBuilder creates a TxBuilder containing a signed tx from validator 0.
func (s IntegrationTestSuite) mkTxBuilder() client.TxBuilder {
	val := s.network.Validators[0]
	s.Require().NoError(s.network.WaitForNextBlock())

	// prepare txBuilder with msg
	txBuilder := val.ClientCtx.TxConfig.NewTxBuilder()
	feeAmount := sdk.Coins{sdk.NewInt64Coin(s.cfg.BondDenom, 10)}
	gasLimit := testdata.NewTestGasLimit()
	s.Require().NoError(
		txBuilder.SetMsgs(&banktypes.MsgSend{
			FromAddress: val.Address.String(),
			ToAddress:   val.Address.String(),
			Amount:      sdk.Coins{sdk.NewInt64Coin(s.cfg.BondDenom, 10)},
		}),
	)
	txBuilder.SetFeeAmount(feeAmount)
	txBuilder.SetGasLimit(gasLimit)
	txBuilder.SetMemo("foobar")

	// setup txFactory
	txFactory := clienttx.Factory{}.
		WithChainID(val.ClientCtx.ChainID).
		WithKeybase(val.ClientCtx.Keyring).
		WithTxConfig(val.ClientCtx.TxConfig).
		WithSignMode(signing.SignMode_SIGN_MODE_DIRECT)

	// Sign Tx.
	err := authclient.SignTx(txFactory, val.ClientCtx, val.Moniker, txBuilder, false, true)
	s.Require().NoError(err)

	return txBuilder
}

func TestIntegrationTestSuite(t *testing.T) {
	suite.Run(t, new(IntegrationTestSuite))
}
