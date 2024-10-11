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

func TestSASHostClass(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	got, err := fs.SASHostClass()
	if err != nil {
		t.Fatal(err)
	}

	want := SASHostClass{
		"host11": &SASHost{
			Name: "host11",
			SASPhys: []string{
				"phy-11:10", "phy-11:11", "phy-11:12", "phy-11:13",
				"phy-11:14", "phy-11:15", "phy-11:7", "phy-11:8",
				"phy-11:9",
			},
			SASPorts: []string{
				"port-11:0", "port-11:1", "port-11:2",
			},
		},
	}

	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASHost class (-want +got):\n%s", diff)
	}
}

func TestSASHostGetByName(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	hc, err := fs.SASHostClass()
	if err != nil {
		t.Fatal(err)
	}

	want := "host11"
	got := hc.GetByName("host11").Name
	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASHost class (-want +got):\n%s", diff)
	}

	// Doesn't exist.
	got2 := hc.GetByName("host12")
	if got2 != nil {
		t.Fatalf("unexpected GetByName response: got %v want nil", got2)
	}
}

func TestSASHostGetByPhy(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	hc, err := fs.SASHostClass()
	if err != nil {
		t.Fatal(err)
	}

	want := "host11"
	got := hc.GetByPhy("phy-11:11").Name
	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASHost class (-want +got):\n%s", diff)
	}

	// Doesn't exist.
	got2 := hc.GetByPhy("phy-12:0")
	if got2 != nil {
		t.Fatalf("unexpected GetByPhy response: got %v want nil", got2)
	}
}

func TestSASHostGetByPort(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	hc, err := fs.SASHostClass()
	if err != nil {
		t.Fatal(err)
	}

	want := "host11"
	got := hc.GetByPort("port-11:0").Name
	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected SASHost class (-want +got):\n%s", diff)
	}

	// Doesn't exist.
	got2 := hc.GetByPort("port-12:0")
	if got2 != nil {
		t.Fatalf("unexpected GetByPhy response: got %v want nil", got2)
	}
}
