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

package secondary

import (
	"context"
	"fmt"

	"github.com/cockroachdb/errors/errbase"
	"github.com/gogo/protobuf/proto"
)

type withSecondaryError struct {
	cause error

	// secondaryError is an additional error payload that provides
	// additional context towards troubleshooting.
	secondaryError error
}

var _ error = (*withSecondaryError)(nil)
var _ errbase.SafeDetailer = (*withSecondaryError)(nil)
var _ fmt.Formatter = (*withSecondaryError)(nil)
var _ errbase.SafeFormatter = (*withSecondaryError)(nil)

// SafeDetails reports the PII-free details from the secondary error.
func (e *withSecondaryError) SafeDetails() []string {
	var details []string
	for err := e.secondaryError; err != nil; err = errbase.UnwrapOnce(err) {
		sd := errbase.GetSafeDetails(err)
		details = sd.Fill(details)
	}
	return details
}

// Printing a withSecondary reveals the details.
func (e *withSecondaryError) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }

func (e *withSecondaryError) SafeFormatError(p errbase.Printer) (next error) {
	if p.Detail() {
		p.Printf("secondary error attachment\n%+v", e.secondaryError)
	}
	return e.cause
}

func (e *withSecondaryError) Error() string { return e.cause.Error() }
func (e *withSecondaryError) Cause() error  { return e.cause }
func (e *withSecondaryError) Unwrap() error { return e.cause }

func encodeWithSecondaryError(ctx context.Context, err error) (string, []string, proto.Message) {
	e := err.(*withSecondaryError)
	enc := errbase.EncodeError(ctx, e.secondaryError)
	return "", nil, &enc
}

func decodeWithSecondaryError(
	ctx context.Context, cause error, _ string, _ []string, payload proto.Message,
) error {
	enc, ok := payload.(*errbase.EncodedError)
	if !ok {
		// If this ever happens, this means some version of the library
		// (presumably future) changed the payload type, and we're
		// receiving this here. In this case, give up and let
		// DecodeError use the opaque type.
		return nil
	}
	return &withSecondaryError{
		cause:          cause,
		secondaryError: errbase.DecodeError(ctx, *enc),
	}
}

func init() {
	tn := errbase.GetTypeKey((*withSecondaryError)(nil))
	errbase.RegisterWrapperDecoder(tn, decodeWithSecondaryError)
	errbase.RegisterWrapperEncoder(tn, encodeWithSecondaryError)
}
