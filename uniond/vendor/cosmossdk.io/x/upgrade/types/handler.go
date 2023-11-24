package types

import (
	context "context"

	"github.com/cosmos/cosmos-sdk/types/module"
)

// UpgradeHandler specifies the type of function that is called when an upgrade
// is applied.
//
// `fromVM` is a VersionMap of moduleName to fromVersion (unit64), where
// fromVersion denotes the version from which we should migrate the module, the
// target version being the module's latest version in the return VersionMap,
// let's call it `toVM`.
//
// `fromVM` is retrieved from x/upgrade's store, whereas `toVM` is chosen
// arbitrarily by the app developer (and persisted to x/upgrade's store right
// after the upgrade handler runs). In general, `toVM` should map all modules
// to their latest ConsensusVersion so that x/upgrade can track each module's
// latest ConsensusVersion; `fromVM` can be left as-is, but can also be
// modified inside the upgrade handler, e.g. to skip running InitGenesis or
// migrations for certain modules when calling the `module.Manager#RunMigrations`
// function.
//
// Please also refer to docs/core/upgrade.md for more information.
type UpgradeHandler func(ctx context.Context, plan Plan, fromVM module.VersionMap) (module.VersionMap, error)
