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

package procfs

import (
	"reflect"
	"testing"
)

func TestParseCgroupSummaryString(t *testing.T) {
	tests := []struct {
		name          string
		s             string
		shouldErr     bool
		CgroupSummary *CgroupSummary
	}{
		{
			name:      "cpuset simple line",
			s:         "cpuset	7	148	1",
			shouldErr: false,
			CgroupSummary: &CgroupSummary{
				SubsysName: "cpuset",
				Hierarchy:  7,
				Cgroups:    148,
				Enabled:    1,
			},
		},
		{
			name:          "memory cgroup number mis format",
			s:             "memory	9	##	1",
			shouldErr:     true,
			CgroupSummary: nil,
		},
	}

	for i, test := range tests {
		t.Logf("[%02d] test %q", i, test.name)

		CgroupSummary, err := parseCgroupSummaryString(test.s)

		if test.shouldErr && err == nil {
			t.Errorf("%s: expected an error, but none occurred", test.name)
		}
		if !test.shouldErr && err != nil {
			t.Errorf("%s: unexpected error: %v", test.name, err)
		}

		if want, have := test.CgroupSummary, CgroupSummary; !reflect.DeepEqual(want, have) {
			t.Errorf("cgroup:\nwant:\n%+v\nhave:\n%+v", want, have)
		}
	}

}
