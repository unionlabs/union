// Copyright 2019 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package errbase

import (
	"context"
	goErr "errors"
	"fmt"
	"os"

	"github.com/cockroachdb/errors/errorspb"
	"github.com/gogo/protobuf/proto"
	pkgErr "github.com/pkg/errors"
)

// This file provides the library the ability to encode/decode
// standard error types.

// errors.errorString from base Go does not need an encoding
// function, because the base encoding logic in EncodeLeaf() is
// able to extract everything about it.

// we can then decode it exactly.
func decodeErrorString(_ context.Context, msg string, _ []string, _ proto.Message) error {
	return goErr.New(msg)
}

// context.DeadlineExceeded uses a custom type.
func decodeDeadlineExceeded(_ context.Context, _ string, _ []string, _ proto.Message) error {
	return context.DeadlineExceeded
}

// errors.fundamental from github.com/pkg/errors cannot be encoded
// exactly because it includes a non-serializable stack trace
// object. In order to work with it, we encode it by dumping
// the stack trace in a safe reporting detail field, and decode
// it as an opaqueLeaf instance in this package.

func encodePkgFundamental(
	_ context.Context, err error,
) (msg string, safe []string, _ proto.Message) {
	msg = err.Error()
	iErr := err.(interface{ StackTrace() pkgErr.StackTrace })
	safeDetails := []string{fmt.Sprintf("%+v", iErr.StackTrace())}
	return msg, safeDetails, nil
}

// errors.withMessage from github.com/pkg/errors can be encoded
// exactly because it just has a message prefix. The base encoding
// logic in EncodeWrapper() is able to extract everything from it.

// we can then decode it exactly.
func decodeWithMessage(
	_ context.Context, cause error, msgPrefix string, _ []string, _ proto.Message,
) error {
	return pkgErr.WithMessage(cause, msgPrefix)
}

// errors.withStack from github.com/pkg/errors cannot be encoded
// exactly because it includes a non-serializable stack trace
// object. In order to work with it, we encode it by dumping
// the stack trace in a safe reporting detail field, and decode
// it as an opaqueWrapper instance in this package.

func encodePkgWithStack(
	_ context.Context, err error,
) (msgPrefix string, safe []string, _ proto.Message) {
	iErr := err.(interface{ StackTrace() pkgErr.StackTrace })
	safeDetails := []string{fmt.Sprintf("%+v", iErr.StackTrace())}
	return "" /* withStack does not have a message prefix */, safeDetails, nil
}

func encodePathError(
	_ context.Context, err error,
) (msgPrefix string, safe []string, details proto.Message) {
	p := err.(*os.PathError)
	msg := p.Op + " " + p.Path
	details = &errorspb.StringsPayload{
		Details: []string{p.Op, p.Path},
	}
	return msg, []string{p.Op}, details
}

func decodePathError(
	_ context.Context, cause error, _ string, _ []string, payload proto.Message,
) (result error) {
	m, ok := payload.(*errorspb.StringsPayload)
	if !ok || len(m.Details) < 2 {
		// If this ever happens, this means some version of the library
		// (presumably future) changed the payload type, and we're
		// receiving this here. In this case, give up and let
		// DecodeError use the opaque type.
		return nil
	}
	return &os.PathError{
		Op:   m.Details[0],
		Path: m.Details[1],
		Err:  cause,
	}
}

func encodeLinkError(
	_ context.Context, err error,
) (msgPrefix string, safe []string, details proto.Message) {
	p := err.(*os.LinkError)
	msg := p.Op + " " + p.Old + " " + p.New
	details = &errorspb.StringsPayload{
		Details: []string{p.Op, p.Old, p.New},
	}
	return msg, []string{p.Op}, details
}

func decodeLinkError(
	_ context.Context, cause error, _ string, _ []string, payload proto.Message,
) (result error) {
	m, ok := payload.(*errorspb.StringsPayload)
	if !ok || len(m.Details) < 3 {
		// If this ever happens, this means some version of the library
		// (presumably future) changed the payload type, and we're
		// receiving this here. In this case, give up and let
		// DecodeError use the opaque type.
		return nil
	}
	return &os.LinkError{
		Op:  m.Details[0],
		Old: m.Details[1],
		New: m.Details[2],
		Err: cause,
	}
}

func encodeSyscallError(
	_ context.Context, err error,
) (msgPrefix string, safe []string, details proto.Message) {
	p := err.(*os.SyscallError)
	return p.Syscall, nil, nil
}

func decodeSyscallError(
	_ context.Context, cause error, msg string, _ []string, _ proto.Message,
) (result error) {
	return os.NewSyscallError(msg, cause)
}

// OpaqueErrno represents a syscall.Errno error object that
// was constructed on a different OS/platform combination.
type OpaqueErrno struct {
	msg     string
	details *errorspb.ErrnoPayload
}

// Error implements the error interface.
func (o *OpaqueErrno) Error() string { return o.msg }

// Is tests whether this opaque errno object represents a special os error type.
func (o *OpaqueErrno) Is(target error) bool {
	return (target == os.ErrPermission && o.details.IsPermission) ||
		(target == os.ErrExist && o.details.IsExist) ||
		(target == os.ErrNotExist && o.details.IsNotExist)
}

// Temporary tests whether this opaque errno object encodes a temporary error.
func (o *OpaqueErrno) Temporary() bool { return o.details.IsTemporary }

// Timeout tests whether this opaque errno object encodes a timeout error.
func (o *OpaqueErrno) Timeout() bool { return o.details.IsTimeout }

func encodeOpaqueErrno(
	_ context.Context, err error,
) (msg string, safe []string, payload proto.Message) {
	e := err.(*OpaqueErrno)
	return e.Error(), []string{e.Error()}, e.details
}

func init() {
	baseErr := goErr.New("")
	RegisterLeafDecoder(GetTypeKey(baseErr), decodeErrorString)

	RegisterLeafDecoder(GetTypeKey(context.DeadlineExceeded), decodeDeadlineExceeded)

	pkgE := pkgErr.New("")
	RegisterLeafEncoder(GetTypeKey(pkgE), encodePkgFundamental)

	RegisterWrapperDecoder(GetTypeKey(pkgErr.WithMessage(baseErr, "")), decodeWithMessage)

	ws := pkgErr.WithStack(baseErr)
	RegisterWrapperEncoder(GetTypeKey(ws), encodePkgWithStack)

	registerOsPathErrorMigration() // Needed for Go 1.16.
	pKey := GetTypeKey(&os.PathError{})
	RegisterWrapperEncoder(pKey, encodePathError)
	RegisterWrapperDecoder(pKey, decodePathError)

	pKey = GetTypeKey(&os.LinkError{})
	RegisterWrapperEncoder(pKey, encodeLinkError)
	RegisterWrapperDecoder(pKey, decodeLinkError)
	pKey = GetTypeKey(&os.SyscallError{})
	RegisterWrapperEncoder(pKey, encodeSyscallError)
	RegisterWrapperDecoder(pKey, decodeSyscallError)

	RegisterLeafEncoder(GetTypeKey(&OpaqueErrno{}), encodeOpaqueErrno)
}
