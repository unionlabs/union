package beacon

import (
	"context"
	"fmt"
	"net/http"
	"strconv"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/gorilla/mux"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/lookup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	ethpbalpha "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"go.opencensus.io/trace"
)

type syncCommitteeStateRequest struct {
	epoch   *primitives.Epoch
	stateId []byte
}

// GetStateRoot calculates HashTreeRoot for state with given 'stateId'. If stateId is root, same value will be returned.
func (s *Server) GetStateRoot(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "beacon.GetStateRoot")
	defer span.End()

	stateId := mux.Vars(r)["state_id"]
	if stateId == "" {
		httputil.HandleError(w, "state_id is required in URL params", http.StatusBadRequest)
		return
	}

	stateRoot, err := s.Stater.StateRoot(ctx, []byte(stateId))
	if err != nil {
		if rootNotFoundErr, ok := err.(*lookup.StateRootNotFoundError); ok {
			httputil.HandleError(w, "State root not found: "+rootNotFoundErr.Error(), http.StatusNotFound)
			return
		} else if parseErr, ok := err.(*lookup.StateIdParseError); ok {
			httputil.HandleError(w, "Invalid state ID: "+parseErr.Error(), http.StatusBadRequest)
			return
		}
		httputil.HandleError(w, "Could not get state root: "+err.Error(), http.StatusInternalServerError)
		return
	}
	st, err := s.Stater.State(ctx, []byte(stateId))
	if err != nil {
		shared.WriteStateFetchError(w, err)
		return
	}
	isOptimistic, err := helpers.IsOptimistic(ctx, []byte(stateId), s.OptimisticModeFetcher, s.Stater, s.ChainInfoFetcher, s.BeaconDB)
	if err != nil {
		httputil.HandleError(w, "Could not check optimistic status: "+err.Error(), http.StatusInternalServerError)
		return
	}
	blockRoot, err := st.LatestBlockHeader().HashTreeRoot()
	if err != nil {
		httputil.HandleError(w, "Could not calculate root of latest block header: "+err.Error(), http.StatusInternalServerError)
		return
	}
	isFinalized := s.FinalizationFetcher.IsFinalized(ctx, blockRoot)

	resp := &GetStateRootResponse{
		Data: &StateRoot{
			Root: hexutil.Encode(stateRoot),
		},
		ExecutionOptimistic: isOptimistic,
		Finalized:           isFinalized,
	}
	httputil.WriteJson(w, resp)
}

// GetRandao fetches the RANDAO mix for the requested epoch from the state identified by state_id.
// If an epoch is not specified then the RANDAO mix for the state's current epoch will be returned.
// By adjusting the state_id parameter you can query for any historic value of the RANDAO mix.
// Ordinarily states from the same epoch will mutate the RANDAO mix for that epoch as blocks are applied.
func (s *Server) GetRandao(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "beacon.GetRandao")
	defer span.End()

	stateId := mux.Vars(r)["state_id"]
	if stateId == "" {
		httputil.HandleError(w, "state_id is required in URL params", http.StatusBadRequest)
		return
	}
	rawEpoch, e, ok := shared.UintFromQuery(w, r, "epoch", false)
	if !ok {
		return
	}

	st, err := s.Stater.State(ctx, []byte(stateId))
	if err != nil {
		shared.WriteStateFetchError(w, err)
		return
	}

	stEpoch := slots.ToEpoch(st.Slot())
	epoch := stEpoch
	if rawEpoch != "" {
		epoch = primitives.Epoch(e)
	}

	// future epochs and epochs too far back are not supported.
	randaoEpochLowerBound := uint64(0)
	// Lower bound should not underflow.
	if uint64(stEpoch) > uint64(st.RandaoMixesLength()) {
		randaoEpochLowerBound = uint64(stEpoch) - uint64(st.RandaoMixesLength())
	}
	if epoch > stEpoch || uint64(epoch) < randaoEpochLowerBound+1 {
		httputil.HandleError(w, "Epoch is out of range for the randao mixes of the state", http.StatusBadRequest)
		return
	}
	idx := epoch % params.BeaconConfig().EpochsPerHistoricalVector
	randao, err := st.RandaoMixAtIndex(uint64(idx))
	if err != nil {
		httputil.HandleError(w, fmt.Sprintf("Could not get randao mix at index %d: %v", idx, err), http.StatusInternalServerError)
		return
	}

	isOptimistic, err := helpers.IsOptimistic(ctx, []byte(stateId), s.OptimisticModeFetcher, s.Stater, s.ChainInfoFetcher, s.BeaconDB)
	if err != nil {
		httputil.HandleError(w, "Could not check optimistic status: "+err.Error(), http.StatusInternalServerError)
		return
	}

	blockRoot, err := st.LatestBlockHeader().HashTreeRoot()
	if err != nil {
		httputil.HandleError(w, "Could not calculate root of latest block header: "+err.Error(), http.StatusInternalServerError)
		return
	}
	isFinalized := s.FinalizationFetcher.IsFinalized(ctx, blockRoot)

	resp := &GetRandaoResponse{
		Data:                &Randao{Randao: hexutil.Encode(randao)},
		ExecutionOptimistic: isOptimistic,
		Finalized:           isFinalized,
	}
	httputil.WriteJson(w, resp)
}

