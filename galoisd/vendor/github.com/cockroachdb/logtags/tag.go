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

import "fmt"

// Tag is a log tag, which has a string key and an arbitrary value.
// The value must support testing for equality. It can be nil.
type Tag struct {
	key   string
	value interface{}
}

// Key returns the key.
func (t *Tag) Key() string {
	return t.key
}

// Value returns the value.
func (t *Tag) Value() interface{} {
	return t.value
}

// ValueStr returns the value as a string.
func (t *Tag) ValueStr() string {
	if t.value == nil {
		return ""
	}
	switch v := t.value.(type) {
	case string:
		return v
	case fmt.Stringer:
		return v.String()
	default:
		return fmt.Sprint(t.value)
	}
}
