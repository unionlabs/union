package types

import (
	"fmt"
	"time"
)

// DefaultTimePerBlock is the default value for maximum expected time per block (in nanoseconds).
const DefaultTimePerBlock = 30 * time.Second

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
