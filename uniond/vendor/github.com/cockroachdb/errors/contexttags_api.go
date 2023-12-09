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

	"github.com/cockroachdb/errors/contexttags"
	"github.com/cockroachdb/logtags"
)

// WithContextTags captures the k/v pairs stored in the context via the
// `logtags` package and annotates them on the error.
//
// Only the strong representation of values remains available. This is
// because the library cannot guarantee that the underlying value is
// preserved across the network. To avoid creating a stateful interface
// (where the user code needs to know whether an error has traveled
// through the network or not), the library restricts access to the
// value part as strings. See GetContextTags() below.
//
// Detail is shown:
// - via `errors.GetSafeDetails()`.
// - via `GetContextTags()` below.
// - when formatting with `%+v`.
// - in Sentry reports.
func WithContextTags(err error, ctx context.Context) error {
	return contexttags.WithContextTags(err, ctx)
}

// GetContextTags retrieves the k/v pairs stored in the error.
// The sets are returned from outermost to innermost level of cause.
// The returned logtags.Buffer only know about the string
// representation of the values originally captured by the error.
func GetContextTags(err error) []*logtags.Buffer { return contexttags.GetContextTags(err) }
