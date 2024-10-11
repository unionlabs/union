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

func TestDMIClass(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	got, err := fs.DMIClass()
	if err != nil {
		t.Fatal(err)
	}

	empty := ""
	biosDate := "04/12/2021"
	biosRelease := "2.2"
	biosVendor := "Dell Inc."
	biosVersion := "2.2.4"
	boardName := "07PXPY"
	boardSerial := ".7N62AI2.GRTCL6944100GP."
	boardVendor := "Dell Inc."
	boardVersion := "A01"
	chassisSerial := "7N62AI2"
	chassisType := "23"
	chassisVendor := "Dell Inc."
	productFamily := "PowerEdge"
	productName := "PowerEdge R6515"
	productSerial := "7N62AI2"
	productSKU := "SKU=NotProvided;ModelName=PowerEdge R6515"
	productUUID := "83340ca8-cb49-4474-8c29-d2088ca84dd9"
	systemVendor := "Dell Inc."

	want := &DMIClass{
		BiosDate:        &biosDate,
		BiosRelease:     &biosRelease,
		BiosVendor:      &biosVendor,
		BiosVersion:     &biosVersion,
		BoardName:       &boardName,
		BoardSerial:     &boardSerial,
		BoardVendor:     &boardVendor,
		BoardVersion:    &boardVersion,
		ChassisAssetTag: &empty,
		ChassisSerial:   &chassisSerial,
		ChassisType:     &chassisType,
		ChassisVendor:   &chassisVendor,
		ChassisVersion:  &empty,
		ProductFamily:   &productFamily,
		ProductName:     &productName,
		ProductSerial:   &productSerial,
		ProductSKU:      &productSKU,
		ProductUUID:     &productUUID,
		ProductVersion:  &empty,
		SystemVendor:    &systemVendor,
	}

	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected DMI class (-want +got):\n%s", diff)
	}
}
