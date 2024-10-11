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

import "testing"

func TestProcSnmp(t *testing.T) {
	p, err := getProcFixtures(t).Proc(26231)
	if err != nil {
		t.Fatal(err)
	}

	procSnmp, err := p.Snmp()
	if err != nil {
		t.Fatal(err)
	}

	for _, test := range []struct {
		name string
		want float64
		have float64
	}{
		{name: "pid", want: 26231, have: float64(procSnmp.PID)},
		{name: "IP:Forwarding", want: 2, have: *procSnmp.Ip.Forwarding},
		{name: "IP:DefaultTTL", want: 64, have: *procSnmp.Ip.DefaultTTL},
		{name: "Icmp:InMsgs", want: 45, have: *procSnmp.Icmp.InMsgs},
		{name: "IcmpMsg:InType3", want: 45, have: *procSnmp.IcmpMsg.InType3},
		{name: "IcmpMsg:OutType3", want: 50, have: *procSnmp.IcmpMsg.OutType3},
		{name: "TCP:RtoAlgorithm", want: 1, have: *procSnmp.Tcp.RtoAlgorithm},
		{name: "TCP:RtoMin", want: 200, have: *procSnmp.Tcp.RtoMin},
		{name: "Udp:InDatagrams", want: 10179, have: *procSnmp.Udp.InDatagrams},
		{name: "Udp:NoPorts", want: 50, have: *procSnmp.Udp.NoPorts},
		{name: "UdpLite:InDatagrams", want: 0, have: *procSnmp.UdpLite.NoPorts},
		{name: "UdpLite:NoPorts", want: 0, have: *procSnmp.UdpLite.NoPorts},
	} {
		if test.want != test.have {
			t.Errorf("want %s %f, have %f", test.name, test.want, test.have)
		}
	}

}
