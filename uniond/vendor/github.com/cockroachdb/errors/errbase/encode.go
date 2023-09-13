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
	"log"
	"reflect"
	"strings"

	"github.com/cockroachdb/errors/errorspb"
	"github.com/gogo/protobuf/proto"
	"github.com/gogo/protobuf/types"
)

// EncodedError is the type of an encoded (and protobuf-encodable) error.
type EncodedError = errorspb.EncodedError

// EncodeError encodes an error.
func EncodeError(ctx context.Context, err error) EncodedError {
	if cause := UnwrapOnce(err); cause != nil {
		return encodeWrapper(ctx, err, cause)
	}
	// Not a causer.
	return encodeLeaf(ctx, err)
}

// encodeLeaf encodes a leaf error.
func encodeLeaf(ctx context.Context, err error) EncodedError {
	var msg string
	var details errorspb.EncodedErrorDetails

	if e, ok := err.(*opaqueLeaf); ok {
		msg = e.msg
		details = e.details
	} else {
		details.OriginalTypeName, details.ErrorTypeMark.FamilyName, details.ErrorTypeMark.Extension = getTypeDetails(err, false /*onlyFamily*/)

		var payload proto.Message

		// If we have a manually registered encoder, use that.
		typeKey := TypeKey(details.ErrorTypeMark.FamilyName)
		if enc, ok := leafEncoders[typeKey]; ok {
			msg, details.ReportablePayload, payload = enc(ctx, err)
		} else {
			// No encoder. Let's try to manually extract fields.

			// The message comes from Error(). Simple.
			msg = err.Error()

			// If there are known safe details, use them.
			if s, ok := err.(SafeDetailer); ok {
				details.ReportablePayload = s.SafeDetails()
			}

			// If it's also a protobuf message, we'll use that as
			// payload. DecodeLeaf() will know how to turn that back into a
			// full error if there is no decoder.
			payload, _ = err.(proto.Message)
		}
		// If there is a detail payload, encode it.
		details.FullDetails = encodeAsAny(ctx, err, payload)
	}

	return EncodedError{
		Error: &errorspb.EncodedError_Leaf{
			Leaf: &errorspb.EncodedErrorLeaf{
				Message: msg,
				Details: details,
			},
		},
	}
}

// warningFn can be overridden with a suitable logging function using
// SetWarningFn() below.
var warningFn = func(_ context.Context, format string, args ...interface{}) {
	log.Printf(format, args...)
}

// SetWarningFn enables configuration of the warning function.
func SetWarningFn(fn func(context.Context, string, ...interface{})) {
	warningFn = fn
}

func encodeAsAny(ctx context.Context, err error, payload proto.Message) *types.Any {
	if payload == nil {
		return nil
	}

	any, marshalErr := types.MarshalAny(payload)
	if marshalErr != nil {
		warningFn(ctx,
			"error %+v (%T) announces proto message, but marshaling fails: %+v",
			err, err, marshalErr)
		return nil
	}

	return any
}

// encodeWrapper encodes an error wrapper.
func encodeWrapper(ctx context.Context, err, cause error) EncodedError {
	var msg string
	var details errorspb.EncodedErrorDetails

	if e, ok := err.(*opaqueWrapper); ok {
		msg = e.prefix
		details = e.details
	} else {
		details.OriginalTypeName, details.ErrorTypeMark.FamilyName, details.ErrorTypeMark.Extension = getTypeDetails(err, false /*onlyFamily*/)

		var payload proto.Message

		// If we have a manually registered encoder, use that.
		typeKey := TypeKey(details.ErrorTypeMark.FamilyName)
		if enc, ok := encoders[typeKey]; ok {
			msg, details.ReportablePayload, payload = enc(ctx, err)
		} else {
			// No encoder.
			// In that case, we'll try to compute a message prefix
			// manually.
			msg = extractPrefix(err, cause)

			// If there are known safe details, use them.
			if s, ok := err.(SafeDetailer); ok {
				details.ReportablePayload = s.SafeDetails()
			}

			// That's all we can get.
		}
		// If there is a detail payload, encode it.
		details.FullDetails = encodeAsAny(ctx, err, payload)
	}

	return EncodedError{
		Error: &errorspb.EncodedError_Wrapper{
			Wrapper: &errorspb.EncodedWrapper{
				Cause:         EncodeError(ctx, cause),
				MessagePrefix: msg,
				Details:       details,
			},
		},
	}
}

// extractPrefix extracts the prefix from a wrapper's error message.
// For example,
//    err := errors.New("bar")
//    err = errors.Wrap(err, "foo")
//    extractPrefix(err)
// returns "foo".
func extractPrefix(err, cause error) string {
	causeSuffix := cause.Error()
	errMsg := err.Error()

	if strings.HasSuffix(errMsg, causeSuffix) {
		prefix := errMsg[:len(errMsg)-len(causeSuffix)]
		if strings.HasSuffix(prefix, ": ") {
			return prefix[:len(prefix)-2]
		}
	}
	return ""
}

