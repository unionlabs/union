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
	"os"
	"path/filepath"
	"regexp"
)

const sasPortClassPath = "class/sas_port"

type SASPort struct {
	Name       string   // /sys/class/sas_device/<Name>
	SASPhys    []string // /sys/class/sas_device/<Name>/device/phy-*
	Expanders  []string // /sys/class/sas_port/<Name>/device/expander-*
	EndDevices []string // /sys/class/sas_port/<Name>/device/end_device-*
}

type SASPortClass map[string]*SASPort

var (
	sasExpanderDeviceRegexp = regexp.MustCompile(`^expander-[0-9:]+$`)
	sasEndDeviceRegexp      = regexp.MustCompile(`^end_device-[0-9:]+$`)
)

// SASPortClass parses ports in /sys/class/sas_port.
//
// A SAS port in this context is a collection of SAS PHYs operating
// together.  For example, it's common to have 8-lane SAS cards that
// have 2 external connectors, each of which carries 4 SAS lanes over
// a SFF-8088 or SFF-8644 connector.  While it's possible to split
// those 4 lanes into 4 different cables wired directly into
// individual drives, it's more common to connect them all to a SAS
// expander.  This gives you 4x the bandwidth between the expander and
// the SAS host, and is represented by a sas-port object which
// contains 4 sas-phy objects.
func (fs FS) SASPortClass() (SASPortClass, error) {
	path := fs.sys.Path(sasPortClassPath)

	dirs, err := os.ReadDir(path)
	if err != nil {
		return nil, err
	}

	spc := make(SASPortClass, len(dirs))

	for _, d := range dirs {
		port, err := fs.parseSASPort(d.Name())
		if err != nil {
			return nil, err
		}

		spc[port.Name] = port
	}

	return spc, nil
}

// Parse a single sas_port.
func (fs FS) parseSASPort(name string) (*SASPort, error) {
	port := SASPort{Name: name}

	portpath := fs.sys.Path(filepath.Join(sasPortClassPath, name, "device"))

	dirs, err := os.ReadDir(portpath)
	if err != nil {
		return nil, err
	}

	for _, d := range dirs {
		if sasPhyDeviceRegexp.MatchString(d.Name()) {
			port.SASPhys = append(port.SASPhys, d.Name())
		}
		if sasExpanderDeviceRegexp.MatchString(d.Name()) {
			port.Expanders = append(port.Expanders, d.Name())
		}
		if sasEndDeviceRegexp.MatchString(d.Name()) {
			port.EndDevices = append(port.EndDevices, d.Name())
		}
	}

	return &port, nil
}

// GetByName returns the SASPort with the provided name.
func (spc *SASPortClass) GetByName(name string) *SASPort {
	return (*spc)[name]
}

// GetByPhy finds the SASPort that contains the provided PHY name.
func (spc *SASPortClass) GetByPhy(name string) *SASPort {
	for _, d := range *spc {
		for _, p := range d.SASPhys {
			if p == name {
				return d
			}
		}
	}
	return nil
}

// GetByExpander finds the SASPort that contains the provided SAS expander name.
func (spc *SASPortClass) GetByExpander(name string) *SASPort {
	for _, d := range *spc {
		for _, e := range d.Expanders {
			if e == name {
				return d
			}
		}
	}
	return nil
}

// GetByEndDevice finds the SASPort that contains the provided SAS end device name.
func (spc *SASPortClass) GetByEndDevice(name string) *SASPort {
	for _, d := range *spc {
		for _, e := range d.EndDevices {
			if e == name {
				return d
			}
		}
	}
	return nil
}
