package validator

import (
	"context"
	"errors"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/contracts/deposit"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"go.opencensus.io/trace"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

var errPubkeyDoesNotExist = errors.New("pubkey does not exist")
var errHeadstateDoesNotExist = errors.New("head state does not exist")
var errOptimisticMode = errors.New("the node is currently optimistic and cannot serve validators")
var nonExistentIndex = primitives.ValidatorIndex(^uint64(0))

var errParticipation = status.Errorf(codes.Internal, "Failed to obtain epoch participation")

// ValidatorStatus returns the validator status of the current epoch.
// The status response can be one of the following:
//
//	DEPOSITED - validator's deposit has been recognized by Ethereum 1, not yet recognized by Ethereum.
//	PENDING - validator is in Ethereum's activation queue.
//	ACTIVE - validator is active.
//	EXITING - validator has initiated an exit request, or has dropped below the ejection balance and is being kicked out.
//	EXITED - validator is no longer validating.
//	SLASHING - validator has been kicked out due to meeting a slashing condition.
//	UNKNOWN_STATUS - validator does not have a known status in the network.
func (vs *Server) ValidatorStatus(
	ctx context.Context,
	req *ethpb.ValidatorStatusRequest,
) (*ethpb.ValidatorStatusResponse, error) {
	headState, err := vs.HeadFetcher.HeadStateReadOnly(ctx)
	if err != nil {
		return nil, status.Error(codes.Internal, "Could not get head state")
	}

	vStatus, _ := vs.validatorStatus(ctx, headState, req.PublicKey, func() (primitives.ValidatorIndex, error) { return helpers.LastActivatedValidatorIndex(ctx, headState) })
	return vStatus, nil
}

// MultipleValidatorStatus is the same as ValidatorStatus. Supports retrieval of multiple
// validator statuses. Takes a list of public keys or a list of validator indices.
func (vs *Server) MultipleValidatorStatus(
	ctx context.Context,
	req *ethpb.MultipleValidatorStatusRequest,
) (*ethpb.MultipleValidatorStatusResponse, error) {
	if vs.SyncChecker.Syncing() {
		return nil, status.Errorf(codes.Unavailable, "Syncing to latest head, not ready to respond")
	}
	headState, err := vs.HeadFetcher.HeadStateReadOnly(ctx)
	if err != nil {
		return nil, status.Error(codes.Internal, "Could not get head state")
	}
	responseCap := len(req.PublicKeys) + len(req.Indices)
	pubKeys := make([][]byte, 0, responseCap)
	filtered := make(map[[fieldparams.BLSPubkeyLength]byte]bool)
	filtered[[fieldparams.BLSPubkeyLength]byte{}] = true // Filter out keys with all zeros.
	// Filter out duplicate public keys.
	for _, pubKey := range req.PublicKeys {
		pubkeyBytes := bytesutil.ToBytes48(pubKey)
		if !filtered[pubkeyBytes] {
			pubKeys = append(pubKeys, pubKey)
			filtered[pubkeyBytes] = true
		}
	}
	// Convert indices to public keys.
	for _, idx := range req.Indices {
		pubkeyBytes := headState.PubkeyAtIndex(primitives.ValidatorIndex(idx))
		if !filtered[pubkeyBytes] {
			pubKeys = append(pubKeys, pubkeyBytes[:])
			filtered[pubkeyBytes] = true
		}
	}
	// Fetch statuses from beacon state.
	statuses := make([]*ethpb.ValidatorStatusResponse, len(pubKeys))
	indices := make([]primitives.ValidatorIndex, len(pubKeys))
	lastActivated, hpErr := helpers.LastActivatedValidatorIndex(ctx, headState)
	for i, pubKey := range pubKeys {
		statuses[i], indices[i] = vs.validatorStatus(ctx, headState, pubKey, func() (primitives.ValidatorIndex, error) { return lastActivated, hpErr })
	}

	return &ethpb.MultipleValidatorStatusResponse{
		PublicKeys: pubKeys,
		Statuses:   statuses,
		Indices:    indices,
	}, nil
}

// CheckDoppelGanger checks if the provided keys are currently active in the network.
func (vs *Server) CheckDoppelGanger(ctx context.Context, req *ethpb.DoppelGangerRequest) (*ethpb.DoppelGangerResponse, error) {
	if vs.SyncChecker.Syncing() {
		return nil, status.Errorf(codes.Unavailable, "Syncing to latest head, not ready to respond")
	}
	if req == nil || req.ValidatorRequests == nil || len(req.ValidatorRequests) == 0 {
		return &ethpb.DoppelGangerResponse{
			Responses: []*ethpb.DoppelGangerResponse_ValidatorResponse{},
		}, nil
	}
	headState, err := vs.HeadFetcher.HeadStateReadOnly(ctx)
	if err != nil {
		return nil, status.Error(codes.Internal, "Could not get head state")
	}

	// Return early if we are in phase0.
	if headState.Version() == version.Phase0 {
		log.Info("Skipping doppelganger check for Phase 0")

		resp := &ethpb.DoppelGangerResponse{
			Responses: []*ethpb.DoppelGangerResponse_ValidatorResponse{},
		}
		for _, v := range req.ValidatorRequests {
			resp.Responses = append(resp.Responses,
				&ethpb.DoppelGangerResponse_ValidatorResponse{
					PublicKey:       v.PublicKey,
					DuplicateExists: false,
				})
		}
		return resp, nil
	}

	headSlot := headState.Slot()
	currEpoch := slots.ToEpoch(headSlot)

	// If all provided keys are recent we skip this check
	// as we are unable to effectively determine if a doppelganger
	// is active.
	isRecent, resp := checkValidatorsAreRecent(currEpoch, req)
	if isRecent {
		return resp, nil
	}

	// We request a state 32 slots ago. We are guaranteed to have
	// currentSlot > 32 since we assume that we are in Altair's fork.
	prevStateSlot := headSlot - params.BeaconConfig().SlotsPerEpoch
	prevEpochEnd, err := slots.EpochEnd(slots.ToEpoch(prevStateSlot))
	if err != nil {
		return nil, status.Error(codes.Internal, "Could not get previous epoch's end")
	}
	prevState, err := vs.ReplayerBuilder.ReplayerForSlot(prevEpochEnd).ReplayBlocks(ctx)
	if err != nil {
		return nil, status.Error(codes.Internal, "Could not get previous state")
	}

	headCurrentParticipation, err := headState.CurrentEpochParticipation()
	if err != nil {
		return nil, errParticipation
	}
	headPreviousParticipation, err := headState.PreviousEpochParticipation()
	if err != nil {
		return nil, errParticipation
	}
	prevCurrentParticipation, err := prevState.CurrentEpochParticipation()
	if err != nil {
		return nil, errParticipation
	}
	prevPreviousParticipation, err := prevState.PreviousEpochParticipation()
	if err != nil {
		return nil, errParticipation
	}

	resp = &ethpb.DoppelGangerResponse{
		Responses: []*ethpb.DoppelGangerResponse_ValidatorResponse{},
	}
	for _, v := range req.ValidatorRequests {
		// If the validator's last recorded epoch was less than 1 epoch
		// ago, the current doppelganger check will not be able to
		// identify dopplelgangers since an attestation can take up to
		// 31 slots to be included.
		if v.Epoch+2 >= currEpoch {
			resp.Responses = append(resp.Responses,
				&ethpb.DoppelGangerResponse_ValidatorResponse{
					PublicKey:       v.PublicKey,
					DuplicateExists: false,
				})
			continue
		}
		valIndex, ok := prevState.ValidatorIndexByPubkey(bytesutil.ToBytes48(v.PublicKey))
		if !ok {
			// Ignore if validator pubkey doesn't exist.
			continue
		}

		if (headCurrentParticipation[valIndex] != 0) || (headPreviousParticipation[valIndex] != 0) ||
			(prevCurrentParticipation[valIndex] != 0) || (prevPreviousParticipation[valIndex] != 0) {
			log.WithField("ValidatorIndex", valIndex).Infof("Participation flag found")
			resp.Responses = append(resp.Responses,
				&ethpb.DoppelGangerResponse_ValidatorResponse{
					PublicKey:       v.PublicKey,
					DuplicateExists: true,
				})
			continue
		}
		// Mark the public key as valid.
		resp.Responses = append(resp.Responses,
			&ethpb.DoppelGangerResponse_ValidatorResponse{
				PublicKey:       v.PublicKey,
				DuplicateExists: false,
			})
	}
	return resp, nil
}

// activationStatus returns the validator status response for the set of validators
// requested by their pub keys.
func (vs *Server) activationStatus(
	ctx context.Context,
	pubKeys [][]byte,
) (bool, []*ethpb.ValidatorActivationResponse_Status, error) {
	headState, err := vs.HeadFetcher.HeadStateReadOnly(ctx)
	if err != nil {
		return false, nil, err
	}
	activeValidatorExists := false
	statusResponses := make([]*ethpb.ValidatorActivationResponse_Status, len(pubKeys))
	// only run calculation of last activated once per state
	lastActivated, hpErr := helpers.LastActivatedValidatorIndex(ctx, headState)
	for i, pubKey := range pubKeys {
		if ctx.Err() != nil {
			return false, nil, ctx.Err()
		}
		vStatus, idx := vs.validatorStatus(ctx, headState, pubKey, func() (primitives.ValidatorIndex, error) { return lastActivated, hpErr })
		if vStatus == nil {
			continue
		}
		resp := &ethpb.ValidatorActivationResponse_Status{
			Status:    vStatus,
			PublicKey: pubKey,
			Index:     idx,
		}
		statusResponses[i] = resp
		if vStatus.Status == ethpb.ValidatorStatus_ACTIVE {
			activeValidatorExists = true
		}
	}

	return activeValidatorExists, statusResponses, nil
}

// optimisticStatus returns an error if the node is currently optimistic with respect to head.
// by definition, an optimistic node is not a full node. It is unable to produce blocks,
// since an execution engine cannot produce a payload upon an unknown parent.
// It cannot faithfully attest to the head block of the chain, since it has not fully verified that block.
//
// Spec:
// https://github.com/ethereum/consensus-specs/blob/dev/sync/optimistic.md
func (vs *Server) optimisticStatus(ctx context.Context) error {
	if slots.ToEpoch(vs.TimeFetcher.CurrentSlot()) < params.BeaconConfig().BellatrixForkEpoch {
		return nil
	}
	optimistic, err := vs.OptimisticModeFetcher.IsOptimistic(ctx)
	if err != nil {
		return status.Errorf(codes.Internal, "Could not determine if the node is a optimistic node: %v", err)
	}
	if !optimistic {
		return nil
	}

	return status.Errorf(codes.Unavailable, errOptimisticMode.Error())
}

// validatorStatus searches for the requested validator's state and deposit to retrieve its inclusion estimate. Also returns the validators index.
func (vs *Server) validatorStatus(
	ctx context.Context,
	headState state.ReadOnlyBeaconState,
	pubKey []byte,
	lastActiveValidatorFn func() (primitives.ValidatorIndex, error),
) (*ethpb.ValidatorStatusResponse, primitives.ValidatorIndex) {
	ctx, span := trace.StartSpan(ctx, "ValidatorServer.validatorStatus")
	defer span.End()

	// Using ^0 as the default value for index, in case the validators index cannot be determined.
	resp := &ethpb.ValidatorStatusResponse{
		Status:          ethpb.ValidatorStatus_UNKNOWN_STATUS,
		ActivationEpoch: params.BeaconConfig().FarFutureEpoch,
	}
	vStatus, idx, err := statusForPubKey(headState, pubKey)
	if err != nil && err != errPubkeyDoesNotExist {
		tracing.AnnotateError(span, err)
		return resp, nonExistentIndex
	}
	resp.Status = vStatus
	if err != errPubkeyDoesNotExist {
		val, err := headState.ValidatorAtIndexReadOnly(idx)
		if err != nil {
			tracing.AnnotateError(span, err)
			return resp, idx
		}
		resp.ActivationEpoch = val.ActivationEpoch()
	}

	switch resp.Status {
	// Unknown status means the validator has not been put into the state yet.
	case ethpb.ValidatorStatus_UNKNOWN_STATUS:
		// If no connection to ETH1, the deposit block number or position in queue cannot be determined.
		if !vs.Eth1InfoFetcher.ExecutionClientConnected() {
			log.Warn("Not connected to ETH1. Cannot determine validator ETH1 deposit block number")
			return resp, nonExistentIndex
		}
		dep, eth1BlockNumBigInt := vs.DepositFetcher.DepositByPubkey(ctx, pubKey)
		if eth1BlockNumBigInt == nil { // No deposit found in ETH1.
			return resp, nonExistentIndex
		}
		domain, err := signing.ComputeDomain(
			params.BeaconConfig().DomainDeposit,
			nil, /*forkVersion*/
			nil, /*genesisValidatorsRoot*/
		)
		if err != nil {
			log.Warn("Could not compute domain")
			return resp, nonExistentIndex
		}
		if err := deposit.VerifyDepositSignature(dep.Data, domain); err != nil {
			resp.Status = ethpb.ValidatorStatus_INVALID
			log.Warn("Invalid Eth1 deposit")
			return resp, nonExistentIndex
		}
		// Set validator deposit status if their deposit is visible.
		resp.Status = depositStatus(dep.Data.Amount)
		resp.Eth1DepositBlockNumber = eth1BlockNumBigInt.Uint64()

		return resp, nonExistentIndex
	// Deposited, Pending or Partially Deposited mean the validator has been put into the state.
	case ethpb.ValidatorStatus_DEPOSITED, ethpb.ValidatorStatus_PENDING, ethpb.ValidatorStatus_PARTIALLY_DEPOSITED:
		if resp.Status == ethpb.ValidatorStatus_PENDING {
			if vs.DepositFetcher == nil {
				log.Warn("Not connected to ETH1. Cannot determine validator ETH1 deposit.")
			} else {
				// Check if there was a deposit.
				_, eth1BlockNumBigInt := vs.DepositFetcher.DepositByPubkey(ctx, pubKey)
				if eth1BlockNumBigInt != nil {
					resp.Eth1DepositBlockNumber = eth1BlockNumBigInt.Uint64()
				}
			}
		}
		if lastActiveValidatorFn == nil {
			return resp, idx
		}
		lastActivatedvalidatorIndex, err := lastActiveValidatorFn()
		if err != nil {
			return resp, idx
		}
		// Our position in the activation queue is the above index - our validator index.
		if lastActivatedvalidatorIndex < idx {
			resp.PositionInActivationQueue = uint64(idx - lastActivatedvalidatorIndex)
		}
		return resp, idx
	default:
		return resp, idx
	}
}

func checkValidatorsAreRecent(headEpoch primitives.Epoch, req *ethpb.DoppelGangerRequest) (bool, *ethpb.DoppelGangerResponse) {
	validatorsAreRecent := true
	resp := &ethpb.DoppelGangerResponse{
		Responses: []*ethpb.DoppelGangerResponse_ValidatorResponse{},
	}
	for _, v := range req.ValidatorRequests {
		// Due to how balances are reflected for individual
		// validators, we can only effectively determine if a
		// validator voted or not if we are able to look
		// back more than 2 epoch into the past.
		if v.Epoch+2 < headEpoch {
			validatorsAreRecent = false
			// Zero out response if we encounter non-recent validators to
			// guard against potential misuse.
			resp.Responses = []*ethpb.DoppelGangerResponse_ValidatorResponse{}
			break
		}
		resp.Responses = append(resp.Responses,
			&ethpb.DoppelGangerResponse_ValidatorResponse{
				PublicKey:       v.PublicKey,
				DuplicateExists: false,
			})
	}
	return validatorsAreRecent, resp
}

func statusForPubKey(headState state.ReadOnlyBeaconState, pubKey []byte) (ethpb.ValidatorStatus, primitives.ValidatorIndex, error) {
	if headState == nil || headState.IsNil() {
		return ethpb.ValidatorStatus_UNKNOWN_STATUS, 0, errHeadstateDoesNotExist
	}
	idx, ok := headState.ValidatorIndexByPubkey(bytesutil.ToBytes48(pubKey))
	if !ok || uint64(idx) >= uint64(headState.NumValidators()) {
		return ethpb.ValidatorStatus_UNKNOWN_STATUS, 0, errPubkeyDoesNotExist
	}
	return assignmentStatus(headState, idx), idx, nil
}

func assignmentStatus(beaconState state.ReadOnlyBeaconState, validatorIndex primitives.ValidatorIndex) ethpb.ValidatorStatus {
	validator, err := beaconState.ValidatorAtIndexReadOnly(validatorIndex)
	if err != nil {
		return ethpb.ValidatorStatus_UNKNOWN_STATUS
	}
	currentEpoch := time.CurrentEpoch(beaconState)
	farFutureEpoch := params.BeaconConfig().FarFutureEpoch
	validatorBalance := validator.EffectiveBalance()

	if validator.IsNil() {
		return ethpb.ValidatorStatus_UNKNOWN_STATUS
	}
	if currentEpoch < validator.ActivationEligibilityEpoch() {
		return depositStatus(validatorBalance)
	}
	if currentEpoch < validator.ActivationEpoch() {
		return ethpb.ValidatorStatus_PENDING
	}
	if validator.ExitEpoch() == farFutureEpoch {
		return ethpb.ValidatorStatus_ACTIVE
	}
	if currentEpoch < validator.ExitEpoch() {
		if validator.Slashed() {
			return ethpb.ValidatorStatus_SLASHING
		}
		return ethpb.ValidatorStatus_EXITING
	}
	return ethpb.ValidatorStatus_EXITED
}

func depositStatus(depositOrBalance uint64) ethpb.ValidatorStatus {
	if depositOrBalance == 0 {
		return ethpb.ValidatorStatus_PENDING
	} else if depositOrBalance < params.BeaconConfig().MaxEffectiveBalance {
		return ethpb.ValidatorStatus_PARTIALLY_DEPOSITED
	}
	return ethpb.ValidatorStatus_DEPOSITED
}
