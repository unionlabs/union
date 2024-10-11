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

//go:build linux
// +build linux

package sysfs

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/prometheus/procfs/internal/util"
)

const watchdogClassPath = "class/watchdog"

// WatchdogStats contains info from files in /sys/class/watchdog for a single watchdog device.
// https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-watchdog
type WatchdogStats struct {
	Name               string
	Bootstatus         *int64  // /sys/class/watchdog/<Name>/bootstatus
	Options            *string // /sys/class/watchdog/<Name>/options
	FwVersion          *int64  // /sys/class/watchdog/<Name>/fw_version
	Identity           *string // /sys/class/watchdog/<Name>/identity
	Nowayout           *int64  // /sys/class/watchdog/<Name>/nowayout
	State              *string // /sys/class/watchdog/<Name>/state
	Status             *string // /sys/class/watchdog/<Name>/status
	Timeleft           *int64  // /sys/class/watchdog/<Name>/timeleft
	Timeout            *int64  // /sys/class/watchdog/<Name>/timeout
	Pretimeout         *int64  // /sys/class/watchdog/<Name>/pretimeout
	PretimeoutGovernor *string // /sys/class/watchdog/<Name>/pretimeout_governor
	AccessCs0          *int64  // /sys/class/watchdog/<Name>/access_cs0
}

// WatchdogClass is a collection of statistics for every watchdog device in /sys/class/watchdog.
//
// The map keys are the names of the watchdog devices.
type WatchdogClass map[string]WatchdogStats

// WatchdogClass returns info for all watchdog devices read from /sys/class/watchdog.
func (fs FS) WatchdogClass() (WatchdogClass, error) {
	path := fs.sys.Path(watchdogClassPath)

	dirs, err := os.ReadDir(path)
	if err != nil {
		return nil, fmt.Errorf("failed to list watchdog devices at %q: %w", path, err)
	}

	wds := make(WatchdogClass, len(dirs))
	for _, d := range dirs {
		stats, err := fs.parseWatchdog(d.Name())
		if err != nil {
			return nil, err
		}

		wds[stats.Name] = *stats
	}

	return wds, nil
}

func (fs FS) parseWatchdog(wdName string) (*WatchdogStats, error) {
	path := fs.sys.Path(watchdogClassPath, wdName)
	wd := WatchdogStats{Name: wdName}

	for _, f := range [...]string{"bootstatus", "options", "fw_version", "identity", "nowayout", "state", "status", "timeleft", "timeout", "pretimeout", "pretimeout_governor", "access_cs0"} {
		name := filepath.Join(path, f)
		value, err := util.SysReadFile(name)
		if err != nil {
			if os.IsNotExist(err) {
				continue
			}
			return nil, fmt.Errorf("failed to read file %q: %w", name, err)
		}

		vp := util.NewValueParser(value)

		switch f {
		case "bootstatus":
			wd.Bootstatus = vp.PInt64()
		case "options":
			wd.Options = &value
		case "fw_version":
			wd.FwVersion = vp.PInt64()
		case "identity":
			wd.Identity = &value
		case "nowayout":
			wd.Nowayout = vp.PInt64()
		case "state":
			wd.State = &value
		case "status":
			wd.Status = &value
		case "timeleft":
			wd.Timeleft = vp.PInt64()
		case "timeout":
			wd.Timeout = vp.PInt64()
		case "pretimeout":
			wd.Pretimeout = vp.PInt64()
		case "pretimeout_governor":
			wd.PretimeoutGovernor = &value
		case "access_cs0":
			wd.AccessCs0 = vp.PInt64()
		}

		if err := vp.Err(); err != nil {
			return nil, err
		}
	}

	return &wd, nil
}
