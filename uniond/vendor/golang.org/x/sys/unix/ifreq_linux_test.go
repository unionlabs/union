// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build linux

package unix

import (
	"bytes"
	"net"
	"testing"
	"unsafe"
)

// An ifreqUnion is shorthand for a byte array matching the
// architecture-dependent size of an ifreq's union field.
type ifreqUnion = [len(ifreq{}.Ifru)]byte

func TestNewIfreq(t *testing.T) {
	// Interface name too long.
	if _, err := NewIfreq("abcdefghijklmnop"); err != EINVAL {
		t.Fatalf("expected error EINVAL, but got: %v", err)
	}
}

func TestIfreqSize(t *testing.T) {
	// Ensure ifreq (generated) and Ifreq/ifreqData (hand-written to create a
	// safe wrapper and store a pointer field) are identical in size.
	want := unsafe.Sizeof(ifreq{})
	if got := unsafe.Sizeof(Ifreq{}); want != got {
		t.Fatalf("unexpected Ifreq size: got: %d, want: %d", got, want)
	}

	if got := unsafe.Sizeof(ifreqData{}); want != got {
		t.Fatalf("unexpected IfreqData size: got: %d, want: %d", got, want)
	}
}

func TestIfreqName(t *testing.T) {
	// Valid ifreq, expect the hard-coded testIfreq name.
	ifr := testIfreq(t)
	if want, got := ifreqName, ifr.Name(); want != got {
		t.Fatalf("unexpected ifreq name: got: %q, want: %q", got, want)
	}
}

func TestIfreqWithData(t *testing.T) {
	ifr := testIfreq(t)

	// Store pointer data in the ifreq so we can retrieve it and cast back later
	// for comparison.
	want := [5]byte{'h', 'e', 'l', 'l', 'o'}
	ifrd := ifr.withData(unsafe.Pointer(&want[0]))

	// Ensure the memory of the original Ifreq was not modified by SetData.
	if ifr.raw.Ifru != (ifreqUnion{}) {
		t.Fatalf("ifreq was unexpectedly modified: % #x", ifr.raw.Ifru)
	}

	got := *(*[5]byte)(ifrd.data)
	if want != got {
		t.Fatalf("unexpected ifreq data bytes:\n got: % #x\nwant: % #x", got, want)
	}
}

func TestIfreqInet4Addr(t *testing.T) {
	ifr := testIfreq(t)
	in := net.IPv4(192, 0, 2, 1).To4()
	if err := ifr.SetInet4Addr(in); err != nil {
		t.Fatalf("failed to set ifreq IPv4 address: %v", err)
	}

	// Store fixed offset data (AF_INET, IPv4 address) within underlying
	// sockaddr bytes. Everything else should be zeroed.
	want := ifreqUnion{4: 192, 5: 0, 6: 2, 7: 1}
	if isBigEndian {
		want[0] = 0x00
		want[1] = 0x02
	} else {
		want[0] = 0x02
		want[1] = 0x00
	}

	if got := ifr.raw.Ifru; want != got {
		t.Fatalf("unexpected ifreq sockaddr bytes:\n got: % #x\nwant: % #x", got, want)
	}

	got, err := ifr.Inet4Addr()
	if err != nil {
		t.Fatalf("failed to get ifreq IPv4 address: %v", err)
	}
	if !bytes.Equal(in, got) {
		t.Fatalf("unexpected ifreq IPv4 address:\n got: % #x\nwant: % #x", got, in)
	}

	// Invalid input, wrong length.
	if err := ifr.SetInet4Addr([]byte{0xff}); err == nil {
		t.Fatal("expected an error setting invalid IPv4 address, but none occurred")
	}

	// Invalid output, AF_INET is only set by SetInet4Addr input.
	ifr.SetUint32(0xffffffff)
	if _, err := ifr.Inet4Addr(); err == nil {
		t.Fatal("expected an error getting invalid IPv4 address, but none occurred")
	}
}

func TestIfreqUint16(t *testing.T) {
	ifr := testIfreq(t)
	const in = 0x0102
	ifr.SetUint16(in)

	// The layout of the bytes depends on the machine's endianness.
	var want ifreqUnion
	if isBigEndian {
		want[0] = 0x01
		want[1] = 0x02
	} else {
		want[0] = 0x02
		want[1] = 0x01
	}

	if got := ifr.raw.Ifru; want != got {
		t.Fatalf("unexpected ifreq uint16 bytes:\n got: % #x\nwant: % #x", got, want)
	}

	if got := ifr.Uint16(); in != got {
		t.Fatalf("unexpected ifreq uint16: got: %d, want: %d", got, in)
	}
}

func TestIfreqUint32(t *testing.T) {
	ifr := testIfreq(t)
	const in = 0x01020304
	ifr.SetUint32(in)

	// The layout of the bytes depends on the machine's endianness.
	var want ifreqUnion
	if isBigEndian {
		want[0] = 0x01
		want[1] = 0x02
		want[2] = 0x03
		want[3] = 0x04
	} else {
		want[0] = 0x04
		want[1] = 0x03
		want[2] = 0x02
		want[3] = 0x01
	}

	if got := ifr.raw.Ifru; want != got {
		t.Fatalf("unexpected ifreq uint32 bytes:\n got: % #x\nwant: % #x", got, want)
	}

	if got := ifr.Uint32(); in != got {
		t.Fatalf("unexpected ifreq uint32: got: %d, want: %d", got, in)
	}
}

// ifreqName is a hard-coded name for testIfreq.
const ifreqName = "eth0"

// testIfreq returns an Ifreq with a populated interface name.
func testIfreq(t *testing.T) *Ifreq {
	t.Helper()

	ifr, err := NewIfreq(ifreqName)
	if err != nil {
		t.Fatalf("failed to create ifreq: %v", err)
	}

	return ifr
}
