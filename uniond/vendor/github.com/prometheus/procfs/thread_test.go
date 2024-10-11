// Copyright 2022 The Prometheus Authors
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

package procfs

import (
	"reflect"
	"sort"
	"strconv"
	"testing"
)

var (
	testPID  = int(27079)
	testTIDS = [...]int{27079, 27080, 27081, 27082, 27083}
)

func TestAllThreads(t *testing.T) {
	fixFS := getProcFixtures(t)
	threads, err := fixFS.AllThreads(testPID)
	if err != nil {
		t.Fatal(err)
	}
	sort.Sort(threads)
	for i, tid := range testTIDS {
		if wantTID, haveTID := tid, threads[i].PID; wantTID != haveTID {
			t.Errorf("want TID %d, have %d", wantTID, haveTID)
		}
		wantFS := fixFS.proc.Path(strconv.Itoa(testPID), "task")
		haveFS := string(threads[i].fs.proc)
		if wantFS != haveFS {
			t.Errorf("want fs %q, have %q", wantFS, haveFS)
		}
	}
}

func TestThreadStat(t *testing.T) {
	// Pull process and thread stats.
	proc, err := getProcFixtures(t).Proc(testPID)
	if err != nil {
		t.Fatal(err)
	}
	procStat, err := proc.Stat()
	if err != nil {
		t.Fatal(err)
	}

	threads, err := getProcFixtures(t).AllThreads(testPID)
	if err != nil {
		t.Fatal(err)
	}
	sort.Sort(threads)
	threadStats := make([]*ProcStat, len(threads))
	for i, thread := range threads {
		threadStat, err := thread.Stat()
		if err != nil {
			t.Fatal(err)
		}
		threadStats[i] = &threadStat
	}

	// The following fields should be shared between the process and its thread:
	procStatValue := reflect.ValueOf(procStat)
	sharedFields := [...]string{
		"PPID",
		"PGRP",
		"Session",
		"TTY",
		"TPGID",
		"VSize",
		"RSS",
	}

	for i, thread := range threads {
		threadStatValue := reflect.ValueOf(threadStats[i]).Elem()
		for _, f := range sharedFields {
			if want, have := procStatValue.FieldByName(f), threadStatValue.FieldByName(f); want.Interface() != have.Interface() {
				t.Errorf("TID %d, want %s %#v, have %#v", thread.PID, f, want, have)
			}
		}
	}

	// Thread specific fields:
	for i, thread := range threads {
		if want, have := thread.PID, threadStats[i].PID; want != have {
			t.Errorf("TID %d, want PID %d, have %d", thread.PID, want, have)
		}
	}

	// Finally exemplify the relationship between process and constituent
	// threads CPU times: each the former  ~ the sum of the corresponding
	// latter. Require -v flag.
	totalUTime, totalSTime := uint(0), uint(0)
	for _, thread := range threads {
		threadStat, err := thread.Stat()
		if err != nil {
			t.Fatal(err)
		}
		t.Logf("TID %d, UTime %d, STime %d", thread.PID, threadStat.UTime, threadStat.STime)
		totalUTime += threadStat.UTime
		totalSTime += threadStat.STime
	}
	t.Logf("PID %d, UTime %d, STime %d, total threads UTime %d, STime %d", proc.PID, procStat.UTime, procStat.STime, totalUTime, totalSTime)
}
