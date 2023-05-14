package errors

import (
	"fmt"
	"reflect"
)

const (
	// SuccessABCICode declares an ABCI response use 0 to signal that the
	// processing was successful and no error is returned.
	SuccessABCICode uint32 = 0

	// All unclassified errors that do not provide an ABCI code are clubbed
	// under an internal error code and a generic message instead of
	// detailed error string.
	internalABCICodespace        = UndefinedCodespace
	internalABCICode      uint32 = 1
)

// ABCIInfo returns the ABCI error information as consumed by the tendermint
// client. Returned codespace, code, and log message should be used as a ABCI response.
// Any error that does not provide ABCICode information is categorized as error
// with code 1, codespace UndefinedCodespace
// When not running in a debug mode all messages of errors that do not provide
// ABCICode information are replaced with generic "internal error". Errors
// without an ABCICode information as considered internal.
func ABCIInfo(err error, debug bool) (codespace string, code uint32, log string) {
	if errIsNil(err) {
		return "", SuccessABCICode, ""
	}

	encode := defaultErrEncoder
	if debug {
		encode = debugErrEncoder
	}

	return abciCodespace(err), abciCode(err), encode(err)
}

// The debugErrEncoder encodes the error with a stacktrace.
func debugErrEncoder(err error) string {
	return fmt.Sprintf("%+v", err)
}

func defaultErrEncoder(err error) string {
	return err.Error()
}

type coder interface {
	ABCICode() uint32
}

// abciCode tests if given error contains an ABCI code and returns the value of
// it if available. This function is testing for the causer interface as well
// and unwraps the error.
func abciCode(err error) uint32 {
	if errIsNil(err) {
		return SuccessABCICode
	}

	for {
		if c, ok := err.(coder); ok {
			return c.ABCICode()
		}

		if c, ok := err.(causer); ok {
			err = c.Cause()
		} else {
			return internalABCICode
		}
	}
}

type codespacer interface {
	Codespace() string
}

// abciCodespace tests if given error contains a codespace and returns the value of
// it if available. This function is testing for the causer interface as well
// and unwraps the error.
func abciCodespace(err error) string {
	if errIsNil(err) {
		return ""
	}

	for {
		if c, ok := err.(codespacer); ok {
			return c.Codespace()
		}

		if c, ok := err.(causer); ok {
			err = c.Cause()
		} else {
			return internalABCICodespace
		}
	}
}

// errIsNil returns true if value represented by the given error is nil.
//
// Most of the time a simple == check is enough. There is a very narrowed
// spectrum of cases (mostly in tests) where a more sophisticated check is
// required.
func errIsNil(err error) bool {
	if err == nil {
		return true
	}
	if val := reflect.ValueOf(err); val.Kind() == reflect.Ptr {
		return val.IsNil()
	}
	return false
}
