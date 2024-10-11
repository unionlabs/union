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
	"fmt"
	"os"
	"path/filepath"

	"github.com/prometheus/procfs/internal/util"
)

const dmiClassPath = "class/dmi/id"

// DMIClass contains info from files in /sys/class/dmi/id.
type DMIClass struct {
	BiosDate        *string // /sys/class/dmi/id/bios_date
	BiosRelease     *string // /sys/class/dmi/id/bios_release
	BiosVendor      *string // /sys/class/dmi/id/bios_vendor
	BiosVersion     *string // /sys/class/dmi/id/bios_version
	BoardAssetTag   *string // /sys/class/dmi/id/board_asset_tag
	BoardName       *string // /sys/class/dmi/id/board_name
	BoardSerial     *string // /sys/class/dmi/id/board_serial
	BoardVendor     *string // /sys/class/dmi/id/board_vendor
	BoardVersion    *string // /sys/class/dmi/id/board_version
	ChassisAssetTag *string // /sys/class/dmi/id/chassis_asset_tag
	ChassisSerial   *string // /sys/class/dmi/id/chassis_serial
	ChassisType     *string // /sys/class/dmi/id/chassis_type
	ChassisVendor   *string // /sys/class/dmi/id/chassis_vendor
	ChassisVersion  *string // /sys/class/dmi/id/chassis_version
	ProductFamily   *string // /sys/class/dmi/id/product_family
	ProductName     *string // /sys/class/dmi/id/product_name
	ProductSerial   *string // /sys/class/dmi/id/product_serial
	ProductSKU      *string // /sys/class/dmi/id/product_sku
	ProductUUID     *string // /sys/class/dmi/id/product_uuid
	ProductVersion  *string // /sys/class/dmi/id/product_version
	SystemVendor    *string // /sys/class/dmi/id/sys_vendor
}

// DMIClass returns Desktop Management Interface (DMI) information read from /sys/class/dmi.
func (fs FS) DMIClass() (*DMIClass, error) {
	path := fs.sys.Path(dmiClassPath)

	files, err := os.ReadDir(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read directory %q: %w", path, err)
	}

	var dmi DMIClass
	for _, f := range files {
		if !f.Type().IsRegular() {
			continue
		}

		name := f.Name()
		if name == "modalias" || name == "uevent" {
			continue
		}

		filename := filepath.Join(path, name)
		value, err := util.SysReadFile(filename)
		if err != nil {
			if os.IsPermission(err) {
				// Only root is allowed to read the serial and product_uuid files!
				continue
			}
			return nil, fmt.Errorf("failed to read file %q: %w", filename, err)
		}

		switch name {
		case "bios_date":
			dmi.BiosDate = &value
		case "bios_release":
			dmi.BiosRelease = &value
		case "bios_vendor":
			dmi.BiosVendor = &value
		case "bios_version":
			dmi.BiosVersion = &value
		case "board_asset_tag":
			dmi.BoardAssetTag = &value
		case "board_name":
			dmi.BoardName = &value
		case "board_serial":
			dmi.BoardSerial = &value
		case "board_vendor":
			dmi.BoardVendor = &value
		case "board_version":
			dmi.BoardVersion = &value
		case "chassis_asset_tag":
			dmi.ChassisAssetTag = &value
		case "chassis_serial":
			dmi.ChassisSerial = &value
		case "chassis_type":
			dmi.ChassisType = &value
		case "chassis_vendor":
			dmi.ChassisVendor = &value
		case "chassis_version":
			dmi.ChassisVersion = &value
		case "product_family":
			dmi.ProductFamily = &value
		case "product_name":
			dmi.ProductName = &value
		case "product_serial":
			dmi.ProductSerial = &value
		case "product_sku":
			dmi.ProductSKU = &value
		case "product_uuid":
			dmi.ProductUUID = &value
		case "product_version":
			dmi.ProductVersion = &value
		case "sys_vendor":
			dmi.SystemVendor = &value
		}
	}

	return &dmi, nil
}
