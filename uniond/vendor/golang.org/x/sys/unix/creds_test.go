// Copyright 2012 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build linux

package unix_test

import (
	"bytes"
	"errors"
	"net"
	"os"
	"testing"
	"time"

	"golang.org/x/sys/unix"
)

// TestSCMCredentials tests the sending and receiving of credentials
// (PID, UID, GID) in an ancillary message between two UNIX
// sockets. The SO_PASSCRED socket option is enabled on the sending
// socket for this to work.
func TestSCMCredentials(t *testing.T) {
	socketTypeTests := []struct {
		socketType int
		dataLen    int
	}{
		{
			unix.SOCK_STREAM,
			1,
		}, {
			unix.SOCK_DGRAM,
			0,
		},
	}

	for _, tt := range socketTypeTests {
		fds, err := unix.Socketpair(unix.AF_LOCAL, tt.socketType, 0)
		if err != nil {
			t.Fatalf("Socketpair: %v", err)
		}

		err = unix.SetsockoptInt(fds[0], unix.SOL_SOCKET, unix.SO_PASSCRED, 1)
		if err != nil {
			unix.Close(fds[0])
			unix.Close(fds[1])
			t.Fatalf("SetsockoptInt: %v", err)
		}

		srvFile := os.NewFile(uintptr(fds[0]), "server")
		cliFile := os.NewFile(uintptr(fds[1]), "client")
		defer srvFile.Close()
		defer cliFile.Close()

		srv, err := net.FileConn(srvFile)
		if err != nil {
			t.Errorf("FileConn: %v", err)
			return
		}
		defer srv.Close()

		cli, err := net.FileConn(cliFile)
		if err != nil {
			t.Errorf("FileConn: %v", err)
			return
		}
		defer cli.Close()

		var ucred unix.Ucred
		ucred.Pid = int32(os.Getpid())
		ucred.Uid = uint32(os.Getuid())
		ucred.Gid = uint32(os.Getgid())
		oob := unix.UnixCredentials(&ucred)

		// On SOCK_STREAM, this is internally going to send a dummy byte
		n, oobn, err := cli.(*net.UnixConn).WriteMsgUnix(nil, oob, nil)
		if err != nil {
			t.Fatalf("WriteMsgUnix: %v", err)
		}
		if n != 0 {
			t.Fatalf("WriteMsgUnix n = %d, want 0", n)
		}
		if oobn != len(oob) {
			t.Fatalf("WriteMsgUnix oobn = %d, want %d", oobn, len(oob))
		}

		oob2 := make([]byte, 10*len(oob))
		n, oobn2, flags, _, err := srv.(*net.UnixConn).ReadMsgUnix(nil, oob2)
		if err != nil {
			t.Fatalf("ReadMsgUnix: %v", err)
		}
		if flags != 0 && flags != unix.MSG_CMSG_CLOEXEC {
			t.Fatalf("ReadMsgUnix flags = %#x, want 0 or %#x (MSG_CMSG_CLOEXEC)", flags, unix.MSG_CMSG_CLOEXEC)
		}
		if n != tt.dataLen {
			t.Fatalf("ReadMsgUnix n = %d, want %d", n, tt.dataLen)
		}
		if oobn2 != oobn {
			// without SO_PASSCRED set on the socket, ReadMsgUnix will
			// return zero oob bytes
			t.Fatalf("ReadMsgUnix oobn = %d, want %d", oobn2, oobn)
		}
		oob2 = oob2[:oobn2]
		if !bytes.Equal(oob, oob2) {
			t.Fatal("ReadMsgUnix oob bytes don't match")
		}

		scm, err := unix.ParseSocketControlMessage(oob2)
		if err != nil {
			t.Fatalf("ParseSocketControlMessage: %v", err)
		}
		newUcred, err := unix.ParseUnixCredentials(&scm[0])
		if err != nil {
			t.Fatalf("ParseUnixCredentials: %v", err)
		}
		if *newUcred != ucred {
			t.Fatalf("ParseUnixCredentials = %+v, want %+v", newUcred, ucred)
		}
	}
}

