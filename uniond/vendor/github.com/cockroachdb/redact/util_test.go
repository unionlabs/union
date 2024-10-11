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
	"testing"

	"github.com/cockroachdb/redact/builder"
)

func TestSort(t *testing.T) {
	v := []RedactableString{"c", "a", "b"}
	SortStrings(v)
	exp := []RedactableString{"a", "b", "c"}
	if !reflect.DeepEqual(v, exp) {
		t.Errorf("expected %+v, got %+v", exp, v)
	}
}

func TestJoin(t *testing.T) {
	v := []RedactableString{"c", "a", "b"}
	exp := RedactableString("c, a, b")
	act := Join(", ", v)
	if exp != act {
		t.Errorf("expected %q, got %q", exp, act)
	}
}

func TestJoinTo(t *testing.T) {
	testCases := []struct {
		v   interface{}
		exp RedactableString
	}{
		{[]int{1, 2, 3}, `‹1›, ‹2›, ‹3›`},
		{[]string{"unsafe", "wo›rld"}, `‹unsafe›, ‹wo?rld›`},
		{[]RedactableString{"a", "‹b›", "c"}, `a, ‹b›, c`},
		{[]SafeString{"a", "b", "c"}, `a, b, c`},
	}

	for _, tc := range testCases {
		var b builder.StringBuilder
		JoinTo(&b, ", ", tc.v)
		act := b.RedactableString()
		if act != tc.exp {
			t.Errorf("expected %q, got %q", tc.exp, act)
		}
	}
}
