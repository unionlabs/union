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
	"context"
	goErr "errors"
	"fmt"
	"os"
	"reflect"
	"strings"
	"testing"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/errorspb"
	"github.com/cockroachdb/errors/testutils"
	"github.com/kr/pretty"
	pkgErr "github.com/pkg/errors"
)

func network(t *testing.T, err error) error {
	t.Helper()
	enc := errbase.EncodeError(context.Background(), err)
	t.Logf("encoded: %# v", pretty.Formatter(enc))
	newErr := errbase.DecodeError(context.Background(), enc)
	t.Logf("decoded: %# v", pretty.Formatter(newErr))
	return newErr
}

func TestAdaptBaseGoErr(t *testing.T) {
	// Base Go errors are preserved completely.
	origErr := goErr.New("world")
	t.Logf("start err: %# v", pretty.Formatter(origErr))

	newErr := network(t, origErr)

	tt := testutils.T{T: t}
	// The library preserves the error message.
	tt.CheckEqual(newErr.Error(), origErr.Error())

	// It actually preserves the full structure of the message,
	// including its Go type.
	tt.CheckDeepEqual(newErr, origErr)
}

func TestAdaptGoSingleWrapErr(t *testing.T) {
	origErr := fmt.Errorf("an error %w", goErr.New("hello"))
	t.Logf("start err: %# v", pretty.Formatter(origErr))

	newErr := network(t, origErr)

	tt := testutils.T{T: t}
	// The library preserves the cause. It's not possible to preserve the fmt
	// string.
	tt.CheckEqual(newErr.Error(), origErr.Error())
	tt.CheckContains(newErr.Error(), "hello")
}

func TestAdaptBaseGoJoinErr(t *testing.T) {
	origErr := goErr.Join(goErr.New("hello"), goErr.New("world"))
	t.Logf("start err: %# v", pretty.Formatter(origErr))

	newErr := network(t, origErr)

	tt := testutils.T{T: t}
	// The library preserves the error message.
	tt.CheckEqual(newErr.Error(), origErr.Error())

}

func TestAdaptGoMultiWrapErr(t *testing.T) {
	origErr := fmt.Errorf("an error %w and also %w", goErr.New("hello"), goErr.New("world"))
	t.Logf("start err: %# v", pretty.Formatter(origErr))

	newErr := network(t, origErr)

	tt := testutils.T{T: t}
	// The library preserves the causes. It's not possible to preserve the fmt string.
	tt.CheckEqual(newErr.Error(), origErr.Error())
	tt.CheckContains(newErr.Error(), "hello")
	tt.CheckContains(newErr.Error(), "world")
}

func TestAdaptPkgWithMessage(t *testing.T) {
	// Simple message wrappers from github.com/pkg/errors are preserved
	// completely.
	origErr := pkgErr.WithMessage(goErr.New("world"), "hello")
	t.Logf("start err: %# v", pretty.Formatter(origErr))

	newErr := network(t, origErr)

	tt := testutils.T{T: t}
	// The library preserves the error message.
	tt.CheckEqual(newErr.Error(), origErr.Error())

	// It actually preserves the full structure of the message,
	// including its Go type.
	tt.CheckDeepEqual(newErr, origErr)
}

func TestAdaptPkgFundamental(t *testing.T) {
	// The "simple error" from github.com/pkg/errors is not
	// that simple because it contains a stack trace. However,
	// we are happy to preserve this stack trace.
	origErr := pkgErr.New("hello")
	t.Logf("start err: %# v", pretty.Formatter(origErr))

	tt := testutils.T{T: t}

	// Show that there is indeed a stack trace.
	theStack := fmt.Sprintf("%+v", errbase.GetSafeDetails(origErr))
	tt.Check(strings.Contains(theStack, "adapters_test.go"))

	newErr := network(t, origErr)

	// In any case, the library preserves the error message.
	tt.CheckEqual(newErr.Error(), origErr.Error())

	// The decoded error does not compare equal, since
	// we had to change the type to preserve the stack trace.
	tt.Check(!reflect.DeepEqual(origErr, newErr))

	// However, it remembers what type the error is coming from.
	errV := fmt.Sprintf("%+v", newErr)
	tt.Check(strings.Contains(errV, "github.com/pkg/errors/*errors.fundamental"))

	// Also, the decoded error does include the stack trace.
	details := errbase.GetSafeDetails(newErr).SafeDetails
	tt.Check(len(details) > 0 && strings.Contains(details[0], "adapters_test.go"))

	// Moreover, if we re-encode and re-decode, that will be preserved exactly!
	newErr2 := network(t, newErr)
	tt.CheckDeepEqual(newErr2, newErr)
}

