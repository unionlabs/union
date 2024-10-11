// Copyright 2023 The Prometheus Authors
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

package sysfs

import (
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestWatchdogClass(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	got, err := fs.WatchdogClass()
	if err != nil {
		t.Fatal(err)
	}

	var (
		bootstatus int64 = 1
		fwVersion  int64 = 2
		nowayout   int64
		timeleft   int64 = 300
		timeout    int64 = 60
		pretimeout int64 = 120
		accessCs0  int64

		options            = "0x8380"
		identity           = "Software Watchdog"
		state              = "active"
		status             = "0x8000"
		pretimeoutGovernor = "noop"
	)

	want := WatchdogClass{
		"watchdog0": {
			Name:               "watchdog0",
			Bootstatus:         &bootstatus,
			Options:            &options,
			FwVersion:          &fwVersion,
			Identity:           &identity,
			Nowayout:           &nowayout,
			State:              &state,
			Status:             &status,
			Timeleft:           &timeleft,
			Timeout:            &timeout,
			Pretimeout:         &pretimeout,
			PretimeoutGovernor: &pretimeoutGovernor,
			AccessCs0:          &accessCs0,
		},
		"watchdog1": {
			Name: "watchdog1",
		},
	}

	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected watchdog class (-want +got):\n%s", diff)
	}
}
