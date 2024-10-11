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

package sysfs

import (
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestNVMeClass(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	got, err := fs.NVMeClass()
	if err != nil {
		t.Fatal(err)
	}

	want := NVMeClass{
		"nvme0": NVMeDevice{
			Name:             "nvme0",
			FirmwareRevision: "1B2QEXP7",
			Model:            "Samsung SSD 970 PRO 512GB",
			Serial:           "S680HF8N190894I",
			State:            "live",
		},
	}

	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected NVMe class (-want +got):\n%s", diff)
	}
}
