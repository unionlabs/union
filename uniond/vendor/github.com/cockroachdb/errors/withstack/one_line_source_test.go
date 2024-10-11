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
	pkgErr "github.com/pkg/errors"
)

func TestOneLineSource(t *testing.T) {
	tt := testutils.T{T: t}

	simpleErr := errors.New("hello")
	testData := []error{
		withstack.WithStack(simpleErr),
		errbase.DecodeError(context.Background(), errbase.EncodeError(context.Background(), withstack.WithStack(simpleErr))),
		pkgErr.WithStack(simpleErr),
		errbase.DecodeError(context.Background(), errbase.EncodeError(context.Background(), pkgErr.WithStack(simpleErr))),
		pkgErr.New("woo"),
		errbase.DecodeError(context.Background(), errbase.EncodeError(context.Background(), pkgErr.New("woo"))),
	}

	for _, err := range testData {
		tt.Run(err.Error(), func(tt testutils.T) {
			file, line, fn, ok := withstack.GetOneLineSource(err)
			tt.CheckEqual(ok, true)
			tt.CheckEqual(file, "one_line_source_test.go")
			tt.CheckEqual(fn, "TestOneLineSource")
			tt.Check(line > 21)
			if tt.Failed() {
				tt.Logf("looking at: %+v", err)
			}
		})
	}
}

func TestOneLineSourceInner(t *testing.T) {
	tt := testutils.T{T: t}

	// makeErr creates an error where the source context is not this
	// test function.
	simpleErr := makeErr()

	// Make the error wrapped to add additional source context. The rest
	// of the test below will check that GetOneLineSource retrieves the
	// innermost context, not this one.
	testData := []error{
		withstack.WithStack(simpleErr),
		errbase.DecodeError(context.Background(), errbase.EncodeError(context.Background(), withstack.WithStack(simpleErr))),
		pkgErr.WithStack(simpleErr),
		errbase.DecodeError(context.Background(), errbase.EncodeError(context.Background(), pkgErr.WithStack(simpleErr))),
	}

	for _, err := range testData {
		file, line, fn, ok := withstack.GetOneLineSource(err)
		tt.CheckEqual(ok, true)
		tt.CheckEqual(file, "reportable_test.go")
		tt.Check(strings.HasPrefix(fn, "makeErr"))
		tt.Check(line > 21)
	}
}
