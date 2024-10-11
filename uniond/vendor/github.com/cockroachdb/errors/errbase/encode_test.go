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

package errbase_test

import (
	"testing"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/testutils"
)

type myE struct{ marker string }

func (e *myE) Error() string { return "woo" }

func (e *myE) ErrorKeyMarker() string { return e.marker }

var _ errbase.TypeKeyMarker = (*myE)(nil)

// This test shows how the extended type marker changes the visible
// type and thus the identity of an error.
func TestTypeName(t *testing.T) {
	err1 := &myE{"woo"}
	err2 := &myE{""}

	tn1 := errbase.GetTypeMark(err1)
	tn2 := errbase.GetTypeMark(err2)

	tt := testutils.T{T: t}

	tt.Check(!tn1.Equals(tn2))
	tt.CheckEqual(tn1.FamilyName, tn2.FamilyName)
	tt.Check(tn1.Extension != tn2.Extension)
}
