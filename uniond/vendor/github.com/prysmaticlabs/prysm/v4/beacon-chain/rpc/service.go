// Package rpc defines a gRPC server implementing the Ethereum consensus API as needed
// by validator clients and consumers of chain data.
package rpc

import (
	"context"
	"fmt"
	"net"
	"net/http"
	"sync"

	"github.com/gorilla/mux"
	middleware "github.com/grpc-ecosystem/go-grpc-middleware"
	recovery "github.com/grpc-ecosystem/go-grpc-middleware/recovery"
	grpcopentracing "github.com/grpc-ecosystem/go-grpc-middleware/tracing/opentracing"
	grpcprometheus "github.com/grpc-ecosystem/go-grpc-prometheus"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filesystem"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/config"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/events"
	beaconprysm "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/beacon"
	nodeprysm "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/node"
	validatorprysm "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/validator"
	"github.com/sirupsen/logrus"
	"go.opencensus.io/plugin/ocgrpc"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/reflection"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/builder"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache/depositcache"
	blockfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/block"
	opfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/attestations"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/blstoexec"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/slashings"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/synccommittee"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/voluntaryexits"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/blob"
	rpcBuilder "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/builder"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/debug"
	lightclient "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/light-client"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/node"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/rewards"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/validator"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/lookup"
	beaconv1alpha1 "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/v1alpha1/beacon"
	debugv1alpha1 "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/v1alpha1/debug"
	nodev1alpha1 "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/v1alpha1/node"
	validatorv1alpha1 "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/v1alpha1/validator"
	slasherservice "github.com/prysmaticlabs/prysm/v4/beacon-chain/slasher"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	chainSync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/io/logs"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	ethpbv1alpha1 "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

const attestationBufferSize = 100

// Service defining an RPC server for a beacon node.
type Service struct {
	cfg                  *Config
	ctx                  context.Context
	cancel               context.CancelFunc
	listener             net.Listener
	grpcServer           *grpc.Server
	incomingAttestation  chan *ethpbv1alpha1.Attestation
	credentialError      error
	connectedRPCClients  map[net.Addr]bool
	clientConnectionLock sync.Mutex
}

// Config options for the beacon node RPC server.
type Config struct {
	ExecutionPayloadReconstructor execution.PayloadReconstructor
	Host                          string
	Port                          string
	CertFlag                      string
	KeyFlag                       string
	BeaconMonitoringHost          string
	BeaconMonitoringPort          int
	BeaconDB                      db.HeadAccessDatabase
	ChainInfoFetcher              blockchain.ChainInfoFetcher
	HeadFetcher                   blockchain.HeadFetcher
	CanonicalFetcher              blockchain.CanonicalFetcher
	ForkFetcher                   blockchain.ForkFetcher
	ForkchoiceFetcher             blockchain.ForkchoiceFetcher
	FinalizationFetcher           blockchain.FinalizationFetcher
	AttestationReceiver           blockchain.AttestationReceiver
	BlockReceiver                 blockchain.BlockReceiver
	BlobReceiver                  blockchain.BlobReceiver
	ExecutionChainService         execution.Chain
	ChainStartFetcher             execution.ChainStartFetcher
	ExecutionChainInfoFetcher     execution.ChainInfoFetcher
	GenesisTimeFetcher            blockchain.TimeFetcher
	GenesisFetcher                blockchain.GenesisFetcher
	EnableDebugRPCEndpoints       bool
	MockEth1Votes                 bool
	AttestationsPool              attestations.Pool
	ExitPool                      voluntaryexits.PoolManager
	SlashingsPool                 slashings.PoolManager
	SlashingChecker               slasherservice.SlashingChecker
	SyncCommitteeObjectPool       synccommittee.Pool
	BLSChangesPool                blstoexec.PoolManager
	SyncService                   chainSync.Checker
	Broadcaster                   p2p.Broadcaster
	PeersFetcher                  p2p.PeersProvider
	PeerManager                   p2p.PeerManager
	MetadataProvider              p2p.MetadataProvider
	DepositFetcher                cache.DepositFetcher
	PendingDepositFetcher         depositcache.PendingDepositsFetcher
	StateNotifier                 statefeed.Notifier
	BlockNotifier                 blockfeed.Notifier
	OperationNotifier             opfeed.Notifier
	StateGen                      *stategen.State
	MaxMsgSize                    int
	ExecutionEngineCaller         execution.EngineCaller
	OptimisticModeFetcher         blockchain.OptimisticModeFetcher
	BlockBuilder                  builder.BlockBuilder
	Router                        *mux.Router
	ClockWaiter                   startup.ClockWaiter
	BlobStorage                   *filesystem.BlobStorage
	TrackedValidatorsCache        *cache.TrackedValidatorsCache
	PayloadIDCache                *cache.PayloadIDCache
}

// NewService instantiates a new RPC service instance that will
// be registered into a running beacon node.
func NewService(ctx context.Context, cfg *Config) *Service {
	ctx, cancel := context.WithCancel(ctx)
	s := &Service{
		cfg:                 cfg,
		ctx:                 ctx,
		cancel:              cancel,
		incomingAttestation: make(chan *ethpbv1alpha1.Attestation, params.BeaconConfig().DefaultBufferSize),
		connectedRPCClients: make(map[net.Addr]bool),
	}

	address := fmt.Sprintf("%s:%s", s.cfg.Host, s.cfg.Port)
	lis, err := net.Listen("tcp", address)
	if err != nil {
		log.WithError(err).Errorf("Could not listen to port in Start() %s", address)
	}
	s.listener = lis
	log.WithField("address", address).Info("gRPC server listening on port")

	opts := []grpc.ServerOption{
		grpc.StatsHandler(&ocgrpc.ServerHandler{}),
		grpc.StreamInterceptor(middleware.ChainStreamServer(
			recovery.StreamServerInterceptor(
				recovery.WithRecoveryHandlerContext(tracing.RecoveryHandlerFunc),
			),
			grpcprometheus.StreamServerInterceptor,
			grpcopentracing.StreamServerInterceptor(),
			s.validatorStreamConnectionInterceptor,
		)),
		grpc.UnaryInterceptor(middleware.ChainUnaryServer(
			recovery.UnaryServerInterceptor(
				recovery.WithRecoveryHandlerContext(tracing.RecoveryHandlerFunc),
			),
			grpcprometheus.UnaryServerInterceptor,
			grpcopentracing.UnaryServerInterceptor(),
			s.validatorUnaryConnectionInterceptor,
		)),
		grpc.MaxRecvMsgSize(s.cfg.MaxMsgSize),
	}
	if s.cfg.CertFlag != "" && s.cfg.KeyFlag != "" {
		creds, err := credentials.NewServerTLSFromFile(s.cfg.CertFlag, s.cfg.KeyFlag)
		if err != nil {
			log.WithError(err).Fatal("Could not load TLS keys")
		}
		opts = append(opts, grpc.Creds(creds))
	} else {
		log.Warn("You are using an insecure gRPC server. If you are running your beacon node and " +
			"validator on the same machines, you can ignore this message. If you want to know " +
			"how to enable secure connections, see: https://docs.prylabs.network/docs/prysm-usage/secure-grpc")
	}
	s.grpcServer = grpc.NewServer(opts...)

	return s
}

// paranoid build time check to ensure ChainInfoFetcher implements required interfaces
var _ stategen.CanonicalChecker = blockchain.ChainInfoFetcher(nil)
var _ stategen.CurrentSlotter = blockchain.ChainInfoFetcher(nil)

func (s *Service) initializeRewardServerRoutes(rewardsServer *rewards.Server) {
	s.cfg.Router.HandleFunc("/eth/v1/beacon/rewards/blocks/{block_id}", rewardsServer.BlockRewards).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/rewards/attestations/{epoch}", rewardsServer.AttestationRewards).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/rewards/sync_committee/{block_id}", rewardsServer.SyncCommitteeRewards).Methods(http.MethodPost)
}

