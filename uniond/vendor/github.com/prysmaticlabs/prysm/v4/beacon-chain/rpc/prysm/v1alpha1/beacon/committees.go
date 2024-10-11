package beacon

import (
	"context"
	"fmt"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// ListBeaconCommittees for a given epoch.
//
// If no filter criteria is specified, the response returns
// all beacon committees for the current epoch. The results are paginated by default.
func (bs *Server) ListBeaconCommittees(
	ctx context.Context,
	req *ethpb.ListCommitteesRequest,
) (*ethpb.BeaconCommittees, error) {
	currentSlot := bs.GenesisTimeFetcher.CurrentSlot()
	var requestedSlot primitives.Slot
	switch q := req.QueryFilter.(type) {
	case *ethpb.ListCommitteesRequest_Epoch:
		startSlot, err := slots.EpochStart(q.Epoch)
		if err != nil {
			return nil, err
		}
		requestedSlot = startSlot
	case *ethpb.ListCommitteesRequest_Genesis:
		requestedSlot = 0
	default:
		requestedSlot = currentSlot
	}

	requestedEpoch := slots.ToEpoch(requestedSlot)
	currentEpoch := slots.ToEpoch(currentSlot)
	if requestedEpoch > currentEpoch {
		return nil, status.Errorf(
			codes.InvalidArgument,
			"Cannot retrieve information for an future epoch, current epoch %d, requesting %d",
			currentEpoch,
			requestedEpoch,
		)
	}

	committees, activeIndices, err := bs.retrieveCommitteesForEpoch(ctx, requestedEpoch)
	if err != nil {
		return nil, status.Errorf(
			codes.Internal,
			"Could not retrieve committees for epoch %d: %v",
			requestedEpoch,
			err,
		)
	}

	return &ethpb.BeaconCommittees{
		Epoch:                requestedEpoch,
		Committees:           committees.SlotToUint64(),
		ActiveValidatorCount: uint64(len(activeIndices)),
	}, nil
}

func (bs *Server) retrieveCommitteesForEpoch(
	ctx context.Context,
	epoch primitives.Epoch,
) (SlotToCommiteesMap, []primitives.ValidatorIndex, error) {
	startSlot, err := slots.EpochStart(epoch)
	if err != nil {
		return nil, nil, err
	}
	requestedState, err := bs.ReplayerBuilder.ReplayerForSlot(startSlot).ReplayBlocks(ctx)
	if err != nil {
		return nil, nil, status.Errorf(codes.Internal, "error replaying blocks for state at slot %d: %v", startSlot, err)
	}
	seed, err := helpers.Seed(requestedState, epoch, params.BeaconConfig().DomainBeaconAttester)
	if err != nil {
		return nil, nil, status.Error(codes.Internal, "Could not get seed")
	}
	activeIndices, err := helpers.ActiveValidatorIndices(ctx, requestedState, epoch)
	if err != nil {
		return nil, nil, status.Error(codes.Internal, "Could not get active indices")
	}

	committeesListsBySlot, err := computeCommittees(ctx, startSlot, activeIndices, seed)
	if err != nil {
		return nil, nil, status.Errorf(
			codes.InvalidArgument,
			"Could not compute committees for epoch %d: %v",
			slots.ToEpoch(startSlot),
			err,
		)
	}
	return committeesListsBySlot, activeIndices, nil
}

// retrieveCommitteesForRoot uses the provided state root to get the current epoch committees.
// Note: This function is always recommended over retrieveCommitteesForEpoch as states are
// retrieved from the DB for this function, rather than generated.
func (bs *Server) retrieveCommitteesForRoot(
	ctx context.Context,
	root []byte,
) (SlotToCommiteesMap, []primitives.ValidatorIndex, error) {
	requestedState, err := bs.StateGen.StateByRoot(ctx, bytesutil.ToBytes32(root))
	if err != nil {
		return nil, nil, status.Error(codes.Internal, fmt.Sprintf("Could not get state: %v", err))
	}
	epoch := time.CurrentEpoch(requestedState)
	seed, err := helpers.Seed(requestedState, epoch, params.BeaconConfig().DomainBeaconAttester)
	if err != nil {
		return nil, nil, status.Error(codes.Internal, "Could not get seed")
	}
	activeIndices, err := helpers.ActiveValidatorIndices(ctx, requestedState, epoch)
	if err != nil {
		return nil, nil, status.Error(codes.Internal, "Could not get active indices")
	}

	startSlot, err := slots.EpochStart(epoch)
	if err != nil {
		return nil, nil, err
	}
	committeesListsBySlot, err := computeCommittees(ctx, startSlot, activeIndices, seed)
	if err != nil {
		return nil, nil, status.Errorf(
			codes.InvalidArgument,
			"Could not compute committees for epoch %d: %v",
			epoch,
			err,
		)
	}
	return committeesListsBySlot, activeIndices, nil
}

// Compute committees given a start slot, active validator indices, and
// the attester seeds value.
func computeCommittees(
	ctx context.Context,
	startSlot primitives.Slot,
	activeIndices []primitives.ValidatorIndex,
	attesterSeed [32]byte,
) (SlotToCommiteesMap, error) {
	committeesListsBySlot := make(SlotToCommiteesMap, params.BeaconConfig().SlotsPerEpoch)
	for slot := startSlot; slot < startSlot+params.BeaconConfig().SlotsPerEpoch; slot++ {
		var countAtSlot = uint64(len(activeIndices)) / uint64(params.BeaconConfig().SlotsPerEpoch) / params.BeaconConfig().TargetCommitteeSize
		if countAtSlot > params.BeaconConfig().MaxCommitteesPerSlot {
			countAtSlot = params.BeaconConfig().MaxCommitteesPerSlot
		}
		if countAtSlot == 0 {
			countAtSlot = 1
		}
		committeeItems := make([]*ethpb.BeaconCommittees_CommitteeItem, countAtSlot)
		for committeeIndex := uint64(0); committeeIndex < countAtSlot; committeeIndex++ {
			committee, err := helpers.BeaconCommittee(ctx, activeIndices, attesterSeed, slot, primitives.CommitteeIndex(committeeIndex))
			if err != nil {
				return nil, status.Errorf(
					codes.Internal,
					"Could not compute committee for slot %d: %v",
					slot,
					err,
				)
			}
			committeeItems[committeeIndex] = &ethpb.BeaconCommittees_CommitteeItem{
				ValidatorIndices: committee,
			}
		}
		committeesListsBySlot[slot] = &ethpb.BeaconCommittees_CommitteesList{
			Committees: committeeItems,
		}
	}
	return committeesListsBySlot, nil
}

// SlotToCommiteesMap represents <slot, CommitteeList> map.
type SlotToCommiteesMap map[primitives.Slot]*ethpb.BeaconCommittees_CommitteesList

// SlotToUint64 updates map keys to slots (workaround which will be unnecessary if we can cast
// map<uint64, CommitteesList> correctly in beacon_chain.proto)
func (m SlotToCommiteesMap) SlotToUint64() map[uint64]*ethpb.BeaconCommittees_CommitteesList {
	updatedCommittees := make(map[uint64]*ethpb.BeaconCommittees_CommitteesList, len(m))
	for k, v := range m {
		updatedCommittees[uint64(k)] = v
	}
	return updatedCommittees
}
