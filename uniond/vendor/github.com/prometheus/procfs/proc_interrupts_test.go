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
	"testing"
)

func TestProcInterrupts(t *testing.T) {
	p, err := getProcFixtures(t).Proc(26231)
	if err != nil {
		t.Fatal(err)
	}

	interrupts, err := p.Interrupts()
	if err != nil {
		t.Fatal(err)
	}

	if want, have := 47, len(interrupts); want != have {
		t.Errorf("want length %d, have %d", want, have)
	}

	for _, test := range []struct {
		name string
		irq  string
		want Interrupt
	}{
		{
			name: "first line",
			irq:  "0",
			want: Interrupt{
				Info:    "IO-APIC",
				Devices: "2-edge timer",
				Values:  []string{"49", "0", "0", "0"},
			},
		},
		{
			name: "last line",
			irq:  "PIW",
			want: Interrupt{
				Info:    "Posted-interrupt wakeup event",
				Devices: "",
				Values:  []string{"0", "0", "0", "0"},
			},
		},
		{
			name: "empty devices",
			irq:  "LOC",
			want: Interrupt{
				Info:    "Local timer interrupts",
				Devices: "",
				Values:  []string{"10196", "7429", "8542", "8229"},
			},
		},
		{
			name: "single value",
			irq:  "ERR",
			want: Interrupt{
				Info:    "",
				Devices: "",
				Values:  []string{"0"},
			},
		},
	} {
		t.Run(test.name, func(t *testing.T) {
			if value, ok := interrupts[test.irq]; ok {
				if value.Info != test.want.Info {
					t.Errorf("info: want %s, have %s", test.want.Info, value.Info)
				}
				if value.Devices != test.want.Devices {
					t.Errorf("devices: want %s, have %s", test.want.Devices, value.Devices)
				}
				if !reflect.DeepEqual(value.Values, test.want.Values) {
					t.Errorf("values: want %v, have %v", test.want.Values, value.Values)
				}
			} else {
				t.Errorf("IRQ %s not found", test.irq)
			}
		})
	}
}
