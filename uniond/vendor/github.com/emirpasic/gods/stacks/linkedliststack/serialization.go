// Copyright (c) 2015, Emir Pasic. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package linkedliststack

import (
	"github.com/emirpasic/gods/containers"
)

// Assert Serialization implementation
var _ containers.JSONSerializer = (*Stack)(nil)
var _ containers.JSONDeserializer = (*Stack)(nil)

// ToJSON outputs the JSON representation of the stack.
func (stack *Stack) ToJSON() ([]byte, error) {
	return stack.list.ToJSON()
}

// FromJSON populates the stack from the input JSON representation.
func (stack *Stack) FromJSON(data []byte) error {
	return stack.list.FromJSON(data)
}

// UnmarshalJSON @implements json.Unmarshaler
func (stack *Stack) UnmarshalJSON(bytes []byte) error {
	return stack.FromJSON(bytes)
}

// MarshalJSON @implements json.Marshaler
func (stack *Stack) MarshalJSON() ([]byte, error) {
	return stack.ToJSON()
}