func TestAdaptPkgWithStack(t *testing.T) {
	// The "with stack" wrapper from github.com/pkg/errors cannot be
	// serialized exactly, however we are happy to preserve this stack
	// trace.
	origErr := pkgErr.WithStack(goErr.New("hello"))
	t.Logf("start err: %# v", pretty.Formatter(origErr))

	tt := testutils.T{T: t}
	// Show that there is indeed a stack trace.
	theStack := fmt.Sprintf("%+v", errbase.GetSafeDetails(origErr))
	tt.Check(strings.Contains(theStack, "adapters_test.go"))

	newErr := network(t, origErr)

	// In any case, the library preserves the error message.
	tt.CheckEqual(newErr.Error(), origErr.Error())

	// The decoded error does not compare equal, since
	// we had to change the type to preserve the stack trace.
	tt.Check(!reflect.DeepEqual(newErr, origErr))

	// However, it does include the stack trace.
	details := errbase.GetSafeDetails(newErr).SafeDetails
	tt.Check(len(details) > 0 && strings.Contains(details[0], "adapters_test.go"))

	// Moreover, if we re-encode and re-decode, that will be preserved exactly!
	newErr2 := network(t, newErr)
	tt.CheckDeepEqual(newErr2, newErr)
}

func TestAdaptProtoErrors(t *testing.T) {
	// If an error type has a proto representation already,
	// it will be preserved exactly.
	origErr := &errorspb.TestError{}
	t.Logf("start err: %# v", pretty.Formatter(origErr))

	newErr := network(t, origErr)

	tt := testutils.T{T: t}

	// In any case, the library preserves the error message.
	tt.CheckEqual(newErr.Error(), origErr.Error())

	// Moreover, it preserves the entire structure.
	tt.CheckDeepEqual(newErr, origErr)
}

func TestAdaptProtoErrorsWithWrapper(t *testing.T) {
	// proto-native error types are preserved exactly
	// together with their wrappers.
	rErr := &errorspb.TestError{}
	origErr := pkgErr.WithMessage(rErr, "hello roachpb")
	t.Logf("start err: %# v", pretty.Formatter(origErr))

	newErr := network(t, origErr)

	tt := testutils.T{T: t}

	// In any case, the library preserves the error message.
	tt.CheckEqual(newErr.Error(), origErr.Error())

	// Moreover, it preserves the entire structure.
	tt.CheckDeepEqual(newErr, origErr)
}

func TestAdaptContextCanceled(t *testing.T) {
	// context.DeadlineExceeded is preserved exactly.

	tt := testutils.T{T: t}
	newErr := network(t, context.DeadlineExceeded)
	tt.CheckEqual(newErr, context.DeadlineExceeded)
}

func TestAdaptOsErrors(t *testing.T) {
	// The special os error types are preserved exactly.

	tt := testutils.T{T: t}
	var origErr error

	origErr = &os.PathError{Op: "hello", Path: "world", Err: goErr.New("woo")}
	newErr := network(t, origErr)
	tt.CheckDeepEqual(newErr, origErr)

	origErr = &os.LinkError{Op: "hello", Old: "world", New: "universe", Err: goErr.New("woo")}
	newErr = network(t, origErr)
	tt.CheckDeepEqual(newErr, origErr)

	origErr = &os.SyscallError{Syscall: "hello", Err: goErr.New("woo")}
	newErr = network(t, origErr)
	tt.CheckDeepEqual(newErr, origErr)
}
