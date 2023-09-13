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

package telemetrykeys

import (
	"context"
	"fmt"
	"strings"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/redact"
	"github.com/gogo/protobuf/proto"
)

type withTelemetry struct {
	cause error
	keys  []string
}

var _ error = (*withTelemetry)(nil)
var _ errbase.SafeDetailer = (*withTelemetry)(nil)
var _ fmt.Formatter = (*withTelemetry)(nil)
var _ errbase.SafeFormatter = (*withTelemetry)(nil)

func (w *withTelemetry) Error() string { return w.cause.Error() }
func (w *withTelemetry) Cause() error  { return w.cause }
func (w *withTelemetry) Unwrap() error { return w.cause }

func (w *withTelemetry) SafeDetails() []string { return w.keys }

func (w *withTelemetry) Format(s fmt.State, verb rune) { errbase.FormatError(w, s, verb) }

func (w *withTelemetry) SafeFormatError(p errbase.Printer) (next error) {
	if p.Detail() {
		p.Printf("keys: [%s]", redact.Safe(strings.Join(w.keys, " ")))
	}
	return w.cause
}

func decodeWithTelemetry(
	_ context.Context, cause error, _ string, keys []string, _ proto.Message,
) error {
	return &withTelemetry{cause: cause, keys: keys}
}

func init() {
	errbase.RegisterWrapperDecoder(errbase.GetTypeKey((*withTelemetry)(nil)), decodeWithTelemetry)
}
