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

import "github.com/cockroachdb/errors/errbase"

// WithTelemetry annotates err with the given telemetry key(s).
// The telemetry keys must be PII-free.
//
// Detail is shown:
// - via `errors.GetSafeDetails()`.
// - via `GetTelemetryKeys()` below.
// - when formatting with `%+v`.
// - in Sentry reports.
func WithTelemetry(err error, keys ...string) error {
	if err == nil {
		return nil
	}

	return &withTelemetry{cause: err, keys: keys}
}

// GetTelemetryKeys retrieves the (de-duplicated) set of
// all telemetry keys present in the direct causal chain
// of the error. The keys may not be sorted.
func GetTelemetryKeys(err error) []string {
	keys := map[string]struct{}{}
	for ; err != nil; err = errbase.UnwrapOnce(err) {
		if w, ok := err.(*withTelemetry); ok {
			for _, k := range w.keys {
				keys[k] = struct{}{}
			}
		}
	}
	res := make([]string, 0, len(keys))
	for k := range keys {
		res = append(res, k)
	}
	return res
}