func (s *Service) initializeBuilderServerRoutes(builderServer *rpcBuilder.Server) {
	s.cfg.Router.HandleFunc("/eth/v1/builder/states/{state_id}/expected_withdrawals", builderServer.ExpectedWithdrawals).Methods(http.MethodGet)
}

func (s *Service) initializeBlobServerRoutes(blobServer *blob.Server) {
	s.cfg.Router.HandleFunc("/eth/v1/beacon/blob_sidecars/{block_id}", blobServer.Blobs).Methods(http.MethodGet)
}

func (s *Service) initializeValidatorServerRoutes(validatorServer *validator.Server) {
	s.cfg.Router.HandleFunc("/eth/v1/validator/aggregate_attestation", validatorServer.GetAggregateAttestation).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/validator/contribution_and_proofs", validatorServer.SubmitContributionAndProofs).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/validator/aggregate_and_proofs", validatorServer.SubmitAggregateAndProofs).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/validator/sync_committee_contribution", validatorServer.ProduceSyncCommitteeContribution).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/validator/sync_committee_subscriptions", validatorServer.SubmitSyncCommitteeSubscription).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/validator/beacon_committee_subscriptions", validatorServer.SubmitBeaconCommitteeSubscription).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/validator/attestation_data", validatorServer.GetAttestationData).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/validator/register_validator", validatorServer.RegisterValidator).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/validator/duties/attester/{epoch}", validatorServer.GetAttesterDuties).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/validator/duties/proposer/{epoch}", validatorServer.GetProposerDuties).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/validator/duties/sync/{epoch}", validatorServer.GetSyncCommitteeDuties).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/validator/prepare_beacon_proposer", validatorServer.PrepareBeaconProposer).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/validator/liveness/{epoch}", validatorServer.GetLiveness).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v2/validator/blocks/{slot}", validatorServer.ProduceBlockV2).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/validator/blinded_blocks/{slot}", validatorServer.ProduceBlindedBlock).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v3/validator/blocks/{slot}", validatorServer.ProduceBlockV3).Methods(http.MethodGet)
}