func TestPktInfo(t *testing.T) {
	testcases := []struct {
		network string
		address *net.UDPAddr
	}{
		{"udp4", &net.UDPAddr{IP: net.ParseIP("127.0.0.1")}},
		{"udp6", &net.UDPAddr{IP: net.ParseIP("::1")}},
	}
	for _, test := range testcases {
		t.Run(test.network, func(t *testing.T) {
			conn, err := net.ListenUDP(test.network, test.address)
			if errors.Is(err, unix.EADDRNOTAVAIL) || errors.Is(err, unix.EAFNOSUPPORT) {
				t.Skipf("%v is not available", test.address)
			}
			if err != nil {
				t.Fatal("Listen:", err)
			}
			defer conn.Close()

			var pktInfo []byte
			var src net.IP
			switch test.network {
			case "udp4":
				var info4 unix.Inet4Pktinfo
				src = net.ParseIP("127.0.0.2").To4()
				copy(info4.Spec_dst[:], src)
				pktInfo = unix.PktInfo4(&info4)

			case "udp6":
				var info6 unix.Inet6Pktinfo
				src = net.ParseIP("2001:0DB8::1")
				copy(info6.Addr[:], src)
				pktInfo = unix.PktInfo6(&info6)

				raw, err := conn.SyscallConn()
				if err != nil {
					t.Fatal("SyscallConn:", err)
				}
				var opErr error
				err = raw.Control(func(fd uintptr) {
					opErr = unix.SetsockoptInt(int(fd), unix.SOL_IPV6, unix.IPV6_FREEBIND, 1)
				})
				if err != nil {
					t.Fatal("Control:", err)
				}
				if errors.Is(opErr, unix.ENOPROTOOPT) {
					// Happens on android-amd64-emu, maybe Android has disabled
					// IPV6_FREEBIND?
					t.Skip("IPV6_FREEBIND not supported")
				}
				if opErr != nil {
					t.Fatal("Can't enable IPV6_FREEBIND:", opErr)
				}
			}

			msg := []byte{1}
			addr := conn.LocalAddr().(*net.UDPAddr)
			_, _, err = conn.WriteMsgUDP(msg, pktInfo, addr)
			if err != nil {
				t.Fatal("WriteMsgUDP:", err)
			}

			conn.SetReadDeadline(time.Now().Add(100 * time.Millisecond))
			_, _, _, remote, err := conn.ReadMsgUDP(msg, nil)
			if err != nil {
				t.Fatal("ReadMsgUDP:", err)
			}

			if !remote.IP.Equal(src) {
				t.Errorf("Got packet from %v, want %v", remote.IP, src)
			}
		})
	}
}

func TestParseOrigDstAddr(t *testing.T) {
	testcases := []struct {
		network string
		address *net.UDPAddr
	}{
		{"udp4", &net.UDPAddr{IP: net.IPv4(127, 0, 0, 1)}},
		{"udp6", &net.UDPAddr{IP: net.IPv6loopback}},
	}

	for _, test := range testcases {
		t.Run(test.network, func(t *testing.T) {
			conn, err := net.ListenUDP(test.network, test.address)
			if errors.Is(err, unix.EADDRNOTAVAIL) || errors.Is(err, unix.EAFNOSUPPORT) {
				t.Skipf("%v is not available", test.address)
			}
			if err != nil {
				t.Fatal("Listen:", err)
			}
			defer conn.Close()

			raw, err := conn.SyscallConn()
			if err != nil {
				t.Fatal("SyscallConn:", err)
			}

			var opErr error
			err = raw.Control(func(fd uintptr) {
				switch test.network {
				case "udp4":
					opErr = unix.SetsockoptInt(int(fd), unix.SOL_IP, unix.IP_RECVORIGDSTADDR, 1)
				case "udp6":
					opErr = unix.SetsockoptInt(int(fd), unix.SOL_IPV6, unix.IPV6_RECVORIGDSTADDR, 1)
				}
			})
			if err != nil {
				t.Fatal("Control:", err)
			}
			if opErr != nil {
				t.Fatal("Can't enable RECVORIGDSTADDR:", err)
			}

			msg := []byte{1}
			addr := conn.LocalAddr().(*net.UDPAddr)
			_, err = conn.WriteToUDP(msg, addr)
			if err != nil {
				t.Fatal("WriteToUDP:", err)
			}

			conn.SetReadDeadline(time.Now().Add(100 * time.Millisecond))
			oob := make([]byte, unix.CmsgSpace(unix.SizeofSockaddrInet6))
			_, oobn, _, _, err := conn.ReadMsgUDP(msg, oob)
			if err != nil {
				t.Fatal("ReadMsgUDP:", err)
			}

			scms, err := unix.ParseSocketControlMessage(oob[:oobn])
			if err != nil {
				t.Fatal("ParseSocketControlMessage:", err)
			}

			sa, err := unix.ParseOrigDstAddr(&scms[0])
			if err != nil {
				t.Fatal("ParseOrigDstAddr:", err)
			}

			switch test.network {
			case "udp4":
				sa4, ok := sa.(*unix.SockaddrInet4)
				if !ok {
					t.Fatalf("Got %T not *SockaddrInet4", sa)
				}

				lo := net.IPv4(127, 0, 0, 1)
				if addr := net.IP(sa4.Addr[:]); !lo.Equal(addr) {
					t.Errorf("Got address %v, want %v", addr, lo)
				}

				if sa4.Port != addr.Port {
					t.Errorf("Got port %d, want %d", sa4.Port, addr.Port)
				}

			case "udp6":
				sa6, ok := sa.(*unix.SockaddrInet6)
				if !ok {
					t.Fatalf("Got %T, want *SockaddrInet6", sa)
				}

				if addr := net.IP(sa6.Addr[:]); !net.IPv6loopback.Equal(addr) {
					t.Errorf("Got address %v, want %v", addr, net.IPv6loopback)
				}

				if sa6.Port != addr.Port {
					t.Errorf("Got port %d, want %d", sa6.Port, addr.Port)
				}
			}
		})
	}
}
