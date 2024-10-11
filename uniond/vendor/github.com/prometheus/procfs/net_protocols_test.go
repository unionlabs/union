// Copyright 2020 The Prometheus Authors
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
	"strings"
	"testing"
)

func TestParseCapabilities(t *testing.T) {
	rawStr := "y  y  y  y  y  y  y  y  y  y  y  y  y  n  y  y  y  y  y\n"
	have := NetProtocolCapabilities{}
	err := have.parseCapabilities(strings.Fields(rawStr))
	if err != nil {
		t.Fatal(err)
	}

	want := NetProtocolCapabilities{true, true, true, true, true, true, true, true, true, true, true, true, true, false, true, true, true, true, true}
	if want != have {
		t.Errorf("want %+v\nhave %+v\n", want, have)
	}
}

func TestProtocolsParseLine(t *testing.T) {
	rawStr := "TCP       1984  93064  1225378   no     320   yes  kernel      y  y  y  y  y  y  y  y  y  y  y  y  y  n  y  y  y  y  y\n"
	protocols := NetProtocolStats{}
	have, err := protocols.parseLine(rawStr)
	if err != nil {
		t.Fatal(err)
	}

	want := NetProtocolStatLine{"TCP", 1984, 93064, 1225378, 0, 320, true, "kernel", NetProtocolCapabilities{true, true, true, true, true, true, true, true, true, true, true, true, true, false, true, true, true, true, true}}
	if want != *have {
		t.Errorf("want %+v\nhave %+v\n", want, have)
	}
}

func TestProtocolsParseProtocols(t *testing.T) {
	fs, err := NewFS(procTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	protocolStats, err := fs.NetProtocols()
	if err != nil {
		t.Fatal(err)
	}

	lines := map[string]NetProtocolStatLine{
		"PACKET":    {"PACKET", 1344, 2, -1, -1, 0, false, "kernel", NetProtocolCapabilities{false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false}},
		"PINGv6":    {"PINGv6", 1112, 0, -1, -1, 0, true, "kernel", NetProtocolCapabilities{true, true, true, false, false, true, false, false, true, true, true, true, false, true, true, true, true, true, false}},
		"RAWv6":     {"RAWv6", 1112, 1, -1, -1, 0, true, "kernel", NetProtocolCapabilities{true, true, true, false, true, true, true, false, true, true, true, true, false, true, true, true, true, false, false}},
		"UDPLITEv6": {"UDPLITEv6", 1216, 0, 57, -1, 0, true, "kernel", NetProtocolCapabilities{true, true, true, false, true, true, true, false, true, true, true, true, false, false, false, true, true, true, false}},
		"UDPv6":     {"UDPv6", 1216, 10, 57, -1, 0, true, "kernel", NetProtocolCapabilities{true, true, true, false, true, true, true, false, true, true, true, true, false, false, false, true, true, true, false}},
		"TCPv6":     {"TCPv6", 2144, 1937, 1225378, 0, 320, true, "kernel", NetProtocolCapabilities{true, true, true, true, true, true, true, true, true, true, true, true, true, false, true, true, true, true, true}},
		"UNIX":      {"UNIX", 1024, 120, -1, -1, 0, true, "kernel", NetProtocolCapabilities{false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false}},
		"UDP-Lite":  {"UDP-Lite", 1024, 0, 57, -1, 0, true, "kernel", NetProtocolCapabilities{true, true, true, false, true, true, true, false, true, true, true, true, true, false, false, true, true, true, false}},
		"PING":      {"PING", 904, 0, -1, -1, 0, true, "kernel", NetProtocolCapabilities{true, true, true, false, false, true, false, false, true, true, true, true, false, true, true, true, true, true, false}},
		"RAW":       {"RAW", 912, 0, -1, -1, 0, true, "kernel", NetProtocolCapabilities{true, true, true, false, true, true, true, false, true, true, true, true, false, true, true, true, true, false, false}},
		"UDP":       {"UDP", 1024, 73, 57, -1, 0, true, "kernel", NetProtocolCapabilities{true, true, true, false, true, true, true, false, true, true, true, true, true, false, false, true, true, true, false}},
		"TCP":       {"TCP", 1984, 93064, 1225378, 1, 320, true, "kernel", NetProtocolCapabilities{true, true, true, true, true, true, true, true, true, true, true, true, true, false, true, true, true, true, true}},
		"NETLINK":   {"NETLINK", 1040, 16, -1, -1, 0, false, "kernel", NetProtocolCapabilities{false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false}},
	}

	if want, have := len(lines), len(protocolStats); want != have {
		t.Errorf("want %d parsed net/protocols lines, have %d", want, have)
	}
	for _, line := range protocolStats {
		if want, have := lines[line.Name], line; want != have {
			t.Errorf("%s: want %v, have %v", line.Name, want, have)
		}
	}
}
