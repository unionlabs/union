package server

import (
	"fmt"
	"strings"

	"github.com/spf13/cast"

	pruningtypes "cosmossdk.io/store/pruning/types"

	"github.com/cosmos/cosmos-sdk/server/types"
)

// GetPruningOptionsFromFlags parses command flags and returns the correct
// PruningOptions. If a pruning strategy is provided, that will be parsed and
// returned, otherwise, it is assumed custom pruning options are provided.
func GetPruningOptionsFromFlags(appOpts types.AppOptions) (pruningtypes.PruningOptions, error) {
	strategy := strings.ToLower(cast.ToString(appOpts.Get(FlagPruning)))

	switch strategy {
	case pruningtypes.PruningOptionDefault, pruningtypes.PruningOptionNothing, pruningtypes.PruningOptionEverything:
		return pruningtypes.NewPruningOptionsFromString(strategy), nil

	case pruningtypes.PruningOptionCustom:
		opts := pruningtypes.NewCustomPruningOptions(
			cast.ToUint64(appOpts.Get(FlagPruningKeepRecent)),
			cast.ToUint64(appOpts.Get(FlagPruningInterval)),
		)

		if err := opts.Validate(); err != nil {
			return opts, fmt.Errorf("invalid custom pruning options: %w", err)
		}

		return opts, nil

	default:
		return pruningtypes.PruningOptions{}, fmt.Errorf("unknown pruning strategy %s", strategy)
	}
}
