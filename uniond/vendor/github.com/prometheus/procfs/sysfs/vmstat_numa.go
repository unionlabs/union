// Copyright 2020 The Prometheus Authors
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
	"bufio"
	"bytes"
	"fmt"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"

	"github.com/prometheus/procfs/internal/util"
)

var (
	nodePattern      = "devices/system/node/node[0-9]*"
	nodeNumberRegexp = regexp.MustCompile(`.*devices/system/node/node([0-9]*)`)
)

type VMStat struct {
	NrFreePages                uint64
	NrZoneInactiveAnon         uint64
	NrZoneActiveAnon           uint64
	NrZoneInactiveFile         uint64
	NrZoneActiveFile           uint64
	NrZoneUnevictable          uint64
	NrZoneWritePending         uint64
	NrMlock                    uint64
	NrPageTablePages           uint64
	NrKernelStack              uint64
	NrBounce                   uint64
	NrZspages                  uint64
	NrFreeCma                  uint64
	NumaHit                    uint64
	NumaMiss                   uint64
	NumaForeign                uint64
	NumaInterleave             uint64
	NumaLocal                  uint64
	NumaOther                  uint64
	NrInactiveAnon             uint64
	NrActiveAnon               uint64
	NrInactiveFile             uint64
	NrActiveFile               uint64
	NrUnevictable              uint64
	NrSlabReclaimable          uint64
	NrSlabUnreclaimable        uint64
	NrIsolatedAnon             uint64
	NrIsolatedFile             uint64
	WorkingsetNodes            uint64
	WorkingsetRefault          uint64
	WorkingsetActivate         uint64
	WorkingsetRestore          uint64
	WorkingsetNodereclaim      uint64
	NrAnonPages                uint64
	NrMapped                   uint64
	NrFilePages                uint64
	NrDirty                    uint64
	NrWriteback                uint64
	NrWritebackTemp            uint64
	NrShmem                    uint64
	NrShmemHugepages           uint64
	NrShmemPmdmapped           uint64
	NrFileHugepages            uint64
	NrFilePmdmapped            uint64
	NrAnonTransparentHugepages uint64
	NrVmscanWrite              uint64
	NrVmscanImmediateReclaim   uint64
	NrDirtied                  uint64
	NrWritten                  uint64
	NrKernelMiscReclaimable    uint64
	NrFollPinAcquired          uint64
	NrFollPinReleased          uint64
}

func (fs FS) VMStatNUMA() (map[int]VMStat, error) {
	m := make(map[int]VMStat)
	nodes, err := filepath.Glob(fs.sys.Path(nodePattern))
	if err != nil {
		return nil, err
	}

	for _, node := range nodes {
		nodeNumbers := nodeNumberRegexp.FindStringSubmatch(node)
		if len(nodeNumbers) != 2 {
			continue
		}
		nodeNumber, err := strconv.Atoi(nodeNumbers[1])
		if err != nil {
			return nil, err
		}
		file, err := util.ReadFileNoStat(filepath.Join(node, "vmstat"))
		if err != nil {
			return nil, err
		}
		nodeStats, err := parseVMStatNUMA(file)
		if err != nil {
			return nil, err
		}
		m[nodeNumber] = nodeStats
	}
	return m, nil
}

