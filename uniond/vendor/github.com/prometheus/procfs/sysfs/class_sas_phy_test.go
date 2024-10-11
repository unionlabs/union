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

func TestSASPhyClass(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	got, err := fs.SASPhyClass()
	if err != nil {
		t.Fatal(err)
	}

	want := SASPhyClass{
		"phy-11:0:2": {
			Name:                       "phy-11:0:2",
			SASAddress:                 "0x5000ccab0200947e",
			SASPort:                    "port-11:0:0",
			DeviceType:                 "edge expander",
			InitiatorPortProtocols:     []string{"smp"},
			InvalidDwordCount:          18,
			LossOfDwordSyncCount:       1,
			MaximumLinkrate:            12,
			MaximumLinkrateHW:          12,
			MinimumLinkrate:            1.5,
			MinimumLinkrateHW:          1.5,
			NegotiatedLinkrate:         6,
			PhyIdentifier:              "2",
			RunningDisparityErrorCount: 18,
			TargetPortProtocols:        []string{"smp"},
		},
		"phy-11:0:4": {
			Name:                       "phy-11:0:4",
			SASAddress:                 "0x5000ccab0200947e",
			SASPort:                    "port-11:0:1",
			DeviceType:                 "edge expander",
			InitiatorPortProtocols:     []string{"smp"},
			InvalidDwordCount:          1,
			MaximumLinkrate:            12,
			MaximumLinkrateHW:          12,
			MinimumLinkrate:            1.5,
			MinimumLinkrateHW:          1.5,
			NegotiatedLinkrate:         12,
			PhyIdentifier:              "4",
			RunningDisparityErrorCount: 1,
			TargetPortProtocols:        []string{"smp"},
		},
		"phy-11:0:6": {
			Name:                       "phy-11:0:6",
			SASAddress:                 "0x5000ccab0200947e",
			SASPort:                    "port-11:0:2",
			DeviceType:                 "edge expander",
			InitiatorPortProtocols:     []string{"smp"},
			InvalidDwordCount:          19,
			LossOfDwordSyncCount:       1,
			MaximumLinkrate:            12,
			MaximumLinkrateHW:          12,
			MinimumLinkrate:            1.5,
			MinimumLinkrateHW:          1.5,
			NegotiatedLinkrate:         6,
			PhyIdentifier:              "6",
			RunningDisparityErrorCount: 20,
			TargetPortProtocols:        []string{"smp"},
		},
		"phy-11:10": {
			Name:                   "phy-11:10",
			SASAddress:             "0x500062b2047b51c4",
			SASPort:                "port-11:0",
			DeviceType:             "end device",
			InitiatorPortProtocols: []string{"smp", "stp", "ssp"},
			MaximumLinkrate:        12,
			MaximumLinkrateHW:      12,
			MinimumLinkrate:        3,
			MinimumLinkrateHW:      1.5,
			NegotiatedLinkrate:     12,
			PhyIdentifier:          "10",
			TargetPortProtocols:    []string{"none"},
		},
		"phy-11:11": {
			Name:                   "phy-11:11",
			SASAddress:             "0x500062b2047b51c4",
			SASPort:                "port-11:0",
			DeviceType:             "end device",
			InitiatorPortProtocols: []string{"smp", "stp", "ssp"},
			MaximumLinkrate:        12,
			MaximumLinkrateHW:      12,
			MinimumLinkrate:        3,
			MinimumLinkrateHW:      1.5,
			NegotiatedLinkrate:     12,
			PhyIdentifier:          "11",
			TargetPortProtocols:    []string{"none"},
		},
		"phy-11:12": {
			Name:                   "phy-11:12",
			SASAddress:             "0x500062b2047b51c4",
			SASPort:                "port-11:1",
			DeviceType:             "end device",
			InitiatorPortProtocols: []string{"smp", "stp", "ssp"},
			MaximumLinkrate:        12,
			MaximumLinkrateHW:      12,
			MinimumLinkrate:        3,
			MinimumLinkrateHW:      1.5,
			NegotiatedLinkrate:     6,
			PhyIdentifier:          "12",
			TargetPortProtocols:    []string{"none"},
		},
		"phy-11:13": {
			Name:                   "phy-11:13",
			SASAddress:             "0x500062b2047b51c4",
			SASPort:                "port-11:1",
			DeviceType:             "end device",
			InitiatorPortProtocols: []string{"smp", "stp", "ssp"},
			MaximumLinkrate:        12,
			MaximumLinkrateHW:      12,
			MinimumLinkrate:        3,
			MinimumLinkrateHW:      1.5,
			NegotiatedLinkrate:     6,
			PhyIdentifier:          "13",
			TargetPortProtocols:    []string{"none"},
		},
		"phy-11:14": {
			Name:                   "phy-11:14",
			SASAddress:             "0x500062b2047b51c4",
			SASPort:                "port-11:1",
			DeviceType:             "end device",
			InitiatorPortProtocols: []string{"smp", "stp", "ssp"},
			MaximumLinkrate:        12,
			MaximumLinkrateHW:      12,
			MinimumLinkrate:        3,
			MinimumLinkrateHW:      1.5,
			NegotiatedLinkrate:     6,
			PhyIdentifier:          "14",
			TargetPortProtocols:    []string{"none"},
		},
		"phy-11:15": {
			Name:                   "phy-11:15",
			SASAddress:             "0x500062b2047b51c4",
			SASPort:                "port-11:1",
			DeviceType:             "end device",
			InitiatorPortProtocols: []string{"smp", "stp", "ssp"},
			MaximumLinkrate:        12,
			MaximumLinkrateHW:      12,
			MinimumLinkrate:        3,
			MinimumLinkrateHW:      1.5,
			NegotiatedLinkrate:     6,
			PhyIdentifier:          "15",
			TargetPortProtocols:    []string{"none"},
		},
		"phy-11:7": {
			Name:                   "phy-11:7",
			SASAddress:             "0x500062b2047b51c4",
			SASPort:                "port-11:2",
			DeviceType:             "end device",
			InitiatorPortProtocols: []string{"smp", "stp", "ssp"},
			MaximumLinkrate:        12,
			MaximumLinkrateHW:      12,
			MinimumLinkrate:        3,
			MinimumLinkrateHW:      1.5,
			NegotiatedLinkrate:     12,
			PhyIdentifier:          "7",
			TargetPortProtocols:    []string{"none"},
		},
		"phy-11:8": {
			Name:                   "phy-11:8",
			SASAddress:             "0x500062b2047b51c4",
			SASPort:                "port-11:0",
			DeviceType:             "end device",
			InitiatorPortProtocols: []string{"smp", "stp", "ssp"},
			MaximumLinkrate:        12,
			MaximumLinkrateHW:      12,
			MinimumLinkrate:        3,
			MinimumLinkrateHW:      1.5,
			NegotiatedLinkrate:     12,
			PhyIdentifier:          "8",
			TargetPortProtocols:    []string{"none"},
		},
		"phy-11:9": {
			Name:                   "phy-11:9",
			SASAddress:             "0x500062b2047b51c4",
			SASPort:                "port-11:0",
			DeviceType:             "end device",
			InitiatorPortProtocols: []string{"smp", "stp", "ssp"},
			MaximumLinkrate:        12,
			MaximumLinkrateHW:      12,
			MinimumLinkrate:        3,
			MinimumLinkrateHW:      1.5,
			NegotiatedLinkrate:     12,
			PhyIdentifier:          "9",
			TargetPortProtocols:    []string{"none"},
		},
	}

	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASPhy class (-want +got):\n%s", diff)
	}
}

func TestSASPhyGetByName(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	pc, err := fs.SASPhyClass()
	if err != nil {
		t.Fatal(err)
	}

	want := "phy-11:9"
	got := pc.GetByName("phy-11:9").Name
	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASPhy class (-want +got):\n%s", diff)
	}

	// Doesn't exist.
	got2 := pc.GetByName("phy-12:0")
	if got2 != nil {
		t.Fatalf("unexpected GetByName response: got %v want nil", got2)
	}
}
