package tendermint

import (
	errorsmod "cosmossdk.io/errors"
)

// IBC tendermint client sentinel errors
var (
	ErrInvalidChainID          = errorsmod.Register(ModuleName, 2, "invalid chain-id")
	ErrInvalidTrustingPeriod   = errorsmod.Register(ModuleName, 3, "invalid trusting period")
	ErrInvalidUnbondingPeriod  = errorsmod.Register(ModuleName, 4, "invalid unbonding period")
	ErrInvalidHeaderHeight     = errorsmod.Register(ModuleName, 5, "invalid header height")
	ErrInvalidHeader           = errorsmod.Register(ModuleName, 6, "invalid header")
	ErrInvalidMaxClockDrift    = errorsmod.Register(ModuleName, 7, "invalid max clock drift")
	ErrProcessedTimeNotFound   = errorsmod.Register(ModuleName, 8, "processed time not found")
	ErrProcessedHeightNotFound = errorsmod.Register(ModuleName, 9, "processed height not found")
	ErrDelayPeriodNotPassed    = errorsmod.Register(ModuleName, 10, "packet-specified delay period has not been reached")
	ErrTrustingPeriodExpired   = errorsmod.Register(ModuleName, 11, "time since latest trusted state has passed the trusting period")
	ErrUnbondingPeriodExpired  = errorsmod.Register(ModuleName, 12, "time since latest trusted state has passed the unbonding period")
	ErrInvalidProofSpecs       = errorsmod.Register(ModuleName, 13, "invalid proof specs")
	ErrInvalidValidatorSet     = errorsmod.Register(ModuleName, 14, "invalid validator set")
)
