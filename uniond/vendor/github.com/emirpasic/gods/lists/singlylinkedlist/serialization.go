// Copyright (c) 2015, Emir Pasic. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package singlylinkedlist

import (
	"encoding/json"
	"github.com/emirpasic/gods/containers"
)

// Assert Serialization implementation
var _ containers.JSONSerializer = (*List)(nil)
var _ containers.JSONDeserializer = (*List)(nil)

// ToJSON outputs the JSON representation of list's elements.
func (list *List) ToJSON() ([]byte, error) {
	return json.Marshal(list.Values())
}

// FromJSON populates list's elements from the input JSON representation.
func (list *List) FromJSON(data []byte) error {
	elements := []interface{}{}
	err := json.Unmarshal(data, &elements)
	if err == nil {
		list.Clear()
		list.Add(elements...)
	}
	return err
}

// UnmarshalJSON @implements json.Unmarshaler
func (list *List) UnmarshalJSON(bytes []byte) error {
	return list.FromJSON(bytes)
}

// MarshalJSON @implements json.Marshaler
func (list *List) MarshalJSON() ([]byte, error) {
	return list.ToJSON()
}
