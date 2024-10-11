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

package fmtforward

import (
	"fmt"
	"testing"
)

// TestMakeFormat verifies that the makeFormat() helper is able to
// reproduce the format given as input to fmt function.
func TestMakeFormat(t *testing.T) {
	testData := []string{
		"%c", "%v", "%q",
		"%3f", "%.3f", "%2.3f",
		"%# v", "%012s",
		"%+v", "%-12s",
	}

	for _, test := range testData {
		justV, revFmt := getFormat(test)
		if (test == "%v") != justV {
			t.Errorf("%q: expected justV %v, got %v", test, test == "%v", justV)
		}
		if revFmt != test {
			t.Errorf("%q: got %q instead", test, revFmt)
		}
	}
}

type formatTester struct {
	fn func(fmt.State, rune)
}

func (f formatTester) Format(s fmt.State, verb rune) {
	f.fn(s, verb)
}

func getFormat(testFmt string) (justV bool, revFmt string) {
	f := formatTester{func(s fmt.State, verb rune) {
		justV, revFmt = MakeFormat(s, verb)
	}}
	_ = fmt.Sprintf(testFmt, f)
	return
}
