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

package errbase_test

import (
	"errors"
	"fmt"
	"testing"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/testutils"
	pkgErr "github.com/pkg/errors"
)

// This test demonstrates how to use errbase.UnwrapOnce and errbase.UnwrapAll to
// access causes.
func TestUnwrap(t *testing.T) {
	tt := testutils.T{T: t}

	err := errors.New("hello")

	tt.CheckEqual(errbase.UnwrapOnce(err), nil)
	tt.CheckEqual(errbase.UnwrapAll(err), err)

	// WithMessage is guaranteed to add just one layer of wrapping.
	err2 := pkgErr.WithMessage(err, "woo")

	tt.CheckEqual(errbase.UnwrapOnce(err2), err)
	tt.CheckEqual(errbase.UnwrapAll(err2), err)

	err3 := pkgErr.WithMessage(err2, "woo")

	tt.CheckEqual(errbase.UnwrapOnce(err3), err2)
	tt.CheckEqual(errbase.UnwrapAll(err3), err)
}

// This test demonstrates how errbase.UnwrapOnce/errbase.UnwrapAll are able to use
// either Cause() or errbase.Unwrap().
func TestMixedErrorWrapping(t *testing.T) {
	tt := testutils.T{T: t}

	err := errors.New("hello")
	err2 := pkgErr.WithMessage(err, "woo")
	err3 := &myWrapper{cause: err2}

	tt.CheckEqual(errbase.UnwrapOnce(err3), err2)
	tt.CheckEqual(errbase.UnwrapAll(err3), err)
}

func TestMultiErrorUnwrap(t *testing.T) {
	tt := testutils.T{T: t}

	err := errors.New("hello")
	err2 := pkgErr.WithMessage(err, "woo")
	err3 := fmt.Errorf("%w %w", err, err2)

	tt.CheckEqual(errbase.UnwrapOnce(err3), nil)
	tt.CheckEqual(errbase.UnwrapAll(err3), err3)
	tt.CheckDeepEqual(errbase.UnwrapMulti(err3), []error{err, err2})
}

type myWrapper struct{ cause error }

func (w *myWrapper) Error() string { return w.cause.Error() }
func (w *myWrapper) Unwrap() error { return w.cause }
