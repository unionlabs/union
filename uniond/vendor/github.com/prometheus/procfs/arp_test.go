// Copyright 2019 The Prometheus Authors
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
	"net"
	"testing"
)

func TestARP(t *testing.T) {
	fs, err := NewFS(procTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	arpFile, err := fs.GatherARPEntries()
	if err != nil {
		t.Fatal(err)
	}

	if want, got := "192.168.224.1", arpFile[0].IPAddr.String(); want != got {
		t.Errorf("want 192.168.224.1, got %s", got)
	}

	if want, got := "00:50:56:c0:00:08", arpFile[0].HWAddr.String(); want != got {
		t.Errorf("want %s, got %s", want, got)
	}

	if want, got := "ens33", arpFile[0].Device; want != got {
		t.Errorf("want ens33, got %s", got)
	}

	if want, got := true, arpFile[0].IsComplete(); want != got {
		t.Errorf("want %t, got %t", want, got)
	}

	if want, got := "192.168.224.2", arpFile[1].IPAddr.String(); want != got {
		t.Errorf("want 192.168.224.2, got %s", got)
	}

	if want, got := make(net.HardwareAddr, 6).String(), arpFile[1].HWAddr.String(); want != got {
		t.Errorf("expected empty MAC, got %s", got)
	}

	if want, got := "ens33", arpFile[1].Device; want != got {
		t.Errorf("want %s, got %s", want, got)
	}

	if want, got := byte(0x0), arpFile[1].Flags; want != got {
		t.Errorf("want %b, got %b", want, got)
	}

	if want, got := false, arpFile[1].IsComplete(); want != got {
		t.Errorf("want %t, got %t", want, got)
	}
}
