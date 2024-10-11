package rewards

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"
	"strings"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/epoch/precompute"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/wealdtech/go-bytesutil"
	"go.opencensus.io/trace"
)

// BlockRewards is an HTTP handler for Beacon API getBlockRewards.
func (s *Server) BlockRewards(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "beacon.BlockRewards")
	defer span.End()
	segments := strings.Split(r.URL.Path, "/")
	blockId := segments[len(segments)-1]

	blk, err := s.Blocker.Block(r.Context(), []byte(blockId))
	if !shared.WriteBlockFetchError(w, blk, err) {
		return
	}
	if blk.Version() == version.Phase0 {
		httputil.HandleError(w, "Block rewards are not supported for Phase 0 blocks", http.StatusBadRequest)
		return
	}

	optimistic, err := s.OptimisticModeFetcher.IsOptimistic(r.Context())
	if err != nil {
		httputil.HandleError(w, "Could not get optimistic mode info: "+err.Error(), http.StatusInternalServerError)
		return
	}
	blkRoot, err := blk.Block().HashTreeRoot()
	if err != nil {
		httputil.HandleError(w, "Could not get block root: "+err.Error(), http.StatusInternalServerError)
		return
	}
	blockRewards, httpError := s.BlockRewardFetcher.GetBlockRewardsData(ctx, blk.Block())
	if httpError != nil {
		httputil.WriteError(w, httpError)
		return
	}
	response := &BlockRewardsResponse{
		Data:                blockRewards,
		ExecutionOptimistic: optimistic,
		Finalized:           s.FinalizationFetcher.IsFinalized(ctx, blkRoot),
	}
	httputil.WriteJson(w, response)
}

// AttestationRewards retrieves attestation reward info for validators specified by array of public keys or validator index.
// If no array is provided, return reward info for every validator.
func (s *Server) AttestationRewards(w http.ResponseWriter, r *http.Request) {
	st, ok := s.attRewardsState(w, r)
	if !ok {
		return
	}
	bal, vals, valIndices, ok := attRewardsBalancesAndVals(w, r, st)
	if !ok {
		return
	}
	totalRewards, ok := totalAttRewards(w, st, bal, vals, valIndices)
	if !ok {
		return
	}
	idealRewards, ok := idealAttRewards(w, st, bal, vals)
	if !ok {
		return
	}

	optimistic, err := s.OptimisticModeFetcher.IsOptimistic(r.Context())
	if err != nil {
		httputil.HandleError(w, "Could not get optimistic mode info: "+err.Error(), http.StatusInternalServerError)
		return
	}
	blkRoot, err := st.LatestBlockHeader().HashTreeRoot()
	if err != nil {
		httputil.HandleError(w, "Could not get block root: "+err.Error(), http.StatusInternalServerError)
		return
	}

	resp := &AttestationRewardsResponse{
		Data: AttestationRewards{
			IdealRewards: idealRewards,
			TotalRewards: totalRewards,
		},
		ExecutionOptimistic: optimistic,
		Finalized:           s.FinalizationFetcher.IsFinalized(r.Context(), blkRoot),
	}
	httputil.WriteJson(w, resp)
}

