// Copyright 2020 The Cockroach Authors.
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

package oserror

import (
	"os"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/errbase"
)

// Portable analogs of some common system call errors.
//
// Errors returned from this package may be tested against these errors
// with errors.Is.
var (
	ErrInvalid    = os.ErrInvalid
	ErrPermission = os.ErrPermission
	ErrExist      = os.ErrExist
	ErrNotExist   = os.ErrNotExist
	ErrClosed     = os.ErrClosed
)

// IsPermission returns a boolean indicating whether the error is
// known to report that permission is denied. It is satisfied by
// ErrPermission as well as some syscall errors.
//
// This function differs from os.IsPermission() in that it
// can identify an error through wrapping layers.
func IsPermission(err error) bool {
	// errors.Is() is not able to peek through os.SyscallError,
	// whereas os.IsPermission() can. Conversely, os.IsPermission()
	// cannot peek through Unwrap, whereas errors.Is() can. So we
	// need to try both.
	if errors.Is(err, ErrPermission) || os.IsPermission(errors.UnwrapAll(err)) {
		return true
	}
	// If a syscall errno representing ErrPermission was encoded on a
	// different platform, and decoded here, then it will show up as
	// neither a syscall errno here nor an ErrPermission; instead it
	// shows up as an OpaqueErrno. We test this here.
	if o := (*errbase.OpaqueErrno)(nil); errors.As(err, &o) {
		return o.Is(ErrPermission)
	}
	return false
}

// IsExist returns a boolean indicating whether the error is known to report
// that a file or directory already exists. It is satisfied by ErrExist as
// well as some syscall errors.
//
// This function differs from os.IsExist() in that it
// can identify an error through wrapping layers.
func IsExist(err error) bool {
	// errors.Is() is not able to peek through os.SyscallError,
	// whereas os.IsExist() can. Conversely, os.IsExist()
	// cannot peek through Unwrap, whereas errors.Is() can. So we
	// need to try both.
	if errors.Is(err, ErrExist) || os.IsExist(errors.UnwrapAll(err)) {
		return true
	}
	// If a syscall errno representing ErrExist was encoded on a
	// different platform, and decoded here, then it will show up as
	// neither a syscall errno here nor an ErrExist; instead it
	// shows up as an OpaqueErrno. We test this here.
	if o := (*errbase.OpaqueErrno)(nil); errors.As(err, &o) {
		return o.Is(ErrExist)
	}
	return false
}

// IsNotExist returns a boolean indicating whether the error is known to
// report that a file or directory does not exist. It is satisfied by
// ErrNotExist as well as some syscall errors.
//
// This function differs from os.IsNotExist() in that it
// can identify an error through wrapping layers.
func IsNotExist(err error) bool {
	// errors.Is() is not able to peek through os.SyscallError,
	// whereas os.IsNotExist() can. Conversely, os.IsNotExist()
	// cannot peek through Unwrap, whereas errors.Is() can. So we
	// need to try both.
	if errors.Is(err, ErrNotExist) || os.IsNotExist(errors.UnwrapAll(err)) {
		return true
	}
	// If a syscall errno representing ErrNotExist was encoded on a
	// different platform, and decoded here, then it will show up as
	// neither a syscall errno here nor an ErrNotExist; instead it
	// shows up as an OpaqueErrno. We test this here.
	if o := (*errbase.OpaqueErrno)(nil); errors.As(err, &o) {
		return o.Is(ErrNotExist)
	}
	return false
}

// IsTimeout returns a boolean indicating whether the error is known
// to report that a timeout occurred.
//
// This function differs from os.IsTimeout() in that it
// can identify an error through wrapping layers.
func IsTimeout(err error) bool {
	// os.IsTimeout() cannot peek through Unwrap. We need errors.If()
	// for that.
	_, ok := errors.If(err, func(err error) (interface{}, bool) {
		return nil, os.IsTimeout(err)
	})
	return ok
}
