// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.20

package errgroup_test

import (
	"context"
	"errors"
	"testing"

	"golang.org/x/sync/errgroup"
)

func TestCancelCause(t *testing.T) {
	errDoom := errors.New("group_test: doomed")

	cases := []struct {
		errs []error
		want error
	}{
		{want: nil},
		{errs: []error{nil}, want: nil},
		{errs: []error{errDoom}, want: errDoom},
		{errs: []error{errDoom, nil}, want: errDoom},
	}

	for _, tc := range cases {
		g, ctx := errgroup.WithContext(context.Background())

		for _, err := range tc.errs {
			err := err
			g.TryGo(func() error { return err })
		}

		if err := g.Wait(); err != tc.want {
			t.Errorf("after %T.TryGo(func() error { return err }) for err in %v\n"+
				"g.Wait() = %v; want %v",
				g, tc.errs, err, tc.want)
		}

		if tc.want == nil {
			tc.want = context.Canceled
		}

		if err := context.Cause(ctx); err != tc.want {
			t.Errorf("after %T.TryGo(func() error { return err }) for err in %v\n"+
				"context.Cause(ctx) = %v; tc.want %v",
				g, tc.errs, err, tc.want)
		}
	}
}
