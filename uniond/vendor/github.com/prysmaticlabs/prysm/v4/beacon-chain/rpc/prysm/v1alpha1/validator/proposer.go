package validator

import (
	"context"
	"fmt"
	"strings"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	emptypb "github.com/golang/protobuf/ptypes/empty"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/builder"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	blockfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/block"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/kv"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// eth1DataNotification is a latch to stop flooding logs with the same warning.
var eth1DataNotification bool

const (
	eth1dataTimeout           = 2 * time.Second
	defaultBuilderBoostFactor = uint64(100)
)

// GetBeaconBlock is called by a proposer during its assigned slot to request a block to sign
// by passing in the slot and the signed randao reveal of the slot.
func (vs *Server) GetBeaconBlock(ctx context.Context, req *ethpb.BlockRequest) (*ethpb.GenericBeaconBlock, error) {
	ctx, span := trace.StartSpan(ctx, "ProposerServer.GetBeaconBlock")
	defer span.End()
	span.AddAttributes(trace.Int64Attribute("slot", int64(req.Slot)))

	t, err := slots.ToTime(uint64(vs.TimeFetcher.GenesisTime().Unix()), req.Slot)
	if err != nil {
		log.WithError(err).Error("Could not convert slot to time")
	}
	log.WithFields(logrus.Fields{
		"slot":               req.Slot,
		"sinceSlotStartTime": time.Since(t),
	}).Info("Begin building block")

	// A syncing validator should not produce a block.
	if vs.SyncChecker.Syncing() {
		return nil, status.Error(codes.Unavailable, "Syncing to latest head, not ready to respond")
	}

	// process attestations and update head in forkchoice
	vs.ForkchoiceFetcher.UpdateHead(ctx, vs.TimeFetcher.CurrentSlot())
	headRoot := vs.ForkchoiceFetcher.CachedHeadRoot()
	parentRoot := vs.ForkchoiceFetcher.GetProposerHead()
	if parentRoot != headRoot {
		blockchain.LateBlockAttemptedReorgCount.Inc()
		log.WithFields(logrus.Fields{
			"slot":       req.Slot,
			"parentRoot": fmt.Sprintf("%#x", parentRoot),
			"headRoot":   fmt.Sprintf("%#x", headRoot),
		}).Warn("late block attempted reorg failed")
	}

	// An optimistic validator MUST NOT produce a block (i.e., sign across the DOMAIN_BEACON_PROPOSER domain).
	if slots.ToEpoch(req.Slot) >= params.BeaconConfig().BellatrixForkEpoch {
		if err := vs.optimisticStatus(ctx); err != nil {
			return nil, status.Errorf(codes.Unavailable, "Validator is not ready to propose: %v", err)
		}
	}

	sBlk, err := getEmptyBlock(req.Slot)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not prepare block: %v", err)
	}
	head, err := vs.HeadFetcher.HeadState(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not get head state: %v", err)
	}
	head, err = transition.ProcessSlotsUsingNextSlotCache(ctx, head, parentRoot[:], req.Slot)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not process slots up to %d: %v", req.Slot, err)
	}

	// Set slot, graffiti, randao reveal, and parent root.
	sBlk.SetSlot(req.Slot)
	sBlk.SetGraffiti(req.Graffiti)
	sBlk.SetRandaoReveal(req.RandaoReveal)
	sBlk.SetParentRoot(parentRoot[:])

	// Set proposer index.
	idx, err := helpers.BeaconProposerIndex(ctx, head)
	if err != nil {
		return nil, fmt.Errorf("could not calculate proposer index %v", err)
	}
	sBlk.SetProposerIndex(idx)

	builderBoostFactor := defaultBuilderBoostFactor
	if req.BuilderBoostFactor != nil {
		builderBoostFactor = req.BuilderBoostFactor.Value
	}

	if err = vs.BuildBlockParallel(ctx, sBlk, head, req.SkipMevBoost, builderBoostFactor); err != nil {
		return nil, errors.Wrap(err, "could not build block in parallel")
	}

	sr, err := vs.computeStateRoot(ctx, sBlk)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not compute state root: %v", err)
	}
	sBlk.SetStateRoot(sr)

	log.WithFields(logrus.Fields{
		"slot":               req.Slot,
		"sinceSlotStartTime": time.Since(t),
		"validator":          sBlk.Block().ProposerIndex(),
	}).Info("Finished building block")

	// Blob cache is updated after BuildBlockParallel
	return vs.constructGenericBeaconBlock(sBlk, bundleCache.get(req.Slot))
}

