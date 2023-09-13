// Copyright 2018 The Cockroach Authors.
//
// Use of this software is governed by the Business Source License included
// in the file licenses/BSL.txt and at www.mariadb.com/bsl11.
//
// Change Date: 2022-10-01
//
// On the date above, in accordance with the Business Source License, use
// of this software will be governed by the Apache License, Version 2.0,
// included in the file licenses/APL.txt and at
// https://www.apache.org/licenses/LICENSE-2.0

package logtags

import "context"

// contextLogTagsKey is an empty type for the handle associated with the log
// tags (*Buffer) value (see context.Value).
type contextLogTagsKey struct{}

// FromContext returns the tags stored in the context (by WithTags, AddTag, or
// AddTags).
func FromContext(ctx context.Context) *Buffer {
	val := ctx.Value(contextLogTagsKey{})
	if val == nil {
		return nil
	}
	return val.(*Buffer)
}

// WithTags returns a context with the given tags. Any existing tags are
// ignored.
func WithTags(ctx context.Context, tags *Buffer) context.Context {
	return context.WithValue(ctx, contextLogTagsKey{}, tags)
}

// AddTag returns a context that has the tags in the given context plus another
// tag. Tags are deduplicated (see Buffer.AddTag).
func AddTag(ctx context.Context, key string, value interface{}) context.Context {
	b := FromContext(ctx)
	return WithTags(ctx, b.Add(key, value))
}

// RemoveTag returns a context that has the tags in the given context except the
// tag with key `key`. If such a tag does not exist, the given context is
// returned.
func RemoveTag(ctx context.Context, key string) context.Context {
	b := FromContext(ctx)
	newB, ok := b.Remove(key)
	if !ok {
		return ctx
	}
	return WithTags(ctx, newB)
}

// AddTags returns a context that has the tags in the given context plus another
// set of tags. Tags are deduplicated (see Buffer.AddTags).
func AddTags(ctx context.Context, tags *Buffer) context.Context {
	b := FromContext(ctx)
	newB := b.Merge(tags)
	if newB == b {
		return ctx
	}
	return WithTags(ctx, newB)
}
