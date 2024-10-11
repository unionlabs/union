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

package procfs

import (
	"reflect"
	"testing"
)

func TestWireless(t *testing.T) {
	fs, err := NewFS(procTestFixtures)
	if err != nil {
		t.Fatalf("failed to open procfs: %v", err)
	}

	got, err := fs.Wireless()
	if err != nil {
		t.Fatal(err)
	}

	expected := []*Wireless{
		{
			Name:           "wlan0",
			Status:         1,
			QualityLink:    2,
			QualityLevel:   3,
			QualityNoise:   4,
			DiscardedNwid:  5,
			DiscardedCrypt: 6,
			DiscardedFrag:  7,
			DiscardedRetry: 8,
			DiscardedMisc:  9,
			MissedBeacon:   10,
		},
		{
			Name:           "wlan1",
			Status:         16,
			QualityLink:    9,
			QualityLevel:   8,
			QualityNoise:   7,
			DiscardedNwid:  6,
			DiscardedCrypt: 5,
			DiscardedFrag:  4,
			DiscardedRetry: 3,
			DiscardedMisc:  2,
			MissedBeacon:   1,
		},
	}

	if len(got) != len(expected) {
		t.Fatalf("unexpected number of interfaces parsed %d, expected %d", len(got), len(expected))
	}

	for i, iface := range got {
		if !reflect.DeepEqual(iface, expected[i]) {
			t.Errorf("unexpected interface got %+v, expected %+v", iface, expected[i])
		}
	}
}
