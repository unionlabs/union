package validator

import (
	"encoding/json"
	"net/http"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"go.opencensus.io/trace"
)

type PerformanceRequest struct {
	PublicKeys [][]byte                    `json:"public_keys,omitempty"`
	Indices    []primitives.ValidatorIndex `json:"indices,omitempty"`
}

type PerformanceResponse struct {
	PublicKeys                    [][]byte `json:"public_keys,omitempty"`
	CorrectlyVotedSource          []bool   `json:"correctly_voted_source,omitempty"`
	CorrectlyVotedTarget          []bool   `json:"correctly_voted_target,omitempty"`
	CorrectlyVotedHead            []bool   `json:"correctly_voted_head,omitempty"`
	CurrentEffectiveBalances      []uint64 `json:"current_effective_balances,omitempty"`
	BalancesBeforeEpochTransition []uint64 `json:"balances_before_epoch_transition,omitempty"`
	BalancesAfterEpochTransition  []uint64 `json:"balances_after_epoch_transition,omitempty"`
	MissingValidators             [][]byte `json:"missing_validators,omitempty"`
	InactivityScores              []uint64 `json:"inactivity_scores,omitempty"`
}

// GetValidatorPerformance is an HTTP handler for GetValidatorPerformance.
func (s *Server) GetValidatorPerformance(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.GetValidatorPerformance")
	defer span.End()

	var req PerformanceRequest
	if r.Body != http.NoBody {
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			handleHTTPError(w, "Could not decode request body: "+err.Error(), http.StatusBadRequest)
			return
		}
	}
	computed, err := s.CoreService.ComputeValidatorPerformance(
		ctx,
		&ethpb.ValidatorPerformanceRequest{
			PublicKeys: req.PublicKeys,
			Indices:    req.Indices,
		},
	)
	if err != nil {
		handleHTTPError(w, "Could not compute validator performance: "+err.Err.Error(), core.ErrorReasonToHTTP(err.Reason))
		return
	}
	response := &PerformanceResponse{
		PublicKeys:                    computed.PublicKeys,
		CorrectlyVotedSource:          computed.CorrectlyVotedSource,
		CorrectlyVotedTarget:          computed.CorrectlyVotedTarget, // In altair, when this is true then the attestation was definitely included.
		CorrectlyVotedHead:            computed.CorrectlyVotedHead,
		CurrentEffectiveBalances:      computed.CurrentEffectiveBalances,
		BalancesBeforeEpochTransition: computed.BalancesBeforeEpochTransition,
		BalancesAfterEpochTransition:  computed.BalancesAfterEpochTransition,
		MissingValidators:             computed.MissingValidators,
		InactivityScores:              computed.InactivityScores, // Only populated in Altair
	}
	httputil.WriteJson(w, response)
}

func handleHTTPError(w http.ResponseWriter, message string, code int) {
	errJson := &httputil.DefaultJsonError{
		Message: message,
		Code:    code,
	}
	httputil.WriteError(w, errJson)
}
