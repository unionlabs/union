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

package errors

import (
	"context"
	"fmt"

	"github.com/cockroachdb/errors/errbase"
)

// UnwrapOnce accesses the direct cause of the error if any, otherwise
// returns nil.
//
// It supports both errors implementing causer (`Cause()` method, from
// github.com/pkg/errors) and `Wrapper` (`Unwrap()` method, from the
// Go 2 error proposal).
func UnwrapOnce(err error) error { return errbase.UnwrapOnce(err) }

// UnwrapAll accesses the root cause object of the error.
// If the error has no cause (leaf error), it is returned directly.
func UnwrapAll(err error) error { return errbase.UnwrapAll(err) }

// EncodedError is the type of an encoded (and protobuf-encodable) error.
type EncodedError = errbase.EncodedError

// EncodeError encodes an error.
func EncodeError(ctx context.Context, err error) EncodedError { return errbase.EncodeError(ctx, err) }

// DecodeError decodes an error.
func DecodeError(ctx context.Context, enc EncodedError) error { return errbase.DecodeError(ctx, enc) }

// SafeDetailer is an interface that can be implemented by errors that
// can provide PII-free additional strings suitable for reporting or
// telemetry.
type SafeDetailer = errbase.SafeDetailer

// GetAllSafeDetails collects the safe details from the given error object
// and all its causes.
// The details are collected from outermost to innermost level of cause.
func GetAllSafeDetails(err error) []SafeDetailPayload { return errbase.GetAllSafeDetails(err) }

// GetSafeDetails collects the safe details from the given error
// object. If it is a wrapper, only the details from the wrapper are
// returned.
func GetSafeDetails(err error) (payload SafeDetailPayload) { return errbase.GetSafeDetails(err) }

// SafeDetailPayload captures the safe strings for one
// level of wrapping.
type SafeDetailPayload = errbase.SafeDetailPayload

// RegisterLeafDecoder can be used to register new leaf error types to
// the library. Registered types will be decoded using their own
// Go type when an error is decoded. Wrappers that have not been
// registered will be decoded using the opaqueLeaf type.
//
// Note: if the error type has been migrated from a previous location
// or a different type, ensure that RegisterTypeMigration() was called
// prior to RegisterLeafDecoder().
func RegisterLeafDecoder(typeName TypeKey, decoder LeafDecoder) {
	errbase.RegisterLeafDecoder(typeName, decoder)
}

// TypeKey identifies an error for the purpose of looking up decoders.
// It is equivalent to the "family name" in ErrorTypeMarker.
type TypeKey = errbase.TypeKey

// GetTypeKey retrieve the type key for a given error object. This
// is meant for use in combination with the Register functions.
func GetTypeKey(err error) TypeKey { return errbase.GetTypeKey(err) }

// LeafDecoder is to be provided (via RegisterLeafDecoder above)
// by additional wrapper types not yet known to this library.
// A nil return indicates that decoding was not successful.
type LeafDecoder = errbase.LeafDecoder

// RegisterWrapperDecoder can be used to register new wrapper types to
// the library. Registered wrappers will be decoded using their own
// Go type when an error is decoded. Wrappers that have not been
// registered will be decoded using the opaqueWrapper type.
//
// Note: if the error type has been migrated from a previous location
// or a different type, ensure that RegisterTypeMigration() was called
// prior to RegisterWrapperDecoder().
func RegisterWrapperDecoder(typeName TypeKey, decoder WrapperDecoder) {
	errbase.RegisterWrapperDecoder(typeName, decoder)
}

// WrapperDecoder is to be provided (via RegisterWrapperDecoder above)
// by additional wrapper types not yet known to this library.
// A nil return indicates that decoding was not successful.
type WrapperDecoder = errbase.WrapperDecoder

