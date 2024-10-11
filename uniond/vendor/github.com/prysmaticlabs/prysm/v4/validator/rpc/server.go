package rpc

import (
	"context"
	"fmt"
	"net"
	"net/http"
	"path/filepath"
	"time"

	"github.com/gorilla/mux"
	middleware "github.com/grpc-ecosystem/go-grpc-middleware"
	recovery "github.com/grpc-ecosystem/go-grpc-middleware/recovery"
	grpcopentracing "github.com/grpc-ecosystem/go-grpc-middleware/tracing/opentracing"
	grpcprometheus "github.com/grpc-ecosystem/go-grpc-prometheus"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/api"
	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/io/logs"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/client"
	iface "github.com/prysmaticlabs/prysm/v4/validator/client/iface"
	"github.com/prysmaticlabs/prysm/v4/validator/db"
	"github.com/sirupsen/logrus"
	"go.opencensus.io/plugin/ocgrpc"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/reflection"
)

// Config options for the gRPC server.
type Config struct {
	ValidatorGatewayHost     string
	ValidatorGatewayPort     int
	ValidatorMonitoringHost  string
	ValidatorMonitoringPort  int
	BeaconClientEndpoint     string
	ClientMaxCallRecvMsgSize int
	ClientGrpcRetries        uint
	ClientGrpcRetryDelay     time.Duration
	ClientGrpcHeaders        []string
	ClientWithCert           string
	Host                     string
	Port                     string
	CertFlag                 string
	KeyFlag                  string
	ValDB                    db.Database
	WalletDir                string
	ValidatorService         *client.ValidatorService
	SyncChecker              client.SyncChecker
	GenesisFetcher           client.GenesisFetcher
	WalletInitializedFeed    *event.Feed
	NodeGatewayEndpoint      string
	Router                   *mux.Router
	Wallet                   *wallet.Wallet
}

// Server defining a gRPC server for the remote signer API.
type Server struct {
	logsStreamer              logs.Streamer
	streamLogsBufferSize      int
	beaconChainClient         iface.BeaconChainClient
	beaconNodeClient          iface.NodeClient
	beaconNodeValidatorClient iface.ValidatorClient
	beaconNodeHealthClient    ethpb.HealthClient
	valDB                     db.Database
	ctx                       context.Context
	cancel                    context.CancelFunc
	beaconClientEndpoint      string
	clientMaxCallRecvMsgSize  int
	clientGrpcRetries         uint
	clientGrpcRetryDelay      time.Duration
	clientGrpcHeaders         []string
	clientWithCert            string
	host                      string
	port                      string
	listener                  net.Listener
	withCert                  string
	withKey                   string
	credentialError           error
	grpcServer                *grpc.Server
	jwtSecret                 []byte
	validatorService          *client.ValidatorService
	syncChecker               client.SyncChecker
	genesisFetcher            client.GenesisFetcher
	walletDir                 string
	wallet                    *wallet.Wallet
	walletInitializedFeed     *event.Feed
	walletInitialized         bool
	nodeGatewayEndpoint       string
	validatorMonitoringHost   string
	validatorMonitoringPort   int
	validatorGatewayHost      string
	validatorGatewayPort      int
	beaconApiEndpoint         string
	beaconApiTimeout          time.Duration
	router                    *mux.Router
}

// NewServer instantiates a new gRPC server.
func NewServer(ctx context.Context, cfg *Config) *Server {
	ctx, cancel := context.WithCancel(ctx)
	server := &Server{
		ctx:                      ctx,
		cancel:                   cancel,
		logsStreamer:             logs.NewStreamServer(),
		streamLogsBufferSize:     1000, // Enough to handle most bursts of logs in the validator client.
		host:                     cfg.Host,
		port:                     cfg.Port,
		withCert:                 cfg.CertFlag,
		withKey:                  cfg.KeyFlag,
		beaconClientEndpoint:     cfg.BeaconClientEndpoint,
		clientMaxCallRecvMsgSize: cfg.ClientMaxCallRecvMsgSize,
		clientGrpcRetries:        cfg.ClientGrpcRetries,
		clientGrpcRetryDelay:     cfg.ClientGrpcRetryDelay,
		clientGrpcHeaders:        cfg.ClientGrpcHeaders,
		clientWithCert:           cfg.ClientWithCert,
		valDB:                    cfg.ValDB,
		validatorService:         cfg.ValidatorService,
		syncChecker:              cfg.SyncChecker,
		genesisFetcher:           cfg.GenesisFetcher,
		walletDir:                cfg.WalletDir,
		walletInitializedFeed:    cfg.WalletInitializedFeed,
		walletInitialized:        cfg.Wallet != nil,
		wallet:                   cfg.Wallet,
		nodeGatewayEndpoint:      cfg.NodeGatewayEndpoint,
		validatorMonitoringHost:  cfg.ValidatorMonitoringHost,
		validatorMonitoringPort:  cfg.ValidatorMonitoringPort,
		validatorGatewayHost:     cfg.ValidatorGatewayHost,
		validatorGatewayPort:     cfg.ValidatorGatewayPort,
		router:                   cfg.Router,
	}
	// immediately register routes to override any catchalls
	if err := server.InitializeRoutes(); err != nil {
		log.WithError(err).Fatal("Could not initialize routes")
	}
	return server
}

