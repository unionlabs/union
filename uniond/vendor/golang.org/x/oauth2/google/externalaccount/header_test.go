// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package externalaccount

import (
	"runtime"
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestGoVersion(t *testing.T) {
	testVersion := func(v string) func() string {
		return func() string {
			return v
		}
	}
	for _, tst := range []struct {
		v    func() string
		want string
	}{
		{
			testVersion("go1.19"),
			"1.19.0",
		},
		{
			testVersion("go1.21-20230317-RC01"),
			"1.21.0-20230317-RC01",
		},
		{
			testVersion("devel +abc1234"),
			"abc1234",
		},
		{
			testVersion("this should be unknown"),
			versionUnknown,
		},
	} {
		version = tst.v
		got := goVersion()
		if diff := cmp.Diff(got, tst.want); diff != "" {
			t.Errorf("got(-),want(+):\n%s", diff)
		}
	}
	version = runtime.Version
}
