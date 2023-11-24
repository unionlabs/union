package types

import (
	"cosmossdk.io/errors"
)

// x/authz module sentinel errors
var (
	// ErrNoModuleVersionFound error if there is no version found in the module-version map
	ErrNoModuleVersionFound = errors.Register(ModuleName, 2, "module version not found")
	// ErrNoUpgradePlanFound error if there is no scheduled upgrade plan found
	ErrNoUpgradePlanFound = errors.Register(ModuleName, 3, "upgrade plan not found")
	// ErrNoUpgradedClientFound error if there is no upgraded client for the next version
	ErrNoUpgradedClientFound = errors.Register(ModuleName, 4, "upgraded client not found")
	// ErrNoUpgradedConsensusStateFound error if there is no upgraded consensus state for the next version
	ErrNoUpgradedConsensusStateFound = errors.Register(ModuleName, 5, "upgraded consensus state not found")
	// ErrInvalidSigner error if the authority is not the signer for a proposal message
	ErrInvalidSigner = errors.Register(ModuleName, 6, "expected authority account as only signer for proposal message")
)
