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
	"errors"
	"reflect"
	"testing"
)

func makeUint64(v uint64) *uint64 {
	return &v
}

func TestCPUTopology(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}
	cpus, err := fs.CPUs()
	if err != nil {
		t.Fatal(err)
	}
	if want, have := 3, len(cpus); want != have {
		t.Errorf("incorrect number of CPUs, have %v, want %v", want, have)
	}
	if want, have := "0", cpus[0].Number(); want != have {
		t.Errorf("incorrect name, have %v, want %v", want, have)
	}
	cpu0Topology, err := cpus[0].Topology()
	if err != nil {
		t.Fatal(err)
	}
	if want, have := "0", cpu0Topology.CoreID; want != have {
		t.Errorf("incorrect core ID, have %v, want %v", want, have)
	}
	if want, have := "0-7", cpu0Topology.CoreSiblingsList; want != have {
		t.Errorf("incorrect core siblings list, have %v, want %v", want, have)
	}
	cpu1Topology, err := cpus[1].Topology()
	if err != nil {
		t.Fatal(err)
	}
	if want, have := "0", cpu1Topology.PhysicalPackageID; want != have {
		t.Errorf("incorrect package ID, have %v, want %v", want, have)
	}
	if want, have := "1,5", cpu1Topology.ThreadSiblingsList; want != have {
		t.Errorf("incorrect thread siblings list, have %v, want %v", want, have)
	}
}

func TestCPUThermalThrottle(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}
	cpus, err := fs.CPUs()
	if err != nil {
		t.Fatal(err)
	}
	if want, have := 3, len(cpus); want != have {
		t.Errorf("incorrect number of CPUs, have %v, want %v", want, have)
	}
	cpu0Throttle, err := cpus[0].ThermalThrottle()
	if err != nil {
		t.Fatal(err)
	}
	if want, have := uint64(34818), cpu0Throttle.PackageThrottleCount; want != have {
		t.Errorf("incorrect package throttle count, have %v, want %v", want, have)
	}

	cpu1Throttle, err := cpus[1].ThermalThrottle()
	if err != nil {
		t.Fatal(err)
	}
	if want, have := uint64(523), cpu1Throttle.CoreThrottleCount; want != have {
		t.Errorf("incorrect core throttle count, have %v, want %v", want, have)
	}
}

func TestSystemCpufreq(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	c, err := fs.SystemCpufreq()
	if err != nil {
		t.Fatal(err)
	}

	systemCpufreq := []SystemCPUCpufreqStats{
		// The following files are missing for the first CPU:
		// * `cpuinfo_cur_freq`
		// * `time_in_state`
		// * `total_trans`
		{
			Name:                             "0",
			CpuinfoCurrentFrequency:          nil,
			CpuinfoMinimumFrequency:          makeUint64(800000),
			CpuinfoMaximumFrequency:          makeUint64(2400000),
			CpuinfoTransitionLatency:         makeUint64(0),
			ScalingCurrentFrequency:          makeUint64(1219917),
			ScalingMinimumFrequency:          makeUint64(800000),
			ScalingMaximumFrequency:          makeUint64(2400000),
			AvailableGovernors:               "performance powersave",
			Driver:                           "intel_pstate",
			Governor:                         "powersave",
			RelatedCpus:                      "0",
			SetSpeed:                         "<unsupported>",
			CpuinfoFrequencyDuration:         nil,
			CpuinfoFrequencyTransitionsTotal: nil,
			CpuinfoTransitionTable: &[][]uint64{
				{0, 3600000, 3400000, 3200000, 3000000, 2800000},
				{3600000, 0, 5, 0, 0, 0},
				{3400000, 4, 0, 2, 0, 0},
				{3200000, 0, 1, 0, 2, 0},
				{3000000, 0, 0, 1, 0, 3},
				{2800000, 0, 0, 0, 2, 0},
			},
		},
		// The following files are missing for the second CPU:
		// * `scaling_cur_freq`
		// * `trans_table`
		{
			Name:                     "1",
			CpuinfoCurrentFrequency:  makeUint64(1200195),
			CpuinfoMinimumFrequency:  makeUint64(1200000),
			CpuinfoMaximumFrequency:  makeUint64(3300000),
			CpuinfoTransitionLatency: makeUint64(4294967295),
			ScalingCurrentFrequency:  nil,
			ScalingMinimumFrequency:  makeUint64(1200000),
			ScalingMaximumFrequency:  makeUint64(3300000),
			AvailableGovernors:       "performance powersave",
			Driver:                   "intel_pstate",
			Governor:                 "powersave",
			RelatedCpus:              "1",
			SetSpeed:                 "<unsupported>",
			CpuinfoFrequencyDuration: &map[uint64]uint64{
				3600000: 2089,
				3400000: 136,
				3200000: 34,
				3000000: 67,
				2800000: 172488,
			},
			CpuinfoFrequencyTransitionsTotal: makeUint64(20),
			CpuinfoTransitionTable:           nil,
		},
	}

	if !reflect.DeepEqual(systemCpufreq, c) {
		t.Errorf("Result not correct: want %v, have %v", systemCpufreq, c)
	}
}

