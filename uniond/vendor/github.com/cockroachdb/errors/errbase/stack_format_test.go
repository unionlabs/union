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
	"fmt"
	"regexp"
	"strings"
	"testing"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/testutils"
	pkgErr "github.com/pkg/errors"
)

func TestSimplifyStacks(t *testing.T) {
	leaf := func() error {
		return pkgErr.New("hello world")
	}
	wrapper := func() error {
		err := leaf()
		return pkgErr.WithStack(err)
	}
	errWrapper := wrapper()
	t.Logf("error: %+v", errWrapper)

	t.Run("low level API", func(t *testing.T) {
		tt := testutils.T{t}
		// Extract the stack trace from the leaf.
		errLeaf := errbase.UnwrapOnce(errWrapper)
		leafP, ok := errLeaf.(errbase.StackTraceProvider)
		if !ok {
			t.Fatal("leaf error does not provide stack trace")
		}
		leafT := leafP.StackTrace()
		spv := fmtClean(leafT)
		t.Logf("-- leaf trace --%+v", spv)
		if !strings.Contains(spv, "TestSimplifyStacks") {
			t.Fatalf("expected test function in trace, got:%v", spv)
		}
		leafLines := strings.Split(spv, "\n")

		// Extract the stack trace from the wrapper.
		wrapperP, ok := errWrapper.(errbase.StackTraceProvider)
		if !ok {
			t.Fatal("wrapper error does not provide stack trace")
		}
		wrapperT := wrapperP.StackTrace()
		spv = fmtClean(wrapperT)
		t.Logf("-- wrapper trace --%+v", spv)
		wrapperLines := strings.Split(spv, "\n")

		// Sanity check before we verify the result.
		tt.Check(len(wrapperLines) > 0)
		tt.CheckDeepEqual(wrapperLines[3:], leafLines[5:])

		// Elide the suffix and verify that we arrive to the same result.
		simplified, hasElided := errbase.ElideSharedStackTraceSuffix(leafT, wrapperT)
		spv = fmtClean(simplified)
		t.Logf("-- simplified (%v) --%+v", hasElided, spv)
		simplifiedLines := strings.Split(spv, "\n")
		tt.CheckDeepEqual(simplifiedLines, wrapperLines[0:3])
	})

	t.Run("high level API", func(t *testing.T) {
		tt := testutils.T{t}

		spv := fmtClean(errbase.Formattable(errWrapper))
		tt.CheckStringEqual(spv, `hello world
(1)
  -- stack trace:
  | github.com/cockroachdb/errors/errbase_test.TestSimplifyStacks.func2
  | <tab><path>:<lineno>
  | [...repeated from below...]
Wraps: (2) hello world
  | github.com/cockroachdb/errors/errbase_test.TestSimplifyStacks.func1
  | <tab><path>:<lineno>
  | github.com/cockroachdb/errors/errbase_test.TestSimplifyStacks.func2
  | <tab><path>:<lineno>
  | github.com/cockroachdb/errors/errbase_test.TestSimplifyStacks
  | <tab><path>:<lineno>
  | testing.tRunner
  | <tab><path>:<lineno>
  | runtime.goexit
  | <tab><path>:<lineno>
Error types: (1) *errors.withStack (2) *errors.fundamental`)
	})
}

func fmtClean(x interface{}) string {
	spv := fmt.Sprintf("%+v", x)
	spv = fileref.ReplaceAllString(spv, "<path>:<lineno>")
	spv = strings.ReplaceAll(spv, "\t", "<tab>")
	return spv
}

var fileref = regexp.MustCompile(`([a-zA-Z0-9\._/@-]*\.(?:go|s):\d+)`)
