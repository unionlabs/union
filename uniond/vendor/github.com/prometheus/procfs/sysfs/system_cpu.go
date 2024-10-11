// Copyright 2018 The Prometheus Authors
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

	"golang.org/x/sync/errgroup"

	"github.com/prometheus/procfs/internal/util"
)

// CPU represents a path to a CPU located in `/sys/devices/system/cpu/cpu[0-9]*`.
type CPU string

// Number returns the ID number of the given CPU.
func (c CPU) Number() string {
	return strings.TrimPrefix(filepath.Base(string(c)), "cpu")
}

// CPUTopology contains data located in `/sys/devices/system/cpu/cpu[0-9]*/topology`.
type CPUTopology struct {
	CoreID             string
	CoreSiblingsList   string
	PhysicalPackageID  string
	ThreadSiblingsList string
}

// CPUThermalThrottle contains data from `/sys/devices/system/cpu/cpu[0-9]*/thermal_throttle`.
type CPUThermalThrottle struct {
	CoreThrottleCount    uint64
	PackageThrottleCount uint64
}

// SystemCPUCpufreqStats contains stats from `/sys/devices/system/cpu/cpu[0-9]*/cpufreq/...`.
type SystemCPUCpufreqStats struct {
	Name                     string
	CpuinfoCurrentFrequency  *uint64
	CpuinfoMinimumFrequency  *uint64
	CpuinfoMaximumFrequency  *uint64
	CpuinfoTransitionLatency *uint64
	ScalingCurrentFrequency  *uint64
	ScalingMinimumFrequency  *uint64
	ScalingMaximumFrequency  *uint64
	AvailableGovernors       string
	Driver                   string
	Governor                 string
	RelatedCpus              string
	SetSpeed                 string
	// Refer `CONFIG_CPU_FREQ_STAT`: https://www.kernel.org/doc/html/latest/cpu-freq/cpufreq-stats.html#configuring-cpufreq-stats
	CpuinfoFrequencyDuration         *map[uint64]uint64
	CpuinfoFrequencyTransitionsTotal *uint64
	CpuinfoTransitionTable           *[][]uint64
}

// CPUs returns a slice of all CPUs in `/sys/devices/system/cpu`.
func (fs FS) CPUs() ([]CPU, error) {
	cpuPaths, err := filepath.Glob(fs.sys.Path("devices/system/cpu/cpu[0-9]*"))
	if err != nil {
		return nil, err
	}
	cpus := make([]CPU, len(cpuPaths))
	for i, cpu := range cpuPaths {
		cpus[i] = CPU(cpu)
	}
	return cpus, nil
}

// Topology gets the topology information for a single CPU from `/sys/devices/system/cpu/cpuN/topology`.
func (c CPU) Topology() (*CPUTopology, error) {
	cpuTopologyPath := filepath.Join(string(c), "topology")
	if _, err := os.Stat(cpuTopologyPath); err != nil {
		return nil, err
	}
	t, err := parseCPUTopology(cpuTopologyPath)
	if err != nil {
		return nil, err
	}
	return t, nil
}

func parseCPUTopology(cpuPath string) (*CPUTopology, error) {
	t := CPUTopology{}
	var err error
	t.CoreID, err = util.SysReadFile(filepath.Join(cpuPath, "core_id"))
	if err != nil {
		return nil, err
	}
	t.PhysicalPackageID, err = util.SysReadFile(filepath.Join(cpuPath, "physical_package_id"))
	if err != nil {
		return nil, err
	}
	t.CoreSiblingsList, err = util.SysReadFile(filepath.Join(cpuPath, "core_siblings_list"))
	if err != nil {
		return nil, err
	}
	t.ThreadSiblingsList, err = util.SysReadFile(filepath.Join(cpuPath, "thread_siblings_list"))
	if err != nil {
		return nil, err
	}
	return &t, nil
}

// ThermalThrottle gets the cpu throttle count information for a single CPU from `/sys/devices/system/cpu/cpuN/thermal_throttle`.
func (c CPU) ThermalThrottle() (*CPUThermalThrottle, error) {
	cpuPath := filepath.Join(string(c), "thermal_throttle")
	if _, err := os.Stat(cpuPath); err != nil {
		return nil, err
	}
	t, err := parseCPUThermalThrottle(cpuPath)
	if err != nil {
		return nil, err
	}
	return t, nil
}

func parseCPUThermalThrottle(cpuPath string) (*CPUThermalThrottle, error) {
	t := CPUThermalThrottle{}
	var err error
	t.PackageThrottleCount, err = util.ReadUintFromFile(filepath.Join(cpuPath, "package_throttle_count"))
	if err != nil {
		return nil, err
	}
	t.CoreThrottleCount, err = util.ReadUintFromFile(filepath.Join(cpuPath, "core_throttle_count"))
	if err != nil {
		return nil, err
	}
	return &t, nil
}