func TestIsolatedParsingCPU(t *testing.T) {
	var testParams = []struct {
		in  []byte
		res []uint16
		err error
	}{
		{[]byte(""), []uint16{}, nil},
		{[]byte("1\n"), []uint16{1}, nil},
		{[]byte("1"), []uint16{1}, nil},
		{[]byte("1,2"), []uint16{1, 2}, nil},
		{[]byte("1-2"), []uint16{1, 2}, nil},
		{[]byte("1-3"), []uint16{1, 2, 3}, nil},
		{[]byte("1,2-4"), []uint16{1, 2, 3, 4}, nil},
		{[]byte("1,3-4"), []uint16{1, 3, 4}, nil},
		{[]byte("1,3-4,7,20-21"), []uint16{1, 3, 4, 7, 20, 21}, nil},

		{[]byte("1,"), []uint16{1}, nil},
		{[]byte("1,2-"), nil, errors.New(`invalid cpu end range: strconv.Atoi: parsing "": invalid syntax`)},
		{[]byte("1,-3"), nil, errors.New(`invalid cpu start range: strconv.Atoi: parsing "": invalid syntax`)},
	}
	for _, params := range testParams {
		t.Run("blabla", func(t *testing.T) {
			res, err := parseCPURange(params.in)
			if !reflect.DeepEqual(res, params.res) {
				t.Fatalf("should have %v result: got %v", params.res, res)
			}
			if err != nil && params.err != nil && err.Error() != params.err.Error() {
				t.Fatalf("should have '%v' error: got '%v'", params.err, err)
			}
			if (err == nil || params.err == nil) && err != params.err {
				t.Fatalf("should have %v error: got %v", params.err, err)
			}

		})
	}
}
func TestIsolatedCPUs(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}
	isolated, err := fs.IsolatedCPUs()
	expected := []uint16{1, 2, 3, 4, 5, 6, 7, 9}
	if !reflect.DeepEqual(isolated, expected) {
		t.Errorf("Result not correct: want %v, have %v", expected, isolated)
	}
	if err != nil {
		t.Errorf("Error not correct: want %v, have %v", nil, err)
	}
}

func TestBinSearch(t *testing.T) {
	var testParams = []struct {
		elem  uint16
		elems []uint16
		res   bool
	}{
		{3, []uint16{1, 3, 5, 7, 9}, true},
		{4, []uint16{1, 3, 5, 7, 9}, false},
		{2, []uint16{}, false},
	}

	for _, param := range testParams {
		res := binSearch(param.elem, &param.elems)

		if res != param.res {
			t.Fatalf("Result not correct: want %v, have %v", param.res, res)
		}

	}
}
