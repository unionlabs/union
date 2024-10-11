package validator

import "fmt"

// ErrNoFlag takes a flag name and returns a formatted error representing no flag was provided.
func errNoFlag(flagName string) error {
	return fmt.Errorf("no --%s flag value was provided", flagName)
}
