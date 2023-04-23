package types

import (
	"fmt"
	"time"

	paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"
)

// DefaultTimePerBlock is the default value for maximum expected time per block (in nanoseconds).
const DefaultTimePerBlock = 30 * time.Second

// KeyMaxExpectedTimePerBlock is store's key for MaxExpectedTimePerBlock parameter
var KeyMaxExpectedTimePerBlock = []byte("MaxExpectedTimePerBlock")

// ParamKeyTable type declaration for parameters
func ParamKeyTable() paramtypes.KeyTable {
	return paramtypes.NewKeyTable().RegisterParamSet(&Params{})
}

// NewParams creates a new parameter configuration for the ibc connection module
func NewParams(timePerBlock uint64) Params {
	return Params{
		MaxExpectedTimePerBlock: timePerBlock,
	}
}

// DefaultParams is the default parameter configuration for the ibc connection module
func DefaultParams() Params {
	return NewParams(uint64(DefaultTimePerBlock))
}

// Validate ensures MaxExpectedTimePerBlock is non-zero
func (p Params) Validate() error {
	if p.MaxExpectedTimePerBlock == 0 {
		return fmt.Errorf("MaxExpectedTimePerBlock cannot be zero")
	}
	return nil
}

// ParamSetPairs implements params.ParamSet
func (p *Params) ParamSetPairs() paramtypes.ParamSetPairs {
	return paramtypes.ParamSetPairs{
		paramtypes.NewParamSetPair(KeyMaxExpectedTimePerBlock, p.MaxExpectedTimePerBlock, validateParams),
	}
}

func validateParams(i interface{}) error {
	_, ok := i.(uint64)
	if !ok {
		return fmt.Errorf("invalid parameter. expected %T, got type: %T", uint64(1), i)
	}
	return nil
}
