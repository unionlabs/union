// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21 && darwin

package quic

import (
	"encoding/binary"

	"golang.org/x/sys/unix"
)

// See udp.go.
const (
	udpECNSupport              = true
	udpInvalidLocalAddrIsError = true
)

// Confusingly, on Darwin the contents of the IP_TOS option differ depending on whether
// it is used as an inbound or outbound cmsg.

func parseIPTOS(b []byte) (ecnBits, bool) {
	// Single byte. The low two bits are the ECN field.
	if len(b) != 1 {
		return 0, false
	}
	return ecnBits(b[0] & ecnMask), true
}

func appendCmsgECNv4(b []byte, ecn ecnBits) []byte {
	// 32-bit integer.
	// https://github.com/apple/darwin-xnu/blob/2ff845c2e033bd0ff64b5b6aa6063a1f8f65aa32/bsd/netinet/in_tclass.c#L1062-L1073
	b, data := appendCmsg(b, unix.IPPROTO_IP, unix.IP_TOS, 4)
	binary.NativeEndian.PutUint32(data, uint32(ecn))
	return b
}