// SyncCommitteeRewards retrieves rewards info for sync committee members specified by array of public keys or validator index.
// If no array is provided, return reward info for every committee member.
func (s *Server) SyncCommitteeRewards(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "beacon.SyncCommitteeRewards")
	defer span.End()
	segments := strings.Split(r.URL.Path, "/")
	blockId := segments[len(segments)-1]

	blk, err := s.Blocker.Block(r.Context(), []byte(blockId))
	if !shared.WriteBlockFetchError(w, blk, err) {
		return
	}
	if blk.Version() == version.Phase0 {
		httputil.HandleError(w, "Sync committee rewards are not supported for Phase 0", http.StatusBadRequest)
		return
	}

	st, httpErr := s.BlockRewardFetcher.GetStateForRewards(ctx, blk.Block())
	if httpErr != nil {
		httputil.WriteError(w, httpErr)
		return
	}
	sa, err := blk.Block().Body().SyncAggregate()
	if err != nil {
		httputil.HandleError(w, "Could not get sync aggregate: "+err.Error(), http.StatusInternalServerError)
		return
	}

	vals, valIndices, ok := syncRewardsVals(w, r, st)
	if !ok {
		return
	}
	preProcessBals := make([]uint64, len(vals))
	for i, valIdx := range valIndices {
		preProcessBals[i], err = st.BalanceAtIndex(valIdx)
		if err != nil {
			httputil.HandleError(w, "Could not get validator's balance: "+err.Error(), http.StatusInternalServerError)
			return
		}
	}

	_, proposerReward, err := altair.ProcessSyncAggregate(r.Context(), st, sa)
	if err != nil {
		httputil.HandleError(w, "Could not get sync aggregate rewards: "+err.Error(), http.StatusInternalServerError)
		return
	}

	rewards := make([]int, len(preProcessBals))
	proposerIndex := blk.Block().ProposerIndex()
	for i, valIdx := range valIndices {
		bal, err := st.BalanceAtIndex(valIdx)
		if err != nil {
			httputil.HandleError(w, "Could not get validator's balance: "+err.Error(), http.StatusInternalServerError)
			return
		}
		rewards[i] = int(bal - preProcessBals[i]) // lint:ignore uintcast
		if valIdx == proposerIndex {
			rewards[i] = rewards[i] - int(proposerReward) // lint:ignore uintcast
		}
	}

	optimistic, err := s.OptimisticModeFetcher.IsOptimistic(r.Context())
	if err != nil {
		httputil.HandleError(w, "Could not get optimistic mode info: "+err.Error(), http.StatusInternalServerError)
		return
	}
	blkRoot, err := blk.Block().HashTreeRoot()
	if err != nil {
		httputil.HandleError(w, "Could not get block root: "+err.Error(), http.StatusInternalServerError)
		return
	}

	scRewards := make([]SyncCommitteeReward, len(valIndices))
	for i, valIdx := range valIndices {
		scRewards[i] = SyncCommitteeReward{
			ValidatorIndex: strconv.FormatUint(uint64(valIdx), 10),
			Reward:         strconv.Itoa(rewards[i]),
		}
	}
	response := &SyncCommitteeRewardsResponse{
		Data:                scRewards,
		ExecutionOptimistic: optimistic,
		Finalized:           s.FinalizationFetcher.IsFinalized(r.Context(), blkRoot),
	}
	httputil.WriteJson(w, response)
}

func (s *Server) attRewardsState(w http.ResponseWriter, r *http.Request) (state.BeaconState, bool) {
	segments := strings.Split(r.URL.Path, "/")
	requestedEpoch, err := strconv.ParseUint(segments[len(segments)-1], 10, 64)
	if err != nil {
		httputil.HandleError(w, "Could not decode epoch: "+err.Error(), http.StatusBadRequest)
		return nil, false
	}
	if primitives.Epoch(requestedEpoch) < params.BeaconConfig().AltairForkEpoch {
		httputil.HandleError(w, "Attestation rewards are not supported for Phase 0", http.StatusNotFound)
		return nil, false
	}
	currentEpoch := uint64(slots.ToEpoch(s.TimeFetcher.CurrentSlot()))
	if requestedEpoch+1 >= currentEpoch {
		httputil.HandleError(w,
			"Attestation rewards are available after two epoch transitions to ensure all attestations have a chance of inclusion",
			http.StatusNotFound)
		return nil, false
	}
	nextEpochEnd, err := slots.EpochEnd(primitives.Epoch(requestedEpoch + 1))
	if err != nil {
		httputil.HandleError(w, "Could not get next epoch's ending slot: "+err.Error(), http.StatusInternalServerError)
		return nil, false
	}
	st, err := s.Stater.StateBySlot(r.Context(), nextEpochEnd)
	if err != nil {
		httputil.HandleError(w, "Could not get state for epoch's starting slot: "+err.Error(), http.StatusInternalServerError)
		return nil, false
	}
	return st, true
}