func parseVMStatNUMA(r []byte) (VMStat, error) {
	var (
		vmStat  = VMStat{}
		scanner = bufio.NewScanner(bytes.NewReader(r))
	)

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" {
			continue
		}
		parts := strings.Fields(line)
		if len(parts) != 2 {
			return vmStat, fmt.Errorf("line scan did not return 2 fields: %s", line)
		}

		fv, err := strconv.ParseUint(parts[1], 10, 64)
		if err != nil {
			return vmStat, fmt.Errorf("invalid value in vmstat: %w", err)
		}
		switch parts[0] {
		case "nr_free_pages":
			vmStat.NrFreePages = fv
		case "nr_zone_inactive_anon":
			vmStat.NrZoneInactiveAnon = fv
		case "nr_zone_active_anon":
			vmStat.NrZoneActiveAnon = fv
		case "nr_zone_inactive_file":
			vmStat.NrZoneInactiveFile = fv
		case "nr_zone_active_file":
			vmStat.NrZoneActiveFile = fv
		case "nr_zone_unevictable":
			vmStat.NrZoneUnevictable = fv
		case "nr_zone_write_pending":
			vmStat.NrZoneWritePending = fv
		case "nr_mlock":
			vmStat.NrMlock = fv
		case "nr_page_table_pages":
			vmStat.NrPageTablePages = fv
		case "nr_kernel_stack":
			vmStat.NrKernelStack = fv
		case "nr_bounce":
			vmStat.NrBounce = fv
		case "nr_zspages":
			vmStat.NrZspages = fv
		case "nr_free_cma":
			vmStat.NrFreeCma = fv
		case "numa_hit":
			vmStat.NumaHit = fv
		case "numa_miss":
			vmStat.NumaMiss = fv
		case "numa_foreign":
			vmStat.NumaForeign = fv
		case "numa_interleave":
			vmStat.NumaInterleave = fv
		case "numa_local":
			vmStat.NumaLocal = fv
		case "numa_other":
			vmStat.NumaOther = fv
		case "nr_inactive_anon":
			vmStat.NrInactiveAnon = fv
		case "nr_active_anon":
			vmStat.NrActiveAnon = fv
		case "nr_inactive_file":
			vmStat.NrInactiveFile = fv
		case "nr_active_file":
			vmStat.NrActiveFile = fv
		case "nr_unevictable":
			vmStat.NrUnevictable = fv
		case "nr_slab_reclaimable":
			vmStat.NrSlabReclaimable = fv
		case "nr_slab_unreclaimable":
			vmStat.NrSlabUnreclaimable = fv
		case "nr_isolated_anon":
			vmStat.NrIsolatedAnon = fv
		case "nr_isolated_file":
			vmStat.NrIsolatedFile = fv
		case "workingset_nodes":
			vmStat.WorkingsetNodes = fv
		case "workingset_refault":
			vmStat.WorkingsetRefault = fv
		case "workingset_activate":
			vmStat.WorkingsetActivate = fv
		case "workingset_restore":
			vmStat.WorkingsetRestore = fv
		case "workingset_nodereclaim":
			vmStat.WorkingsetNodereclaim = fv
		case "nr_anon_pages":
			vmStat.NrAnonPages = fv
		case "nr_mapped":
			vmStat.NrMapped = fv
		case "nr_file_pages":
			vmStat.NrFilePages = fv
		case "nr_dirty":
			vmStat.NrDirty = fv
		case "nr_writeback":
			vmStat.NrWriteback = fv
		case "nr_writeback_temp":
			vmStat.NrWritebackTemp = fv
		case "nr_shmem":
			vmStat.NrShmem = fv
		case "nr_shmem_hugepages":
			vmStat.NrShmemHugepages = fv
		case "nr_shmem_pmdmapped":
			vmStat.NrShmemPmdmapped = fv
		case "nr_file_hugepages":
			vmStat.NrFileHugepages = fv
		case "nr_file_pmdmapped":
			vmStat.NrFilePmdmapped = fv
		case "nr_anon_transparent_hugepages":
			vmStat.NrAnonTransparentHugepages = fv
		case "nr_vmscan_write":
			vmStat.NrVmscanWrite = fv
		case "nr_vmscan_immediate_reclaim":
			vmStat.NrVmscanImmediateReclaim = fv
		case "nr_dirtied":
			vmStat.NrDirtied = fv
		case "nr_written":
			vmStat.NrWritten = fv
		case "nr_kernel_misc_reclaimable":
			vmStat.NrKernelMiscReclaimable = fv
		case "nr_foll_pin_acquired":
			vmStat.NrFollPinAcquired = fv
		case "nr_foll_pin_released":
			vmStat.NrFollPinReleased = fv
		}

	}
	return vmStat, scanner.Err()
}
