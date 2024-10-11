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

package exthttp

import (
	"context"
	"fmt"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/markers"
	"github.com/gogo/protobuf/proto"
)

// This file demonstrates how to add a wrapper type not otherwise
// known to the rest of the library.

// withHTTPCode is our wrapper type.
type withHTTPCode struct {
	cause error
	code  int
}

// WrapWithHTTPCode adds a HTTP code to an existing error.
func WrapWithHTTPCode(err error, code int) error {
	if err == nil {
		return nil
	}
	return &withHTTPCode{cause: err, code: code}
}

// GetHTTPCode retrieves the HTTP code from a stack of causes.
func GetHTTPCode(err error, defaultCode int) int {
	if v, ok := markers.If(err, func(err error) (interface{}, bool) {
		if w, ok := err.(*withHTTPCode); ok {
			return w.code, true
		}
		return nil, false
	}); ok {
		return v.(int)
	}
	return defaultCode
}

// it's an error.
func (w *withHTTPCode) Error() string { return w.cause.Error() }

// it's also a wrapper.
func (w *withHTTPCode) Cause() error  { return w.cause }
func (w *withHTTPCode) Unwrap() error { return w.cause }

// it knows how to format itself.
func (w *withHTTPCode) Format(s fmt.State, verb rune) { errors.FormatError(w, s, verb) }

// SafeFormatter implements errors.SafeFormatter.
// Note: see the documentation of errbase.SafeFormatter for details
// on how to implement this. In particular beware of not emitting
// unsafe strings.
func (w *withHTTPCode) SafeFormatError(p errors.Printer) (next error) {
	if p.Detail() {
		p.Printf("http code: %d", w.code)
	}
	return w.cause
}

// it's an encodable error.
func encodeWithHTTPCode(_ context.Context, err error) (string, []string, proto.Message) {
	w := err.(*withHTTPCode)
	details := []string{fmt.Sprintf("HTTP %d", w.code)}
	payload := &EncodedHTTPCode{Code: uint32(w.code)}
	return "", details, payload
}

// it's a decodable error.
func decodeWithHTTPCode(
	_ context.Context, cause error, _ string, _ []string, payload proto.Message,
) error {
	wp := payload.(*EncodedHTTPCode)
	return &withHTTPCode{cause: cause, code: int(wp.Code)}
}

func init() {
	errbase.RegisterWrapperEncoder(errbase.GetTypeKey((*withHTTPCode)(nil)), encodeWithHTTPCode)
	errbase.RegisterWrapperDecoder(errbase.GetTypeKey((*withHTTPCode)(nil)), decodeWithHTTPCode)
}
