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

package contexttags

import (
	"context"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/logtags"
	"github.com/cockroachdb/redact"
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
	if err == nil {
		return nil
	}
	tags := logtags.FromContext(ctx)
	if tags == nil {
		return err
	}
	return &withContext{cause: err, tags: tags}
}

// GetContextTags retrieves the k/v pairs stored in the error.
// The sets are returned from outermost to innermost level of cause.
// The returned logtags.Buffer only know about the string
// representation of the values originally captured by the error.
func GetContextTags(err error) (res []*logtags.Buffer) {
	for e := err; e != nil; e = errbase.UnwrapOnce(e) {
		if w, ok := e.(*withContext); ok {
			b := w.tags
			// Ensure that the buffer does not contain any non-string.
			if hasNonStringValue(b) {
				b = convertToStringsOnly(b)
			}
			res = append(res, b)
		}
	}
	return res
}

func hasNonStringValue(b *logtags.Buffer) bool {
	for _, t := range b.Get() {
		v := t.Value()
		if v == nil {
			return true
		}
		if _, ok := v.(string); !ok {
			return true
		}
	}
	return false
}

func convertToStringsOnly(b *logtags.Buffer) (res *logtags.Buffer) {
	for _, t := range b.Get() {
		res = res.Add(t.Key(), t.ValueStr())
	}
	return res
}

func redactTags(b *logtags.Buffer) []string {
	res := make([]string, len(b.Get()))
	redactableTagsIterate(b, func(i int, r redact.RedactableString) {
		res[i] = r.Redact().StripMarkers()
	})
	return res
}

func redactableTagsIterate(b *logtags.Buffer, fn func(i int, s redact.RedactableString)) {
	var empty redact.SafeString
	for i, t := range b.Get() {
		k := t.Key()
		v := t.Value()
		eq := empty
		var val interface{} = empty
		if v != nil {
			if len(k) > 1 {
				eq = "="
			}
			val = v
		}
		res := redact.Sprintf("%s%s%v", redact.Safe(k), eq, val)
		fn(i, res)
	}
}
