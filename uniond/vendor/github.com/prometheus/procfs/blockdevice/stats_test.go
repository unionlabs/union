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

package blockdevice

import (
	"os"
	"reflect"
	"testing"
)

const (
	failMsgFormat  = "%v, expected %v, actual %v"
	procfsFixtures = "testdata/fixtures/proc"
	sysfsFixtures  = "testdata/fixtures/sys"
)

func TestDiskstats(t *testing.T) {
	blockdevice, err := NewFS(procfsFixtures, sysfsFixtures)
	if err != nil {
		t.Fatalf("failed to access blockdevice fs: %v", err)
	}
	diskstats, err := blockdevice.ProcDiskstats()
	if err != nil {
		t.Fatal(err)
	}
	expectedNumOfDevices := 52
	if len(diskstats) != expectedNumOfDevices {
		t.Errorf(failMsgFormat, "Incorrect number of devices", expectedNumOfDevices, len(diskstats))
	}
	if diskstats[0].DeviceName != "ram0" {
		t.Errorf(failMsgFormat, "Incorrect device name", "ram0", diskstats[0].DeviceName)
	}
	if diskstats[1].IoStatsCount != 14 {
		t.Errorf(failMsgFormat, "Incorrect number of stats read", 14, diskstats[0].IoStatsCount)
	}
	if diskstats[24].WriteIOs != 28444756 {
		t.Errorf(failMsgFormat, "Incorrect writes completed", 28444756, diskstats[24].WriteIOs)
	}
	if diskstats[48].DiscardTicks != 11130 {
		t.Errorf(failMsgFormat, "Incorrect discard time", 11130, diskstats[48].DiscardTicks)
	}
	if diskstats[48].IoStatsCount != 18 {
		t.Errorf(failMsgFormat, "Incorrect number of stats read", 18, diskstats[48].IoStatsCount)
	}
	if diskstats[49].IoStatsCount != 20 {
		t.Errorf(failMsgFormat, "Incorrect number of stats read", 20, diskstats[50].IoStatsCount)
	}
	if diskstats[49].FlushRequestsCompleted != 127 {
		t.Errorf(failMsgFormat, "Incorrect number of flash requests completed", 127, diskstats[50].FlushRequestsCompleted)
	}
	if diskstats[49].TimeSpentFlushing != 182 {
		t.Errorf(failMsgFormat, "Incorrect time spend flushing", 182, diskstats[50].TimeSpentFlushing)
	}
}

func TestBlockDevice(t *testing.T) {
	blockdevice, err := NewFS("testdata/fixtures/proc", "testdata/fixtures/sys")
	if err != nil {
		t.Fatalf("failed to access blockdevice fs: %v", err)
	}
	devices, err := blockdevice.SysBlockDevices()
	if err != nil {
		t.Fatal(err)
	}
	expectedNumOfDevices := 8
	if len(devices) != expectedNumOfDevices {
		t.Fatalf(failMsgFormat, "Incorrect number of devices", expectedNumOfDevices, len(devices))
	}
	if devices[0] != "dm-0" {
		t.Errorf(failMsgFormat, "Incorrect device name", "dm-0", devices[0])
	}
	device0stats, count, err := blockdevice.SysBlockDeviceStat(devices[0])
	if err != nil {
		t.Fatal(err)
	}
	if count != 11 {
		t.Errorf(failMsgFormat, "Incorrect number of stats read", 11, count)
	}
	if device0stats.ReadIOs != 6447303 {
		t.Errorf(failMsgFormat, "Incorrect read I/Os", 6447303, device0stats.ReadIOs)
	}
	if device0stats.WeightedIOTicks != 6088971 {
		t.Errorf(failMsgFormat, "Incorrect time in queue", 6088971, device0stats.WeightedIOTicks)
	}
	device7stats, count, err := blockdevice.SysBlockDeviceStat(devices[7])
	if count != 15 {
		t.Errorf(failMsgFormat, "Incorrect number of stats read", 15, count)
	}
	if err != nil {
		t.Fatal(err)
	}
	if device7stats.WriteSectors != 286915323 {
		t.Errorf(failMsgFormat, "Incorrect write merges", 286915323, device7stats.WriteSectors)
	}
	if device7stats.DiscardTicks != 12 {
		t.Errorf(failMsgFormat, "Incorrect discard ticks", 12, device7stats.DiscardTicks)
	}
	blockQueueStatExpected := BlockQueueStats{
		AddRandom:            1,
		DAX:                  0,
		DiscardGranularity:   0,
		DiscardMaxHWBytes:    0,
		DiscardMaxBytes:      0,
		HWSectorSize:         512,
		IOPoll:               0,
		IOPollDelay:          -1,
		IOTimeout:            30000,
		IOStats:              1,
		LogicalBlockSize:     512,
		MaxHWSectorsKB:       32767,
		MaxIntegritySegments: 0,
		MaxSectorsKB:         1280,
		MaxSegments:          168,
		MaxSegmentSize:       65536,
		MinimumIOSize:        512,
		NoMerges:             0,
		NRRequests:           64,
		OptimalIOSize:        0,
		PhysicalBlockSize:    512,
		ReadAHeadKB:          128,
		Rotational:           1,
		RQAffinity:           1,
		SchedulerList:        []string{"mq-deadline", "kyber", "bfq", "none"},
		SchedulerCurrent:     "bfq",
		WriteCache:           "write back",
		WriteSameMaxBytes:    0,
		WBTLatUSec:           75000,
		ThrottleSampleTime:   nil,
		Zoned:                "none",
		NRZones:              0,
		ChunkSectors:         0,
		FUA:                  0,
		MaxDiscardSegments:   1,
		WriteZeroesMaxBytes:  0,
	}

	blockQueueStat, err := blockdevice.SysBlockDeviceQueueStats(devices[7])
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(blockQueueStat, blockQueueStatExpected) {
		t.Errorf("Incorrect BlockQueueStat, expected: \n%+v, got: \n%+v", blockQueueStatExpected, blockQueueStat)
	}
}