// Start the gRPC server.
func (s *Server) Start() {
	// Setup the gRPC server options and TLS configuration.
	address := fmt.Sprintf("%s:%s", s.host, s.port)
	lis, err := net.Listen("tcp", address)
	if err != nil {
		log.WithError(err).Errorf("Could not listen to port in Start() %s", address)
	}
	s.listener = lis

	// Register interceptors for metrics gathering as well as our
	// own, custom JWT unary interceptor.
	opts := []grpc.ServerOption{
		grpc.StatsHandler(&ocgrpc.ServerHandler{}),
		grpc.UnaryInterceptor(middleware.ChainUnaryServer(
			recovery.UnaryServerInterceptor(
				recovery.WithRecoveryHandlerContext(tracing.RecoveryHandlerFunc),
			),
			grpcprometheus.UnaryServerInterceptor,
			grpcopentracing.UnaryServerInterceptor(),
			s.JWTInterceptor(),
		)),
	}
	grpcprometheus.EnableHandlingTimeHistogram()
	if s.withCert != "" && s.withKey != "" {
		creds, err := credentials.NewServerTLSFromFile(s.withCert, s.withKey)
		if err != nil {
			log.WithError(err).Fatal("Could not load TLS keys")
		}
		opts = append(opts, grpc.Creds(creds))
		log.WithFields(logrus.Fields{
			"crt-path": s.withCert,
			"key-path": s.withKey,
		}).Info("Loaded TLS certificates")
	}
	s.grpcServer = grpc.NewServer(opts...)

	// Register a gRPC client to the beacon node.
	if err := s.registerBeaconClient(); err != nil {
		log.WithError(err).Fatal("Could not register beacon chain gRPC client")
	}

	// Register services available for the gRPC server.
	reflection.Register(s.grpcServer)

	// routes needs to be set before the server calls the server function
	go func() {
		if s.listener != nil {
			if err := s.grpcServer.Serve(s.listener); err != nil {
				log.WithError(err).Error("Could not serve")
			}
		}
	}()

	log.WithField("address", address).Info("gRPC server listening on address")
	if s.walletDir != "" {
		token, err := s.initializeAuthToken(s.walletDir)
		if err != nil {
			log.WithError(err).Error("Could not initialize web auth token")
			return
		}
		validatorWebAddr := fmt.Sprintf("%s:%d", s.validatorGatewayHost, s.validatorGatewayPort)
		authTokenPath := filepath.Join(s.walletDir, AuthTokenFileName)
		logValidatorWebAuth(validatorWebAddr, token, authTokenPath)
		go s.refreshAuthTokenFromFileChanges(s.ctx, authTokenPath)
	}
}

