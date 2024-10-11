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
	"path/filepath"
	"strings"

	"github.com/prometheus/procfs/internal/util"
)

// Mdraid holds info parsed from relevant files in the /sys/block/md*/md directory.
type Mdraid struct {
	Device          string            // Kernel device name of array.
	Level           string            // mdraid level.
	ArrayState      string            // State of the array.
	MetadataVersion string            // mdraid metadata version.
	Disks           uint64            // Number of devices in a fully functional array.
	Components      []MdraidComponent // mdraid component devices.
	UUID            string            // UUID of the array.

	// The following item is only valid for raid0, 4, 5, 6 and 10.
	ChunkSize uint64 // Chunk size

	// The following items are only valid for raid1, 4, 5, 6 and 10.
	DegradedDisks uint64  // Number of degraded disks in the array.
	SyncAction    string  // Current sync action.
	SyncCompleted float64 // Fraction (0-1) representing the completion status of current sync operation.
}

type MdraidComponent struct {
	Device string // Kernel device name.
	State  string // Current state of device.
}

// Mdraids gathers information and statistics about mdraid devices present. Based on upstream
// kernel documentation https://docs.kernel.org/admin-guide/md.html.
func (fs FS) Mdraids() ([]Mdraid, error) {
	matches, err := filepath.Glob(fs.sys.Path("block/md*/md"))
	if err != nil {
		return nil, err
	}

	mdraids := make([]Mdraid, 0)

	for _, m := range matches {
		md := Mdraid{Device: filepath.Base(filepath.Dir(m))}
		path := fs.sys.Path("block", md.Device, "md")

		if val, err := util.SysReadFile(filepath.Join(path, "level")); err == nil {
			md.Level = val
		} else {
			return mdraids, err
		}

		// Array state can be one of: clear, inactive, readonly, read-auto, clean, active,
		// write-pending, active-idle.
		if val, err := util.SysReadFile(filepath.Join(path, "array_state")); err == nil {
			md.ArrayState = val
		} else {
			return mdraids, err
		}

		if val, err := util.SysReadFile(filepath.Join(path, "metadata_version")); err == nil {
			md.MetadataVersion = val
		} else {
			return mdraids, err
		}

		if val, err := util.ReadUintFromFile(filepath.Join(path, "raid_disks")); err == nil {
			md.Disks = val
		} else {
			return mdraids, err
		}

		if val, err := util.SysReadFile(filepath.Join(path, "uuid")); err == nil {
			md.UUID = val
		} else {
			return mdraids, err
		}

		if devs, err := filepath.Glob(filepath.Join(path, "dev-*")); err == nil {
			for _, dev := range devs {
				comp := MdraidComponent{Device: strings.TrimPrefix(filepath.Base(dev), "dev-")}

				// Component state can be a comma-separated list of: faulty, in_sync, writemostly,
				// blocked, spare, write_error, want_replacement, replacement.
				if val, err := util.SysReadFile(filepath.Join(dev, "state")); err == nil {
					comp.State = val
				} else {
					return mdraids, err
				}

				md.Components = append(md.Components, comp)
			}
		} else {
			return mdraids, err
		}

		switch md.Level {
		case "raid0", "raid4", "raid5", "raid6", "raid10":
			if val, err := util.ReadUintFromFile(filepath.Join(path, "chunk_size")); err == nil {
				md.ChunkSize = val
			} else {
				return mdraids, err
			}
		}

		switch md.Level {
		case "raid1", "raid4", "raid5", "raid6", "raid10":
			if val, err := util.ReadUintFromFile(filepath.Join(path, "degraded")); err == nil {
				md.DegradedDisks = val
			} else {
				return mdraids, err
			}

			// Array sync action can be one of: resync, recover, idle, check, repair.
			if val, err := util.SysReadFile(filepath.Join(path, "sync_action")); err == nil {
				md.SyncAction = val
			} else {
				return mdraids, err
			}

			if val, err := util.SysReadFile(filepath.Join(path, "sync_completed")); err == nil {
				if val != "none" {
					var a, b uint64

					// File contains two values representing the fraction of number of completed
					// sectors divided by number of total sectors to process.
					if _, err := fmt.Sscanf(val, "%d / %d", &a, &b); err == nil {
						md.SyncCompleted = float64(a) / float64(b)
					} else {
						return mdraids, err
					}
				}
			} else {
				return mdraids, err
			}
		}

		mdraids = append(mdraids, md)
	}

	return mdraids, nil
}
