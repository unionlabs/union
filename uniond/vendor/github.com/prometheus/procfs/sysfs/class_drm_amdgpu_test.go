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
	"reflect"
	"testing"
)

func TestClassDRMCardAMDGPUStats(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	drmTest, err := fs.ClassDRMCardAMDGPUStats()
	if err != nil {
		t.Fatal(err)
	}

	classDRMCardStats := []ClassDRMCardAMDGPUStats{
		{
			Name:                          "card0",
			GPUBusyPercent:                4,
			MemoryGTTSize:                 8573157376,
			MemoryGTTUsed:                 144560128,
			MemoryVisibleVRAMSize:         8573157376,
			MemoryVisibleVRAMUsed:         1490378752,
			MemoryVRAMSize:                8573157376,
			MemoryVRAMUsed:                1490378752,
			MemoryVRAMVendor:              "samsung",
			PowerDPMForcePerformanceLevel: "manual",
			UniqueID:                      "0123456789abcdef",
		},
	}

	if !reflect.DeepEqual(classDRMCardStats, drmTest) {
		t.Errorf("Result not correct: want %v, have %v", classDRMCardStats, drmTest)
	}
}
