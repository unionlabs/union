package types

import "cosmossdk.io/errors"

// x/slashing module sentinel errors
var (
	ErrNoValidatorForAddress        = errors.Register(ModuleName, 2, "address is not associated with any known validator")
	ErrBadValidatorAddr             = errors.Register(ModuleName, 3, "validator does not exist for that address")
	ErrValidatorJailed              = errors.Register(ModuleName, 4, "validator still jailed; cannot be unjailed")
	ErrValidatorNotJailed           = errors.Register(ModuleName, 5, "validator not jailed; cannot be unjailed")
	ErrMissingSelfDelegation        = errors.Register(ModuleName, 6, "validator has no self-delegation; cannot be unjailed")
	ErrSelfDelegationTooLowToUnjail = errors.Register(ModuleName, 7, "validator's self delegation less than minimum; cannot be unjailed")
	ErrNoSigningInfoFound           = errors.Register(ModuleName, 8, "no validator signing info found")
	ErrValidatorTombstoned          = errors.Register(ModuleName, 9, "validator already tombstoned")
)
