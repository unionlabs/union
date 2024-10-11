// Copyright 2015 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package rpc

import "fmt"

// HTTPError is returned by client operations when the HTTP status code of the
// response is not a 2xx status.
type HTTPError struct {
	StatusCode int
	Status     string
	Body       []byte
}

func (err HTTPError) Error() string {
	if len(err.Body) == 0 {
		return err.Status
	}
	return fmt.Sprintf("%v: %s", err.Status, err.Body)
}

// Error wraps RPC errors, which contain an error code in addition to the message.
type Error interface {
	Error() string  // returns the message
	ErrorCode() int // returns the code
}

// A DataError contains some data in addition to the error message.
type DataError interface {
	Error() string          // returns the message
	ErrorData() interface{} // returns the error data
}

// Error types defined below are the built-in JSON-RPC errors.

var (
	_ Error = new(methodNotFoundError)
	_ Error = new(subscriptionNotFoundError)
	_ Error = new(parseError)
	_ Error = new(invalidRequestError)
	_ Error = new(invalidMessageError)
	_ Error = new(invalidParamsError)
	_ Error = new(internalServerError)
)

const (
	errcodeDefault          = -32000
	errcodeTimeout          = -32002
	errcodeResponseTooLarge = -32003
	errcodePanic            = -32603
	errcodeMarshalError     = -32603

	legacyErrcodeNotificationsUnsupported = -32001
)

const (
	errMsgTimeout          = "request timed out"
	errMsgResponseTooLarge = "response too large"
	errMsgBatchTooLarge    = "batch too large"
)

type methodNotFoundError struct{ method string }

func (e *methodNotFoundError) ErrorCode() int { return -32601 }

func (e *methodNotFoundError) Error() string {
	return fmt.Sprintf("the method %s does not exist/is not available", e.method)
}

type notificationsUnsupportedError struct{}

func (e notificationsUnsupportedError) Error() string {
	return "notifications not supported"
}

func (e notificationsUnsupportedError) ErrorCode() int { return -32601 }

// Is checks for equivalence to another error. Here we define that all errors with code
// -32601 (method not found) are equivalent to notificationsUnsupportedError. This is
// done to enable the following pattern:
//
//	sub, err := client.Subscribe(...)
//	if errors.Is(err, rpc.ErrNotificationsUnsupported) {
//		// server doesn't support subscriptions
//	}
func (e notificationsUnsupportedError) Is(other error) bool {
	if other == (notificationsUnsupportedError{}) {
		return true
	}
	rpcErr, ok := other.(Error)
	if ok {
		code := rpcErr.ErrorCode()
		return code == -32601 || code == legacyErrcodeNotificationsUnsupported
	}
	return false
}

type subscriptionNotFoundError struct{ namespace, subscription string }

func (e *subscriptionNotFoundError) ErrorCode() int { return -32601 }

func (e *subscriptionNotFoundError) Error() string {
	return fmt.Sprintf("no %q subscription in %s namespace", e.subscription, e.namespace)
}

// Invalid JSON was received by the server.
type parseError struct{ message string }

func (e *parseError) ErrorCode() int { return -32700 }

func (e *parseError) Error() string { return e.message }

// received message isn't a valid request
type invalidRequestError struct{ message string }

func (e *invalidRequestError) ErrorCode() int { return -32600 }

func (e *invalidRequestError) Error() string { return e.message }

// received message is invalid
type invalidMessageError struct{ message string }

func (e *invalidMessageError) ErrorCode() int { return -32700 }

func (e *invalidMessageError) Error() string { return e.message }

// unable to decode supplied params, or an invalid number of parameters
type invalidParamsError struct{ message string }

func (e *invalidParamsError) ErrorCode() int { return -32602 }

func (e *invalidParamsError) Error() string { return e.message }

// internalServerError is used for server errors during request processing.
type internalServerError struct {
	code    int
	message string
}

func (e *internalServerError) ErrorCode() int { return e.code }

func (e *internalServerError) Error() string { return e.message }
