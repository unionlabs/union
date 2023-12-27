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

import "github.com/cockroachdb/errors/telemetrykeys"

// WithTelemetry annotates err with the given telemetry key(s).
// The telemetry keys must be PII-free.
//
// Detail is shown:
// - via `errors.GetSafeDetails()`.
// - via `GetTelemetryKeys()` below.
// - when formatting with `%+v`.
// - in Sentry reports.
func WithTelemetry(err error, keys ...string) error { return telemetrykeys.WithTelemetry(err, keys...) }

// GetTelemetryKeys retrieves the (de-duplicated) set of
// all telemetry keys present in the direct causal chain
// of the error. The keys may not be sorted.
func GetTelemetryKeys(err error) []string { return telemetrykeys.GetTelemetryKeys(err) }
