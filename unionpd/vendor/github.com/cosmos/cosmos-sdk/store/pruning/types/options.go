package types

import (
	"errors"
	"fmt"
)

// PruningOptions defines the pruning strategy used when determining which
// heights are removed from disk when committing state.
type PruningOptions struct {
	// KeepRecent defines how many recent heights to keep on disk.
	KeepRecent uint64

	// Interval defines when the pruned heights are removed from disk.
	Interval uint64

	// Strategy defines the kind of pruning strategy. See below for more information on each.
	Strategy PruningStrategy
}

type PruningStrategy int

// Pruning option string constants
const (
	PruningOptionDefault    = "default"
	PruningOptionEverything = "everything"
	PruningOptionNothing    = "nothing"
	PruningOptionCustom     = "custom"
)

const (
	// PruningDefault defines a pruning strategy where the last 362880 heights are
	// kept where to-be pruned heights are pruned at every 10th height.
	// The last 362880 heights are kept(approximately 3.5 weeks worth of state) assuming the typical
	// block time is 6s. If these values do not match the applications' requirements, use the "custom" option.
	PruningDefault PruningStrategy = iota
	// PruningEverything defines a pruning strategy where all committed heights are
	// deleted, storing only the current height and last 2 states. To-be pruned heights are
	// pruned at every 10th height.
	PruningEverything
	// PruningNothing defines a pruning strategy where all heights are kept on disk.
	// This is the only stretegy where KeepEvery=1 is allowed with state-sync snapshots disabled.
	PruningNothing
	// PruningCustom defines a pruning strategy where the user specifies the pruning.
	PruningCustom
	// PruningUndefined defines an undefined pruning strategy. It is to be returned by stores that do not support pruning.
	PruningUndefined
)

const (
	pruneEverythingKeepRecent = 2
	pruneEverythingInterval   = 10
)

var (
	ErrPruningIntervalZero       = errors.New("'pruning-interval' must not be 0. If you want to disable pruning, select pruning = \"nothing\"")
	ErrPruningIntervalTooSmall   = fmt.Errorf("'pruning-interval' must not be less than %d. For the most aggressive pruning, select pruning = \"everything\"", pruneEverythingInterval)
	ErrPruningKeepRecentTooSmall = fmt.Errorf("'pruning-keep-recent' must not be less than %d. For the most aggressive pruning, select pruning = \"everything\"", pruneEverythingKeepRecent)
)

func NewPruningOptions(pruningStrategy PruningStrategy) PruningOptions {
	switch pruningStrategy {
	case PruningDefault:
		return PruningOptions{
			KeepRecent: 362880,
			Interval:   10,
			Strategy:   PruningDefault,
		}
	case PruningEverything:
		return PruningOptions{
			KeepRecent: pruneEverythingKeepRecent,
			Interval:   pruneEverythingInterval,
			Strategy:   PruningEverything,
		}
	case PruningNothing:
		return PruningOptions{
			KeepRecent: 0,
			Interval:   0,
			Strategy:   PruningNothing,
		}
	default:
		return PruningOptions{
			Strategy: PruningCustom,
		}
	}
}

func NewCustomPruningOptions(keepRecent, interval uint64) PruningOptions {
	return PruningOptions{
		KeepRecent: keepRecent,
		Interval:   interval,
		Strategy:   PruningCustom,
	}
}

func (po PruningOptions) GetPruningStrategy() PruningStrategy {
	return po.Strategy
}

func (po PruningOptions) Validate() error {
	if po.Strategy == PruningNothing {
		return nil
	}
	if po.Interval == 0 {
		return ErrPruningIntervalZero
	}
	if po.Interval < pruneEverythingInterval {
		return ErrPruningIntervalTooSmall
	}
	if po.KeepRecent < pruneEverythingKeepRecent {
		return ErrPruningKeepRecentTooSmall
	}
	return nil
}

func NewPruningOptionsFromString(strategy string) PruningOptions {
	switch strategy {
	case PruningOptionEverything:
		return NewPruningOptions(PruningEverything)

	case PruningOptionNothing:
		return NewPruningOptions(PruningNothing)

	case PruningOptionDefault:
		return NewPruningOptions(PruningDefault)

	default:
		return NewPruningOptions(PruningDefault)
	}
}
