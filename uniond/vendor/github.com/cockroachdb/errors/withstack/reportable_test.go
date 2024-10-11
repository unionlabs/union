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

package withstack_test

import (
	"context"
	"errors"
	"strings"
	"testing"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/testutils"
	"github.com/cockroachdb/errors/withstack"
	"github.com/cockroachdb/errors/withstack/internal"
	"github.com/kr/pretty"
	pkgErr "github.com/pkg/errors"
)

func TestReportableStackTrace(t *testing.T) {
	baseErr := errors.New("hello")

	t.Run("pkgErr", func(t *testing.T) {
		err := internal.Run(func() error { return pkgErr.WithStack(baseErr) })
		t.Run("local", func(t *testing.T) {
			checkStackTrace(t, err, 0)
		})
		enc := errbase.EncodeError(context.Background(), err)
		err = errbase.DecodeError(context.Background(), enc)
		t.Run("remote", func(t *testing.T) {
			checkStackTrace(t, err, 0)
		})
	})

	t.Run("pkgFundamental", func(t *testing.T) {
		err := internal.Run(func() error { return pkgErr.New("hello") })
		t.Run("local", func(t *testing.T) {
			checkStackTrace(t, err, 0)
		})
		enc := errbase.EncodeError(context.Background(), err)
		err = errbase.DecodeError(context.Background(), enc)
		t.Run("remote", func(t *testing.T) {
			checkStackTrace(t, err, 0)
		})
	})

	t.Run("withStack", func(t *testing.T) {
		err := internal.Run(func() error { return withstack.WithStack(baseErr) })
		t.Run("local", func(t *testing.T) {
			checkStackTrace(t, err, 0)
		})
		enc := errbase.EncodeError(context.Background(), err)
		err = errbase.DecodeError(context.Background(), enc)
		t.Run("remote", func(t *testing.T) {
			checkStackTrace(t, err, 0)
		})
	})

	t.Run("withStack depth", func(t *testing.T) {
		err := internal.Run(makeErr)
		checkStackTrace(t, err, 1)
	})
	t.Run("withStack nontrival depth", func(t *testing.T) {
		err := internal.Run(makeErr3)
		checkStackTrace(t, err, 0)
	})
}

func makeErr() error  { return makeErr2() }
func makeErr2() error { return withstack.WithStack(errors.New("")) }

func makeErr3() error { return makeErr4() }
func makeErr4() error { return withstack.WithStackDepth(errors.New(""), 1) }

func checkStackTrace(t *testing.T, err error, expectedDepth int) {
	tt := testutils.T{T: t}

	t.Logf("looking at err %# v", pretty.Formatter(err))

	r := withstack.GetReportableStackTrace(err)
	tt.Assert(r != nil)

	// We're expecting the Run() functions in second position.
	tt.Assert(len(r.Frames) >= expectedDepth+2)

	for i, f := range r.Frames {
		t.Logf("frame %d:", i)
		t.Logf("absolute path: %s", f.AbsPath)
		t.Logf("file: %s", f.Filename)
		t.Logf("line: %d", f.Lineno)
		t.Logf("module: %s", f.Module)
		t.Logf("function: %s", f.Function)
	}

	// The reportable frames are in reversed order. For the test,
	// we want to look at them in the "good" order.
	for i, j := 0, len(r.Frames)-1; i < j; i, j = i+1, j-1 {
		r.Frames[i], r.Frames[j] = r.Frames[j], r.Frames[i]
	}

	for i := expectedDepth; i < expectedDepth+2; i++ {
		f := r.Frames[i]
		tt.Check(strings.Contains(f.Filename, "/errors/") || strings.Contains(f.Filename, "/errors@"))

		tt.Check(strings.HasSuffix(f.AbsPath, f.Filename))

		switch i {
		case expectedDepth:
			tt.Check(strings.HasSuffix(f.Filename, "reportable_test.go"))

		case expectedDepth + 1, expectedDepth + 2:
			tt.Check(strings.HasSuffix(f.Filename, "internal/run.go"))

			tt.Check(strings.HasSuffix(f.Module, "withstack/internal"))

			tt.Check(strings.HasPrefix(f.Function, "Run"))
		}
	}

	// Check that Run2() is after Run() in the source code.
	tt.Check(r.Frames[expectedDepth+1].Lineno != 0 &&
		r.Frames[expectedDepth+2].Lineno != 0 &&
		(r.Frames[expectedDepth+1].Lineno > r.Frames[expectedDepth+2].Lineno))
}
