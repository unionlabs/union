package features

import "github.com/urfave/cli/v2"

// Deprecated flags list.
const deprecatedUsage = "DEPRECATED. DO NOT USE."

var (
	// To deprecate a feature flag, first copy the example below, then insert deprecated flag in `deprecatedFlags`.
	exampleDeprecatedFeatureFlag = &cli.StringFlag{
		Name:   "name",
		Usage:  deprecatedUsage,
		Hidden: true,
	}
	deprecatedDisableVecHTR = &cli.BoolFlag{
		Name:   "disable-vectorized-htr",
		Usage:  deprecatedUsage,
		Hidden: true,
	}
	deprecatedEnableReorgLateBlocks = &cli.BoolFlag{
		Name:   "enable-reorg-late-blocks",
		Usage:  deprecatedUsage,
		Hidden: true,
	}
	deprecatedDisableGossipBatchAggregation = &cli.BoolFlag{
		Name:   "disable-gossip-batch-aggregation",
		Usage:  deprecatedUsage,
		Hidden: true,
	}
	deprecatedBuildBlockParallel = &cli.BoolFlag{
		Name:   "build-block-parallel",
		Usage:  deprecatedUsage,
		Hidden: true,
	}

	deprecatedEnableRegistrationCache = &cli.BoolFlag{
		Name:   "enable-registration-cache",
		Usage:  deprecatedUsage,
		Hidden: true,
	}
	deprecatedAggregateParallel = &cli.BoolFlag{
		Name:   "aggregate-parallel",
		Usage:  deprecatedUsage,
		Hidden: true,
	}
	deprecatedEnableOptionalEngineMethods = &cli.BoolFlag{
		Name:   "enable-optional-engine-methods",
		Usage:  deprecatedUsage,
		Hidden: true,
	}
	deprecatedDisableBuildBlockParallel = &cli.BoolFlag{
		Name:   "disable-build-block-parallel",
		Usage:  deprecatedUsage,
		Hidden: true,
	}
	deprecatedDisableReorgLateBlocks = &cli.BoolFlag{
		Name:   "disable-reorg-late-blocks",
		Usage:  deprecatedUsage,
		Hidden: true,
	}
	deprecatedDisableOptionalEngineMethods = &cli.BoolFlag{
		Name:   "disable-optional-engine-methods",
		Usage:  deprecatedUsage,
		Hidden: true,
	}
	deprecatedDisableAggregateParallel = &cli.BoolFlag{
		Name:   "disable-aggregate-parallel",
		Usage:  deprecatedUsage,
		Hidden: true,
	}
)

// Deprecated flags for both the beacon node and validator client.
var deprecatedFlags = []cli.Flag{
	exampleDeprecatedFeatureFlag,
	deprecatedDisableVecHTR,
	deprecatedEnableReorgLateBlocks,
	deprecatedDisableGossipBatchAggregation,
	deprecatedBuildBlockParallel,
	deprecatedEnableRegistrationCache,
	deprecatedAggregateParallel,
	deprecatedEnableOptionalEngineMethods,
	deprecatedDisableBuildBlockParallel,
	deprecatedDisableReorgLateBlocks,
	deprecatedDisableOptionalEngineMethods,
	deprecatedDisableAggregateParallel,
}

// deprecatedBeaconFlags contains flags that are still used by other components
// and therefore cannot be added to deprecatedFlags
var deprecatedBeaconFlags []cli.Flag