func (s *Service) initializeNodeServerRoutes(nodeServer *node.Server) {
	s.cfg.Router.HandleFunc("/eth/v1/node/syncing", nodeServer.GetSyncStatus).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/node/identity", nodeServer.GetIdentity).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/node/peers/{peer_id}", nodeServer.GetPeer).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/node/peers", nodeServer.GetPeers).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/node/peer_count", nodeServer.GetPeerCount).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/node/version", nodeServer.GetVersion).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/node/health", nodeServer.GetHealth).Methods(http.MethodGet)
}

func (s *Service) initializeBeaconServerRoutes(beaconServer *beacon.Server) {
	s.cfg.Router.HandleFunc("/eth/v1/beacon/states/{state_id}/committees", beaconServer.GetCommittees).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/states/{state_id}/fork", beaconServer.GetStateFork).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/states/{state_id}/root", beaconServer.GetStateRoot).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/states/{state_id}/sync_committees", beaconServer.GetSyncCommittees).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/states/{state_id}/randao", beaconServer.GetRandao).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/blocks", beaconServer.PublishBlock).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/blinded_blocks", beaconServer.PublishBlindedBlock).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v2/beacon/blocks", beaconServer.PublishBlockV2).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v2/beacon/blinded_blocks", beaconServer.PublishBlindedBlockV2).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/blocks/{block_id}", beaconServer.GetBlock).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v2/beacon/blocks/{block_id}", beaconServer.GetBlockV2).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/blocks/{block_id}/attestations", beaconServer.GetBlockAttestations).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/blinded_blocks/{block_id}", beaconServer.GetBlindedBlock).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/blocks/{block_id}/root", beaconServer.GetBlockRoot).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/pool/attestations", beaconServer.ListAttestations).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/pool/attestations", beaconServer.SubmitAttestations).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/pool/voluntary_exits", beaconServer.ListVoluntaryExits).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/pool/voluntary_exits", beaconServer.SubmitVoluntaryExit).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/pool/sync_committees", beaconServer.SubmitSyncCommitteeSignatures).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/pool/bls_to_execution_changes", beaconServer.ListBLSToExecutionChanges).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/pool/bls_to_execution_changes", beaconServer.SubmitBLSToExecutionChanges).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/pool/attester_slashings", beaconServer.GetAttesterSlashings).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/pool/attester_slashings", beaconServer.SubmitAttesterSlashing).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/pool/proposer_slashings", beaconServer.GetProposerSlashings).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/pool/proposer_slashings", beaconServer.SubmitProposerSlashing).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/headers", beaconServer.GetBlockHeaders).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/headers/{block_id}", beaconServer.GetBlockHeader).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/genesis", beaconServer.GetGenesis).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/states/{state_id}/finality_checkpoints", beaconServer.GetFinalityCheckpoints).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/states/{state_id}/validators", beaconServer.GetValidators).Methods(http.MethodGet, http.MethodPost)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/states/{state_id}/validators/{validator_id}", beaconServer.GetValidator).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/states/{state_id}/validator_balances", beaconServer.GetValidatorBalances).Methods(http.MethodGet, http.MethodPost)
}

