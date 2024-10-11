package types

import sdkmath "cosmossdk.io/math"

// create a new DelegatorStartingInfo
func NewDelegatorStartingInfo(previousPeriod uint64, stake sdkmath.LegacyDec, height uint64) DelegatorStartingInfo {
	return DelegatorStartingInfo{
		PreviousPeriod: previousPeriod,
		Stake:          stake,
		Height:         height,
	}
}
