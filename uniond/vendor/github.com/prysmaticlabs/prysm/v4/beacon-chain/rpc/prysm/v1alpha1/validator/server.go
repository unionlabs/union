// Package validator defines a gRPC validator service implementation, providing
// critical endpoints for validator clients to submit blocks/attestations to the
// beacon node, receive assignments, and more.
package validator

import (
	"context"
	"time"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/builder"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache/depositcache"
	blockfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/block"
	opfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/attestations"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/blstoexec"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/slashings"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/synccommittee"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/voluntaryexits"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/emptypb"
)

// Server defines a server implementation of the gRPC Validator service,
// providing RPC endpoints for obtaining validator assignments per epoch, the slots
// and committees in which particular validators need to perform their responsibilities,
// and more.
type Server struct {
	Ctx                    context.Context
	PayloadIDCache         *cache.PayloadIDCache
	TrackedValidatorsCache *cache.TrackedValidatorsCache
	HeadFetcher            blockchain.HeadFetcher
	ForkFetcher            blockchain.ForkFetcher
	ForkchoiceFetcher      blockchain.ForkchoiceFetcher
	GenesisFetcher         blockchain.GenesisFetcher
	FinalizationFetcher    blockchain.FinalizationFetcher
	TimeFetcher            blockchain.TimeFetcher
	BlockFetcher           execution.POWBlockFetcher
	DepositFetcher         cache.DepositFetcher
	ChainStartFetcher      execution.ChainStartFetcher
	Eth1InfoFetcher        execution.ChainInfoFetcher
	OptimisticModeFetcher  blockchain.OptimisticModeFetcher
	SyncChecker            sync.Checker
	StateNotifier          statefeed.Notifier
	BlockNotifier          blockfeed.Notifier
	P2P                    p2p.Broadcaster
	AttPool                attestations.Pool
	SlashingsPool          slashings.PoolManager
	ExitPool               voluntaryexits.PoolManager
	SyncCommitteePool      synccommittee.Pool
	BlockReceiver          blockchain.BlockReceiver
	BlobReceiver           blockchain.BlobReceiver
	MockEth1Votes          bool
	Eth1BlockFetcher       execution.POWBlockFetcher
	PendingDepositsFetcher depositcache.PendingDepositsFetcher
	OperationNotifier      opfeed.Notifier
	StateGen               stategen.StateManager
	ReplayerBuilder        stategen.ReplayerBuilder
	BeaconDB               db.HeadAccessDatabase
	ExecutionEngineCaller  execution.EngineCaller
	BlockBuilder           builder.BlockBuilder
	BLSChangesPool         blstoexec.PoolManager
	ClockWaiter            startup.ClockWaiter
	CoreService            *core.Service
}

// WaitForActivation checks if a validator public key exists in the active validator registry of the current
// beacon state, if not, then it creates a stream which listens for canonical states which contain
// the validator with the public key as an active validator record.
func (vs *Server) WaitForActivation(req *ethpb.ValidatorActivationRequest, stream ethpb.BeaconNodeValidator_WaitForActivationServer) error {
	activeValidatorExists, validatorStatuses, err := vs.activationStatus(stream.Context(), req.PublicKeys)
	if err != nil {
		return status.Errorf(codes.Internal, "Could not fetch validator status: %v", err)
	}
	res := &ethpb.ValidatorActivationResponse{
		Statuses: validatorStatuses,
	}
	if activeValidatorExists {
		return stream.Send(res)
	}
	if err := stream.Send(res); err != nil {
		return status.Errorf(codes.Internal, "Could not send response over stream: %v", err)
	}

	for {
		select {
		// Pinging every slot for activation.
		case <-time.After(time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second):
			activeValidatorExists, validatorStatuses, err := vs.activationStatus(stream.Context(), req.PublicKeys)
			if err != nil {
				return status.Errorf(codes.Internal, "Could not fetch validator status: %v", err)
			}
			res := &ethpb.ValidatorActivationResponse{
				Statuses: validatorStatuses,
			}
			if activeValidatorExists {
				return stream.Send(res)
			}
			if err := stream.Send(res); err != nil {
				return status.Errorf(codes.Internal, "Could not send response over stream: %v", err)
			}
		case <-stream.Context().Done():
			return status.Error(codes.Canceled, "Stream context canceled")
		case <-vs.Ctx.Done():
			return status.Error(codes.Canceled, "RPC context canceled")
		}
	}
}

