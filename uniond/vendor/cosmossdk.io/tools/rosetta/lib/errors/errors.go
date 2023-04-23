package errors

// errors.go contains all the errors returned by the adapter implementation
// plus some extra utilities to parse those errors

import (
	"fmt"

	grpccodes "google.golang.org/grpc/codes"
	grpcstatus "google.golang.org/grpc/status"

	"github.com/coinbase/rosetta-sdk-go/types"
)

// ListErrors lists all the registered errors
func ListErrors() []*types.Error {
	return registry.list()
}

// SealAndListErrors seals the registry and lists its errors
func SealAndListErrors() []*types.Error {
	registry.seal()
	return registry.list()
}

// Error defines an error that can be converted to a Rosetta API error.
type Error struct {
	rosErr *types.Error
}

func (e *Error) Error() string {
	if e.rosErr == nil {
		return ErrUnknown.Error()
	}
	return fmt.Sprintf("rosetta: (%d) %s", e.rosErr.Code, e.rosErr.Message)
}

// Is implements errors.Is for *Error, two errors are considered equal
// if their error codes are identical
func (e *Error) Is(err error) bool {
	// assert it can be casted
	rosErr, ok := err.(*Error)
	if rosErr == nil || !ok {
		return false
	}
	// check that both *Error's are correctly initialized to avoid dereference panics
	if rosErr.rosErr == nil || e.rosErr == nil {
		return false
	}
	// messages are equal if their error codes match
	return rosErr.rosErr.Code == e.rosErr.Code
}

// WrapError wraps the rosetta error with additional context
func WrapError(err *Error, msg string) *Error {
	return &Error{rosErr: &types.Error{
		Code:        err.rosErr.Code,
		Message:     err.rosErr.Message,
		Description: err.rosErr.Description,
		Retriable:   err.rosErr.Retriable,
		Details: map[string]interface{}{
			"info": msg,
		},
	}}
}

// ToRosetta attempts to converting an error into a rosetta
// error, if the error cannot be converted it will be parsed as unknown
func ToRosetta(err error) *types.Error {
	// if it's null or not known
	rosErr, ok := err.(*Error)
	if rosErr == nil || !ok {
		return ToRosetta(WrapError(ErrUnknown, ErrUnknown.Error()))
	}
	return rosErr.rosErr
}

// FromGRPCToRosettaError converts a gRPC error to rosetta error
func FromGRPCToRosettaError(err error) *Error {
	status, ok := grpcstatus.FromError(err)
	if !ok {
		return WrapError(ErrUnknown, err.Error())
	}
	switch status.Code() {
	case grpccodes.NotFound:
		return WrapError(ErrNotFound, status.Message())
	case grpccodes.FailedPrecondition:
		return WrapError(ErrBadArgument, status.Message())
	case grpccodes.InvalidArgument:
		return WrapError(ErrBadArgument, status.Message())
	case grpccodes.Internal:
		return WrapError(ErrInternal, status.Message())
	default:
		return WrapError(ErrUnknown, status.Message())
	}
}

func RegisterError(code int32, message string, retryable bool, description string) *Error {
	e := &Error{rosErr: &types.Error{
		Code:        code,
		Message:     message,
		Description: &description,
		Retriable:   retryable,
		Details:     nil,
	}}
	registry.add(e)
	return e
}

// Default error list
var (
	// ErrUnknown defines an unknown error, if this is returned it means
	// the library is ignoring an error
	ErrUnknown = RegisterError(0, "unknown", false, "unknown error")
	// ErrOffline is returned when there is an attempt to query an endpoint in offline mode
	ErrOffline = RegisterError(1, "cannot query endpoint in offline mode", false, "returned when querying an online endpoint in offline mode")
	// ErrNetworkNotSupported is returned when there is an attempt to query a network which is not supported
	ErrNetworkNotSupported = RegisterError(2, "network is not supported", false, "returned when querying a non supported network")
	// ErrCodec is returned when there's an error while marshalling or unmarshalling data
	ErrCodec = RegisterError(3, "encode/decode error", true, "returned when there are errors encoding or decoding information to and from the node")
	// ErrInvalidOperation is returned when the operation supplied to rosetta is not a valid one
	ErrInvalidOperation = RegisterError(4, "invalid operation", false, "returned when the operation is not valid")
	// ErrInvalidTransaction is returned when the provided hex bytes of a TX are not valid
	ErrInvalidTransaction = RegisterError(5, "invalid transaction", false, "returned when the transaction is invalid")
	// ErrInvalidAddress is returned when the byte of the address are bad
	ErrInvalidAddress = RegisterError(7, "invalid address", false, "returned when the address is malformed")
	// ErrInvalidPubkey is returned when the public key is invalid
	ErrInvalidPubkey = RegisterError(8, "invalid pubkey", false, "returned when the public key is invalid")
	// ErrInterpreting is returned when there are errors interpreting the data from the node, most likely related to breaking changes, version incompatibilities
	ErrInterpreting = RegisterError(9, "error interpreting data from node", false, "returned when there are issues interpreting requests or response from node")
	ErrInvalidMemo  = RegisterError(11, "invalid memo", false, "returned when the memo is invalid")
	// ErrBadArgument is returned when the request is malformed
	ErrBadArgument = RegisterError(400, "bad argument", false, "request is malformed")
	// ErrNotFound is returned when the required object was not found
	// retry is set to true because something that is not found now
	// might be found later, example: a TX
	ErrNotFound = RegisterError(404, "not found", true, "returned when the node does not find what the client is asking for")
	// ErrInternal is returned when the node is experiencing internal errors
	ErrInternal = RegisterError(500, "internal error", false, "returned when the node experiences internal errors")
	// ErrBadGateway is returned when there are problems interacting with the nodes
	ErrBadGateway = RegisterError(502, "bad gateway", true, "return when the node is unreachable")
	// ErrNotImplemented is returned when a method is not implemented yet
	ErrNotImplemented = RegisterError(14, "not implemented", false, "returned when querying an endpoint which is not implemented")
	// ErrUnsupportedCurve is returned when the curve specified is not supported
	ErrUnsupportedCurve = RegisterError(15, "unsupported curve, expected secp256k1", false, "returned when using an unsupported crypto curve")
)
