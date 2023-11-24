package types

import (
	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// create a new ValidatorHistoricalRewards
func NewValidatorHistoricalRewards(cumulativeRewardRatio sdk.DecCoins, referenceCount uint32) ValidatorHistoricalRewards {
	return ValidatorHistoricalRewards{
		CumulativeRewardRatio: cumulativeRewardRatio,
		ReferenceCount:        referenceCount,
	}
}

// create a new ValidatorCurrentRewards
func NewValidatorCurrentRewards(rewards sdk.DecCoins, period uint64) ValidatorCurrentRewards {
	return ValidatorCurrentRewards{
		Rewards: rewards,
		Period:  period,
	}
}

// return the initial accumulated commission (zero)
func InitialValidatorAccumulatedCommission() ValidatorAccumulatedCommission {
	return ValidatorAccumulatedCommission{}
}

// create a new ValidatorSlashEvent
func NewValidatorSlashEvent(validatorPeriod uint64, fraction sdkmath.LegacyDec) ValidatorSlashEvent {
	return ValidatorSlashEvent{
		ValidatorPeriod: validatorPeriod,
		Fraction:        fraction,
	}
}