// GetSyncCommittees retrieves the sync committees for the given epoch.
// If the epoch is not passed in, then the sync committees for the epoch of the state will be obtained.
func (s *Server) GetSyncCommittees(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "beacon.GetSyncCommittees")
	defer span.End()

	stateId := mux.Vars(r)["state_id"]
	if stateId == "" {
		httputil.HandleError(w, "state_id is required in URL params", http.StatusBadRequest)
		return
	}
	rawEpoch, e, ok := shared.UintFromQuery(w, r, "epoch", false)
	if !ok {
		return
	}
	epoch := primitives.Epoch(e)

	currentSlot := s.GenesisTimeFetcher.CurrentSlot()
	currentEpoch := slots.ToEpoch(currentSlot)
	currentPeriodStartEpoch, err := slots.SyncCommitteePeriodStartEpoch(currentEpoch)
	if err != nil {
		httputil.HandleError(w, fmt.Sprintf("Could not calculate start period for slot %d: %v", currentSlot, err), http.StatusInternalServerError)
		return
	}

	requestNextCommittee := false
	if rawEpoch != "" {
		reqPeriodStartEpoch, err := slots.SyncCommitteePeriodStartEpoch(epoch)
		if err != nil {
			httputil.HandleError(w, fmt.Sprintf("Could not calculate start period for epoch %d: %v", e, err), http.StatusInternalServerError)
			return
		}
		if reqPeriodStartEpoch > currentPeriodStartEpoch+params.BeaconConfig().EpochsPerSyncCommitteePeriod {
			httputil.HandleError(
				w,
				fmt.Sprintf("Could not fetch sync committee too far in the future (requested epoch %d, current epoch %d)", e, currentEpoch),
				http.StatusBadRequest,
			)
			return
		}
		if reqPeriodStartEpoch > currentPeriodStartEpoch {
			requestNextCommittee = true
			epoch = currentPeriodStartEpoch
		}
	}

	syncCommitteeReq := &syncCommitteeStateRequest{
		epoch:   nil,
		stateId: []byte(stateId),
	}
	if rawEpoch != "" {
		syncCommitteeReq.epoch = &epoch
	}
	st, ok := s.stateForSyncCommittee(ctx, w, syncCommitteeReq)
	if !ok {
		return
	}

	var committeeIndices []string
	var committee *ethpbalpha.SyncCommittee
	if requestNextCommittee {
		// Get the next sync committee and sync committee indices from the state.
		committeeIndices, committee, err = nextCommitteeIndicesFromState(st)
		if err != nil {
			httputil.HandleError(w, "Could not get next sync committee indices: "+err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		// Get the current sync committee and sync committee indices from the state.
		committeeIndices, committee, err = currentCommitteeIndicesFromState(st)
		if err != nil {
			httputil.HandleError(w, "Could not get current sync committee indices: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}
	subcommittees, err := extractSyncSubcommittees(st, committee)
	if err != nil {
		httputil.HandleError(w, "Could not extract sync subcommittees: "+err.Error(), http.StatusInternalServerError)
		return
	}

	isOptimistic, err := helpers.IsOptimistic(ctx, []byte(stateId), s.OptimisticModeFetcher, s.Stater, s.ChainInfoFetcher, s.BeaconDB)
	if err != nil {
		httputil.HandleError(w, "Could not check optimistic status: "+err.Error(), http.StatusInternalServerError)
		return
	}

	blockRoot, err := st.LatestBlockHeader().HashTreeRoot()
	if err != nil {
		httputil.HandleError(w, "Could not calculate root of latest block header: "+err.Error(), http.StatusInternalServerError)
		return
	}
	isFinalized := s.FinalizationFetcher.IsFinalized(ctx, blockRoot)

	resp := GetSyncCommitteeResponse{
		Data: &SyncCommitteeValidators{
			Validators:          committeeIndices,
			ValidatorAggregates: subcommittees,
		},
		ExecutionOptimistic: isOptimistic,
		Finalized:           isFinalized,
	}
	httputil.WriteJson(w, resp)
}

func committeeIndicesFromState(st state.BeaconState, committee *ethpbalpha.SyncCommittee) ([]string, *ethpbalpha.SyncCommittee, error) {
	committeeIndices := make([]string, len(committee.Pubkeys))
	for i, key := range committee.Pubkeys {
		index, ok := st.ValidatorIndexByPubkey(bytesutil.ToBytes48(key))
		if !ok {
			return nil, nil, fmt.Errorf(
				"validator index not found for pubkey %#x",
				bytesutil.Trunc(key),
			)
		}
		committeeIndices[i] = strconv.FormatUint(uint64(index), 10)
	}
	return committeeIndices, committee, nil
}

func currentCommitteeIndicesFromState(st state.BeaconState) ([]string, *ethpbalpha.SyncCommittee, error) {
	committee, err := st.CurrentSyncCommittee()
	if err != nil {
		return nil, nil, fmt.Errorf(
			"could not get sync committee: %v", err,
		)
	}

	return committeeIndicesFromState(st, committee)
}

func nextCommitteeIndicesFromState(st state.BeaconState) ([]string, *ethpbalpha.SyncCommittee, error) {
	committee, err := st.NextSyncCommittee()
	if err != nil {
		return nil, nil, fmt.Errorf(
			"could not get sync committee: %v", err,
		)
	}

	return committeeIndicesFromState(st, committee)
}

func extractSyncSubcommittees(st state.BeaconState, committee *ethpbalpha.SyncCommittee) ([][]string, error) {
	subcommitteeCount := params.BeaconConfig().SyncCommitteeSubnetCount
	subcommittees := make([][]string, subcommitteeCount)
	for i := uint64(0); i < subcommitteeCount; i++ {
		pubkeys, err := altair.SyncSubCommitteePubkeys(committee, primitives.CommitteeIndex(i))
		if err != nil {
			return nil, fmt.Errorf(
				"failed to get subcommittee pubkeys: %v", err,
			)
		}
		subcommittee := make([]string, len(pubkeys))
		for j, key := range pubkeys {
			index, ok := st.ValidatorIndexByPubkey(bytesutil.ToBytes48(key))
			if !ok {
				return nil, fmt.Errorf(
					"validator index not found for pubkey %#x",
					bytesutil.Trunc(key),
				)
			}
			subcommittee[j] = strconv.FormatUint(uint64(index), 10)
		}
		subcommittees[i] = subcommittee
	}
	return subcommittees, nil
}

func (s *Server) stateForSyncCommittee(ctx context.Context, w http.ResponseWriter, req *syncCommitteeStateRequest) (state.BeaconState, bool) {
	if req.epoch != nil {
		slot, err := slots.EpochStart(*req.epoch)
		if err != nil {
			httputil.HandleError(w, fmt.Sprintf("Could not calculate start slot for epoch %d: %v", *req.epoch, err), http.StatusInternalServerError)
			return nil, false
		}
		st, err := s.Stater.State(ctx, []byte(strconv.FormatUint(uint64(slot), 10)))
		if err != nil {
			shared.WriteStateFetchError(w, err)
			return nil, false
		}
		return st, true
	}
	st, err := s.Stater.State(ctx, req.stateId)
	if err != nil {
		shared.WriteStateFetchError(w, err)
		return nil, false
	}
	return st, true
}
