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
	"errors"
	"fmt"
	"path/filepath"
	"regexp"
	"syscall"

	"github.com/prometheus/procfs/internal/util"
)

const (
	// Supported device drivers.
	deviceDriverAMDGPU = "amdgpu"
)

// ClassDRMCardAMDGPUStats contains info from files in
// /sys/class/drm/card<card>/device for a single amdgpu card.
// Not all cards expose all metrics.
// https://www.kernel.org/doc/html/latest/gpu/amdgpu.html
type ClassDRMCardAMDGPUStats struct {
	Name                          string // The card name.
	GPUBusyPercent                uint64 // How busy the GPU is as a percentage.
	MemoryGTTSize                 uint64 // The size of the graphics translation table (GTT) block in bytes.
	MemoryGTTUsed                 uint64 // The used amount of the graphics translation table (GTT) block in bytes.
	MemoryVisibleVRAMSize         uint64 // The size of visible VRAM in bytes.
	MemoryVisibleVRAMUsed         uint64 // The used amount of visible VRAM in bytes.
	MemoryVRAMSize                uint64 // The size of VRAM in bytes.
	MemoryVRAMUsed                uint64 // The used amount of VRAM in bytes.
	MemoryVRAMVendor              string // The VRAM vendor name.
	PowerDPMForcePerformanceLevel string // The current power performance level.
	UniqueID                      string // The unique ID of the GPU that will persist from machine to machine.
}

// ClassDRMCardAMDGPUStats returns DRM card metrics for all amdgpu cards.
func (fs FS) ClassDRMCardAMDGPUStats() ([]ClassDRMCardAMDGPUStats, error) {
	cards, err := filepath.Glob(fs.sys.Path("class/drm/card[0-9]"))
	if err != nil {
		return nil, err
	}

	var stats []ClassDRMCardAMDGPUStats
	for _, card := range cards {
		cardStats, err := parseClassDRMAMDGPUCard(card)
		if err != nil {
			if errors.Is(err, syscall.ENODATA) {
				continue
			}
			return nil, err
		}
		cardStats.Name = filepath.Base(card)
		stats = append(stats, cardStats)
	}
	return stats, nil
}

func parseClassDRMAMDGPUCard(card string) (ClassDRMCardAMDGPUStats, error) {
	uevent, err := util.SysReadFile(filepath.Join(card, "device/uevent"))
	if err != nil {
		return ClassDRMCardAMDGPUStats{}, err
	}

	match, err := regexp.MatchString(fmt.Sprintf("DRIVER=%s", deviceDriverAMDGPU), uevent)
	if err != nil {
		return ClassDRMCardAMDGPUStats{}, err
	}
	if !match {
		return ClassDRMCardAMDGPUStats{}, nil
	}

	stats := ClassDRMCardAMDGPUStats{Name: card}
	// Read only specific files for faster data gathering.
	if v, err := readDRMCardField(card, "gpu_busy_percent"); err == nil {
		stats.GPUBusyPercent = *util.NewValueParser(v).PUInt64()
	}
	if v, err := readDRMCardField(card, "mem_info_gtt_total"); err == nil {
		stats.MemoryGTTSize = *util.NewValueParser(v).PUInt64()
	}
	if v, err := readDRMCardField(card, "mem_info_gtt_used"); err == nil {
		stats.MemoryGTTUsed = *util.NewValueParser(v).PUInt64()
	}
	if v, err := readDRMCardField(card, "mem_info_vis_vram_total"); err == nil {
		stats.MemoryVisibleVRAMSize = *util.NewValueParser(v).PUInt64()
	}
	if v, err := readDRMCardField(card, "mem_info_vis_vram_used"); err == nil {
		stats.MemoryVisibleVRAMUsed = *util.NewValueParser(v).PUInt64()
	}
	if v, err := readDRMCardField(card, "mem_info_vram_total"); err == nil {
		stats.MemoryVRAMSize = *util.NewValueParser(v).PUInt64()
	}
	if v, err := readDRMCardField(card, "mem_info_vram_used"); err == nil {
		stats.MemoryVRAMUsed = *util.NewValueParser(v).PUInt64()
	}
	if v, err := readDRMCardField(card, "mem_info_vram_vendor"); err == nil {
		stats.MemoryVRAMVendor = v
	}
	if v, err := readDRMCardField(card, "power_dpm_force_performance_level"); err == nil {
		stats.PowerDPMForcePerformanceLevel = v
	}
	if v, err := readDRMCardField(card, "unique_id"); err == nil {
		stats.UniqueID = v
	}

	return stats, nil
}