func (vs *Server) BuildBlockParallel(ctx context.Context, sBlk interfaces.SignedBeaconBlock, head state.BeaconState, skipMevBoost bool, builderBoostFactor uint64) error {
	// Build consensus fields in background
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()

		// Set eth1 data.
		eth1Data, err := vs.eth1DataMajorityVote(ctx, head)
		if err != nil {
			eth1Data = &ethpb.Eth1Data{DepositRoot: params.BeaconConfig().ZeroHash[:], BlockHash: params.BeaconConfig().ZeroHash[:]}
			log.WithError(err).Error("Could not get eth1data")
		}
		sBlk.SetEth1Data(eth1Data)

		// Set deposit and attestation.
		deposits, atts, err := vs.packDepositsAndAttestations(ctx, head, eth1Data) // TODO: split attestations and deposits
		if err != nil {
			sBlk.SetDeposits([]*ethpb.Deposit{})
			sBlk.SetAttestations([]*ethpb.Attestation{})
			log.WithError(err).Error("Could not pack deposits and attestations")
		} else {
			sBlk.SetDeposits(deposits)
			sBlk.SetAttestations(atts)
		}

		// Set slashings.
		validProposerSlashings, validAttSlashings := vs.getSlashings(ctx, head)
		sBlk.SetProposerSlashings(validProposerSlashings)
		sBlk.SetAttesterSlashings(validAttSlashings)

		// Set exits.
		sBlk.SetVoluntaryExits(vs.getExits(head, sBlk.Block().Slot()))

		// Set sync aggregate. New in Altair.
		vs.setSyncAggregate(ctx, sBlk)

		// Set bls to execution change. New in Capella.
		vs.setBlsToExecData(sBlk, head)
	}()

	localPayload, overrideBuilder, err := vs.getLocalPayload(ctx, sBlk.Block(), head)
	if err != nil {
		return status.Errorf(codes.Internal, "Could not get local payload: %v", err)
	}

	// There's no reason to try to get a builder bid if local override is true.
	var builderPayload interfaces.ExecutionData
	var builderKzgCommitments [][]byte
	overrideBuilder = overrideBuilder || skipMevBoost // Skip using mev-boost if requested by the caller.
	if !overrideBuilder {
		builderPayload, builderKzgCommitments, err = vs.getBuilderPayloadAndBlobs(ctx, sBlk.Block().Slot(), sBlk.Block().ProposerIndex())
		if err != nil {
			builderGetPayloadMissCount.Inc()
			log.WithError(err).Error("Could not get builder payload")
		}
	}

	if err := setExecutionData(ctx, sBlk, localPayload, builderPayload, builderKzgCommitments, builderBoostFactor); err != nil {
		return status.Errorf(codes.Internal, "Could not set execution data: %v", err)
	}

	wg.Wait() // Wait until block is built via consensus and execution fields.

	return nil
}

// ProposeBeaconBlock handles the proposal of beacon blocks.
func (vs *Server) ProposeBeaconBlock(ctx context.Context, req *ethpb.GenericSignedBeaconBlock) (*ethpb.ProposeResponse, error) {
	ctx, span := trace.StartSpan(ctx, "ProposerServer.ProposeBeaconBlock")
	defer span.End()

	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	block, err := blocks.NewSignedBeaconBlock(req.Block)
	if err != nil {
		return nil, status.Errorf(codes.InvalidArgument, "%s: %v", "decode block failed", err)
	}

	var sidecars []*ethpb.BlobSidecar
	if block.IsBlinded() {
		block, sidecars, err = vs.handleBlindedBlock(ctx, block)
	} else {
		sidecars, err = vs.handleUnblindedBlock(block, req)
	}
	if err != nil {
		return nil, status.Errorf(codes.Internal, "%s: %v", "handle block failed", err)
	}

	root, err := block.Block().HashTreeRoot()
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not hash tree root: %v", err)
	}

	var wg sync.WaitGroup
	errChan := make(chan error, 1)

	wg.Add(1)
	go func() {
		defer wg.Done()
		if err := vs.broadcastReceiveBlock(ctx, block, root); err != nil {
			errChan <- errors.Wrap(err, "broadcast/receive block failed")
			return
		}
		errChan <- nil
	}()

	if err := vs.broadcastAndReceiveBlobs(ctx, sidecars, root); err != nil {
		return nil, status.Errorf(codes.Internal, "Could not broadcast/receive blobs: %v", err)
	}

	wg.Wait()
	if err := <-errChan; err != nil {
		return nil, status.Errorf(codes.Internal, "Could not broadcast/receive block: %v", err)
	}

	return &ethpb.ProposeResponse{BlockRoot: root[:]}, nil
}

