package core

import (
	"net/http"

	"google.golang.org/grpc/codes"
)

type ErrorReason uint8

const (
	Internal = iota
	Unavailable
	BadRequest
	NotFound
	// Add more errors as needed
)

type RpcError struct {
	Err    error
	Reason ErrorReason
}

func ErrorReasonToGRPC(reason ErrorReason) codes.Code {
	switch reason {
	case Internal:
		return codes.Internal
	case Unavailable:
		return codes.Unavailable
	case BadRequest:
		return codes.InvalidArgument
	case NotFound:
		return codes.NotFound
	// Add more cases for other error reasons as needed
	default:
		return codes.Internal
	}
}

func ErrorReasonToHTTP(reason ErrorReason) int {
	switch reason {
	case Internal:
		return http.StatusInternalServerError
	case Unavailable:
		return http.StatusServiceUnavailable
	case BadRequest:
		return http.StatusBadRequest
	case NotFound:
		return http.StatusNotFound
	// Add more cases for other error reasons as needed
	default:
		return http.StatusInternalServerError
	}
}