func (s *Service) initializeConfigRoutes() {
	s.cfg.Router.HandleFunc("/eth/v1/config/deposit_contract", config.GetDepositContract).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/config/fork_schedule", config.GetForkSchedule).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/config/spec", config.GetSpec).Methods(http.MethodGet)
}

func (s *Service) initializeEventsServerRoutes(eventsServer *events.Server) {
	s.cfg.Router.HandleFunc("/eth/v1/events", eventsServer.StreamEvents).Methods(http.MethodGet)
}

func (s *Service) initializeLightClientServerRoutes(lightClientServer *lightclient.Server) {
	s.cfg.Router.HandleFunc("/eth/v1/beacon/light_client/bootstrap/{block_root}", lightClientServer.GetLightClientBootstrap).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/light_client/updates", lightClientServer.GetLightClientUpdatesByRange).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/light_client/finality_update", lightClientServer.GetLightClientFinalityUpdate).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/beacon/light_client/optimistic_update", lightClientServer.GetLightClientOptimisticUpdate).Methods(http.MethodGet)
}

func (s *Service) initializeDebugServerRoutes(debugServer *debug.Server) {
	s.cfg.Router.HandleFunc("/eth/v1/debug/beacon/states/{state_id}", debugServer.GetBeaconStateSSZ).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v2/debug/beacon/states/{state_id}", debugServer.GetBeaconStateV2).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v2/debug/beacon/heads", debugServer.GetForkChoiceHeadsV2).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/eth/v1/debug/fork_choice", debugServer.GetForkChoice).Methods(http.MethodGet)
}

// prysm internal routes
func (s *Service) initializePrysmBeaconServerRoutes(beaconServerPrysm *beaconprysm.Server) {
	s.cfg.Router.HandleFunc("/prysm/v1/beacon/weak_subjectivity", beaconServerPrysm.GetWeakSubjectivity).Methods(http.MethodGet)
}

func (s *Service) initializePrysmNodeServerRoutes(nodeServerPrysm *nodeprysm.Server) {
	s.cfg.Router.HandleFunc("/prysm/node/trusted_peers", nodeServerPrysm.ListTrustedPeer).Methods(http.MethodGet)
	s.cfg.Router.HandleFunc("/prysm/node/trusted_peers", nodeServerPrysm.AddTrustedPeer).Methods(http.MethodPost)
	s.cfg.Router.HandleFunc("/prysm/node/trusted_peers/{peer_id}", nodeServerPrysm.RemoveTrustedPeer).Methods(http.MethodDelete)
}

func (s *Service) initializePrysmValidatorServerRoutes(validatorServerPrysm *validatorprysm.Server) {
	s.cfg.Router.HandleFunc("/prysm/validators/performance", validatorServerPrysm.GetValidatorPerformance).Methods(http.MethodPost)
	// /eth/v1/beacon/states/{state_id}/validator_count is not a beacon API, it's a custom endpoint
	s.cfg.Router.HandleFunc("/eth/v1/beacon/states/{state_id}/validator_count", validatorServerPrysm.GetValidatorCount).Methods(http.MethodGet)
}

