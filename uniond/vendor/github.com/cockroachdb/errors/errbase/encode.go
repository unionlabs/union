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
	return encodeLeaf(ctx, err, UnwrapMulti(err))
}

// encodeLeaf encodes a leaf error. This function accepts a `causes`
// argument because we encode multi-cause errors using the Leaf
// protobuf. This was done to enable backwards compatibility when
// introducing this functionality since the Wrapper type already has a
// required single `cause` field.
func encodeLeaf(ctx context.Context, err error, causes []error) EncodedError {
	var msg string
	var details errorspb.EncodedErrorDetails

	if e, ok := err.(*opaqueLeaf); ok {
		msg = e.msg
		details = e.details
	} else if e, ok := err.(*opaqueLeafCauses); ok {
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

	var cs []*EncodedError
	if len(causes) > 0 {
		cs = make([]*EncodedError, len(causes))
		for i, ee := range causes {
			ee := EncodeError(ctx, ee)
			cs[i] = &ee
		}
	}

	return EncodedError{
		Error: &errorspb.EncodedError_Leaf{
			Leaf: &errorspb.EncodedErrorLeaf{
				Message:          msg,
				Details:          details,
				MultierrorCauses: cs,
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
	messageType := Prefix

	if e, ok := err.(*opaqueWrapper); ok {
		// We delegate all knowledge of the error string
		// to the original encoder and do not try to re-engineer
		// the prefix out of the error. This helps maintain
		// backward compatibility with earlier versions of the
		// encoder which don't have any understanding of
		// error string ownership by the wrapper.
		msg = e.prefix
		details = e.details
		messageType = e.messageType
	} else {
		details.OriginalTypeName, details.ErrorTypeMark.FamilyName, details.ErrorTypeMark.Extension = getTypeDetails(err, false /*onlyFamily*/)

		var payload proto.Message

		// If we have a manually registered encoder, use that.
		typeKey := TypeKey(details.ErrorTypeMark.FamilyName)
		if enc, ok := encoders[typeKey]; ok {
			msg, details.ReportablePayload, payload, messageType = enc(ctx, err)
		} else {
			// No encoder.
			// In that case, we'll try to compute a message prefix
			// manually.
			msg, messageType = extractPrefix(err, cause)

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
				Cause:       EncodeError(ctx, cause),
				Message:     msg,
				Details:     details,
				MessageType: errorspb.MessageType(messageType),
			},
		},
	}
}

// extractPrefix extracts the prefix from a wrapper's error message.
// For example,
//
//	err := errors.New("bar")
//	err = errors.Wrap(err, "foo")
//	extractPrefix(err)
//
// returns "foo".
//
// If a presumed wrapper does not have a message prefix, it is assumed
// to override the entire error message and `extractPrefix` returns
// the entire message and the boolean `true` to signify that the causes
// should not be appended to it.
func extractPrefix(err, cause error) (string, MessageType) {
	causeSuffix := cause.Error()
	errMsg := err.Error()

	if strings.HasSuffix(errMsg, causeSuffix) {
		prefix := errMsg[:len(errMsg)-len(causeSuffix)]
		// If error msg matches exactly then this is a wrapper
		// with no message of its own.
		if len(prefix) == 0 {
			return "", Prefix
		}
		if strings.HasSuffix(prefix, ": ") {
			return prefix[:len(prefix)-2], Prefix
		}
	}
	// If we don't have the cause as a suffix, then we have
	// some other string as our error msg, preserve that and
	// mark as override
	return errMsg, FullMessage
}

func getTypeDetails(
	err error, onlyFamily bool,
) (origTypeName string, typeKeyFamily string, typeKeyExtension string) {
	// If we have received an error of type not known locally,
	// we still know its type name. Return that.
	switch t := err.(type) {
	case *opaqueLeaf:
		return t.details.OriginalTypeName, t.details.ErrorTypeMark.FamilyName, t.details.ErrorTypeMark.Extension
	case *opaqueLeafCauses:
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

// RegisterMultiCauseEncoder can be used to register new multi-cause
// error types to the library. Registered types will be encoded using
// their own Go type when an error is encoded. Multi-cause wrappers
// that have not been registered will be encoded using the
// opaqueWrapper type.
func RegisterMultiCauseEncoder(theType TypeKey, encoder MultiCauseEncoder) {
	// This implementation is a simple wrapper around `LeafEncoder`
	// because we implemented multi-cause error wrapper encoding into a
	// `Leaf` instead of a `Wrapper` for smoother backwards
	// compatibility support. Exposing this detail to consumers of the
	// API is confusing and hence avoided. The causes of the error are
	// encoded separately regardless of this encoder's implementation.
	RegisterLeafEncoder(theType, encoder)
}

// MultiCauseEncoder is to be provided (via RegisterMultiCauseEncoder
// above) by additional multi-cause wrapper types not yet known to this
// library. The encoder will automatically extract and encode the
// causes of this error by calling `Unwrap()` and expecting a slice of
// errors.
type MultiCauseEncoder = func(ctx context.Context, err error) (msg string, safeDetails []string, payload proto.Message)

// RegisterWrapperEncoder can be used to register new wrapper types to
// the library. Registered wrappers will be encoded using their own
// Go type when an error is encoded. Wrappers that have not been
// registered will be encoded using the opaqueWrapper type.
//
// Note: if the error type has been migrated from a previous location
// or a different type, ensure that RegisterTypeMigration() was called
// prior to RegisterWrapperEncoder().
func RegisterWrapperEncoder(theType TypeKey, encoder WrapperEncoder) {
	RegisterWrapperEncoderWithMessageType(
		theType,
		func(ctx context.Context, err error) (
			msgPrefix string,
			safeDetails []string,
			payload proto.Message,
			messageType MessageType,
		) {
			prefix, details, payload := encoder(ctx, err)
			return prefix, details, payload, messageType
		})
}

// RegisterWrapperEncoderWithMessageType can be used to register
// new wrapper types to the library. Registered wrappers will be
// encoded using their own Go type when an error is encoded. Wrappers
// that have not been registered will be encoded using the
// opaqueWrapper type.
//
// This function differs from RegisterWrapperEncoder by allowing the
// caller to explicitly decide whether the wrapper owns the entire
// error message or not. Otherwise, the relationship is inferred.
//
// Note: if the error type has been migrated from a previous location
// or a different type, ensure that RegisterTypeMigration() was called
// prior to RegisterWrapperEncoder().
func RegisterWrapperEncoderWithMessageType(
	theType TypeKey, encoder WrapperEncoderWithMessageType,
) {
	if encoder == nil {
		delete(encoders, theType)
	} else {
		encoders[theType] = encoder
	}
}

// WrapperEncoder is to be provided (via RegisterWrapperEncoder above)
// by additional wrapper types not yet known to this library.
type WrapperEncoder func(ctx context.Context, err error) (
	msgPrefix string,
	safeDetails []string,
	payload proto.Message,
)

// MessageType is used to encode information about an error message
// within a wrapper error type. This information is used to affect
// display logic.
type MessageType errorspb.MessageType

// Values below should match the ones in errorspb.MessageType for
// direct conversion.
const (
	// Prefix denotes an error message that should be prepended to the
	// message of its cause.
	Prefix MessageType = MessageType(errorspb.MessageType_PREFIX)
	// FullMessage denotes an error message that contains the text of its
	// causes and can be displayed standalone.
	FullMessage = MessageType(errorspb.MessageType_FULL_MESSAGE)
)

// WrapperEncoderWithMessageType is to be provided (via
// RegisterWrapperEncoderWithMessageType above) by additional wrapper
// types not yet known to this library. This encoder returns an
// additional enum which indicates whether the wrapper owns the error
// message completely instead of simply being a prefix with the error
// message of its causes appended to it. This information is encoded
// along with the prefix in order to provide context during error
// display.
type WrapperEncoderWithMessageType func(ctx context.Context, err error) (
	msgPrefix string,
	safeDetails []string,
	payload proto.Message,
	messageType MessageType,
)

// registry for RegisterWrapperType.
var encoders = map[TypeKey]WrapperEncoderWithMessageType{}