func attRewardsBalancesAndVals(
	w http.ResponseWriter,
	r *http.Request,
	st state.BeaconState,
) (*precompute.Balance, []*precompute.Validator, []primitives.ValidatorIndex, bool) {
	allVals, bal, err := altair.InitializePrecomputeValidators(r.Context(), st)
	if err != nil {
		httputil.HandleError(w, "Could not initialize precompute validators: "+err.Error(), http.StatusBadRequest)
		return nil, nil, nil, false
	}
	allVals, bal, err = altair.ProcessEpochParticipation(r.Context(), st, bal, allVals)
	if err != nil {
		httputil.HandleError(w, "Could not process epoch participation: "+err.Error(), http.StatusBadRequest)
		return nil, nil, nil, false
	}
	valIndices, ok := requestedValIndices(w, r, st, allVals)
	if !ok {
		return nil, nil, nil, false
	}
	if len(valIndices) == len(allVals) {
		return bal, allVals, valIndices, true
	} else {
		filteredVals := make([]*precompute.Validator, len(valIndices))
		for i, valIx := range valIndices {
			filteredVals[i] = allVals[valIx]
		}
		return bal, filteredVals, valIndices, true
	}
}

// idealAttRewards returns rewards for hypothetical, perfectly voting validators
// whose effective balances are over EJECTION_BALANCE and match balances in passed in validators.
func idealAttRewards(
	w http.ResponseWriter,
	st state.BeaconState,
	bal *precompute.Balance,
	vals []*precompute.Validator,
) ([]IdealAttestationReward, bool) {
	idealValsCount := uint64(16)
	minIdealBalance := uint64(17)
	maxIdealBalance := minIdealBalance + idealValsCount - 1
	idealRewards := make([]IdealAttestationReward, 0, idealValsCount)
	idealVals := make([]*precompute.Validator, 0, idealValsCount)
	increment := params.BeaconConfig().EffectiveBalanceIncrement
	for i := minIdealBalance; i <= maxIdealBalance; i++ {
		for _, v := range vals {
			if v.CurrentEpochEffectiveBalance/1e9 == i {
				effectiveBalance := i * increment
				idealVals = append(idealVals, &precompute.Validator{
					IsActivePrevEpoch:            true,
					IsSlashed:                    false,
					CurrentEpochEffectiveBalance: effectiveBalance,
					IsPrevEpochSourceAttester:    true,
					IsPrevEpochTargetAttester:    true,
					IsPrevEpochHeadAttester:      true,
				})
				idealRewards = append(idealRewards, IdealAttestationReward{
					EffectiveBalance: strconv.FormatUint(effectiveBalance, 10),
					Inactivity:       strconv.FormatUint(0, 10),
				})
				break
			}
		}
	}
	deltas, err := altair.AttestationsDelta(st, bal, idealVals)
	if err != nil {
		httputil.HandleError(w, "Could not get attestations delta: "+err.Error(), http.StatusInternalServerError)
		return nil, false
	}
	for i, d := range deltas {
		idealRewards[i].Head = strconv.FormatUint(d.HeadReward, 10)
		if d.SourcePenalty > 0 {
			idealRewards[i].Source = fmt.Sprintf("-%s", strconv.FormatUint(d.SourcePenalty, 10))
		} else {
			idealRewards[i].Source = strconv.FormatUint(d.SourceReward, 10)
		}
		if d.TargetPenalty > 0 {
			idealRewards[i].Target = fmt.Sprintf("-%s", strconv.FormatUint(d.TargetPenalty, 10))
		} else {
			idealRewards[i].Target = strconv.FormatUint(d.TargetReward, 10)
		}
		if d.InactivityPenalty > 0 {
			idealRewards[i].Inactivity = fmt.Sprintf("-%s", strconv.FormatUint(d.InactivityPenalty, 10))
		} else {
			idealRewards[i].Inactivity = strconv.FormatUint(d.InactivityPenalty, 10)
		}
	}
	return idealRewards, true
}

func totalAttRewards(
	w http.ResponseWriter,
	st state.BeaconState,
	bal *precompute.Balance,
	vals []*precompute.Validator,
	valIndices []primitives.ValidatorIndex,
) ([]TotalAttestationReward, bool) {
	totalRewards := make([]TotalAttestationReward, len(valIndices))
	for i, v := range valIndices {
		totalRewards[i] = TotalAttestationReward{ValidatorIndex: strconv.FormatUint(uint64(v), 10)}
	}
	deltas, err := altair.AttestationsDelta(st, bal, vals)
	if err != nil {
		httputil.HandleError(w, "Could not get attestations delta: "+err.Error(), http.StatusInternalServerError)
		return nil, false
	}
	for i, d := range deltas {
		totalRewards[i].Head = strconv.FormatUint(d.HeadReward, 10)
		if d.SourcePenalty > 0 {
			totalRewards[i].Source = fmt.Sprintf("-%s", strconv.FormatUint(d.SourcePenalty, 10))
		} else {
			totalRewards[i].Source = strconv.FormatUint(d.SourceReward, 10)
		}
		if d.TargetPenalty > 0 {
			totalRewards[i].Target = fmt.Sprintf("-%s", strconv.FormatUint(d.TargetPenalty, 10))
		} else {
			totalRewards[i].Target = strconv.FormatUint(d.TargetReward, 10)
		}
		if d.InactivityPenalty > 0 {
			totalRewards[i].Inactivity = fmt.Sprintf("-%s", strconv.FormatUint(d.InactivityPenalty, 10))
		} else {
			totalRewards[i].Inactivity = strconv.FormatUint(d.InactivityPenalty, 10)
		}
	}
	return totalRewards, true
}

