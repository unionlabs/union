package blockchain

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/time"
)

var stateDefragmentationTime = promauto.NewSummary(prometheus.SummaryOpts{
	Name: "head_state_defragmentation_milliseconds",
	Help: "Milliseconds it takes to defragment the head state",
})

// This method defragments our state, so that any specific fields which have
// a higher number of fragmented indexes are reallocated to a new separate slice for
// that field.
func (s *Service) defragmentState(st state.BeaconState) {
	if !features.Get().EnableExperimentalState {
		return
	}
	startTime := time.Now()
	st.Defragment()
	elapsedTime := time.Since(startTime)
	stateDefragmentationTime.Observe(float64(elapsedTime.Milliseconds()))
}