func TestBlockDmInfo(t *testing.T) {
	blockdevice, err := NewFS("testdata/fixtures/proc", "testdata/fixtures/sys")
	if err != nil {
		t.Fatalf("failed to access blockdevice fs: %v", err)
	}
	devices, err := blockdevice.SysBlockDevices()
	if err != nil {
		t.Fatal(err)
	}
	dm0Info, err := blockdevice.SysBlockDeviceMapperInfo(devices[0])
	if err != nil {
		t.Fatal(err)
	}

	dm0InfoExpected := DeviceMapperInfo{
		Name:                      "vg0--lv_root",
		RqBasedSeqIOMergeDeadline: 0,
		Suspended:                 0,
		UseBlkMQ:                  0,
		UUID:                      "LVM-3zSHSR5Nbf4j7g6auAAefWY2CMaX01theZYEvQyecVsm2WtX3iY5q51qq5dWWOq7",
	}
	if !reflect.DeepEqual(dm0Info, dm0InfoExpected) {
		t.Errorf("Incorrect BlockQueueStat, expected: \n%+v, got: \n%+v", dm0InfoExpected, dm0Info)
	}

	dm1Info, err := blockdevice.SysBlockDeviceMapperInfo(devices[1])
	if err != nil {
		if _, ok := err.(*os.PathError); ok {
			// Fail the test if there's an error other than PathError.
			if !os.IsNotExist(err) {
				t.Fatal(err)
			}
		} else {
			t.Fatal(err)
		}
	} else {
		t.Fatal("SysBlockDeviceMapperInfo on sda was supposed to fail.")
	}
	dm1InfoExpected := DeviceMapperInfo{}
	if !reflect.DeepEqual(dm1Info, dm1InfoExpected) {
		t.Errorf("Incorrect BlockQueueStat, expected: \n%+v, got: \n%+v", dm0InfoExpected, dm0Info)
	}
}

func TestSysBlockDeviceUnderlyingDevices(t *testing.T) {
	blockdevice, err := NewFS("testdata/fixtures/proc", "testdata/fixtures/sys")
	if err != nil {
		t.Fatalf("failed to access blockdevice fs: %v", err)
	}
	devices, err := blockdevice.SysBlockDevices()
	if err != nil {
		t.Fatal(err)
	}

	underlying0, err := blockdevice.SysBlockDeviceUnderlyingDevices(devices[0])
	if err != nil {
		t.Fatal(err)
	}
	underlying0Expected := UnderlyingDeviceInfo{
		DeviceNames: []string{"sda"},
	}
	if !reflect.DeepEqual(underlying0, underlying0Expected) {
		t.Errorf("Incorrect BlockQueueStat, expected: \n%+v, got: \n%+v", underlying0Expected, underlying0)
	}
}
