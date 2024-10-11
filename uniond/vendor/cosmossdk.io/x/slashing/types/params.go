package types

import (
	"fmt"
	"time"

	"cosmossdk.io/math"
)

// Default parameter namespace
const (
	DefaultSignedBlocksWindow   = int64(100)
	DefaultDowntimeJailDuration = 60 * 10 * time.Second
)

var (
	DefaultMinSignedPerWindow      = math.LegacyNewDecWithPrec(5, 1)
	DefaultSlashFractionDoubleSign = math.LegacyNewDec(1).Quo(math.LegacyNewDec(20))
	DefaultSlashFractionDowntime   = math.LegacyNewDec(1).Quo(math.LegacyNewDec(100))
)

// NewParams creates a new Params object
func NewParams(
	signedBlocksWindow int64, minSignedPerWindow math.LegacyDec, downtimeJailDuration time.Duration,
	slashFractionDoubleSign, slashFractionDowntime math.LegacyDec,
) Params {
	return Params{
		SignedBlocksWindow:      signedBlocksWindow,
		MinSignedPerWindow:      minSignedPerWindow,
		DowntimeJailDuration:    downtimeJailDuration,
		SlashFractionDoubleSign: slashFractionDoubleSign,
		SlashFractionDowntime:   slashFractionDowntime,
	}
}

// DefaultParams defines the parameters for this module
func DefaultParams() Params {
	return NewParams(
		DefaultSignedBlocksWindow,
		DefaultMinSignedPerWindow,
		DefaultDowntimeJailDuration,
		DefaultSlashFractionDoubleSign,
		DefaultSlashFractionDowntime,
	)
}

// Validate validates the params
func (p Params) Validate() error {
	if err := validateSignedBlocksWindow(p.SignedBlocksWindow); err != nil {
		return err
	}
	if err := validateMinSignedPerWindow(p.MinSignedPerWindow); err != nil {
		return err
	}
	if err := validateDowntimeJailDuration(p.DowntimeJailDuration); err != nil {
		return err
	}
	if err := validateSlashFractionDoubleSign(p.SlashFractionDoubleSign); err != nil {
		return err
	}
	if err := validateSlashFractionDowntime(p.SlashFractionDowntime); err != nil {
		return err
	}
	return nil
}

func validateSignedBlocksWindow(i interface{}) error {
	v, ok := i.(int64)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	if v <= 0 {
		return fmt.Errorf("signed blocks window must be positive: %d", v)
	}

	return nil
}

func validateMinSignedPerWindow(i interface{}) error {
	v, ok := i.(math.LegacyDec)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	if v.IsNil() {
		return fmt.Errorf("min signed per window cannot be nil: %s", v)
	}
	if v.IsNegative() {
		return fmt.Errorf("min signed per window cannot be negative: %s", v)
	}
	if v.GT(math.LegacyOneDec()) {
		return fmt.Errorf("min signed per window too large: %s", v)
	}

	return nil
}

func validateDowntimeJailDuration(i interface{}) error {
	v, ok := i.(time.Duration)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	if v <= 0 {
		return fmt.Errorf("downtime jail duration must be positive: %s", v)
	}

	return nil
}

func validateSlashFractionDoubleSign(i interface{}) error {
	v, ok := i.(math.LegacyDec)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	if v.IsNil() {
		return fmt.Errorf("double sign slash fraction cannot be nil: %s", v)
	}
	if v.IsNegative() {
		return fmt.Errorf("double sign slash fraction cannot be negative: %s", v)
	}
	if v.GT(math.LegacyOneDec()) {
		return fmt.Errorf("double sign slash fraction too large: %s", v)
	}

	return nil
}

func validateSlashFractionDowntime(i interface{}) error {
	v, ok := i.(math.LegacyDec)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	if v.IsNil() {
		return fmt.Errorf("downtime slash fraction cannot be nil: %s", v)
	}
	if v.IsNegative() {
		return fmt.Errorf("downtime slash fraction cannot be negative: %s", v)
	}
	if v.GT(math.LegacyOneDec()) {
		return fmt.Errorf("downtime slash fraction too large: %s", v)
	}

	return nil
}

// MinSignedPerWindowInt returns min signed per window as an integer (vs the decimal in the param)
func (p *Params) MinSignedPerWindowInt() int64 {
	signedBlocksWindow := p.SignedBlocksWindow
	minSignedPerWindow := p.MinSignedPerWindow

	// NOTE: RoundInt64 will never panic as minSignedPerWindow is
	//       less than 1.
	return minSignedPerWindow.MulInt64(signedBlocksWindow).RoundInt64()
}