func getTypeDetails(
	err error, onlyFamily bool,
) (origTypeName string, typeKeyFamily string, typeKeyExtension string) {
	// If we have received an error of type not known locally,
	// we still know its type name. Return that.
	switch t := err.(type) {
	case *opaqueLeaf:
		return t.details.OriginalTypeName, t.details.ErrorTypeMark.FamilyName, t.details.ErrorTypeMark.Extension
	case *opaqueWrapper:
		return t.details.OriginalTypeName, t.details.ErrorTypeMark.FamilyName, t.details.ErrorTypeMark.Extension
	}

	// Compute the full error name, for reporting and printing details.
	tn := getFullTypeName(err)
	// Compute a family name, used to find decoders and to compare error identities.
	fm := tn
	if prevKey, ok := backwardRegistry[TypeKey(tn)]; ok {
		fm = string(prevKey)
	}

	if onlyFamily {
		return tn, fm, ""
	}

	// If the error has an extra type marker, add it.
	// This is not used by the base functionality but
	// is hooked into by the barrier subsystem.
	var em string
	if tm, ok := err.(TypeKeyMarker); ok {
		em = tm.ErrorKeyMarker()
	}
	return tn, fm, em
}

// TypeKeyMarker can be implemented by errors that wish to extend
// their type name as seen by GetTypeKey().
//
// Note: the key marker is considered safe for reporting and
// is included in sentry reports.
type TypeKeyMarker interface {
	ErrorKeyMarker() string
}

func getFullTypeName(err error) string {
	t := reflect.TypeOf(err)
	pkgPath := getPkgPath(t)
	return makeTypeKey(pkgPath, t.String())
}

func makeTypeKey(pkgPath, typeNameString string) string {
	return pkgPath + "/" + typeNameString
}

// getPkgPath extract the package path for a Go type. We'll do some
// extra work for typical types that did not get a name, for example
// *E has the package path of E.
func getPkgPath(t reflect.Type) string {
	pkgPath := t.PkgPath()
	if pkgPath != "" {
		return pkgPath
	}
	// Try harder.
	switch t.Kind() {
	case reflect.Array, reflect.Chan, reflect.Map, reflect.Ptr, reflect.Slice:
		return getPkgPath(t.Elem())
	}
	// Nothing to report.
	return ""
}

// TypeKey identifies an error for the purpose of looking up decoders.
// It is equivalent to the "family name" in ErrorTypeMarker.
type TypeKey string

// GetTypeKey retrieve the type key for a given error object. This
// is meant for use in combination with the Register functions.
func GetTypeKey(err error) TypeKey {
	_, familyName, _ := getTypeDetails(err, true /*onlyFamily*/)
	return TypeKey(familyName)
}

// GetTypeMark retrieves the ErrorTypeMark for a given error object.
// This is meant for use in the markers sub-package.
func GetTypeMark(err error) errorspb.ErrorTypeMark {
	_, familyName, extension := getTypeDetails(err, false /*onlyFamily*/)
	return errorspb.ErrorTypeMark{FamilyName: familyName, Extension: extension}
}

// RegisterLeafEncoder can be used to register new leaf error types to
// the library. Registered types will be encoded using their own
// Go type when an error is encoded. Wrappers that have not been
// registered will be encoded using the opaqueLeaf type.
//
// Note: if the error type has been migrated from a previous location
// or a different type, ensure that RegisterTypeMigration() was called
// prior to RegisterLeafEncoder().
func RegisterLeafEncoder(theType TypeKey, encoder LeafEncoder) {
	if encoder == nil {
		delete(leafEncoders, theType)
	} else {
		leafEncoders[theType] = encoder
	}
}

// LeafEncoder is to be provided (via RegisterLeafEncoder above)
// by additional wrapper types not yet known to this library.
type LeafEncoder = func(ctx context.Context, err error) (msg string, safeDetails []string, payload proto.Message)

// registry for RegisterLeafEncoder.
var leafEncoders = map[TypeKey]LeafEncoder{}

// RegisterWrapperEncoder can be used to register new wrapper types to
// the library. Registered wrappers will be encoded using their own
// Go type when an error is encoded. Wrappers that have not been
// registered will be encoded using the opaqueWrapper type.
//
// Note: if the error type has been migrated from a previous location
// or a different type, ensure that RegisterTypeMigration() was called
// prior to RegisterWrapperEncoder().
func RegisterWrapperEncoder(theType TypeKey, encoder WrapperEncoder) {
	if encoder == nil {
		delete(encoders, theType)
	} else {
		encoders[theType] = encoder
	}
}

// WrapperEncoder is to be provided (via RegisterWrapperEncoder above)
// by additional wrapper types not yet known to this library.
type WrapperEncoder = func(ctx context.Context, err error) (msgPrefix string, safeDetails []string, payload proto.Message)

// registry for RegisterWrapperType.
var encoders = map[TypeKey]WrapperEncoder{}
