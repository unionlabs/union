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

// Sadly the go 2/1.13 design for errors has promoted the name
// `Unwrap()` for the method that accesses the cause, whilst the
// ecosystem has already chosen `Cause()`. In order to unwrap
// reliably, we must thus support both.
//
// See: https://github.com/golang/go/issues/31778

// UnwrapOnce accesses the direct cause of the error if any, otherwise
// returns nil.
//
// It supports both errors implementing causer (`Cause()` method, from
// github.com/pkg/errors) and `Wrapper` (`Unwrap()` method, from the
// Go 2 error proposal).
//
// UnwrapOnce treats multi-errors (those implementing the
// `Unwrap() []error` interface as leaf-nodes since they cannot
// reasonably be iterated through to a single cause. These errors
// are typically constructed as a result of `fmt.Errorf` which results
// in a `wrapErrors` instance that contains an interpolated error
// string along with a list of causes.
//
// The go stdlib does not define output on `Unwrap()` for a multi-cause
// error, so we default to nil here.
func UnwrapOnce(err error) (cause error) {
	switch e := err.(type) {
	case interface{ Cause() error }:
		return e.Cause()
	case interface{ Unwrap() error }:
		return e.Unwrap()
	}
	return nil
}

// UnwrapAll accesses the root cause object of the error.
// If the error has no cause (leaf error), it is returned directly.
// UnwrapAll treats multi-errors as leaf nodes.
func UnwrapAll(err error) error {
	for {
		if cause := UnwrapOnce(err); cause != nil {
			err = cause
			continue
		}
		break
	}
	return err
}

// UnwrapMulti access the slice of causes that an error contains, if it is a
// multi-error.
func UnwrapMulti(err error) []error {
	if me, ok := err.(interface{ Unwrap() []error }); ok {
		return me.Unwrap()
	}
	return nil
}
