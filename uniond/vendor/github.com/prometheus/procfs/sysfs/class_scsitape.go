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
	"regexp"

	"github.com/prometheus/procfs/internal/util"
)

const scsiTapeClassPath = "class/scsi_tape"

type SCSITapeCounters struct {
	WriteNs      uint64 // /sys/class/scsi_tape/<Name>/stats/write_ns
	ReadByteCnt  uint64 // /sys/class/scsi_tape/<Name>/stats/read_byte_cnt
	IoNs         uint64 // /sys/class/scsi_tape/<Name>/stats/io_ns
	WriteCnt     uint64 // /sys/class/scsi_tape/<Name>/stats/write_cnt
	ResidCnt     uint64 // /sys/class/scsi_tape/<Name>/stats/resid_cnt
	ReadNs       uint64 // /sys/class/scsi_tape/<Name>/stats/read_ns
	InFlight     uint64 // /sys/class/scsi_tape/<Name>/stats/in_flight
	OtherCnt     uint64 // /sys/class/scsi_tape/<Name>/stats/other_cnt
	ReadCnt      uint64 // /sys/class/scsi_tape/<Name>/stats/read_cnt
	WriteByteCnt uint64 // /sys/class/scsi_tape/<Name>/stats/write_byte_cnt
}

type SCSITape struct {
	Name     string           // /sys/class/scsi_tape/<Name>
	Counters SCSITapeCounters // /sys/class/scsi_tape/<Name>/statistics/*
}

type SCSITapeClass map[string]SCSITape

// SCSITapeClass parses st[0-9]+ devices in /sys/class/scsi_tape.
func (fs FS) SCSITapeClass() (SCSITapeClass, error) {
	path := fs.sys.Path(scsiTapeClassPath)

	dirs, err := os.ReadDir(path)
	if err != nil {
		return nil, err
	}

	// There are n?st[0-9]+[a-b]? variants depending on device features.
	// n/2 is probably overestimated but never underestimated
	stc := make(SCSITapeClass, len(dirs)/2)
	validDevice := regexp.MustCompile(`^st[0-9]+$`)

	for _, d := range dirs {
		if !validDevice.Match([]byte(d.Name())) {
			continue
		}
		tape, err := fs.parseSCSITape(d.Name())
		if err != nil {
			return nil, err
		}

		stc[tape.Name] = *tape
	}

	return stc, nil
}

// Parse a single scsi_tape.
func (fs FS) parseSCSITape(name string) (*SCSITape, error) {
	path := fs.sys.Path(scsiTapeClassPath, name)
	tape := SCSITape{Name: name}

	counters, err := parseSCSITapeStatistics(path)
	if err != nil {
		return nil, err
	}
	tape.Counters = *counters

	return &tape, nil
}

// parseSCSITapeStatistics parses metrics from a single tape.
func parseSCSITapeStatistics(tapePath string) (*SCSITapeCounters, error) {
	var counters SCSITapeCounters

	path := filepath.Join(tapePath, "stats")
	files, err := os.ReadDir(path)
	if err != nil {
		return nil, err
	}

	for _, f := range files {
		name := filepath.Join(path, f.Name())
		value, err := util.SysReadFile(name)
		if err != nil {
			return nil, fmt.Errorf("failed to read file %q: %w", name, err)
		}

		vp := util.NewValueParser(value)
		switch f.Name() {
		case "in_flight":
			counters.InFlight = *vp.PUInt64()
		case "io_ns":
			counters.IoNs = *vp.PUInt64()
		case "other_cnt":
			counters.OtherCnt = *vp.PUInt64()
		case "read_byte_cnt":
			counters.ReadByteCnt = *vp.PUInt64()
		case "read_cnt":
			counters.ReadCnt = *vp.PUInt64()
		case "read_ns":
			counters.ReadNs = *vp.PUInt64()
		case "resid_cnt":
			counters.ResidCnt = *vp.PUInt64()
		case "write_byte_cnt":
			counters.WriteByteCnt = *vp.PUInt64()
		case "write_cnt":
			counters.WriteCnt = *vp.PUInt64()
		case "write_ns":
			counters.WriteNs = *vp.PUInt64()
		}

		if err := vp.Err(); err != nil {
			return nil, err
		}

	}

	return &counters, nil
}
