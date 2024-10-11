package builder

import (
	"fmt"
	"net/http"
	"strconv"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/gorilla/mux"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// ExpectedWithdrawals get the withdrawals computed from the specified state, that will be included in the block that gets built on the specified state.
func (s *Server) ExpectedWithdrawals(w http.ResponseWriter, r *http.Request) {
	// Retrieve beacon state
	stateId := mux.Vars(r)["state_id"]
	if stateId == "" {
		httputil.WriteError(w, &httputil.DefaultJsonError{
			Message: "state_id is required in URL params",
			Code:    http.StatusBadRequest,
		})
		return
	}
	st, err := s.Stater.State(r.Context(), []byte(stateId))
	if err != nil {
		httputil.WriteError(w, handleWrapError(err, "could not retrieve state", http.StatusNotFound))
		return
	}
	queryParam := r.URL.Query().Get("proposal_slot")
	var proposalSlot primitives.Slot
	if queryParam != "" {
		pSlot, err := strconv.ParseUint(queryParam, 10, 64)
		if err != nil {
			httputil.WriteError(w, handleWrapError(err, "invalid proposal slot value", http.StatusBadRequest))
			return
		}
		proposalSlot = primitives.Slot(pSlot)
	} else {
		proposalSlot = st.Slot() + 1
	}
	// Perform sanity checks on proposal slot before computing state
	capellaStart, err := slots.EpochStart(params.BeaconConfig().CapellaForkEpoch)
	if err != nil {
		httputil.WriteError(w, handleWrapError(err, "could not calculate Capella start slot", http.StatusInternalServerError))
		return
	}
	if proposalSlot < capellaStart {
		httputil.WriteError(w, &httputil.DefaultJsonError{
			Message: "expected withdrawals are not supported before Capella fork",
			Code:    http.StatusBadRequest,
		})
		return
	}
	if proposalSlot <= st.Slot() {
		httputil.WriteError(w, &httputil.DefaultJsonError{
			Message: fmt.Sprintf("proposal slot must be bigger than state slot. proposal slot: %d, state slot: %d", proposalSlot, st.Slot()),
			Code:    http.StatusBadRequest,
		})
		return
	}
	lookAheadLimit := uint64(params.BeaconConfig().SlotsPerEpoch.Mul(uint64(params.BeaconConfig().MaxSeedLookahead)))
	if st.Slot().Add(lookAheadLimit) <= proposalSlot {
		httputil.WriteError(w, &httputil.DefaultJsonError{
			Message: fmt.Sprintf("proposal slot cannot be >= %d slots ahead of state slot", lookAheadLimit),
			Code:    http.StatusBadRequest,
		})
		return
	}
	// Get metadata for response
	isOptimistic, err := s.OptimisticModeFetcher.IsOptimistic(r.Context())
	if err != nil {
		httputil.WriteError(w, handleWrapError(err, "could not get optimistic mode info", http.StatusInternalServerError))
		return
	}
	root, err := helpers.BlockRootAtSlot(st, st.Slot()-1)
	if err != nil {
		httputil.WriteError(w, handleWrapError(err, "could not get block root", http.StatusInternalServerError))
		return
	}
	var blockRoot = [32]byte(root)
	isFinalized := s.FinalizationFetcher.IsFinalized(r.Context(), blockRoot)
	// Advance state forward to proposal slot
	st, err = transition.ProcessSlots(r.Context(), st, proposalSlot)
	if err != nil {
		httputil.WriteError(w, &httputil.DefaultJsonError{
			Message: "could not process slots",
			Code:    http.StatusInternalServerError,
		})
		return
	}
	withdrawals, err := st.ExpectedWithdrawals()
	if err != nil {
		httputil.WriteError(w, &httputil.DefaultJsonError{
			Message: "could not get expected withdrawals",
			Code:    http.StatusInternalServerError,
		})
		return
	}
	httputil.WriteJson(w, &ExpectedWithdrawalsResponse{
		ExecutionOptimistic: isOptimistic,
		Finalized:           isFinalized,
		Data:                buildExpectedWithdrawalsData(withdrawals),
	})
}

func buildExpectedWithdrawalsData(withdrawals []*enginev1.Withdrawal) []*ExpectedWithdrawal {
	data := make([]*ExpectedWithdrawal, len(withdrawals))
	for i, withdrawal := range withdrawals {
		data[i] = &ExpectedWithdrawal{
			Address:        hexutil.Encode(withdrawal.Address),
			Amount:         strconv.FormatUint(withdrawal.Amount, 10),
			Index:          strconv.FormatUint(withdrawal.Index, 10),
			ValidatorIndex: strconv.FormatUint(uint64(withdrawal.ValidatorIndex), 10),
		}
	}
	return data
}

func handleWrapError(err error, message string, code int) *httputil.DefaultJsonError {
	return &httputil.DefaultJsonError{
		Message: errors.Wrapf(err, message).Error(),
		Code:    code,
	}
}
