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

package procfs

import (
	"bytes"
	"reflect"
	"testing"
)

func TestParseNetRoute(t *testing.T) {
	var netRoute = []byte(`Iface            Destination  Gateway   Flags  RefCnt  Use  Metric  Mask      MTU  Window  IRTT
eno16780032      00000000     9503A8C0  0003   0       0    100     00000000  0    0       0
eno16780032      0000A8C0     00000000  0001   0       0    100     0000FFFF  0    0       0`)

	r := bytes.NewReader(netRoute)
	parsed, _ := parseNetRoute(r)
	want := []NetRouteLine{
		{
			Iface:       "eno16780032",
			Destination: 0,
			Gateway:     2500044992,
			Flags:       3,
			RefCnt:      0,
			Use:         0,
			Metric:      100,
			Mask:        0,
			MTU:         0,
			Window:      0,
			IRTT:        0,
		},
		{
			Iface:       "eno16780032",
			Destination: 43200,
			Gateway:     0,
			Flags:       1,
			RefCnt:      0,
			Use:         0,
			Metric:      100,
			Mask:        65535,
			MTU:         0,
			Window:      0,
			IRTT:        0,
		},
	}
	if !reflect.DeepEqual(want, parsed) {
		t.Errorf("want %v, parsed %v", want, parsed)
	}
}