// Start the gRPC server.
func (s *Service) Start() {
	grpcprometheus.EnableHandlingTimeHistogram()

	var stateCache stategen.CachedGetter
	if s.cfg.StateGen != nil {
		stateCache = s.cfg.StateGen.CombinedCache()
	}
	withCache := stategen.WithCache(stateCache)
	ch := stategen.NewCanonicalHistory(s.cfg.BeaconDB, s.cfg.ChainInfoFetcher, s.cfg.ChainInfoFetcher, withCache)
	stater := &lookup.BeaconDbStater{
		BeaconDB:           s.cfg.BeaconDB,
		ChainInfoFetcher:   s.cfg.ChainInfoFetcher,
		GenesisTimeFetcher: s.cfg.GenesisTimeFetcher,
		StateGenService:    s.cfg.StateGen,
		ReplayerBuilder:    ch,
	}
	blocker := &lookup.BeaconDbBlocker{
		BeaconDB:         s.cfg.BeaconDB,
		ChainInfoFetcher: s.cfg.ChainInfoFetcher,
		BlobStorage:      s.cfg.BlobStorage,
	}
	rewardFetcher := &rewards.BlockRewardService{Replayer: ch}

	s.initializeRewardServerRoutes(&rewards.Server{
		Blocker:               blocker,
		OptimisticModeFetcher: s.cfg.OptimisticModeFetcher,
		FinalizationFetcher:   s.cfg.FinalizationFetcher,
		TimeFetcher:           s.cfg.GenesisTimeFetcher,
		Stater:                stater,
		HeadFetcher:           s.cfg.HeadFetcher,
		BlockRewardFetcher:    rewardFetcher,
	})
	s.initializeBuilderServerRoutes(&rpcBuilder.Server{
		FinalizationFetcher:   s.cfg.FinalizationFetcher,
		OptimisticModeFetcher: s.cfg.OptimisticModeFetcher,
		Stater:                stater,
	})
	s.initializeBlobServerRoutes(&blob.Server{
		Blocker: blocker,
	})

	coreService := &core.Service{
		HeadFetcher:           s.cfg.HeadFetcher,
		GenesisTimeFetcher:    s.cfg.GenesisTimeFetcher,
		SyncChecker:           s.cfg.SyncService,
		Broadcaster:           s.cfg.Broadcaster,
		SyncCommitteePool:     s.cfg.SyncCommitteeObjectPool,
		OperationNotifier:     s.cfg.OperationNotifier,
		AttestationCache:      cache.NewAttestationCache(),
		StateGen:              s.cfg.StateGen,
		P2P:                   s.cfg.Broadcaster,
		FinalizedFetcher:      s.cfg.FinalizationFetcher,
		OptimisticModeFetcher: s.cfg.OptimisticModeFetcher,
	}

	validatorServer := &validatorv1alpha1.Server{
		Ctx:                    s.ctx,
		AttPool:                s.cfg.AttestationsPool,
		ExitPool:               s.cfg.ExitPool,
		HeadFetcher:            s.cfg.HeadFetcher,
		ForkFetcher:            s.cfg.ForkFetcher,
		ForkchoiceFetcher:      s.cfg.ForkchoiceFetcher,
		GenesisFetcher:         s.cfg.GenesisFetcher,
		FinalizationFetcher:    s.cfg.FinalizationFetcher,
		TimeFetcher:            s.cfg.GenesisTimeFetcher,
		BlockFetcher:           s.cfg.ExecutionChainService,
		DepositFetcher:         s.cfg.DepositFetcher,
		ChainStartFetcher:      s.cfg.ChainStartFetcher,
		Eth1InfoFetcher:        s.cfg.ExecutionChainService,
		OptimisticModeFetcher:  s.cfg.OptimisticModeFetcher,
		SyncChecker:            s.cfg.SyncService,
		StateNotifier:          s.cfg.StateNotifier,
		BlockNotifier:          s.cfg.BlockNotifier,
		OperationNotifier:      s.cfg.OperationNotifier,
		P2P:                    s.cfg.Broadcaster,
		BlockReceiver:          s.cfg.BlockReceiver,
		BlobReceiver:           s.cfg.BlobReceiver,
		MockEth1Votes:          s.cfg.MockEth1Votes,
		Eth1BlockFetcher:       s.cfg.ExecutionChainService,
		PendingDepositsFetcher: s.cfg.PendingDepositFetcher,
		SlashingsPool:          s.cfg.SlashingsPool,
		StateGen:               s.cfg.StateGen,
		SyncCommitteePool:      s.cfg.SyncCommitteeObjectPool,
		ReplayerBuilder:        ch,
		ExecutionEngineCaller:  s.cfg.ExecutionEngineCaller,
		BeaconDB:               s.cfg.BeaconDB,
		BlockBuilder:           s.cfg.BlockBuilder,
		BLSChangesPool:         s.cfg.BLSChangesPool,
		ClockWaiter:            s.cfg.ClockWaiter,
		CoreService:            coreService,
		TrackedValidatorsCache: s.cfg.TrackedValidatorsCache,
		PayloadIDCache:         s.cfg.PayloadIDCache,
	}
	s.initializeValidatorServerRoutes(&validator.Server{
		HeadFetcher:            s.cfg.HeadFetcher,
		TimeFetcher:            s.cfg.GenesisTimeFetcher,
		SyncChecker:            s.cfg.SyncService,
		OptimisticModeFetcher:  s.cfg.OptimisticModeFetcher,
		AttestationsPool:       s.cfg.AttestationsPool,
		PeerManager:            s.cfg.PeerManager,
		Broadcaster:            s.cfg.Broadcaster,
		V1Alpha1Server:         validatorServer,
		Stater:                 stater,
		SyncCommitteePool:      s.cfg.SyncCommitteeObjectPool,
		ChainInfoFetcher:       s.cfg.ChainInfoFetcher,
		BeaconDB:               s.cfg.BeaconDB,
		BlockBuilder:           s.cfg.BlockBuilder,
		OperationNotifier:      s.cfg.OperationNotifier,
		TrackedValidatorsCache: s.cfg.TrackedValidatorsCache,
		PayloadIDCache:         s.cfg.PayloadIDCache,
		CoreService:            coreService,
		BlockRewardFetcher:     rewardFetcher,
	})

	nodeServer := &nodev1alpha1.Server{
		LogsStreamer:         logs.NewStreamServer(),
		StreamLogsBufferSize: 1000, // Enough to handle bursts of beacon node logs for gRPC streaming.
		BeaconDB:             s.cfg.BeaconDB,
		Server:               s.grpcServer,
		SyncChecker:          s.cfg.SyncService,
		GenesisTimeFetcher:   s.cfg.GenesisTimeFetcher,
		PeersFetcher:         s.cfg.PeersFetcher,
		PeerManager:          s.cfg.PeerManager,
		GenesisFetcher:       s.cfg.GenesisFetcher,
		POWChainInfoFetcher:  s.cfg.ExecutionChainInfoFetcher,
		BeaconMonitoringHost: s.cfg.BeaconMonitoringHost,
		BeaconMonitoringPort: s.cfg.BeaconMonitoringPort,
	}
	s.initializeNodeServerRoutes(&node.Server{
		BeaconDB:                  s.cfg.BeaconDB,
		Server:                    s.grpcServer,
		SyncChecker:               s.cfg.SyncService,
		OptimisticModeFetcher:     s.cfg.OptimisticModeFetcher,
		GenesisTimeFetcher:        s.cfg.GenesisTimeFetcher,
		PeersFetcher:              s.cfg.PeersFetcher,
		PeerManager:               s.cfg.PeerManager,
		MetadataProvider:          s.cfg.MetadataProvider,
		HeadFetcher:               s.cfg.HeadFetcher,
		ExecutionChainInfoFetcher: s.cfg.ExecutionChainInfoFetcher,
	})

	beaconChainServer := &beaconv1alpha1.Server{
		Ctx:                         s.ctx,
		BeaconDB:                    s.cfg.BeaconDB,
		AttestationsPool:            s.cfg.AttestationsPool,
		SlashingsPool:               s.cfg.SlashingsPool,
		OptimisticModeFetcher:       s.cfg.OptimisticModeFetcher,
		HeadFetcher:                 s.cfg.HeadFetcher,
		FinalizationFetcher:         s.cfg.FinalizationFetcher,
		CanonicalFetcher:            s.cfg.CanonicalFetcher,
		ChainStartFetcher:           s.cfg.ChainStartFetcher,
		DepositFetcher:              s.cfg.DepositFetcher,
		BlockFetcher:                s.cfg.ExecutionChainService,
		GenesisTimeFetcher:          s.cfg.GenesisTimeFetcher,
		StateNotifier:               s.cfg.StateNotifier,
		BlockNotifier:               s.cfg.BlockNotifier,
		AttestationNotifier:         s.cfg.OperationNotifier,
		Broadcaster:                 s.cfg.Broadcaster,
		StateGen:                    s.cfg.StateGen,
		SyncChecker:                 s.cfg.SyncService,
		ReceivedAttestationsBuffer:  make(chan *ethpbv1alpha1.Attestation, attestationBufferSize),
		CollectedAttestationsBuffer: make(chan []*ethpbv1alpha1.Attestation, attestationBufferSize),
		ReplayerBuilder:             ch,
		CoreService:                 coreService,
	}
	s.initializeBeaconServerRoutes(&beacon.Server{
		CanonicalHistory:              ch,
		BeaconDB:                      s.cfg.BeaconDB,
		AttestationsPool:              s.cfg.AttestationsPool,
		SlashingsPool:                 s.cfg.SlashingsPool,
		ChainInfoFetcher:              s.cfg.ChainInfoFetcher,
		GenesisTimeFetcher:            s.cfg.GenesisTimeFetcher,
		BlockNotifier:                 s.cfg.BlockNotifier,
		OperationNotifier:             s.cfg.OperationNotifier,
		Broadcaster:                   s.cfg.Broadcaster,
		BlockReceiver:                 s.cfg.BlockReceiver,
		StateGenService:               s.cfg.StateGen,
		Stater:                        stater,
		Blocker:                       blocker,
		OptimisticModeFetcher:         s.cfg.OptimisticModeFetcher,
		HeadFetcher:                   s.cfg.HeadFetcher,
		TimeFetcher:                   s.cfg.GenesisTimeFetcher,
		VoluntaryExitsPool:            s.cfg.ExitPool,
		V1Alpha1ValidatorServer:       validatorServer,
		SyncChecker:                   s.cfg.SyncService,
		ExecutionPayloadReconstructor: s.cfg.ExecutionPayloadReconstructor,
		BLSChangesPool:                s.cfg.BLSChangesPool,
		FinalizationFetcher:           s.cfg.FinalizationFetcher,
		ForkchoiceFetcher:             s.cfg.ForkchoiceFetcher,
		CoreService:                   coreService,
	})

	s.initializeConfigRoutes()

	s.initializeEventsServerRoutes(&events.Server{
		StateNotifier:     s.cfg.StateNotifier,
		OperationNotifier: s.cfg.OperationNotifier,
		HeadFetcher:       s.cfg.HeadFetcher,
		ChainInfoFetcher:  s.cfg.ChainInfoFetcher,
	})

	s.initializeLightClientServerRoutes(&lightclient.Server{
		Blocker:     blocker,
		Stater:      stater,
		HeadFetcher: s.cfg.HeadFetcher,
	})

	ethpbv1alpha1.RegisterNodeServer(s.grpcServer, nodeServer)
	ethpbv1alpha1.RegisterHealthServer(s.grpcServer, nodeServer)
	ethpbv1alpha1.RegisterBeaconChainServer(s.grpcServer, beaconChainServer)
	if s.cfg.EnableDebugRPCEndpoints {
		log.Info("Enabled debug gRPC endpoints")
		debugServer := &debugv1alpha1.Server{
			GenesisTimeFetcher: s.cfg.GenesisTimeFetcher,
			BeaconDB:           s.cfg.BeaconDB,
			StateGen:           s.cfg.StateGen,
			HeadFetcher:        s.cfg.HeadFetcher,
			PeerManager:        s.cfg.PeerManager,
			PeersFetcher:       s.cfg.PeersFetcher,
			ReplayerBuilder:    ch,
		}
		s.initializeDebugServerRoutes(&debug.Server{
			BeaconDB:              s.cfg.BeaconDB,
			HeadFetcher:           s.cfg.HeadFetcher,
			Stater:                stater,
			OptimisticModeFetcher: s.cfg.OptimisticModeFetcher,
			ForkFetcher:           s.cfg.ForkFetcher,
			ForkchoiceFetcher:     s.cfg.ForkchoiceFetcher,
			FinalizationFetcher:   s.cfg.FinalizationFetcher,
			ChainInfoFetcher:      s.cfg.ChainInfoFetcher,
		})
		ethpbv1alpha1.RegisterDebugServer(s.grpcServer, debugServer)
	}
	ethpbv1alpha1.RegisterBeaconNodeValidatorServer(s.grpcServer, validatorServer)
	// Register reflection service on gRPC server.
	reflection.Register(s.grpcServer)

	validatorServer.PruneBlobsBundleCacheRoutine()

	s.initializePrysmBeaconServerRoutes(&beaconprysm.Server{
		SyncChecker:           s.cfg.SyncService,
		HeadFetcher:           s.cfg.HeadFetcher,
		TimeFetcher:           s.cfg.GenesisTimeFetcher,
		OptimisticModeFetcher: s.cfg.OptimisticModeFetcher,
		CanonicalHistory:      ch,
		BeaconDB:              s.cfg.BeaconDB,
	})

	s.initializePrysmNodeServerRoutes(&nodeprysm.Server{
		BeaconDB:                  s.cfg.BeaconDB,
		SyncChecker:               s.cfg.SyncService,
		OptimisticModeFetcher:     s.cfg.OptimisticModeFetcher,
		GenesisTimeFetcher:        s.cfg.GenesisTimeFetcher,
		PeersFetcher:              s.cfg.PeersFetcher,
		PeerManager:               s.cfg.PeerManager,
		MetadataProvider:          s.cfg.MetadataProvider,
		HeadFetcher:               s.cfg.HeadFetcher,
		ExecutionChainInfoFetcher: s.cfg.ExecutionChainInfoFetcher,
	})

	s.initializePrysmValidatorServerRoutes(&validatorprysm.Server{
		GenesisTimeFetcher:    s.cfg.GenesisTimeFetcher,
		HeadFetcher:           s.cfg.HeadFetcher,
		SyncChecker:           s.cfg.SyncService,
		CoreService:           coreService,
		OptimisticModeFetcher: s.cfg.OptimisticModeFetcher,
		Stater:                stater,
		ChainInfoFetcher:      s.cfg.ChainInfoFetcher,
		BeaconDB:              s.cfg.BeaconDB,
		FinalizationFetcher:   s.cfg.FinalizationFetcher,
	})

	go func() {
		if s.listener != nil {
			if err := s.grpcServer.Serve(s.listener); err != nil {
				log.WithError(err).Errorf("Could not serve gRPC")
			}
		}
	}()
}