// handleBlindedBlock processes blinded beacon blocks.
func (vs *Server) handleBlindedBlock(ctx context.Context, block interfaces.SignedBeaconBlock) (interfaces.SignedBeaconBlock, []*ethpb.BlobSidecar, error) {
	if block.Version() < version.Bellatrix {
		return nil, nil, errors.New("pre-Bellatrix blinded block")
	}
	if vs.BlockBuilder == nil || !vs.BlockBuilder.Configured() {
		return nil, nil, errors.New("unconfigured block builder")
	}

	copiedBlock, err := block.Copy()
	if err != nil {
		return nil, nil, err
	}

	payload, bundle, err := vs.BlockBuilder.SubmitBlindedBlock(ctx, block)
	if err != nil {
		return nil, nil, errors.Wrap(err, "submit blinded block failed")
	}

	if err := copiedBlock.Unblind(payload); err != nil {
		return nil, nil, errors.Wrap(err, "unblind failed")
	}

	sidecars, err := unblindBlobsSidecars(copiedBlock, bundle)
	if err != nil {
		return nil, nil, errors.Wrap(err, "unblind sidecars failed")
	}

	return copiedBlock, sidecars, nil
}

// handleUnblindedBlock processes unblinded beacon blocks.
func (vs *Server) handleUnblindedBlock(block interfaces.SignedBeaconBlock, req *ethpb.GenericSignedBeaconBlock) ([]*ethpb.BlobSidecar, error) {
	dbBlockContents := req.GetDeneb()
	if dbBlockContents == nil {
		return nil, nil
	}
	return buildBlobSidecars(block, dbBlockContents.Blobs, dbBlockContents.KzgProofs)
}

// broadcastReceiveBlock broadcasts a block and handles its reception.
func (vs *Server) broadcastReceiveBlock(ctx context.Context, block interfaces.SignedBeaconBlock, root [32]byte) error {
	protoBlock, err := block.Proto()
	if err != nil {
		return errors.Wrap(err, "protobuf conversion failed")
	}
	if err := vs.P2P.Broadcast(ctx, protoBlock); err != nil {
		return errors.Wrap(err, "broadcast failed")
	}
	vs.BlockNotifier.BlockFeed().Send(&feed.Event{
		Type: blockfeed.ReceivedBlock,
		Data: &blockfeed.ReceivedBlockData{SignedBlock: block},
	})
	return vs.BlockReceiver.ReceiveBlock(ctx, block, root, nil)
}

// broadcastAndReceiveBlobs handles the broadcasting and reception of blob sidecars.
func (vs *Server) broadcastAndReceiveBlobs(ctx context.Context, sidecars []*ethpb.BlobSidecar, root [32]byte) error {
	for i, sc := range sidecars {
		if err := vs.P2P.BroadcastBlob(ctx, uint64(i), sc); err != nil {
			return errors.Wrap(err, "broadcast blob failed")
		}

		readOnlySc, err := blocks.NewROBlobWithRoot(sc, root)
		if err != nil {
			return errors.Wrap(err, "ROBlob creation failed")
		}
		verifiedBlob := blocks.NewVerifiedROBlob(readOnlySc)
		if err := vs.BlobReceiver.ReceiveBlob(ctx, verifiedBlob); err != nil {
			return errors.Wrap(err, "receive blob failed")
		}
		vs.OperationNotifier.OperationFeed().Send(&feed.Event{
			Type: operation.BlobSidecarReceived,
			Data: &operation.BlobSidecarReceivedData{Blob: &verifiedBlob},
		})
	}
	return nil
}

