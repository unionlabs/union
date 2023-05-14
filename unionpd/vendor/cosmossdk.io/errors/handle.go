package errors

import "fmt"

// AssertNil panics on error
// Should be only used with interface methods, which require return error, but the
// error is always nil
func AssertNil(err error) {
	if err != nil {
		panic(fmt.Errorf("logic error - this should never happen. %w", err))
	}
}
