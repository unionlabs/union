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

package rfmt

import (
	"reflect"
	"testing"

	w "github.com/cockroachdb/redact/internal/redact"
)

func TestCustomSafeTypes(t *testing.T) {
	defer func(prev map[reflect.Type]bool) { safeTypeRegistry = prev }(safeTypeRegistry)
	RegisterSafeType(reflect.TypeOf(int32(123)))

	// Also a struct containing the safe type.
	type s struct {
		v int32
	}
	x := s{v: 999}

	actual := Sprint(123,
		int32(456),
		x,
		reflect.ValueOf(int32(789)),
		reflect.ValueOf(x),
	)
	const expected = `‹123› 456 {999} 789 {999}`
	if actual != expected {
		t.Errorf("expected %q, got %q", expected, actual)
	}

	// Unsafe can override.
	actual = Sprint(123,
		w.Unsafe(int32(456)),
		w.Unsafe(x),
		w.Unsafe(reflect.ValueOf(int32(789))),
		w.Unsafe(reflect.ValueOf(x)),
	)
	const expected2 = `‹123› ‹456› ‹{999}› ‹789› ‹{999}›`
	if actual != expected2 {
		t.Errorf("expected %q, got %q", expected2, actual)
	}

	// Safe can override unsafe.
	actual = Sprint(123,
		w.Safe(w.Unsafe(int32(456))),
		w.Safe(w.Unsafe(x)),
		w.Safe(w.Unsafe(reflect.ValueOf(int32(789)))),
		w.Safe(w.Unsafe(reflect.ValueOf(x))),
	)
	const expected3 = `‹123› 456 {999} 789 {999}`
	if actual != expected3 {
		t.Errorf("expected %q, got %q", expected3, actual)
	}
}
