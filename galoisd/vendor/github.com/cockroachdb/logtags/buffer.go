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

import (
	"fmt"
	"strings"
)

const staticSlots = 4

// Buffer is an immutable list of Tags.
type Buffer struct {
	tags     []Tag
	prealloc [staticSlots]Tag
}

// SingleTagBuffer returns a Buffer with a single tag.
func SingleTagBuffer(key string, value interface{}) *Buffer {
	b := &Buffer{}
	b.init(1, 1)
	b.tags[0] = Tag{key: key, value: value}
	return b
}

// Get returns the tags, as a slice. This slice must not be modified.
func (b *Buffer) Get() []Tag {
	return b.tags
}

// GetTag returns the tag corresponding to the given key. If the tag doesn't
// exist, the bool return value will be false.
func (b *Buffer) GetTag(key string) (Tag, bool) {
	if b == nil {
		return Tag{}, false
	}
	for _, t := range b.tags {
		if t.Key() == key {
			return t, true
		}
	}
	return Tag{}, false
}

// Add returns a new buffer with one more tag. If the tag has the same key as an
// earlier tag, that tag is overwritten.
// The receiver can be nil.
func (b *Buffer) Add(key string, value interface{}) *Buffer {
	if b == nil {
		return SingleTagBuffer(key, value)
	}
	res := &Buffer{}
	res.init(len(b.tags), len(b.tags)+1)
	copy(res.tags, b.tags)
	res.addOrReplace(key, value)
	return res
}

// Remove returns a new Buffer with the tag with key `key` removed. If the tag
// does not exist, the receiver is returned (unchanged). The bool return value
// is true if the tag existed.
func (b *Buffer) Remove(key string) (*Buffer, bool) {
	if b == nil {
		return nil, false
	}
	for i, t := range b.tags {
		if t.Key() == key {
			res := &Buffer{}
			res.init(len(b.tags)-1, 0 /* capacityHint */)
			copy(res.tags, b.tags[:i])
			copy(res.tags[i:], b.tags[i+1:])
			return res, true
		}
	}
	return b, false
}

// Merge returns a new buffer which contains tags from the receiver, followed by
// the tags from <other>.
//
// If both buffers have the same tag, the tag will appear only one time, with
// the value it has in <other>. It can appear either in the place of the tag in
// <b> or the tag in <other> (depending on which is deemed more efficient).
//
// The method can return <b> or <other> if the result is identical with one of
// them.
//
// The receiver can be nil.
func (b *Buffer) Merge(other *Buffer) *Buffer {
	if b == nil || len(b.tags) == 0 {
		return other
	}
	if other == nil || len(other.tags) == 0 {
		return b
	}

	// Check for a common case where b's tags are a subsequence of the
	// other tags. In practice this happens when we start with an annotated
	// context and we annotate it again at a lower level (with more specific
	// information). Frequent examples seen in practice are when b has a node ID
	// tag and other has both a node and a store ID tag; and when b has a node ID
	// and store ID tag and other has node, store, and replica tags.
	if diff := len(other.tags) - len(b.tags); diff >= 0 {
		i, j := 0, 0
		for i < len(b.tags) && j-i <= diff {
			if b.tags[i].key == other.tags[j].key {
				i++
			}
			j++
		}
		if i == len(b.tags) {
			return other
		}
	}

	// Another common case is when we aren't adding any new tags or values; find
	// and ignore the longest prefix of redundant tags.
	i := 0
	for ; i < len(other.tags); i++ {
		idx := b.find(other.tags[i].key)
		if idx == -1 || b.tags[idx].value != other.tags[i].value {
			break
		}
	}
	if i == len(other.tags) {
		return b
	}

	res := &Buffer{}
	res.init(len(b.tags), len(b.tags)+len(other.tags)-i)
	copy(res.tags, b.tags)

	for ; i < len(other.tags); i++ {
		res.addOrReplace(other.tags[i].key, other.tags[i].value)
	}
	return res
}

// String returns a string representation of the tags.
func (b *Buffer) String() string {
	var buf strings.Builder
	b.FormatToString(&buf)
	return buf.String()
}

// FormatToString emits the k/v pairs to a strings.Builder.
//   - the k/v pairs are separated by commas (& no spaces).
//   - if there is no value, only the key is printed.
//   - if there is a value, and the key is just 1 character long,
//     the key and the value are concatenated.
//     This supports e.g. printing k="n", v=123 as "n123".
//   - otherwise, it prints "k=v".
func (b *Buffer) FormatToString(buf *strings.Builder) {
	comma := ""
	for _, t := range b.Get() {
		buf.WriteString(comma)
		buf.WriteString(t.Key())
		if v := t.Value(); v != nil && v != "" {
			if len(t.Key()) > 1 {
				buf.WriteByte('=')
			}
			fmt.Fprint(buf, v)
		}
		comma = ","
	}
}

// init initializes b.tags to a slice of len `length` (filled with zero values).
// The slice may have capacity capacityHint. If capacityHint is less than
// length, it is ignored.
func (b *Buffer) init(length, capacityHint int) {
	if length <= staticSlots {
		// Even if capacityHint is larger that staticSlots, we still want to try to
		// avoid the allocation (especially since tags frequently get deduplicated).
		b.tags = b.prealloc[:length]
	} else {
		if capacityHint < length {
			capacityHint = length
		}
		b.tags = make([]Tag, length, capacityHint)
	}
}

func (b *Buffer) addOrReplace(key string, value interface{}) {
	for i := range b.tags {
		if b.tags[i].key == key {
			b.tags[i].value = value
			return
		}
	}
	b.tags = append(b.tags, Tag{key: key, value: value})
}

// find returns the position of the tag with the given key, or -1 if
// it is not found.
func (b *Buffer) find(key string) int {
	for i := range b.tags {
		if b.tags[i].key == key {
			return i
		}
	}
	return -1
}
