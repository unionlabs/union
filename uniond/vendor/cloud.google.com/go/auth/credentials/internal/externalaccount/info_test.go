// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