// PrepareBeaconProposer caches and updates the fee recipient for the given proposer.
func (vs *Server) PrepareBeaconProposer(
	_ context.Context, request *ethpb.PrepareBeaconProposerRequest,
) (*emptypb.Empty, error) {
	var validatorIndices []primitives.ValidatorIndex

	for _, r := range request.Recipients {
		recipient := hexutil.Encode(r.FeeRecipient)
		if !common.IsHexAddress(recipient) {
			return nil, status.Errorf(codes.InvalidArgument, fmt.Sprintf("Invalid fee recipient address: %v", recipient))
		}
		// Use default address if the burn address is return
		feeRecipient := primitives.ExecutionAddress(r.FeeRecipient)
		if feeRecipient == primitives.ExecutionAddress([20]byte{}) {
			feeRecipient = primitives.ExecutionAddress(params.BeaconConfig().DefaultFeeRecipient)
			if feeRecipient == primitives.ExecutionAddress([20]byte{}) {
				log.WithField("validatorIndex", r.ValidatorIndex).Warn("fee recipient is the burn address")
			}
		}
		val := cache.TrackedValidator{
			Active:       true, // TODO: either check or add the field in the request
			Index:        r.ValidatorIndex,
			FeeRecipient: feeRecipient,
		}
		vs.TrackedValidatorsCache.Set(val)
		validatorIndices = append(validatorIndices, r.ValidatorIndex)
	}
	if len(validatorIndices) != 0 {
		log.WithFields(logrus.Fields{
			"validatorCount": len(validatorIndices),
		}).Info("Updated fee recipient addresses for validator indices")
	}
	return &emptypb.Empty{}, nil
}

// GetFeeRecipientByPubKey returns a fee recipient from the beacon node's settings or db based on a given public key
func (vs *Server) GetFeeRecipientByPubKey(ctx context.Context, request *ethpb.FeeRecipientByPubKeyRequest) (*ethpb.FeeRecipientByPubKeyResponse, error) {
	ctx, span := trace.StartSpan(ctx, "validator.GetFeeRecipientByPublicKey")
	defer span.End()
	if request == nil {
		return nil, status.Errorf(codes.InvalidArgument, "request was empty")
	}

	resp, err := vs.ValidatorIndex(ctx, &ethpb.ValidatorIndexRequest{PublicKey: request.PublicKey})
	if err != nil {
		if strings.Contains(err.Error(), "Could not find validator index") {
			return &ethpb.FeeRecipientByPubKeyResponse{
				FeeRecipient: params.BeaconConfig().DefaultFeeRecipient.Bytes(),
			}, nil
		} else {
			log.WithError(err).Error("An error occurred while retrieving validator index")
			return nil, err
		}
	}
	address, err := vs.BeaconDB.FeeRecipientByValidatorID(ctx, resp.GetIndex())
	if err != nil {
		if errors.Is(err, kv.ErrNotFoundFeeRecipient) {
			return &ethpb.FeeRecipientByPubKeyResponse{
				FeeRecipient: params.BeaconConfig().DefaultFeeRecipient.Bytes(),
			}, nil
		} else {
			log.WithError(err).Error("An error occurred while retrieving fee recipient from db")
			return nil, status.Errorf(codes.Internal, err.Error())
		}
	}
	return &ethpb.FeeRecipientByPubKeyResponse{
		FeeRecipient: address.Bytes(),
	}, nil
}

// computeStateRoot computes the state root after a block has been processed through a state transition and
// returns it to the validator client.
func (vs *Server) computeStateRoot(ctx context.Context, block interfaces.ReadOnlySignedBeaconBlock) ([]byte, error) {
	beaconState, err := vs.StateGen.StateByRoot(ctx, block.Block().ParentRoot())
	if err != nil {
		return nil, errors.Wrap(err, "could not retrieve beacon state")
	}
	root, err := transition.CalculateStateRoot(
		ctx,
		beaconState,
		block,
	)
	if err != nil {
		return nil, errors.Wrapf(err, "could not calculate state root at slot %d", beaconState.Slot())
	}

	log.WithField("beaconStateRoot", fmt.Sprintf("%#x", root)).Debugf("Computed state root")
	return root[:], nil
}

// SubmitValidatorRegistrations submits validator registrations.
func (vs *Server) SubmitValidatorRegistrations(ctx context.Context, reg *ethpb.SignedValidatorRegistrationsV1) (*emptypb.Empty, error) {
	if vs.BlockBuilder == nil || !vs.BlockBuilder.Configured() {
		return &emptypb.Empty{}, status.Errorf(codes.InvalidArgument, "Could not register block builder: %v", builder.ErrNoBuilder)
	}

	if err := vs.BlockBuilder.RegisterValidator(ctx, reg.Messages); err != nil {
		return nil, status.Errorf(codes.InvalidArgument, "Could not register block builder: %v", err)
	}

	return &emptypb.Empty{}, nil
}
