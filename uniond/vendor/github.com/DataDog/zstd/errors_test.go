package zstd

import (
	"testing"
)

const (
	// ErrorUpperBound is the upper bound to error number, currently only used in test
	// If this needs to be updated, check in zstd_errors.h what the max is
	ErrorUpperBound = 1000
)

// TestFindIsDstSizeTooSmallError tests that there is at least one error code that
// corresponds to dst size too small
func TestFindIsDstSizeTooSmallError(t *testing.T) {
	found := 0
	for i := -1; i > -ErrorUpperBound; i-- {
		e := ErrorCode(i)
		if IsDstSizeTooSmallError(e) {
			found++
		}
	}

	if found == 0 {
		t.Fatal("Couldn't find an error code for DstSizeTooSmall error, please make sure we didn't change the error string")
	} else if found > 1 {
		t.Fatal("IsDstSizeTooSmallError found multiple error codes matching, this shouldn't be the case")
	}
}
