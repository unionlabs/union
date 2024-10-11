package types

import (
	"fmt"
)

func ValidateEpochIdentifierInterface(i interface{}) error {
	v, ok := i.(string)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}
	if err := ValidateEpochIdentifierString(v); err != nil {
		return err
	}

	return nil
}

func ValidateEpochIdentifierString(s string) error {
	if s == "" {
		return fmt.Errorf("empty distribution epoch identifier: %+v", s)
	}
	return nil
}