func binSearch(elem uint16, elemSlice *[]uint16) bool {
	if len(*elemSlice) == 0 {
		return false
	}

	if len(*elemSlice) == 1 {
		return elem == (*elemSlice)[0]
	}

	start := 0
	end := len(*elemSlice) - 1

	var mid int

	for start <= end {
		mid = (start + end) / 2
		if (*elemSlice)[mid] == elem {
			return true
		} else if (*elemSlice)[mid] > elem {
			end = mid - 1
		} else if (*elemSlice)[mid] < elem {
			start = mid + 1
		}
	}

	return false
}

func filterOfflineCPUs(offlineCpus *[]uint16, cpus *[]string) ([]string, error) {
	var filteredCPUs []string
	for _, cpu := range *cpus {
		cpuName := strings.TrimPrefix(filepath.Base(cpu), "cpu")
		cpuNameUint16, err := strconv.Atoi(cpuName)
		if err != nil {
			return nil, err
		}
		if !binSearch(uint16(cpuNameUint16), offlineCpus) {
			filteredCPUs = append(filteredCPUs, cpu)
		}
	}

	return filteredCPUs, nil
}

// SystemCpufreq returns CPU frequency metrics for all CPUs.
func (fs FS) SystemCpufreq() ([]SystemCPUCpufreqStats, error) {
	var g errgroup.Group

	cpus, err := filepath.Glob(fs.sys.Path("devices/system/cpu/cpu[0-9]*"))
	if err != nil {
		return nil, err
	}

	line, err := util.ReadFileNoStat(fs.sys.Path("devices/system/cpu/offline"))
	if err != nil {
		return nil, err
	}

	if strings.TrimSpace(string(line)) != "" {
		offlineCPUs, err := parseCPURange(line)
		if err != nil {
			return nil, err
		}

		if len(offlineCPUs) > 0 {
			cpus, err = filterOfflineCPUs(&offlineCPUs, &cpus)
			if err != nil {
				return nil, err
			}
		}
	}

	systemCpufreq := make([]SystemCPUCpufreqStats, len(cpus))
	for i, cpu := range cpus {
		cpuName := strings.TrimPrefix(filepath.Base(cpu), "cpu")

		cpuCpufreqPath := filepath.Join(cpu, "cpufreq")
		_, err = os.Stat(cpuCpufreqPath)
		if os.IsNotExist(err) {
			continue
		} else if err != nil {
			return nil, err
		}

		// Execute the parsing of each CPU in parallel.
		// This is done because the kernel intentionally delays access to each CPU by
		// 50 milliseconds to avoid DDoSing possibly expensive functions.
		i := i // https://golang.org/doc/faq#closures_and_goroutines
		g.Go(func() error {
			cpufreq, err := parseCpufreqCpuinfo(cpuCpufreqPath)
			if err == nil {
				cpufreq.Name = cpuName
				systemCpufreq[i] = *cpufreq
			}
			return err
		})
	}

	if err = g.Wait(); err != nil {
		return nil, err
	}

	if len(systemCpufreq) == 0 {
		return nil, fmt.Errorf("could not find any cpufreq files: %w", os.ErrNotExist)
	}

	return systemCpufreq, nil
}

