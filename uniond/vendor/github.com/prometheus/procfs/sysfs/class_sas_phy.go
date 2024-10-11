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
	"strconv"
	"strings"

	"github.com/prometheus/procfs/internal/util"
)

const sasPhyClassPath = "class/sas_phy"

type SASPhy struct {
	Name                       string   // /sys/class/sas_phy/<Name>
	SASAddress                 string   // /sys/class/sas_phy/<Name>/sas_address
	SASPort                    string   // /sys/class/sas_phy/<Name>/device/ports
	DeviceType                 string   // /sys/class/sas_phy/<Name>/device_type
	InitiatorPortProtocols     []string // /sys/class/sas_phy/<Name>/initiator_port_protocols
	InvalidDwordCount          int      // /sys/class/sas_phy/<Name>/invalid_dword_count
	LossOfDwordSyncCount       int      // /sys/class/sas_phy/<Name>/loss_of_dword_sync_count
	MaximumLinkrate            float64  // /sys/class/sas_phy/<Name>/maximum_linkrate
	MaximumLinkrateHW          float64  // /sys/class/sas_phy/<Name>/maximum_linkrate_hw
	MinimumLinkrate            float64  // /sys/class/sas_phy/<Name>/minimum_linkrate
	MinimumLinkrateHW          float64  // /sys/class/sas_phy/<Name>/minimum_linkrate_hw
	NegotiatedLinkrate         float64  // /sys/class/sas_phy/<Name>/negotiated_linkrate
	PhyIdentifier              string   // /sys/class/sas_phy/<Name>/phy_identifier
	PhyResetProblemCount       int      // /sys/class/sas_phy/<Name>/phy_reset_problem_count
	RunningDisparityErrorCount int      // /sys/class/sas_phy/<Name>/running_disparity_error_count
	TargetPortProtocols        []string // /sys/class/sas_phy/<Name>/target_port_protocols
}

type SASPhyClass map[string]*SASPhy

// SASPhyClass parses entries in /sys/class/sas_phy.
func (fs FS) SASPhyClass() (SASPhyClass, error) {
	path := fs.sys.Path(sasPhyClassPath)

	dirs, err := os.ReadDir(path)
	if err != nil {
		return nil, err
	}

	spc := make(SASPhyClass, len(dirs))

	for _, d := range dirs {
		phy, err := fs.parseSASPhy(d.Name())
		if err != nil {
			return nil, err
		}

		spc[phy.Name] = phy
	}

	return spc, nil
}

// Parse a single sas_phy.
func (fs FS) parseSASPhy(name string) (*SASPhy, error) {
	phy := SASPhy{Name: name}

	phypath := fs.sys.Path(filepath.Join(sasPhyClassPath, name))
	phydevicepath := filepath.Join(phypath, "device")

	link, err := os.Readlink(filepath.Join(phydevicepath, "port"))

	if err == nil {
		if sasPortDeviceRegexp.MatchString(filepath.Base(link)) {
			phy.SASPort = filepath.Base(link)
		}
	}

	files, err := os.ReadDir(phypath)
	if err != nil {
		return nil, err
	}
	for _, f := range files {
		name := filepath.Join(phypath, f.Name())
		fileinfo, _ := os.Stat(name)
		if fileinfo.Mode().IsRegular() {
			value, err := util.SysReadFile(name)
			if err != nil {
				if os.IsPermission(err) {
					continue
				}
				return nil, fmt.Errorf("failed to read file %q: %w", name, err)
			}

			vp := util.NewValueParser(value)
			switch f.Name() {
			case "sas_address":
				phy.SASAddress = value
			case "device_type":
				phy.DeviceType = value
			case "initiator_port_protocols":
				phy.InitiatorPortProtocols = strings.Split(value, ", ")
			case "invalid_dword_count":
				phy.InvalidDwordCount = vp.Int()
			case "loss_of_dword_sync_count":
				phy.LossOfDwordSyncCount = vp.Int()
			case "maximum_linkrate":
				phy.MaximumLinkrate = parseLinkrate(value)
			case "maximum_linkrate_hw":
				phy.MaximumLinkrateHW = parseLinkrate(value)
			case "minimum_linkrate":
				phy.MinimumLinkrate = parseLinkrate(value)
			case "minimum_linkrate_hw":
				phy.MinimumLinkrateHW = parseLinkrate(value)
			case "negotiated_linkrate":
				phy.NegotiatedLinkrate = parseLinkrate(value)
			case "phy_identifier":
				phy.PhyIdentifier = value
			case "phy_reset_problem_count":
				phy.PhyResetProblemCount = vp.Int()
			case "running_disparity_error_count":
				phy.RunningDisparityErrorCount = vp.Int()
			case "target_port_protocols":
				phy.TargetPortProtocols = strings.Split(value, ", ")
			}

			if err := vp.Err(); err != nil {
				return nil, err
			}
		}
	}

	return &phy, nil
}

// parseLinkRate turns the kernel's SAS linkrate values into floats.
// The kernel returns values like "12.0 Gbit".  Valid speeds are
// currently 1.5, 3.0, 6.0, 12.0, and up.  This is a float to cover
// the 1.5 Gbps case.  A value of 0 is returned if the speed can't be
// parsed.
func parseLinkrate(value string) float64 {
	f := strings.Split(value, " ")[0]
	gb, err := strconv.ParseFloat(f, 64)
	if err != nil {
		return 0
	}
	return gb
}

// GetByName returns the SASPhy with the provided name.
func (spc *SASPhyClass) GetByName(name string) *SASPhy {
	return (*spc)[name]
}
