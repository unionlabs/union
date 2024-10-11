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

package prometheus

import (
	"bytes"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"testing"

	"github.com/prometheus/common/expfmt"
	"github.com/prometheus/procfs"

	dto "github.com/prometheus/client_model/go"
)

func TestProcessCollector(t *testing.T) {
	if _, err := procfs.Self(); err != nil {
		t.Skipf("skipping TestProcessCollector, procfs not available: %s", err)
	}

	registry := NewPedanticRegistry()
	if err := registry.Register(NewProcessCollector(ProcessCollectorOpts{})); err != nil {
		t.Fatal(err)
	}
	if err := registry.Register(NewProcessCollector(ProcessCollectorOpts{
		PidFn:        func() (int, error) { return os.Getpid(), nil },
		Namespace:    "foobar",
		ReportErrors: true, // No errors expected, just to see if none are reported.
	})); err != nil {
		t.Fatal(err)
	}

	mfs, err := registry.Gather()
	if err != nil {
		t.Fatal(err)
	}

	var buf bytes.Buffer
	for _, mf := range mfs {
		if _, err := expfmt.MetricFamilyToText(&buf, mf); err != nil {
			t.Fatal(err)
		}
	}

	for _, re := range []*regexp.Regexp{
		regexp.MustCompile("\nprocess_cpu_seconds_total [0-9]"),
		regexp.MustCompile("\nprocess_max_fds [1-9]"),
		regexp.MustCompile("\nprocess_open_fds [1-9]"),
		regexp.MustCompile("\nprocess_virtual_memory_max_bytes (-1|[1-9])"),
		regexp.MustCompile("\nprocess_virtual_memory_bytes [1-9]"),
		regexp.MustCompile("\nprocess_resident_memory_bytes [1-9]"),
		regexp.MustCompile("\nprocess_start_time_seconds [0-9.]{10,}"),
		regexp.MustCompile("\nprocess_network_receive_bytes_total [0-9]+"),
		regexp.MustCompile("\nprocess_network_transmit_bytes_total [0-9]+"),
		regexp.MustCompile("\nfoobar_process_cpu_seconds_total [0-9]"),
		regexp.MustCompile("\nfoobar_process_max_fds [1-9]"),
		regexp.MustCompile("\nfoobar_process_open_fds [1-9]"),
		regexp.MustCompile("\nfoobar_process_virtual_memory_max_bytes (-1|[1-9])"),
		regexp.MustCompile("\nfoobar_process_virtual_memory_bytes [1-9]"),
		regexp.MustCompile("\nfoobar_process_resident_memory_bytes [1-9]"),
		regexp.MustCompile("\nfoobar_process_start_time_seconds [0-9.]{10,}"),
		regexp.MustCompile("\nfoobar_process_network_receive_bytes_total [0-9]+"),
		regexp.MustCompile("\nfoobar_process_network_transmit_bytes_total [0-9]+"),
	} {
		if !re.Match(buf.Bytes()) {
			t.Errorf("want body to match %s\n%s", re, buf.String())
		}
	}

	brokenProcessCollector := NewProcessCollector(ProcessCollectorOpts{
		PidFn:        func() (int, error) { return 0, errors.New("boo") },
		ReportErrors: true,
	})

	ch := make(chan Metric)
	go func() {
		brokenProcessCollector.Collect(ch)
		close(ch)
	}()
	n := 0
	for m := range ch {
		n++
		pb := &dto.Metric{}
		err := m.Write(pb)
		if err == nil {
			t.Error("metric collected from broken process collector is unexpectedly valid")
		}
	}
	if n != 1 {
		t.Errorf("%d metrics collected, want 1", n)
	}
}

func TestNewPidFileFn(t *testing.T) {
	folderPath, err := os.Getwd()
	if err != nil {
		t.Error("failed to get current path")
	}
	mockPidFilePath := filepath.Join(folderPath, "mockPidFile")
	defer os.Remove(mockPidFilePath)

	testCases := []struct {
		mockPidFile       func()
		expectedErrPrefix string
		expectedPid       int
		desc              string
	}{
		{
			mockPidFile: func() {
				os.Remove(mockPidFilePath)
			},
			expectedErrPrefix: "can't read pid file",
			expectedPid:       0,
			desc:              "no existed pid file",
		},
		{
			mockPidFile: func() {
				os.Remove(mockPidFilePath)
				f, _ := os.Create(mockPidFilePath)
				f.Write([]byte("abc"))
				f.Close()
			},
			expectedErrPrefix: "can't parse pid file",
			expectedPid:       0,
			desc:              "existed pid file, error pid number",
		},
		{
			mockPidFile: func() {
				os.Remove(mockPidFilePath)
				f, _ := os.Create(mockPidFilePath)
				f.Write([]byte("123"))
				f.Close()
			},
			expectedErrPrefix: "",
			expectedPid:       123,
			desc:              "existed pid file, correct pid number",
		},
	}

	for _, tc := range testCases {
		fn := NewPidFileFn(mockPidFilePath)
		if fn == nil {
			t.Error("Should not get nil PidFileFn")
		}

		tc.mockPidFile()

		if pid, err := fn(); pid != tc.expectedPid || (err != nil && !strings.HasPrefix(err.Error(), tc.expectedErrPrefix)) {
			fmt.Println(err.Error())
			t.Error(tc.desc)
		}
	}
}