// ValidatorIndex is called by a validator to get its index location in the beacon state.
func (vs *Server) ValidatorIndex(ctx context.Context, req *ethpb.ValidatorIndexRequest) (*ethpb.ValidatorIndexResponse, error) {
	st, err := vs.HeadFetcher.HeadStateReadOnly(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not determine head state: %v", err)
	}
	if st == nil || st.IsNil() {
		return nil, status.Errorf(codes.Internal, "head state is empty")
	}
	index, ok := st.ValidatorIndexByPubkey(bytesutil.ToBytes48(req.PublicKey))
	if !ok {
		return nil, status.Errorf(codes.NotFound, "Could not find validator index for public key %#x", req.PublicKey)
	}

	return &ethpb.ValidatorIndexResponse{Index: index}, nil
}

// DomainData fetches the current domain version information from the beacon state.
func (vs *Server) DomainData(ctx context.Context, request *ethpb.DomainRequest) (*ethpb.DomainResponse, error) {
	fork, err := forks.Fork(request.Epoch)
	if err != nil {
		return nil, err
	}
	headGenesisValidatorsRoot := vs.HeadFetcher.HeadGenesisValidatorsRoot()
	isExitDomain := [4]byte(request.Domain) == params.BeaconConfig().DomainVoluntaryExit
	if isExitDomain {
		hs, err := vs.HeadFetcher.HeadStateReadOnly(ctx)
		if err != nil {
			return nil, err
		}
		if hs.Version() >= version.Deneb {
			fork = &ethpb.Fork{
				PreviousVersion: params.BeaconConfig().CapellaForkVersion,
				CurrentVersion:  params.BeaconConfig().CapellaForkVersion,
				Epoch:           params.BeaconConfig().CapellaForkEpoch,
			}
		}
	}
	dv, err := signing.Domain(fork, request.Epoch, bytesutil.ToBytes4(request.Domain), headGenesisValidatorsRoot[:])
	if err != nil {
		return nil, err
	}
	return &ethpb.DomainResponse{
		SignatureDomain: dv,
	}, nil
}

// WaitForChainStart queries the logs of the Deposit Contract in order to verify the beacon chain
// has started its runtime and validators begin their responsibilities. If it has not, it then
// subscribes to an event stream triggered by the powchain service whenever the ChainStart log does
// occur in the Deposit Contract on ETH 1.0.
func (vs *Server) WaitForChainStart(_ *emptypb.Empty, stream ethpb.BeaconNodeValidator_WaitForChainStartServer) error {
	head, err := vs.HeadFetcher.HeadStateReadOnly(stream.Context())
	if err != nil {
		return status.Errorf(codes.Internal, "Could not retrieve head state: %v", err)
	}
	if head != nil && !head.IsNil() {
		res := &ethpb.ChainStartResponse{
			Started:               true,
			GenesisTime:           head.GenesisTime(),
			GenesisValidatorsRoot: head.GenesisValidatorsRoot(),
		}
		return stream.Send(res)
	}

	clock, err := vs.ClockWaiter.WaitForClock(vs.Ctx)
	if err != nil {
		return status.Error(codes.Canceled, "Context canceled")
	}
	log.WithField("starttime", clock.GenesisTime()).Debug("Received chain started event")
	log.Debug("Sending genesis time notification to connected validator clients")
	gvr := clock.GenesisValidatorsRoot()
	res := &ethpb.ChainStartResponse{
		Started:               true,
		GenesisTime:           uint64(clock.GenesisTime().Unix()),
		GenesisValidatorsRoot: gvr[:],
	}
	return stream.Send(res)
}

// PruneBlobsBundleCacheRoutine prunes the blobs bundle cache at 6s mark of the slot.
func (vs *Server) PruneBlobsBundleCacheRoutine() {
	go func() {
		clock, err := vs.ClockWaiter.WaitForClock(vs.Ctx)
		if err != nil {
			log.WithError(err).Error("PruneBlobsBundleCacheRoutine failed to receive genesis data")
			return
		}

		pruneInterval := time.Second * time.Duration(params.BeaconConfig().SecondsPerSlot/2)
		ticker := slots.NewSlotTickerWithIntervals(clock.GenesisTime(), []time.Duration{pruneInterval})
		for {
			select {
			case <-vs.Ctx.Done():
				return
			case slotInterval := <-ticker.C():
				bundleCache.prune(slotInterval.Slot)
			}
		}
	}()
}
