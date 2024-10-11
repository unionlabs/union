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
	"testing"
)

func TestProcNetstat(t *testing.T) {
	p, err := getProcFixtures(t).Proc(26231)
	if err != nil {
		t.Fatal(err)
	}

	procNetstat, err := p.Netstat()
	if err != nil {
		t.Fatal(err)
	}

	for _, test := range []struct {
		name string
		want float64
		have float64
	}{
		{name: "pid", want: 26231, have: float64(procNetstat.PID)},
		{name: "TcpExt:SyncookiesSent", want: 0, have: *procNetstat.TcpExt.SyncookiesSent},
		{name: "TcpExt:EmbryonicRsts", want: 1, have: *procNetstat.TcpExt.EmbryonicRsts},
		{name: "TcpExt:TW", want: 83, have: *procNetstat.TcpExt.TW},
		{name: "TcpExt:PAWSEstab", want: 3640, have: *procNetstat.TcpExt.PAWSEstab},

		{name: "IpExt:InNoRoutes", want: 0, have: *procNetstat.IpExt.InNoRoutes},
		{name: "IpExt:InMcastPkts", want: 208, have: *procNetstat.IpExt.InMcastPkts},
		{name: "IpExt:OutMcastPkts", want: 214, have: *procNetstat.IpExt.OutMcastPkts},
	} {
		if test.want != test.have {
			t.Errorf("want %s %f, have %f", test.name, test.want, test.have)
		}
	}
}