func parseCpufreqCpuinfo(cpuPath string) (*SystemCPUCpufreqStats, error) {
	uintFiles := []string{
		"cpuinfo_cur_freq",
		"cpuinfo_max_freq",
		"cpuinfo_min_freq",
		"cpuinfo_transition_latency",
		"scaling_cur_freq",
		"scaling_max_freq",
		"scaling_min_freq",
	}
	uintOut := make([]*uint64, len(uintFiles))

	for i, f := range uintFiles {
		v, err := util.ReadUintFromFile(filepath.Join(cpuPath, f))
		if err != nil {
			if os.IsNotExist(err) || os.IsPermission(err) {
				continue
			}
			return &SystemCPUCpufreqStats{}, err
		}

		uintOut[i] = &v
	}

	stringFiles := []string{
		"scaling_available_governors",
		"scaling_driver",
		"scaling_governor",
		"related_cpus",
		"scaling_setspeed",
	}
	stringOut := make([]string, len(stringFiles))
	var err error

	for i, f := range stringFiles {
		stringOut[i], err = util.SysReadFile(filepath.Join(cpuPath, f))
		if err != nil {
			return &SystemCPUCpufreqStats{}, err
		}
	}

	// "total_trans" is the total number of times the CPU has changed frequency.
	var cpuinfoFrequencyTransitionsTotal *uint64
	cpuinfoFrequencyTransitionsTotalUint, err := util.ReadUintFromFile(filepath.Join(cpuPath, "stats", "total_trans"))
	if err != nil {
		if !(os.IsNotExist(err) || os.IsPermission(err)) {
			return &SystemCPUCpufreqStats{}, err
		}
	} else {
		cpuinfoFrequencyTransitionsTotal = &cpuinfoFrequencyTransitionsTotalUint
	}

	// "time_in_state" is the total time spent at each frequency.
	var cpuinfoFrequencyDuration *map[uint64]uint64
	cpuinfoFrequencyDurationString, err := util.ReadFileNoStat(filepath.Join(cpuPath, "stats", "time_in_state"))
	if err != nil {
		if !(os.IsNotExist(err) || os.IsPermission(err)) {
			return &SystemCPUCpufreqStats{}, err
		}
	} else {
		cpuinfoFrequencyDuration = &map[uint64]uint64{}
		for _, line := range strings.Split(string(cpuinfoFrequencyDurationString), "\n") {
			if line == "" {
				continue
			}
			fields := strings.Fields(line)
			if len(fields) != 2 {
				return &SystemCPUCpufreqStats{}, fmt.Errorf("unexpected number of fields in time_in_state: %v", fields)
			}
			freq, err := strconv.ParseUint(fields[0], 10, 64)
			if err != nil {
				return &SystemCPUCpufreqStats{}, err
			}
			duration, err := strconv.ParseUint(fields[1], 10, 64)
			if err != nil {
				return &SystemCPUCpufreqStats{}, err
			}
			(*cpuinfoFrequencyDuration)[freq] = duration
		}
	}

	// "trans_table" contains information about all the CPU frequency transitions.
	var cpuinfoTransitionTable *[][]uint64
	cpuinfoTransitionTableString, err := util.ReadFileNoStat(filepath.Join(cpuPath, "stats", "trans_table"))
	if err != nil {
		if !(os.IsNotExist(err) || os.IsPermission(err)) {
			return &SystemCPUCpufreqStats{}, err
		}
	} else {
		cpuinfoTransitionTable = &[][]uint64{}
		for i, line := range strings.Split(string(cpuinfoTransitionTableString), "\n") {
			// Skip the "From: To" header.
			if i == 0 || line == "" {
				continue
			}
			fields := strings.Fields(line)
			fields[0] = strings.TrimSuffix(fields[0], ":")
			cpuinfoTransitionTableRow := make([]uint64, len(fields))
			for i := range fields {
				if len(fields[i]) == 0 {
					continue
				}
				f, err := strconv.ParseUint(fields[i], 10, 64)
				if err != nil {
					return &SystemCPUCpufreqStats{}, err
				}
				cpuinfoTransitionTableRow[i] = f
			}
			*cpuinfoTransitionTable = append(*cpuinfoTransitionTable, cpuinfoTransitionTableRow)
		}
	}

	return &SystemCPUCpufreqStats{
		CpuinfoCurrentFrequency:          uintOut[0],
		CpuinfoMaximumFrequency:          uintOut[1],
		CpuinfoMinimumFrequency:          uintOut[2],
		CpuinfoTransitionLatency:         uintOut[3],
		ScalingCurrentFrequency:          uintOut[4],
		ScalingMaximumFrequency:          uintOut[5],
		ScalingMinimumFrequency:          uintOut[6],
		AvailableGovernors:               stringOut[0],
		Driver:                           stringOut[1],
		Governor:                         stringOut[2],
		RelatedCpus:                      stringOut[3],
		SetSpeed:                         stringOut[4],
		CpuinfoFrequencyDuration:         cpuinfoFrequencyDuration,
		CpuinfoFrequencyTransitionsTotal: cpuinfoFrequencyTransitionsTotal,
		CpuinfoTransitionTable:           cpuinfoTransitionTable,
	}, nil
}

func (fs FS) IsolatedCPUs() ([]uint16, error) {
	isolcpus, err := os.ReadFile(fs.sys.Path("devices/system/cpu/isolated"))
	if err != nil {
		return nil, err
	}

	return parseCPURange(isolcpus)
}

func parseCPURange(data []byte) ([]uint16, error) {

	var cpusInt = []uint16{}

	for _, cpu := range strings.Split(strings.TrimSuffix(string(data), "\n"), ",") {
		if cpu == "" {
			continue
		}
		if strings.Contains(cpu, "-") {
			ranges := strings.Split(cpu, "-")
			if len(ranges) != 2 {
				return nil, fmt.Errorf("invalid cpu range: %s", cpu)
			}
			startRange, err := strconv.Atoi(ranges[0])
			if err != nil {
				return nil, fmt.Errorf("invalid cpu start range: %w", err)
			}
			endRange, err := strconv.Atoi(ranges[1])
			if err != nil {
				return nil, fmt.Errorf("invalid cpu end range: %w", err)
			}

			for i := startRange; i <= endRange; i++ {
				cpusInt = append(cpusInt, uint16(i))
			}
			continue
		}

		cpuN, err := strconv.Atoi(cpu)
		if err != nil {
			return nil, err
		}
		cpusInt = append(cpusInt, uint16(cpuN))
	}
	return cpusInt, nil
}
