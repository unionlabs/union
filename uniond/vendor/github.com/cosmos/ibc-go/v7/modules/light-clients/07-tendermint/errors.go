package tendermint

import (
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// IBC tendermint client sentinel errors
var (
	ErrInvalidChainID          = sdkerrors.Register(ModuleName, 2, "invalid chain-id")
	ErrInvalidTrustingPeriod   = sdkerrors.Register(ModuleName, 3, "invalid trusting period")
	ErrInvalidUnbondingPeriod  = sdkerrors.Register(ModuleName, 4, "invalid unbonding period")
	ErrInvalidHeaderHeight     = sdkerrors.Register(ModuleName, 5, "invalid header height")
	ErrInvalidHeader           = sdkerrors.Register(ModuleName, 6, "invalid header")
	ErrInvalidMaxClockDrift    = sdkerrors.Register(ModuleName, 7, "invalid max clock drift")
	ErrProcessedTimeNotFound   = sdkerrors.Register(ModuleName, 8, "processed time not found")
	ErrProcessedHeightNotFound = sdkerrors.Register(ModuleName, 9, "processed height not found")
	ErrDelayPeriodNotPassed    = sdkerrors.Register(ModuleName, 10, "packet-specified delay period has not been reached")
	ErrTrustingPeriodExpired   = sdkerrors.Register(ModuleName, 11, "time since latest trusted state has passed the trusting period")
	ErrUnbondingPeriodExpired  = sdkerrors.Register(ModuleName, 12, "time since latest trusted state has passed the unbonding period")
	ErrInvalidProofSpecs       = sdkerrors.Register(ModuleName, 13, "invalid proof specs")
	ErrInvalidValidatorSet     = sdkerrors.Register(ModuleName, 14, "invalid validator set")
)
