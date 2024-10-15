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

package markers

import (
	"context"
	"fmt"
	"reflect"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/errorspb"
	"github.com/cockroachdb/redact"
	"github.com/gogo/protobuf/proto"
)

// Is determines whether one of the causes of the given error or any
// of its causes is equivalent to some reference error.
//
// As in the Go standard library, an error is considered to match a
// reference error if it is equal to that target or if it implements a
// method Is(error) bool such that Is(reference) returns true.
//
// Note: the inverse is not true - making an Is(reference) method
// return false does not imply that errors.Is() also returns
// false. Errors can be equal because their network equality marker is
// the same. To force errors to appear different to Is(), use
// errors.Mark().
//
// Note: if any of the error types has been migrated from a previous
// package location or a different type, ensure that
// RegisterTypeMigration() was called prior to Is().
func Is(err, reference error) bool {
	if reference == nil {
		return err == nil
	}

	isComparable := reflect.TypeOf(reference).Comparable()

	// Direct reference comparison is the fastest, and most
	// likely to be true, so do this first.
	for c := err; c != nil; c = errbase.UnwrapOnce(c) {
		if isComparable && c == reference {
			return true
		}
		// Compatibility with std go errors: if the error object itself
		// implements Is(), try to use that.
		if tryDelegateToIsMethod(c, reference) {
			return true
		}

		// Recursively try multi-error causes, if applicable.
		for _, me := range errbase.UnwrapMulti(c) {
			if Is(me, reference) {
				return true
			}
		}
	}

	if err == nil {
		// Err is nil and reference is non-nil, so it cannot match. We
		// want to short-circuit the loop below in this case, otherwise
		// we're paying the expense of getMark() without need.
		return false
	}

	// Not directly equal. Try harder, using error marks. We don't do
	// this during the loop above as it may be more expensive.
	//
	// Note: there is a more effective recursive algorithm that ensures
	// that any pair of string only gets compared once. Should the
	// following code become a performance bottleneck, that algorithm
	// can be considered instead.
	refMark := getMark(reference)
	for c := err; c != nil; c = errbase.UnwrapOnce(c) {
		if equalMarks(getMark(c), refMark) {
			return true
		}
	}
	return false
}

func tryDelegateToIsMethod(err, reference error) bool {
	if x, ok := err.(interface{ Is(error) bool }); ok && x.Is(reference) {
		return true
	}
	return false
}

// HasType returns true iff err contains an error whose concrete type
// matches that of referenceType.
func HasType(err error, referenceType error) bool {
	typ := reflect.TypeOf(referenceType)
	_, isType := If(err, func(err error) (interface{}, bool) {
		return nil, reflect.TypeOf(err) == typ
	})
	return isType
}

// HasInterface returns true if err contains an error which implements the
// interface pointed to by referenceInterface. The type of referenceInterface
// must be a pointer to an interface type. If referenceInterface is not a
// pointer to an interface, this function will panic.
func HasInterface(err error, referenceInterface interface{}) bool {
	iface := getInterfaceType("HasInterface", referenceInterface)
	_, isType := If(err, func(err error) (interface{}, bool) {
		return nil, reflect.TypeOf(err).Implements(iface)
	})
	return isType
}

func getInterfaceType(context string, referenceInterface interface{}) reflect.Type {
	typ := reflect.TypeOf(referenceInterface)
	if typ == nil || typ.Kind() != reflect.Ptr || typ.Elem().Kind() != reflect.Interface {
		panic(fmt.Errorf("errors.%s: referenceInterface must be a pointer to an interface, "+
			"found %T", context, referenceInterface))
	}
	return typ.Elem()
}

// If iterates on the error's causal chain and returns a predicate's
// return value the first time the predicate returns true.
//
// Note: if any of the error types has been migrated from a previous
// package location or a different type, ensure that
// RegisterTypeMigration() was called prior to If().
func If(err error, pred func(err error) (interface{}, bool)) (interface{}, bool) {
	for c := err; c != nil; c = errbase.UnwrapOnce(c) {
		if v, ok := pred(c); ok {
			return v, ok
		}
	}
	return nil, false
}

