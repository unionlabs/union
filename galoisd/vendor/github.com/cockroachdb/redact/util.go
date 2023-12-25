// Copyright 2020 The Cockroach Authors.
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

package redact

import (
	"reflect"
	"sort"

	"github.com/cockroachdb/redact/builder"
)

// JoinTo writes the given slice of values delimited by the provided
// delimiter to the given SafeWriter.
func JoinTo(w SafeWriter, delim RedactableString, values interface{}) {
	// Ideally, we'd make the interface clearer: JoinTo is a generic function,
	// with a "values" argument of type []T where T is a type variable.
	v := reflect.ValueOf(values)
	if v.Kind() != reflect.Slice {
		// Note a slice: just print the value as-is.
		w.Print(values)
	}
	for i, l := 0, v.Len(); i < l; i++ {
		if i > 0 {
			w.Print(delim)
		}
		w.Print(v.Index(i).Interface())
	}
}

// Join creates a redactable string from the
// given slice of redactable strings, adjoined
// with the provided delimiter.
func Join(delim RedactableString, s []RedactableString) RedactableString {
	var b builder.StringBuilder
	JoinTo(&b, delim, s)
	return b.RedactableString()
}

// SortStrings sorts the provided slice of redactable strings.
func SortStrings(s []RedactableString) {
	sort.Sort(sortableSlice(s))
}

// sortableSlice attaches the methods of sort.Interface to
// []RedactableString, sorting in increasing order.
type sortableSlice []RedactableString

func (p sortableSlice) Len() int           { return len(p) }
func (p sortableSlice) Less(i, j int) bool { return p[i] < p[j] }
func (p sortableSlice) Swap(i, j int)      { p[i], p[j] = p[j], p[i] }
