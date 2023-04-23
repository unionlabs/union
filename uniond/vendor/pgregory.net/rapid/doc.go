// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/*
Package rapid implements utilities for property-based testing.

Rapid checks that properties you define hold for a large number
of automatically generated test cases. If a failure is found, rapid
fails the current test and presents an automatically minimized
version of the failing test case.

Here is what a trivial test using rapid looks like:

	package rapid_test

	import (
		"net"
		"testing"

		"pgregory.net/rapid"
	)

	func TestParseValidIPv4(t *testing.T) {
		const ipv4re = `(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])` +
			`\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])` +
			`\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])` +
			`\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])`

		rapid.Check(t, func(t *rapid.T) {
			addr := rapid.StringMatching(ipv4re).Draw(t, "addr")
			ip := net.ParseIP(addr)
			if ip == nil || ip.String() != addr {
				t.Fatalf("parsed %q into %v", addr, ip)
			}
		})
	}
*/
package rapid
