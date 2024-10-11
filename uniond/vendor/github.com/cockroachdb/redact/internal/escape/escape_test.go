// Copyright 2021 The Cockroach Authors.
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

package escape

import "testing"

func TestInternalEscape(t *testing.T) {
	testCases := []struct {
		input    []byte
		start    int
		bnl      bool
		strip    bool
		expected string
	}{
		{nil, 0, false, false, ""},
		{[]byte(""), 0, false, false, ""},
		{[]byte("abc"), 0, false, false, "abc"},
		{[]byte("‹abc›"), 0, false, false, "?abc?"},
		{[]byte("‹abc›"), 3, false, false, "‹abc?"},
		{[]byte("‹abc›def›ghi"), 3, false, false, "‹abc?def?ghi"},
		{[]byte("‹abc›"), len([]byte("‹abc›")), false, false, "‹abc›"},
		{[]byte("‹abc›‹def›"), len([]byte("‹abc›")), false, false, "‹abc›?def?"},
		{[]byte("‹abc›\n‹d\nef›"), len([]byte("‹abc›")), false, false, "‹abc›\n?d\nef?"},
		{[]byte("abc\n‹d\nef›\n \n\n "), len([]byte("abc")), true, false, "abc›\n‹?d›\n‹ef?›\n‹ ›\n\n‹ "},
		{[]byte("abc\n‹d\nef›\n \n\n "), len([]byte("abc")), true, true, "abc›\n‹?d›\n‹ef?"},
		{[]byte("‹abc› ‹def›"), len([]byte("‹abc› ")), true, true, "‹abc› ?def?"},
		{[]byte("abc‹\ndef"), len([]byte("abc‹")), true, true, "abc\n‹def"},
	}

	for _, tc := range testCases {
		actual := string(InternalEscapeBytes(tc.input, tc.start, tc.bnl, tc.strip))
		if actual != tc.expected {
			t.Errorf("%q/%d: expected %q, got %q", string(tc.input), tc.start, tc.expected, actual)
		}
	}
}