func syncRewardsVals(
	w http.ResponseWriter,
	r *http.Request,
	st state.BeaconState,
) ([]*precompute.Validator, []primitives.ValidatorIndex, bool) {
	allVals, _, err := altair.InitializePrecomputeValidators(r.Context(), st)
	if err != nil {
		httputil.HandleError(w, "Could not initialize precompute validators: "+err.Error(), http.StatusBadRequest)
		return nil, nil, false
	}
	valIndices, ok := requestedValIndices(w, r, st, allVals)
	if !ok {
		return nil, nil, false
	}

	sc, err := st.CurrentSyncCommittee()
	if err != nil {
		httputil.HandleError(w, "Could not get current sync committee: "+err.Error(), http.StatusBadRequest)
		return nil, nil, false
	}
	allScIndices := make([]primitives.ValidatorIndex, len(sc.Pubkeys))
	for i, pk := range sc.Pubkeys {
		valIdx, ok := st.ValidatorIndexByPubkey(bytesutil.ToBytes48(pk))
		if !ok {
			httputil.HandleError(w, fmt.Sprintf("No validator index found for pubkey %#x", pk), http.StatusBadRequest)
			return nil, nil, false
		}
		allScIndices[i] = valIdx
	}

	scIndices := make([]primitives.ValidatorIndex, 0, len(allScIndices))
	scVals := make([]*precompute.Validator, 0, len(allScIndices))
	for _, valIdx := range valIndices {
		for _, scIdx := range allScIndices {
			if valIdx == scIdx {
				scVals = append(scVals, allVals[valIdx])
				scIndices = append(scIndices, valIdx)
				break
			}
		}
	}

	return scVals, scIndices, true
}

func requestedValIndices(w http.ResponseWriter, r *http.Request, st state.BeaconState, allVals []*precompute.Validator) ([]primitives.ValidatorIndex, bool) {
	var rawValIds []string
	if r.Body != http.NoBody {
		if err := json.NewDecoder(r.Body).Decode(&rawValIds); err != nil {
			httputil.HandleError(w, "Could not decode validators: "+err.Error(), http.StatusBadRequest)
			return nil, false
		}
	}
	valIndices := make([]primitives.ValidatorIndex, len(rawValIds))
	for i, v := range rawValIds {
		index, err := strconv.ParseUint(v, 10, 64)
		if err != nil {
			pubkey, err := bytesutil.FromHexString(v)
			if err != nil || len(pubkey) != fieldparams.BLSPubkeyLength {
				httputil.HandleError(w, fmt.Sprintf("%s is not a validator index or pubkey", v), http.StatusBadRequest)
				return nil, false
			}
			var ok bool
			valIndices[i], ok = st.ValidatorIndexByPubkey(bytesutil.ToBytes48(pubkey))
			if !ok {
				httputil.HandleError(w, fmt.Sprintf("No validator index found for pubkey %#x", pubkey), http.StatusBadRequest)
				return nil, false
			}
		} else {
			if index >= uint64(st.NumValidators()) {
				httputil.HandleError(w, fmt.Sprintf("Validator index %d is too large. Maximum allowed index is %d", index, st.NumValidators()-1), http.StatusBadRequest)
				return nil, false
			}
			valIndices[i] = primitives.ValidatorIndex(index)
		}
	}
	if len(valIndices) == 0 {
		valIndices = make([]primitives.ValidatorIndex, len(allVals))
		for i := 0; i < len(allVals); i++ {
			valIndices[i] = primitives.ValidatorIndex(i)
		}
	}

	return valIndices, true
}
