// Copyright 2022 The Prometheus Authors
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

func TestSASDeviceClass(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	got, err := fs.SASDeviceClass()
	if err != nil {
		t.Fatal(err)
	}

	want := SASDeviceClass{
		"end_device-11:0:0": {
			Name:         "end_device-11:0:0",
			SASAddress:   "0x5000ccab02009402",
			BlockDevices: []string{"sdv"},
		},
		"end_device-11:0:1": {
			Name:         "end_device-11:0:1",
			SASAddress:   "0x5000cca26128b1f5",
			BlockDevices: []string{"sdw"},
		},
		"end_device-11:0:2": {
			Name:         "end_device-11:0:2",
			SASAddress:   "0x5000ccab02009406",
			BlockDevices: []string{"sdx"},
		},
		"end_device-11:2": {
			Name:         "end_device-11:2",
			SASAddress:   "0x5000cca0506b5f1d",
			BlockDevices: []string{"sdp"},
		},
		"expander-11:0": {
			Name:       "expander-11:0",
			SASAddress: "0x5000ccab0200947e",
			SASPhys: []string{
				"phy-11:0:10", "phy-11:0:11", "phy-11:0:12",
				"phy-11:0:13", "phy-11:0:14", "phy-11:0:15",
				"phy-11:0:2", "phy-11:0:4", "phy-11:0:6",
				"phy-11:0:7", "phy-11:0:8", "phy-11:0:9",
			},
			SASPorts: []string{
				"port-11:0:0", "port-11:0:1",
				"port-11:0:2",
			},
		},
		"expander-11:1": {
			Name:       "expander-11:1",
			SASAddress: "0x5003048001e8967f",
		},
	}

	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASDevice class (-want +got):\n%s", diff)
	}
}

func TestSASEndDeviceClass(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	got, err := fs.SASEndDeviceClass()
	if err != nil {
		t.Fatal(err)
	}

	want := SASDeviceClass{
		"end_device-11:0:0": {
			Name:         "end_device-11:0:0",
			SASAddress:   "0x5000ccab02009402",
			BlockDevices: []string{"sdv"},
		},
		"end_device-11:0:1": {
			Name:         "end_device-11:0:1",
			SASAddress:   "0x5000cca26128b1f5",
			BlockDevices: []string{"sdw"},
		},
		"end_device-11:0:2": {
			Name:         "end_device-11:0:2",
			SASAddress:   "0x5000ccab02009406",
			BlockDevices: []string{"sdx"},
		},
		"end_device-11:2": {
			Name:         "end_device-11:2",
			SASAddress:   "0x5000cca0506b5f1d",
			BlockDevices: []string{"sdp"},
		},
	}

	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASDevice class (-want +got):\n%s", diff)
	}
}

func TestSASExpanderClass(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	got, err := fs.SASExpanderClass()
	if err != nil {
		t.Fatal(err)
	}

	want := SASDeviceClass{
		"expander-11:0": {
			Name:       "expander-11:0",
			SASAddress: "0x5000ccab0200947e",
			SASPhys: []string{
				"phy-11:0:10", "phy-11:0:11", "phy-11:0:12",
				"phy-11:0:13", "phy-11:0:14", "phy-11:0:15",
				"phy-11:0:2", "phy-11:0:4", "phy-11:0:6",
				"phy-11:0:7", "phy-11:0:8", "phy-11:0:9",
			},
			SASPorts: []string{
				"port-11:0:0", "port-11:0:1",
				"port-11:0:2",
			},
		},
		"expander-11:1": {
			Name:       "expander-11:1",
			SASAddress: "0x5003048001e8967f",
		},
	}

	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASDevice class (-want +got):\n%s", diff)
	}
}

func TestSASDeviceGetByName(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	dc, err := fs.SASDeviceClass()
	if err != nil {
		t.Fatal(err)
	}

	want := "expander-11:0"
	got := dc.GetByName("expander-11:0").Name
	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASDevice name (-want +got):\n%s", diff)
	}

	// Doesn't exist.
	got2 := dc.GetByName("expander-15")
	if got2 != nil {
		t.Fatalf("unexpected GetByName response: got %v want nil", got2)
	}
}

func TestSASDeviceGetByPhy(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	dc, err := fs.SASDeviceClass()
	if err != nil {
		t.Fatal(err)
	}

	want := "expander-11:0"
	got := dc.GetByPhy("phy-11:0:11").Name
	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASDevice class (-want +got):\n%s", diff)
	}

	// Doesn't exist.
	got2 := dc.GetByPhy("phy-12:0")
	if got2 != nil {
		t.Fatalf("unexpected GetByPhy response: got %v want nil", got2)
	}
}

func TestSASDeviceGetByPort(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	dc, err := fs.SASDeviceClass()
	if err != nil {
		t.Fatal(err)
	}

	want := "expander-11:0"
	got := dc.GetByPort("port-11:0:0").Name
	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASDevice class (-want +got):\n%s", diff)
	}

	// Doesn't exist.
	got2 := dc.GetByPort("port-12:0")
	if got2 != nil {
		t.Fatalf("unexpected GetByPhy response: got %v want nil", got2)
	}
}