// RegisterLeafEncoder can be used to register new leaf error types to
// the library. Registered types will be encoded using their own
// Go type when an error is encoded. Wrappers that have not been
// registered will be encoded using the opaqueLeaf type.
//
// Note: if the error type has been migrated from a previous location
// or a different type, ensure that RegisterTypeMigration() was called
// prior to RegisterLeafEncoder().
func RegisterLeafEncoder(typeName TypeKey, encoder LeafEncoder) {
	errbase.RegisterLeafEncoder(typeName, encoder)
}

// LeafEncoder is to be provided (via RegisterLeafEncoder above)
// by additional wrapper types not yet known to this library.
type LeafEncoder = errbase.LeafEncoder

// RegisterWrapperEncoder can be used to register new wrapper types to
// the library. Registered wrappers will be encoded using their own
// Go type when an error is encoded. Wrappers that have not been
// registered will be encoded using the opaqueWrapper type.
//
// Note: if the error type has been migrated from a previous location
// or a different type, ensure that RegisterTypeMigration() was called
// prior to RegisterWrapperEncoder().
func RegisterWrapperEncoder(typeName TypeKey, encoder WrapperEncoder) {
	errbase.RegisterWrapperEncoder(typeName, encoder)
}

// WrapperEncoder is to be provided (via RegisterWrapperEncoder above)
// by additional wrapper types not yet known to this library.
type WrapperEncoder = errbase.WrapperEncoder

// SetWarningFn enables configuration of the warning function.
func SetWarningFn(fn func(context.Context, string, ...interface{})) { errbase.SetWarningFn(fn) }

// A Formatter formats error messages.
//
// NB: Consider implementing SafeFormatter instead. This will ensure
// that error displays can distinguish bits that are PII-safe.
type Formatter = errbase.Formatter

// SafeFormatter is implemented by error leaf or wrapper types that want
// to separate safe and non-safe information when printed out.
//
// When multiple errors are chained (e.g. via errors.Wrap), intermediate
// layers in the error that do not implement SafeError are considered
// “unsafe”
type SafeFormatter = errbase.SafeFormatter

// A Printer formats error messages.
//
// The most common implementation of Printer is the one provided by package fmt
// during Printf (as of Go 1.13). Localization packages such as golang.org/x/text/message
// typically provide their own implementations.
type Printer = errbase.Printer

// FormatError formats an error according to s and verb.
// This is a helper meant for use when implementing the fmt.Formatter
// interface on custom error objects.
//
// If the error implements errors.Formatter, FormatError calls its
// FormatError method of f with an errors.Printer configured according
// to s and verb, and writes the result to s.
//
// Otherwise, if it is a wrapper, FormatError prints out its error prefix,
// then recurses on its cause.
//
// Otherwise, its Error() text is printed.
func FormatError(err error, s fmt.State, verb rune) { errbase.FormatError(err, s, verb) }

// Formattable wraps an error into a fmt.Formatter which
// will provide "smart" formatting even if the outer layer
// of the error does not implement the Formatter interface.
func Formattable(err error) fmt.Formatter { return errbase.Formattable(err) }

// RegisterTypeMigration tells the library that the type of the error
// given as 3rd argument was previously known with type
// previousTypeName, located at previousPkgPath.
//
// The value of previousTypeName must be the result of calling
// reflect.TypeOf(err).String() on the original error object.
// This is usually composed as follows:
//     [*]<shortpackage>.<errortype>
//
// For example, Go's standard error type has name "*errors.errorString".
// The asterisk indicates that `errorString` implements the `error`
// interface via pointer receiver.
//
// Meanwhile, the singleton error type context.DeadlineExceeded
// has name "context.deadlineExceededError", without asterisk
// because the type implements `error` by value.
//
// Remember that the short package name inside the error type name and
// the last component of the package path can be different. This is
// why they must be specified separately.
func RegisterTypeMigration(previousPkgPath, previousTypeName string, newType error) {
	errbase.RegisterTypeMigration(previousPkgPath, previousTypeName, newType)
}
