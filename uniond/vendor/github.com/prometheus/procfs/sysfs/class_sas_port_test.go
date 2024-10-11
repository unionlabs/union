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

func TestSASPortClass(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	got, err := fs.SASPortClass()
	if err != nil {
		t.Fatal(err)
	}

	want := SASPortClass{
		"port-11:0": {
			Name:      "port-11:0",
			SASPhys:   []string{"phy-11:10", "phy-11:11", "phy-11:8", "phy-11:9"},
			Expanders: []string{"expander-11:0"},
		},
		"port-11:0:0": {
			Name:       "port-11:0:0",
			SASPhys:    []string{"phy-11:0:2"},
			EndDevices: []string{"end_device-11:0:0"},
		},
		"port-11:0:1": {
			Name:       "port-11:0:1",
			SASPhys:    []string{"phy-11:0:4"},
			EndDevices: []string{"end_device-11:0:1"},
		},
		"port-11:0:2": {
			Name:       "port-11:0:2",
			SASPhys:    []string{"phy-11:0:6"},
			EndDevices: []string{"end_device-11:0:2"},
		},
		"port-11:1": {
			Name:      "port-11:1",
			SASPhys:   []string{"phy-11:12", "phy-11:13", "phy-11:14", "phy-11:15"},
			Expanders: []string{"expander-11:1"},
		},
	}

	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASDevice class (-want +got):\n%s", diff)
	}
}

func TestSASPortGetByName(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	dc, err := fs.SASPortClass()
	if err != nil {
		t.Fatal(err)
	}

	want := "port-11:0:0"
	got := dc.GetByName("port-11:0:0").Name
	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASPort name (-want +got):\n%s", diff)
	}

	// Doesn't exist.
	got2 := dc.GetByName("port-15")
	if got2 != nil {
		t.Fatalf("unexpected GetByName response: got %v want nil", got2)
	}
}

func TestSASPortGetByPhy(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	dc, err := fs.SASPortClass()
	if err != nil {
		t.Fatal(err)
	}

	want := "port-11:0:2"
	got := dc.GetByPhy("phy-11:0:6").Name
	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASPort class (-want +got):\n%s", diff)
	}

	// Doesn't exist.
	got2 := dc.GetByPhy("phy-12:0")
	if got2 != nil {
		t.Fatalf("unexpected GetByPhy response: got %v want nil", got2)
	}
}

func TestSASPortGetByExpander(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	dc, err := fs.SASPortClass()
	if err != nil {
		t.Fatal(err)
	}

	want := "port-11:0"
	got := dc.GetByExpander("expander-11:0").Name
	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASPort class (-want +got):\n%s", diff)
	}

	// Doesn't exist.
	got2 := dc.GetByExpander("expander-12:0")
	if got2 != nil {
		t.Fatalf("unexpected GetByPhy response: got %v want nil", got2)
	}
}

func TestSASPortGetByEndDevice(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	dc, err := fs.SASPortClass()
	if err != nil {
		t.Fatal(err)
	}

	want := "port-11:0:2"
	got := dc.GetByEndDevice("end_device-11:0:2").Name
	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASPort class (-want +got):\n%s", diff)
	}

	// Doesn't exist.
	got2 := dc.GetByEndDevice("end_device-12:0")
	if got2 != nil {
		t.Fatalf("unexpected GetByPhy response: got %v want nil", got2)
	}
}
