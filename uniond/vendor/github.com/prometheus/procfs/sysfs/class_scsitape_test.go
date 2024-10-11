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

func TestSCSITapeClass(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	got, err := fs.SCSITapeClass()
	if err != nil {
		t.Fatal(err)
	}

	want := SCSITapeClass{
		"st0": SCSITape{
			Name: "st0",
			Counters: SCSITapeCounters{
				WriteNs:      5233597394395,
				ReadByteCnt:  979383912,
				IoNs:         9247011087720,
				WriteCnt:     53772916,
				WriteByteCnt: 1496246784000,
				ResidCnt:     19,
				ReadNs:       33788355744,
				InFlight:     1,
				OtherCnt:     1409,
				ReadCnt:      3741,
			},
		},
	}

	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SCSITape class (-want +got):\n%s", diff)
	}
}
