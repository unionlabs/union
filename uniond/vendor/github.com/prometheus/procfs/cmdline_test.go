// Copyright 2021 The Prometheus Authors
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

//go:build linux
// +build linux

package procfs

import (
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestCmdline(t *testing.T) {
	fs, err := NewFS(procTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	got, err := fs.CmdLine()
	if err != nil {
		t.Fatal(err)
	}

	want := []string{
		"BOOT_IMAGE=/vmlinuz-5.11.0-22-generic",
		"root=UUID=456a0345-450d-4f7b-b7c9-43e3241d99ad",
		"ro",
		"quiet",
		"splash",
		"vt.handoff=7",
	}

	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected CmdLine (-want +got):\n%s", diff)
	}
}
