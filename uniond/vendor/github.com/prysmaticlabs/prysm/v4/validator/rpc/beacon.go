package rpc

import (
	"net/http"

	middleware "github.com/grpc-ecosystem/go-grpc-middleware"
	grpcretry "github.com/grpc-ecosystem/go-grpc-middleware/retry"
	grpcopentracing "github.com/grpc-ecosystem/go-grpc-middleware/tracing/opentracing"
	grpcprometheus "github.com/grpc-ecosystem/go-grpc-prometheus"
	"github.com/pkg/errors"
	grpcutil "github.com/prysmaticlabs/prysm/v4/api/grpc"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/validator/client"
	beaconApi "github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api"
	beaconChainClientFactory "github.com/prysmaticlabs/prysm/v4/validator/client/beacon-chain-client-factory"
	nodeClientFactory "github.com/prysmaticlabs/prysm/v4/validator/client/node-client-factory"
	validatorClientFactory "github.com/prysmaticlabs/prysm/v4/validator/client/validator-client-factory"
	validatorHelpers "github.com/prysmaticlabs/prysm/v4/validator/helpers"
	"google.golang.org/grpc"
)

// Initialize a client connect to a beacon node gRPC endpoint.
func (s *Server) registerBeaconClient() error {
	streamInterceptor := grpc.WithStreamInterceptor(middleware.ChainStreamClient(
		grpcopentracing.StreamClientInterceptor(),
		grpcprometheus.StreamClientInterceptor,
		grpcretry.StreamClientInterceptor(),
	))
	dialOpts := client.ConstructDialOptions(
		s.clientMaxCallRecvMsgSize,
		s.clientWithCert,
		s.clientGrpcRetries,
		s.clientGrpcRetryDelay,
		streamInterceptor,
	)
	if dialOpts == nil {
		return errors.New("no dial options for beacon chain gRPC client")
	}

	s.ctx = grpcutil.AppendHeaders(s.ctx, s.clientGrpcHeaders)

	grpcConn, err := grpc.DialContext(s.ctx, s.beaconClientEndpoint, dialOpts...)
	if err != nil {
		return errors.Wrapf(err, "could not dial endpoint: %s", s.beaconClientEndpoint)
	}
	if s.clientWithCert != "" {
		log.Info("Established secure gRPC connection")
	}
	s.beaconNodeHealthClient = ethpb.NewHealthClient(grpcConn)

	conn := validatorHelpers.NewNodeConnection(
		grpcConn,
		s.beaconApiEndpoint,
		s.beaconApiTimeout,
	)

	restHandler := &beaconApi.BeaconApiJsonRestHandler{
		HttpClient: http.Client{Timeout: s.beaconApiTimeout},
		Host:       s.beaconApiEndpoint,
	}
	s.beaconChainClient = beaconChainClientFactory.NewBeaconChainClient(conn, restHandler)
	s.beaconNodeClient = nodeClientFactory.NewNodeClient(conn, restHandler)
	s.beaconNodeValidatorClient = validatorClientFactory.NewValidatorClient(conn, restHandler)

	return nil
}
