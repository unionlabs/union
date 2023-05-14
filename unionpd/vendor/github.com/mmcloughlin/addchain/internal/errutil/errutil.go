// Package errutil implements common error types and helper functions.
package errutil

import (
	"fmt"
	"io"
)

// AssertionFailure is used for an error resulting from the failure of an
// expected invariant.
func AssertionFailure(format string, args ...interface{}) error {
	return fmt.Errorf("assertion failure: "+format, args...)
}

// UnexpectedType builds an error for an unexpected type, typically in a type switch.
func UnexpectedType(t interface{}) error {
	return AssertionFailure("unexpected type %T", t)
}

// CheckClose closes c. If an error occurs it will be written to the error
// pointer errp, if it doesn't already reference an error. This is intended to
// allow you to properly check errors when defering a close call. In this case
// the error pointer should be the address of a named error return.
func CheckClose(errp *error, c io.Closer) {
	if err := c.Close(); err != nil && *errp == nil {
		*errp = err
	}
}
