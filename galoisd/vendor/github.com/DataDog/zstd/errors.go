package zstd

/*
#include "zstd.h"
*/
import "C"

// ErrorCode is an error returned by the zstd library.
type ErrorCode int

// Error returns the error string given by zstd
func (e ErrorCode) Error() string {
	return C.GoString(C.ZSTD_getErrorName(C.size_t(e)))
}

func cIsError(code int) bool {
	return int(C.ZSTD_isError(C.size_t(code))) != 0
}

// getError returns an error for the return code, or nil if it's not an error
func getError(code int) error {
	if code < 0 && cIsError(code) {
		return ErrorCode(code)
	}
	return nil
}

// IsDstSizeTooSmallError returns whether the error correspond to zstd standard sDstSizeTooSmall error
func IsDstSizeTooSmallError(e error) bool {
	if e != nil && e.Error() == "Destination buffer is too small" {
		return true
	}
	return false
}