// InitializeRoutes initializes pure HTTP REST endpoints for the validator client.
// needs to be called before the Serve function
func (s *Server) InitializeRoutes() error {
	if s.router == nil {
		return errors.New("no router found on server")
	}
	// Adding Auth Interceptor for the routes below
	s.router.Use(s.JwtHttpInterceptor)
	// Register all services, HandleFunc calls, etc.
	// ...
	s.router.HandleFunc("/eth/v1/keystores", s.ListKeystores).Methods(http.MethodGet)
	s.router.HandleFunc("/eth/v1/keystores", s.ImportKeystores).Methods(http.MethodPost)
	s.router.HandleFunc("/eth/v1/keystores", s.DeleteKeystores).Methods(http.MethodDelete)
	s.router.HandleFunc("/eth/v1/remotekeys", s.ListRemoteKeys).Methods(http.MethodGet)
	s.router.HandleFunc("/eth/v1/remotekeys", s.ImportRemoteKeys).Methods(http.MethodPost)
	s.router.HandleFunc("/eth/v1/remotekeys", s.DeleteRemoteKeys).Methods(http.MethodDelete)
	s.router.HandleFunc("/eth/v1/validator/{pubkey}/gas_limit", s.GetGasLimit).Methods(http.MethodGet)
	s.router.HandleFunc("/eth/v1/validator/{pubkey}/gas_limit", s.SetGasLimit).Methods(http.MethodPost)
	s.router.HandleFunc("/eth/v1/validator/{pubkey}/gas_limit", s.DeleteGasLimit).Methods(http.MethodDelete)
	s.router.HandleFunc("/eth/v1/validator/{pubkey}/feerecipient", s.ListFeeRecipientByPubkey).Methods(http.MethodGet)
	s.router.HandleFunc("/eth/v1/validator/{pubkey}/feerecipient", s.SetFeeRecipientByPubkey).Methods(http.MethodPost)
	s.router.HandleFunc("/eth/v1/validator/{pubkey}/feerecipient", s.DeleteFeeRecipientByPubkey).Methods(http.MethodDelete)
	s.router.HandleFunc("/eth/v1/validator/{pubkey}/voluntary_exit", s.SetVoluntaryExit).Methods(http.MethodPost)
	// auth endpoint
	s.router.HandleFunc(api.WebUrlPrefix+"initialize", s.Initialize).Methods(http.MethodGet)
	// accounts endpoints
	s.router.HandleFunc(api.WebUrlPrefix+"accounts", s.ListAccounts).Methods(http.MethodGet)
	s.router.HandleFunc(api.WebUrlPrefix+"accounts/backup", s.BackupAccounts).Methods(http.MethodPost)
	s.router.HandleFunc(api.WebUrlPrefix+"accounts/voluntary-exit", s.VoluntaryExit).Methods(http.MethodPost)
	// web health endpoints
	s.router.HandleFunc(api.WebUrlPrefix+"health/version", s.GetVersion).Methods(http.MethodGet)
	s.router.HandleFunc(api.WebUrlPrefix+"health/logs/validator/stream", s.StreamValidatorLogs).Methods(http.MethodGet)
	s.router.HandleFunc(api.WebUrlPrefix+"health/logs/beacon/stream", s.StreamBeaconLogs).Methods(http.MethodGet)
	// Beacon calls
	s.router.HandleFunc(api.WebUrlPrefix+"beacon/status", s.GetBeaconStatus).Methods(http.MethodGet)
	s.router.HandleFunc(api.WebUrlPrefix+"beacon/summary", s.GetValidatorPerformance).Methods(http.MethodGet)
	s.router.HandleFunc(api.WebUrlPrefix+"beacon/validators", s.GetValidators).Methods(http.MethodGet)
	s.router.HandleFunc(api.WebUrlPrefix+"beacon/balances", s.GetValidatorBalances).Methods(http.MethodGet)
	s.router.HandleFunc(api.WebUrlPrefix+"beacon/peers", s.GetPeers).Methods(http.MethodGet)
	// web wallet endpoints
	s.router.HandleFunc(api.WebUrlPrefix+"wallet", s.WalletConfig).Methods(http.MethodGet)
	s.router.HandleFunc(api.WebUrlPrefix+"wallet/create", s.CreateWallet).Methods(http.MethodPost)
	s.router.HandleFunc(api.WebUrlPrefix+"wallet/keystores/validate", s.ValidateKeystores).Methods(http.MethodPost)
	s.router.HandleFunc(api.WebUrlPrefix+"wallet/recover", s.RecoverWallet).Methods(http.MethodPost)
	// slashing protection endpoints
	s.router.HandleFunc(api.WebUrlPrefix+"slashing-protection/export", s.ExportSlashingProtection).Methods(http.MethodGet)
	s.router.HandleFunc(api.WebUrlPrefix+"slashing-protection/import", s.ImportSlashingProtection).Methods(http.MethodPost)
	log.Info("Initialized REST API routes")
	return nil
}

// Stop the gRPC server.
func (s *Server) Stop() error {
	s.cancel()
	if s.listener != nil {
		s.grpcServer.GracefulStop()
		log.Debug("Initiated graceful stop of server")
	}
	return nil
}

// Status returns nil or credentialError.
func (s *Server) Status() error {
	return s.credentialError
}