// Stop the service.
func (s *Service) Stop() error {
	s.cancel()
	if s.listener != nil {
		s.grpcServer.GracefulStop()
		log.Debug("Initiated graceful stop of gRPC server")
	}
	return nil
}

// Status returns nil or credentialError
func (s *Service) Status() error {
	optimistic, err := s.cfg.OptimisticModeFetcher.IsOptimistic(s.ctx)
	if err != nil {
		return errors.Wrap(err, "failed to check if service is optimistic")
	}
	if optimistic {
		return errors.New("service is optimistic, validators can't perform duties " +
			"please check if execution layer is fully synced")
	}
	if s.cfg.SyncService.Syncing() {
		return errors.New("syncing")
	}
	if s.credentialError != nil {
		return s.credentialError
	}
	return nil
}

// Stream interceptor for new validator client connections to the beacon node.
func (s *Service) validatorStreamConnectionInterceptor(
	srv interface{},
	ss grpc.ServerStream,
	_ *grpc.StreamServerInfo,
	handler grpc.StreamHandler,
) error {
	s.logNewClientConnection(ss.Context())
	return handler(srv, ss)
}

// Unary interceptor for new validator client connections to the beacon node.
func (s *Service) validatorUnaryConnectionInterceptor(
	ctx context.Context,
	req interface{},
	_ *grpc.UnaryServerInfo,
	handler grpc.UnaryHandler,
) (interface{}, error) {
	s.logNewClientConnection(ctx)
	return handler(ctx, req)
}

func (s *Service) logNewClientConnection(ctx context.Context) {
	if features.Get().DisableGRPCConnectionLogs {
		return
	}
	if clientInfo, ok := peer.FromContext(ctx); ok {
		// Check if we have not yet observed this grpc client connection
		// in the running beacon node.
		s.clientConnectionLock.Lock()
		defer s.clientConnectionLock.Unlock()
		if !s.connectedRPCClients[clientInfo.Addr] {
			log.WithFields(logrus.Fields{
				"addr": clientInfo.Addr.String(),
			}).Infof("gRPC client connected to beacon node")
			s.connectedRPCClients[clientInfo.Addr] = true
		}
	}
}
