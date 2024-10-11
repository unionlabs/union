// Copyright 2017 Michal Witkowski. All Rights Reserved.
// See LICENSE for licensing terms.

package grpc_logging

import (
	"context"
	"io"

	"github.com/golang/protobuf/proto"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// ErrorToCode function determines the error code of an error
// This makes using custom errors with grpc middleware easier
type ErrorToCode func(err error) codes.Code

func DefaultErrorToCode(err error) codes.Code {
	return status.Code(err)
}

// Decider function defines rules for suppressing any interceptor logs
type Decider func(fullMethodName string, err error) bool

// DefaultDeciderMethod is the default implementation of decider to see if you should log the call
// by default this if always true so all calls are logged
func DefaultDeciderMethod(fullMethodName string, err error) bool {
	return true
}

// ServerPayloadLoggingDecider is a user-provided function for deciding whether to log the server-side
// request/response payloads
type ServerPayloadLoggingDecider func(ctx context.Context, fullMethodName string, servingObject interface{}) bool

// ClientPayloadLoggingDecider is a user-provided function for deciding whether to log the client-side
// request/response payloads
type ClientPayloadLoggingDecider func(ctx context.Context, fullMethodName string) bool

// JsonPbMarshaller is a marshaller that serializes protobuf messages.
type JsonPbMarshaler interface {
	Marshal(out io.Writer, pb proto.Message) error
}
