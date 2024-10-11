// Copyright 2022 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package procfs

import (
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestSysctlInts(t *testing.T) {
	fs := getProcFixtures(t)

	for _, tc := range []struct {
		sysctl string
		want   []int
	}{
		{"kernel.random.entropy_avail", []int{3943}},
		{"vm.lowmem_reserve_ratio", []int{256, 256, 32, 0, 0}},
	} {
		t.Run(tc.sysctl, func(t *testing.T) {
			got, err := fs.SysctlInts(tc.sysctl)
			if err != nil {
				t.Fatal(err)
			}
			if diff := cmp.Diff(tc.want, got); diff != "" {
				t.Fatalf("unexpected syscall value(-want +got):\n%s", diff)
			}
		})
	}
}

func TestSysctlStrings(t *testing.T) {
	fs := getProcFixtures(t)

	for _, tc := range []struct {
		sysctl string
		want   []string
	}{
		{"kernel.seccomp.actions_avail", []string{"kill_process", "kill_thread", "trap", "errno", "trace", "log", "allow"}},
	} {
		t.Run(tc.sysctl, func(t *testing.T) {
			got, err := fs.SysctlStrings(tc.sysctl)
			if err != nil {
				t.Fatal(err)
			}
			if diff := cmp.Diff(tc.want, got); diff != "" {
				t.Fatalf("unexpected syscall value(-want +got):\n%s", diff)
			}
		})
	}
}

func TestSysctlIntsError(t *testing.T) {
	fs := getProcFixtures(t)

	for _, tc := range []struct {
		sysctl string
		want   string
	}{
		{"kernel.seccomp.actions_avail", "Error Parsing File: field 0 in sysctl kernel.seccomp.actions_avail is not a valid int: strconv.ParseInt: parsing \"kill_process\": invalid syntax"},
	} {
		t.Run(tc.sysctl, func(t *testing.T) {
			_, err := fs.SysctlInts(tc.sysctl)
			if err == nil {
				t.Fatal("Error Parsing File: expected error")
			}
			if diff := cmp.Diff(tc.want, err.Error()); diff != "" {
				t.Fatalf("unexpected syscall value(-want +got):\n%s", diff)
			}
		})
	}
}
