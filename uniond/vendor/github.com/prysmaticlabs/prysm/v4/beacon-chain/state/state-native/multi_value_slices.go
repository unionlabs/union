package state_native

import (
	"runtime"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native/types"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	multi_value_slice "github.com/prysmaticlabs/prysm/v4/container/multi-value-slice"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

var (
	multiValueCountGauge = promauto.NewGaugeVec(prometheus.GaugeOpts{
		Name: "multi_value_object_count",
		Help: "The number of instances that exist for the multivalue slice for a particular field.",
	}, []string{"field"})
	multiValueIndividualElementsCountGauge = promauto.NewGaugeVec(prometheus.GaugeOpts{
		Name: "multi_value_individual_elements_count",
		Help: "The number of individual elements that exist for the multivalue slice object.",
	}, []string{"field"})
	multiValueIndividualElementReferencesCountGauge = promauto.NewGaugeVec(prometheus.GaugeOpts{
		Name: "multi_value_individual_element_references_count",
		Help: "The number of individual element references that exist for the multivalue slice object.",
	}, []string{"field"})
	multiValueAppendedElementsCountGauge = promauto.NewGaugeVec(prometheus.GaugeOpts{
		Name: "multi_value_appended_elements_count",
		Help: "The number of appended elements that exist for the multivalue slice object.",
	}, []string{"field"})
	multiValueAppendedElementReferencesCountGauge = promauto.NewGaugeVec(prometheus.GaugeOpts{
		Name: "multi_value_appended_element_references_count",
		Help: "The number of appended element references that exist for the multivalue slice object.",
	}, []string{"field"})
)

// MultiValueRandaoMixes is a multi-value slice of randao mixes.
type MultiValueRandaoMixes = multi_value_slice.Slice[[32]byte]

// NewMultiValueRandaoMixes creates a new slice whose shared items will be populated with copies of input values.
func NewMultiValueRandaoMixes(mixes [][]byte) *MultiValueRandaoMixes {
	items := make([][32]byte, fieldparams.RandaoMixesLength)
	for i, v := range mixes {
		items[i] = [32]byte(bytesutil.PadTo(v, 32))
	}
	mv := &MultiValueRandaoMixes{}
	mv.Init(items)
	multiValueCountGauge.WithLabelValues(types.RandaoMixes.String()).Inc()
	runtime.SetFinalizer(mv, randaoMixesFinalizer)
	return mv
}

// MultiValueBlockRoots is a multi-value slice of block roots.
type MultiValueBlockRoots = multi_value_slice.Slice[[32]byte]

// NewMultiValueBlockRoots creates a new slice whose shared items will be populated with copies of input values.
func NewMultiValueBlockRoots(roots [][]byte) *MultiValueBlockRoots {
	items := make([][32]byte, fieldparams.BlockRootsLength)
	for i, v := range roots {
		items[i] = [32]byte(bytesutil.PadTo(v, 32))
	}
	mv := &MultiValueBlockRoots{}
	mv.Init(items)
	multiValueCountGauge.WithLabelValues(types.BlockRoots.String()).Inc()
	runtime.SetFinalizer(mv, blockRootsFinalizer)
	return mv
}

// MultiValueStateRoots is a multi-value slice of state roots.
type MultiValueStateRoots = multi_value_slice.Slice[[32]byte]

// NewMultiValueStateRoots creates a new slice whose shared items will be populated with copies of input values.
func NewMultiValueStateRoots(roots [][]byte) *MultiValueStateRoots {
	items := make([][32]byte, fieldparams.StateRootsLength)
	for i, v := range roots {
		items[i] = [32]byte(bytesutil.PadTo(v, 32))
	}
	mv := &MultiValueStateRoots{}
	mv.Init(items)
	multiValueCountGauge.WithLabelValues(types.StateRoots.String()).Inc()
	runtime.SetFinalizer(mv, stateRootsFinalizer)
	return mv
}

// MultiValueBalances is a multi-value slice of balances.
type MultiValueBalances = multi_value_slice.Slice[uint64]

// NewMultiValueBalances creates a new slice whose shared items will be populated with copies of input values.
func NewMultiValueBalances(balances []uint64) *MultiValueBalances {
	items := make([]uint64, len(balances))
	copy(items, balances)
	mv := &MultiValueBalances{}
	mv.Init(items)
	multiValueCountGauge.WithLabelValues(types.Balances.String()).Inc()
	runtime.SetFinalizer(mv, balancesFinalizer)
	return mv
}

// MultiValueInactivityScores is a multi-value slice of inactivity scores.
type MultiValueInactivityScores = multi_value_slice.Slice[uint64]

// NewMultiValueInactivityScores creates a new slice whose shared items will be populated with copies of input values.
func NewMultiValueInactivityScores(scores []uint64) *MultiValueInactivityScores {
	items := make([]uint64, len(scores))
	copy(items, scores)
	mv := &MultiValueInactivityScores{}
	mv.Init(items)
	multiValueCountGauge.WithLabelValues(types.InactivityScores.String()).Inc()
	runtime.SetFinalizer(mv, inactivityScoresFinalizer)
	return mv
}

// MultiValueValidators is a multi-value slice of validator references.
type MultiValueValidators = multi_value_slice.Slice[*ethpb.Validator]

// NewMultiValueValidators creates a new slice whose shared items will be populated with input values.
func NewMultiValueValidators(vals []*ethpb.Validator) *MultiValueValidators {
	mv := &MultiValueValidators{}
	mv.Init(vals)
	multiValueCountGauge.WithLabelValues(types.Validators.String()).Inc()
	runtime.SetFinalizer(mv, validatorsFinalizer)
	return mv
}

// Defragment checks whether each individual multi-value field in our state is fragmented
// and if it is, it will 'reset' the field to create a new multivalue object.
func (b *BeaconState) Defragment() {
	b.lock.Lock()
	defer b.lock.Unlock()
	if b.blockRootsMultiValue != nil && b.blockRootsMultiValue.IsFragmented() {
		initialMVslice := b.blockRootsMultiValue
		b.blockRootsMultiValue = b.blockRootsMultiValue.Reset(b)
		initialMVslice.Detach(b)
		multiValueCountGauge.WithLabelValues(types.BlockRoots.String()).Inc()
		runtime.SetFinalizer(b.blockRootsMultiValue, blockRootsFinalizer)
	}
	if b.stateRootsMultiValue != nil && b.stateRootsMultiValue.IsFragmented() {
		initialMVslice := b.stateRootsMultiValue
		b.stateRootsMultiValue = b.stateRootsMultiValue.Reset(b)
		initialMVslice.Detach(b)
		multiValueCountGauge.WithLabelValues(types.StateRoots.String()).Inc()
		runtime.SetFinalizer(b.stateRootsMultiValue, stateRootsFinalizer)
	}
	if b.randaoMixesMultiValue != nil && b.randaoMixesMultiValue.IsFragmented() {
		initialMVslice := b.randaoMixesMultiValue
		b.randaoMixesMultiValue = b.randaoMixesMultiValue.Reset(b)
		initialMVslice.Detach(b)
		multiValueCountGauge.WithLabelValues(types.RandaoMixes.String()).Inc()
		runtime.SetFinalizer(b.randaoMixesMultiValue, randaoMixesFinalizer)
	}
	if b.balancesMultiValue != nil && b.balancesMultiValue.IsFragmented() {
		initialMVslice := b.balancesMultiValue
		b.balancesMultiValue = b.balancesMultiValue.Reset(b)
		initialMVslice.Detach(b)
		multiValueCountGauge.WithLabelValues(types.Balances.String()).Inc()
		runtime.SetFinalizer(b.balancesMultiValue, balancesFinalizer)
	}
	if b.validatorsMultiValue != nil && b.validatorsMultiValue.IsFragmented() {
		initialMVslice := b.validatorsMultiValue
		b.validatorsMultiValue = b.validatorsMultiValue.Reset(b)
		initialMVslice.Detach(b)
		multiValueCountGauge.WithLabelValues(types.Validators.String()).Inc()
		runtime.SetFinalizer(b.validatorsMultiValue, validatorsFinalizer)
	}
	if b.inactivityScoresMultiValue != nil && b.inactivityScoresMultiValue.IsFragmented() {
		initialMVslice := b.inactivityScoresMultiValue
		b.inactivityScoresMultiValue = b.inactivityScoresMultiValue.Reset(b)
		initialMVslice.Detach(b)
		multiValueCountGauge.WithLabelValues(types.InactivityScores.String()).Inc()
		runtime.SetFinalizer(b.inactivityScoresMultiValue, inactivityScoresFinalizer)
	}
}

func randaoMixesFinalizer(m *MultiValueRandaoMixes) {
	multiValueCountGauge.WithLabelValues(types.RandaoMixes.String()).Dec()
}

func blockRootsFinalizer(m *MultiValueBlockRoots) {
	multiValueCountGauge.WithLabelValues(types.BlockRoots.String()).Dec()
}

func stateRootsFinalizer(m *MultiValueStateRoots) {
	multiValueCountGauge.WithLabelValues(types.StateRoots.String()).Dec()
}

func balancesFinalizer(m *MultiValueBalances) {
	multiValueCountGauge.WithLabelValues(types.Balances.String()).Dec()
}

func validatorsFinalizer(m *MultiValueValidators) {
	multiValueCountGauge.WithLabelValues(types.Validators.String()).Dec()
}

func inactivityScoresFinalizer(m *MultiValueInactivityScores) {
	multiValueCountGauge.WithLabelValues(types.InactivityScores.String()).Dec()
}
