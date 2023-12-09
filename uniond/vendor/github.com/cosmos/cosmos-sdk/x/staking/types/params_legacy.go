package types

import paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"

var (
	KeyUnbondingTime     = []byte("UnbondingTime")
	KeyMaxValidators     = []byte("MaxValidators")
	KeyMaxEntries        = []byte("MaxEntries")
	KeyBondDenom         = []byte("BondDenom")
	KeyHistoricalEntries = []byte("HistoricalEntries")
	KeyMinCommissionRate = []byte("MinCommissionRate")
)

var _ paramtypes.ParamSet = (*Params)(nil)

// Deprecated: now params can be accessed on key `0x51` on the staking store.
// ParamTable for staking module
func ParamKeyTable() paramtypes.KeyTable {
	return paramtypes.NewKeyTable().RegisterParamSet(&Params{})
}

// Deprecated: Implements params.ParamSet
func (p *Params) ParamSetPairs() paramtypes.ParamSetPairs {
	return paramtypes.ParamSetPairs{
		paramtypes.NewParamSetPair(KeyUnbondingTime, &p.UnbondingTime, validateUnbondingTime),
		paramtypes.NewParamSetPair(KeyMaxValidators, &p.MaxValidators, validateMaxValidators),
		paramtypes.NewParamSetPair(KeyMaxEntries, &p.MaxEntries, validateMaxEntries),
		paramtypes.NewParamSetPair(KeyHistoricalEntries, &p.HistoricalEntries, validateHistoricalEntries),
		paramtypes.NewParamSetPair(KeyBondDenom, &p.BondDenom, validateBondDenom),
		paramtypes.NewParamSetPair(KeyMinCommissionRate, &p.MinCommissionRate, validateMinCommissionRate),
	}
}
