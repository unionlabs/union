// Copyright 2020 The Prometheus Authors
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
)

func TestParseVMStatNUMA(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	vmstat, err := fs.VMStatNUMA()
	if err != nil {
		t.Fatal(err)
	}
	if want, got := uint64(1), vmstat[1].NrFreePages; want != got {
		t.Errorf("want vmstat stat nr_free_pages value %d, got %d", want, got)
	}

	if want, got := uint64(5), vmstat[1].NrZoneActiveFile; want != got {
		t.Errorf("want numa stat nr_zone_active_file %d, got %d", want, got)
	}
	if want, got := uint64(7), vmstat[2].NrFreePages; want != got {
		t.Errorf("want vmstat stat nr_free_pages value %d, got %d", want, got)
	}

	if want, got := uint64(11), vmstat[2].NrZoneActiveFile; want != got {
		t.Errorf("want numa stat nr_zone_active_file %d, got %d", want, got)
	}
}
