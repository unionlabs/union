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

package errutil

import "github.com/cockroachdb/redact"

// WithMessage annotates err with a new message.
// If err is nil, WithMessage returns nil.
// The message is considered safe for reporting
// and is included in Sentry reports.
func WithMessage(err error, message string) error {
	if err == nil {
		return nil
	}
	return &withPrefix{
		cause:  err,
		prefix: redact.Sprint(redact.Safe(message)),
	}
}

// WithMessagef annotates err with the format specifier.
// If err is nil, WithMessagef returns nil.
// The message is formatted as per redact.Sprintf,
// to separate safe and unsafe strings for Sentry reporting.
func WithMessagef(err error, format string, args ...interface{}) error {
	if err == nil {
		return nil
	}
	return &withPrefix{
		cause:  err,
		prefix: redact.Sprintf(format, args...),
	}
}
