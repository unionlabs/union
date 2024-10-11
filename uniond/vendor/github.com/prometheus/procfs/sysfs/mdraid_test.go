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

func TestMdraidStats(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	got, err := fs.Mdraids()
	if err != nil {
		t.Fatal(err)
	}

	want := []Mdraid{
		{
			Device:          "md0",
			Level:           "raid0",
			ArrayState:      "clean",
			MetadataVersion: "1.2",
			Disks:           2,
			Components: []MdraidComponent{
				{Device: "sdg", State: "in_sync"},
				{Device: "sdh", State: "in_sync"},
			},
			UUID:      "155f29ff-1716-4107-b362-52307ef86cac",
			ChunkSize: 524288,
		},
		{
			Device:          "md1",
			Level:           "raid1",
			ArrayState:      "clean",
			MetadataVersion: "1.2",
			Disks:           2,
			Components: []MdraidComponent{
				{Device: "sdi", State: "in_sync"},
				{Device: "sdj", State: "in_sync"},
			},
			UUID:       "0fbf5f2c-add2-43c2-bd78-a4be3ab709ef",
			SyncAction: "idle",
		},
		{
			Device:          "md10",
			Level:           "raid10",
			ArrayState:      "clean",
			MetadataVersion: "1.2",
			Disks:           4,
			Components: []MdraidComponent{
				{Device: "sdu", State: "in_sync"},
				{Device: "sdv", State: "in_sync"},
				{Device: "sdw", State: "in_sync"},
				{Device: "sdx", State: "in_sync"},
			},
			UUID:       "0c15f7e7-b159-4b1f-a5cd-a79b5c04b6f5",
			ChunkSize:  524288,
			SyncAction: "idle",
		},
		{
			Device:          "md4",
			Level:           "raid4",
			ArrayState:      "clean",
			MetadataVersion: "1.2",
			Disks:           3,
			Components: []MdraidComponent{
				{Device: "sdk", State: "in_sync"},
				{Device: "sdl", State: "in_sync"},
				{Device: "sdm", State: "in_sync"},
			},
			UUID:       "67f415d5-2c0c-4b69-8e0d-7e20ef553457",
			ChunkSize:  524288,
			SyncAction: "idle",
		},
		{
			Device:          "md5",
			Level:           "raid5",
			ArrayState:      "clean",
			MetadataVersion: "1.2",
			Disks:           3,
			Components: []MdraidComponent{
				{Device: "sdaa", State: "spare"},
				{Device: "sdn", State: "in_sync"},
				{Device: "sdo", State: "in_sync"},
				{Device: "sdp", State: "faulty"},
			},
			UUID:          "7615b98d-f2ba-4d99-bee8-6202d8e130b9",
			ChunkSize:     524288,
			DegradedDisks: 1,
			SyncAction:    "idle",
		},
		{
			Device:          "md6",
			Level:           "raid6",
			ArrayState:      "active",
			MetadataVersion: "1.2",
			Disks:           4,
			Components: []MdraidComponent{
				{Device: "sdq", State: "in_sync"},
				{Device: "sdr", State: "in_sync"},
				{Device: "sds", State: "in_sync"},
				{Device: "sdt", State: "spare"},
			},
			UUID:          "5f529b25-6efd-46e4-99a2-31f6f597be6b",
			ChunkSize:     524288,
			DegradedDisks: 1,
			SyncAction:    "recover",
			SyncCompleted: 0.7500458659491194,
		},
	}

	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected Mdraid (-want +got):\n%s", diff)
	}
}
