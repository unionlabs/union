// Copyright 2017 The Cockroach Authors.
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

package apd

import "testing"

func TestConstWithPrecision(t *testing.T) {
	c := makeConstWithPrecision("123.456789")
	expected := []string{
		"1E+2",           // 0
		"1E+2",           // 1
		"1.2E+2",         // 2
		"123.5", "123.5", // 3, 4
		"123.45679", "123.45679", "123.45679", "123.45679", // 5..8
		"123.456789", "123.456789", "123.456789", // 9+
	}
	for i, e := range expected {
		if s := c.get(uint32(i)).String(); s != e {
			t.Errorf("%d: expected %s, got %s", i, e, s)
		}
	}
}
