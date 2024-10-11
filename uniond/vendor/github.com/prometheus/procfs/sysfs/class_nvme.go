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

const nvmeClassPath = "class/nvme"

// NVMeDevice contains info from files in /sys/class/nvme for a single NVMe device.
type NVMeDevice struct {
	Name             string
	Serial           string // /sys/class/nvme/<Name>/serial
	Model            string // /sys/class/nvme/<Name>/model
	State            string // /sys/class/nvme/<Name>/state
	FirmwareRevision string // /sys/class/nvme/<Name>/firmware_rev
}

// NVMeClass is a collection of every NVMe device in /sys/class/nvme.
//
// The map keys are the names of the NVMe devices.
type NVMeClass map[string]NVMeDevice

// NVMeClass returns info for all NVMe devices read from /sys/class/nvme.
func (fs FS) NVMeClass() (NVMeClass, error) {
	path := fs.sys.Path(nvmeClassPath)

	dirs, err := os.ReadDir(path)
	if err != nil {
		return nil, fmt.Errorf("failed to list NVMe devices at %q: %w", path, err)
	}

	nc := make(NVMeClass, len(dirs))
	for _, d := range dirs {
		device, err := fs.parseNVMeDevice(d.Name())
		if err != nil {
			return nil, err
		}

		nc[device.Name] = *device
	}

	return nc, nil
}

// Parse one NVMe device.
func (fs FS) parseNVMeDevice(name string) (*NVMeDevice, error) {
	path := fs.sys.Path(nvmeClassPath, name)
	device := NVMeDevice{Name: name}

	for _, f := range [...]string{"firmware_rev", "model", "serial", "state"} {
		name := filepath.Join(path, f)
		value, err := util.SysReadFile(name)
		if err != nil {
			return nil, fmt.Errorf("failed to read file %q: %w", name, err)
		}

		switch f {
		case "firmware_rev":
			device.FirmwareRevision = value
		case "model":
			device.Model = value
		case "serial":
			device.Serial = value
		case "state":
			device.State = value
		}
	}

	return &device, nil
}
