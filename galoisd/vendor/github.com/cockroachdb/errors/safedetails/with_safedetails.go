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

package safedetails

import (
	"context"
	"fmt"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/redact"
	"github.com/gogo/protobuf/proto"
)

type withSafeDetails struct {
	cause error

	safeDetails []string
}

func (e *withSafeDetails) SafeDetails() []string {
	return e.safeDetails
}

var _ fmt.Formatter = (*withSafeDetails)(nil)
var _ errbase.SafeFormatter = (*withSafeDetails)(nil)

// Printing a withSecondary reveals the details.
func (e *withSafeDetails) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }

// SafeFormatError implements errbase.SafeFormatter.
func (e *withSafeDetails) SafeFormatError(p errbase.Printer) error {
	if p.Detail() {
		comma := redact.SafeString("")
		if len(e.safeDetails) != 1 {
			plural := redact.SafeString("s")
			if len(e.safeDetails) == 1 {
				plural = ""
			}
			p.Printf("%d safe detail%s enclosed", redact.Safe(len(e.safeDetails)), plural)
			comma = "\n"
		}
		// We hide the details from %+v; they are included
		// during Sentry reporting.
		for _, s := range e.safeDetails {
			p.Printf("%s%s", comma, redact.Safe(s))
			comma = "\n"
		}
	}
	return e.cause
}

func (e *withSafeDetails) Error() string { return e.cause.Error() }
func (e *withSafeDetails) Cause() error  { return e.cause }
func (e *withSafeDetails) Unwrap() error { return e.cause }

func decodeWithSafeDetails(
	_ context.Context, cause error, _ string, safeDetails []string, _ proto.Message,
) error {
	return &withSafeDetails{cause: cause, safeDetails: safeDetails}
}

func init() {
	tn := errbase.GetTypeKey((*withSafeDetails)(nil))
	errbase.RegisterWrapperDecoder(tn, decodeWithSafeDetails)
	// Note: no encoder needed, the default implementation is suitable.
}
