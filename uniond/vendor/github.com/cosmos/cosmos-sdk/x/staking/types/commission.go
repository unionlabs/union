package types

import (
	"time"

	"cosmossdk.io/math"
)

// NewCommissionRates returns an initialized validator commission rates.
func NewCommissionRates(rate, maxRate, maxChangeRate math.LegacyDec) CommissionRates {
	return CommissionRates{
		Rate:          rate,
		MaxRate:       maxRate,
		MaxChangeRate: maxChangeRate,
	}
}

// NewCommission returns an initialized validator commission.
func NewCommission(rate, maxRate, maxChangeRate math.LegacyDec) Commission {
	return Commission{
		CommissionRates: NewCommissionRates(rate, maxRate, maxChangeRate),
		UpdateTime:      time.Unix(0, 0).UTC(),
	}
}

// NewCommissionWithTime returns an initialized validator commission with a specified
// update time which should be the current block BFT time.
func NewCommissionWithTime(rate, maxRate, maxChangeRate math.LegacyDec, updatedAt time.Time) Commission {
	return Commission{
		CommissionRates: NewCommissionRates(rate, maxRate, maxChangeRate),
		UpdateTime:      updatedAt,
	}
}

// Validate performs basic sanity validation checks of initial commission
// parameters. If validation fails, an SDK error is returned.
func (cr CommissionRates) Validate() error {
	switch {
	case cr.MaxRate.IsNegative():
		// max rate cannot be negative
		return ErrCommissionNegative

	case cr.MaxRate.GT(math.LegacyOneDec()):
		// max rate cannot be greater than 1
		return ErrCommissionHuge

	case cr.Rate.IsNegative():
		// rate cannot be negative
		return ErrCommissionNegative

	case cr.Rate.GT(cr.MaxRate):
		// rate cannot be greater than the max rate
		return ErrCommissionGTMaxRate

	case cr.MaxChangeRate.IsNegative():
		// change rate cannot be negative
		return ErrCommissionChangeRateNegative

	case cr.MaxChangeRate.GT(cr.MaxRate):
		// change rate cannot be greater than the max rate
		return ErrCommissionChangeRateGTMaxRate
	}

	return nil
}

// ValidateNewRate performs basic sanity validation checks of a new commission
// rate. If validation fails, an SDK error is returned.
func (c Commission) ValidateNewRate(newRate math.LegacyDec, blockTime time.Time) error {
	switch {
	case blockTime.Sub(c.UpdateTime).Hours() < 24:
		// new rate cannot be changed more than once within 24 hours
		return ErrCommissionUpdateTime

	case newRate.IsNegative():
		// new rate cannot be negative
		return ErrCommissionNegative

	case newRate.GT(c.MaxRate):
		// new rate cannot be greater than the max rate
		return ErrCommissionGTMaxRate

	case newRate.Sub(c.Rate).GT(c.MaxChangeRate):
		// new rate % points change cannot be greater than the max change rate
		return ErrCommissionGTMaxChangeRate
	}

	return nil
}