// IsAny is like Is except that multiple references are compared.
//
// Note: if any of the error types has been migrated from a previous
// package location or a different type, ensure that
// RegisterTypeMigration() was called prior to IsAny().
func IsAny(err error, references ...error) bool {
	if err == nil {
		for _, refErr := range references {
			if refErr == nil {
				return true
			}
		}
		// The mark-based comparison below will never match anything if
		// the error is nil, so don't bother with computing the marks in
		// that case. This avoids the computational expense of computing
		// the reference marks upfront.
		return false
	}

	// First try using direct reference comparison.
	for c := err; c != nil; c = errbase.UnwrapOnce(c) {
		for _, refErr := range references {
			if refErr == nil {
				continue
			}
			isComparable := reflect.TypeOf(refErr).Comparable()
			if isComparable && c == refErr {
				return true
			}
			// Compatibility with std go errors: if the error object itself
			// implements Is(), try to use that.
			if tryDelegateToIsMethod(c, refErr) {
				return true
			}
		}

		// Recursively try multi-error causes, if applicable.
		for _, me := range errbase.UnwrapMulti(c) {
			if IsAny(me, references...) {
				return true
			}
		}
	}

	// Try harder with marks.
	// Note: there is a more effective recursive algorithm that ensures
	// that any pair of string only gets compared once. Should this
	// become a performance bottleneck, that algorithm can be considered
	// instead.
	refMarks := make([]errorMark, 0, len(references))
	for _, refErr := range references {
		if refErr == nil {
			continue
		}
		refMarks = append(refMarks, getMark(refErr))
	}
	for c := err; c != nil; c = errbase.UnwrapOnce(c) {
		errMark := getMark(c)
		for _, refMark := range refMarks {
			if equalMarks(errMark, refMark) {
				return true
			}
		}
	}
	return false
}

type errorMark struct {
	msg   string
	types []errorspb.ErrorTypeMark
}

// equalMarks compares two error markers.
func equalMarks(m1, m2 errorMark) bool {
	if m1.msg != m2.msg {
		return false
	}
	for i, t := range m1.types {
		if !t.Equals(m2.types[i]) {
			return false
		}
	}
	return true
}

// getMark computes a marker for the given error.
func getMark(err error) errorMark {
	if m, ok := err.(*withMark); ok {
		return m.mark
	}
	m := errorMark{msg: safeGetErrMsg(err), types: []errorspb.ErrorTypeMark{errbase.GetTypeMark(err)}}
	for c := errbase.UnwrapOnce(err); c != nil; c = errbase.UnwrapOnce(c) {
		m.types = append(m.types, errbase.GetTypeMark(c))
	}
	return m
}

// safeGetErrMsg extracts an error's Error() but tolerates panics.
func safeGetErrMsg(err error) (result string) {
	defer func() {
		if r := recover(); r != nil {
			result = fmt.Sprintf("(%p).Error() panic: %v", err, r)
		}
	}()
	result = err.Error()
	return
}

// Mark creates an explicit mark for the given error, using
// the same mark as some reference error.
//
// Note: if any of the error types has been migrated from a previous
// package location or a different type, ensure that
// RegisterTypeMigration() was called prior to Mark().
func Mark(err error, reference error) error {
	if err == nil {
		return nil
	}
	refMark := getMark(reference)
	return &withMark{cause: err, mark: refMark}
}

// withMark carries an explicit mark.
type withMark struct {
	cause error
	mark  errorMark
}

var _ error = (*withMark)(nil)
var _ fmt.Formatter = (*withMark)(nil)
var _ errbase.SafeFormatter = (*withMark)(nil)

func (m *withMark) Error() string { return m.cause.Error() }
func (m *withMark) Cause() error  { return m.cause }
func (m *withMark) Unwrap() error { return m.cause }

func (m *withMark) Format(s fmt.State, verb rune) { errbase.FormatError(m, s, verb) }

func (m *withMark) SafeFormatError(p errbase.Printer) error {
	if p.Detail() {
		p.Printf("forced error mark\n")
		p.Printf("%q\n%s::%s",
			m.mark.msg,
			redact.Safe(m.mark.types[0].FamilyName),
			redact.Safe(m.mark.types[0].Extension),
		)
	}
	return m.cause
}

func encodeMark(_ context.Context, err error) (msg string, _ []string, payload proto.Message) {
	m := err.(*withMark)
	payload = &errorspb.MarkPayload{Msg: m.mark.msg, Types: m.mark.types}
	return "", nil, payload
}

func decodeMark(_ context.Context, cause error, _ string, _ []string, payload proto.Message) error {
	m, ok := payload.(*errorspb.MarkPayload)
	if !ok {
		// If this ever happens, this means some version of the library
		// (presumably future) changed the payload type, and we're
		// receiving this here. In this case, give up and let
		// DecodeError use the opaque type.
		return nil
	}
	return &withMark{cause: cause, mark: errorMark{msg: m.Msg, types: m.Types}}
}

func init() {
	errbase.RegisterWrapperEncoder(errbase.GetTypeKey((*withMark)(nil)), encodeMark)
	errbase.RegisterWrapperDecoder(errbase.GetTypeKey((*withMark)(nil)), decodeMark)
}
