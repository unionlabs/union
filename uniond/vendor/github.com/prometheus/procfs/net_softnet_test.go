// Copyright 2019 The Prometheus Authors
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

func TestNetSoftnet(t *testing.T) {
	fs, err := NewFS(procTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	want := []SoftnetStat{
		{
			Processed:         0x00358fe3,
			Dropped:           0x00006283,
			TimeSqueezed:      0x00000000,
			CPUCollision:      0x00000000,
			ReceivedRps:       0x000855fc,
			FlowLimitCount:    0x00000076,
			SoftnetBacklogLen: 0x00000000,
			Index:             0x00000000,
			Width:             13,
		},
		{
			Processed:         0x00953d1a,
			Dropped:           0x00000446,
			TimeSqueezed:      0x000000b1,
			CPUCollision:      0x00000000,
			ReceivedRps:       0x008eeb9a,
			FlowLimitCount:    0x0000002b,
			SoftnetBacklogLen: 0x000000dc,
			Index:             0x00000001,
			Width:             13,
		},
		{
			Processed:      0x00015c73,
			Dropped:        0x00020e76,
			TimeSqueezed:   0xf0000769,
			CPUCollision:   0x00000004,
			ReceivedRps:    0x00000003,
			FlowLimitCount: 0x00000002,
			Index:          0x00000002,
			Width:          11,
		},
		{
			Processed:    0x01663fb2,
			Dropped:      0x00000000,
			TimeSqueezed: 0x0109a4,
			CPUCollision: 0x00020e76,
			Index:        0x00000003,
			Width:        9,
		},
		{
			Processed:    0x00008e78,
			Dropped:      0x00000001,
			TimeSqueezed: 0x00000011,
			CPUCollision: 0x00000020,
			ReceivedRps:  0x00000010,
			Index:        0x00000004,
			Width:        10,
		},
	}

	got, err := fs.NetSoftnetStat()
	if err != nil {
		t.Fatal(err)
	}

	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected softnet stats(-want +got):\n%s", diff)
	}
}

func TestBadSoftnet(t *testing.T) {
	softNetProcFile = "net/softnet_stat.broken"
	fs, err := NewFS(procTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	_, err = fs.NetSoftnetStat()
	if err == nil {
		t.Fatal("expected error, got nil")
	}
}
