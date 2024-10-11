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

package testutils

import "testing"

func id(x int) int { return x }

func TestAssert(t *testing.T) {
	tt := T{T: t}

	tt.Run("the-test", func(t T) {
		t.Check(id(1) == 1)
		t.CheckEqual(1, id(1))
		t.CheckDeepEqual(1, id(1))
		t.Assert(id(1) == 1)
		t.AssertEqual(1, id(1))
		t.AssertDeepEqual(1, id(1))
		t.CheckRegexpEqual("hello", "h.*o")
	})
}
