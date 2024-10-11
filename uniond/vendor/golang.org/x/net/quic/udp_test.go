// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"fmt"
	"net"
	"net/netip"
	"runtime"
	"testing"
)

func TestUDPSourceUnspecified(t *testing.T) {
	// Send datagram with no source address set.
	runUDPTest(t, func(t *testing.T, test udpTest) {
		t.Logf("%v", test.dstAddr)
		data := []byte("source unspecified")
		if err := test.src.Write(datagram{
			b:        data,
			peerAddr: test.dstAddr,
		}); err != nil {
			t.Fatalf("Write: %v", err)
		}
		got := <-test.dgramc
		if !bytes.Equal(got.b, data) {
			t.Errorf("got datagram {%x}, want {%x}", got.b, data)
		}
	})
}

func TestUDPSourceSpecified(t *testing.T) {
	// Send datagram with source address set.
	runUDPTest(t, func(t *testing.T, test udpTest) {
		data := []byte("source specified")
		if err := test.src.Write(datagram{
			b:         data,
			peerAddr:  test.dstAddr,
			localAddr: test.src.LocalAddr(),
		}); err != nil {
			t.Fatalf("Write: %v", err)
		}
		got := <-test.dgramc
		if !bytes.Equal(got.b, data) {
			t.Errorf("got datagram {%x}, want {%x}", got.b, data)
		}
	})
}

func TestUDPSourceInvalid(t *testing.T) {
	// Send datagram with source address set to an address not associated with the connection.
	if !udpInvalidLocalAddrIsError {
		t.Skipf("%v: sending from invalid source succeeds", runtime.GOOS)
	}
	runUDPTest(t, func(t *testing.T, test udpTest) {
		var localAddr netip.AddrPort
		if test.src.LocalAddr().Addr().Is4() {
			localAddr = netip.MustParseAddrPort("127.0.0.2:1234")
		} else {
			localAddr = netip.MustParseAddrPort("[::2]:1234")
		}
		data := []byte("source invalid")
		if err := test.src.Write(datagram{
			b:         data,
			peerAddr:  test.dstAddr,
			localAddr: localAddr,
		}); err == nil {
			t.Errorf("Write with invalid localAddr succeeded; want error")
		}
	})
}

func TestUDPECN(t *testing.T) {
	if !udpECNSupport {
		t.Skipf("%v: no ECN support", runtime.GOOS)
	}
	// Send datagrams with ECN bits set, verify the ECN bits are received.
	runUDPTest(t, func(t *testing.T, test udpTest) {
		for _, ecn := range []ecnBits{ecnNotECT, ecnECT1, ecnECT0, ecnCE} {
			if err := test.src.Write(datagram{
				b:        []byte{1, 2, 3, 4},
				peerAddr: test.dstAddr,
				ecn:      ecn,
			}); err != nil {
				t.Fatalf("Write: %v", err)
			}
			got := <-test.dgramc
			if got.ecn != ecn {
				t.Errorf("sending ECN bits %x, got %x", ecn, got.ecn)
			}
		}
	})
}

type udpTest struct {
	src     *netUDPConn
	dst     *netUDPConn
	dstAddr netip.AddrPort
	dgramc  chan *datagram
}

// runUDPTest calls f with a pair of UDPConns in a matrix of network variations:
// udp, udp4, and udp6, and variations on binding to an unspecified address (0.0.0.0)
// or a specified one.
func runUDPTest(t *testing.T, f func(t *testing.T, u udpTest)) {
	for _, test := range []struct {
		srcNet, srcAddr, dstNet, dstAddr string
	}{
		{"udp4", "127.0.0.1", "udp", ""},
		{"udp4", "127.0.0.1", "udp4", ""},
		{"udp4", "127.0.0.1", "udp4", "127.0.0.1"},
		{"udp6", "::1", "udp", ""},
		{"udp6", "::1", "udp6", ""},
		{"udp6", "::1", "udp6", "::1"},
	} {
		spec := "spec"
		if test.dstAddr == "" {
			spec = "unspec"
		}
		t.Run(fmt.Sprintf("%v/%v/%v", test.srcNet, test.dstNet, spec), func(t *testing.T) {
			// See: https://go.googlesource.com/go/+/refs/tags/go1.22.0/src/net/ipsock.go#47
			// On these platforms, conns with network="udp" cannot accept IPv6.
			switch runtime.GOOS {
			case "dragonfly", "openbsd":
				if test.srcNet == "udp6" && test.dstNet == "udp" {
					t.Skipf("%v: no support for mapping IPv4 address to IPv6", runtime.GOOS)
				}
			case "plan9":
				t.Skipf("ReadMsgUDP not supported on %s", runtime.GOOS)
			}
			if runtime.GOARCH == "wasm" && test.srcNet == "udp6" {
				t.Skipf("%v: IPv6 tests fail when using wasm fake net", runtime.GOARCH)
			}

			srcAddr := netip.AddrPortFrom(netip.MustParseAddr(test.srcAddr), 0)
			srcConn, err := net.ListenUDP(test.srcNet, net.UDPAddrFromAddrPort(srcAddr))
			if err != nil {
				// If ListenUDP fails here, we presumably don't have
				// IPv4/IPv6 configured.
				t.Skipf("ListenUDP(%q, %v) = %v", test.srcNet, srcAddr, err)
			}
			t.Cleanup(func() { srcConn.Close() })
			src, err := newNetUDPConn(srcConn)
			if err != nil {
				t.Fatalf("newNetUDPConn: %v", err)
			}

			var dstAddr netip.AddrPort
			if test.dstAddr != "" {
				dstAddr = netip.AddrPortFrom(netip.MustParseAddr(test.dstAddr), 0)
			}
			dstConn, err := net.ListenUDP(test.dstNet, net.UDPAddrFromAddrPort(dstAddr))
			if err != nil {
				t.Skipf("ListenUDP(%q, nil) = %v", test.dstNet, err)
			}
			dst, err := newNetUDPConn(dstConn)
			if err != nil {
				dstConn.Close()
				t.Fatalf("newNetUDPConn: %v", err)
			}

			dgramc := make(chan *datagram)
			go func() {
				defer close(dgramc)
				dst.Read(func(dgram *datagram) {
					dgramc <- dgram
				})
			}()
			t.Cleanup(func() {
				dstConn.Close()
				for range dgramc {
					t.Errorf("test read unexpected datagram")
				}
			})

			f(t, udpTest{
				src: src,
				dst: dst,
				dstAddr: netip.AddrPortFrom(
					srcAddr.Addr(),
					dst.LocalAddr().Port(),
				),
				dgramc: dgramc,
			})
		})
	}
}
