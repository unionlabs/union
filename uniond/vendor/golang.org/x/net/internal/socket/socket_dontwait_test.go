// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build darwin || dragonfly || freebsd || linux || netbsd || openbsd || solaris

package socket_test

import (
	"bytes"
	"errors"
	"net"
	"runtime"
	"syscall"
	"testing"

	"golang.org/x/net/internal/socket"
	"golang.org/x/net/nettest"
)

func TestUDPDontwait(t *testing.T) {
	c, err := nettest.NewLocalPacketListener("udp")
	if err != nil {
		t.Skipf("not supported on %s/%s: %v", runtime.GOOS, runtime.GOARCH, err)
	}
	defer c.Close()
	cc, err := socket.NewConn(c.(*net.UDPConn))
	if err != nil {
		t.Fatal(err)
	}
	isErrWouldblock := func(err error) bool {
		var errno syscall.Errno
		return errors.As(err, &errno) && (errno == syscall.EAGAIN || errno == syscall.EWOULDBLOCK)
	}

	t.Run("Message-dontwait", func(t *testing.T) {
		// Read before something was sent; expect EWOULDBLOCK
		b := make([]byte, 32)
		rm := socket.Message{
			Buffers: [][]byte{b},
		}
		if err := cc.RecvMsg(&rm, syscall.MSG_DONTWAIT); !isErrWouldblock(err) {
			t.Fatal(err)
		}
		// To trigger EWOULDBLOCK by SendMsg, we have to send faster than what the
		// system/network is able to process. Whether or not we can trigger this
		// depends on the system, specifically on write buffer sizes and the speed
		// of the network interface.
		// We cannot expect to quickly and reliably trigger this, especially not
		// because this test sends data over a (fast) loopback. Consequently, we
		// only check that sending with MSG_DONTWAIT works at all and don't attempt
		// testing that we would eventually get EWOULDBLOCK here.
		data := []byte("HELLO-R-U-THERE")
		wm := socket.Message{
			Buffers: [][]byte{data},
			Addr:    c.LocalAddr(),
		}
		// Send one message, repeat until we don't get EWOULDBLOCK. This will likely succeed at the first attempt.
		for {
			err := cc.SendMsg(&wm, syscall.MSG_DONTWAIT)
			if err == nil {
				break
			} else if !isErrWouldblock(err) {
				t.Fatal(err)
			}
		}
		// Read the message now available; again, this will likely succeed at the first attempt.
		for {
			err := cc.RecvMsg(&rm, syscall.MSG_DONTWAIT)
			if err == nil {
				break
			} else if !isErrWouldblock(err) {
				t.Fatal(err)
			}
		}
		if !bytes.Equal(b[:rm.N], data) {
			t.Fatalf("got %#v; want %#v", b[:rm.N], data)
		}
	})
	switch runtime.GOOS {
	case "android", "linux":
		t.Run("Messages", func(t *testing.T) {
			data := []byte("HELLO-R-U-THERE")
			wmbs := bytes.SplitAfter(data, []byte("-"))
			wms := []socket.Message{
				{Buffers: wmbs[:1], Addr: c.LocalAddr()},
				{Buffers: wmbs[1:], Addr: c.LocalAddr()},
			}
			b := make([]byte, 32)
			rmbs := [][][]byte{{b[:len(wmbs[0])]}, {b[len(wmbs[0]):]}}
			rms := []socket.Message{
				{Buffers: rmbs[0]},
				{Buffers: rmbs[1]},
			}
			_, err := cc.RecvMsgs(rms, syscall.MSG_DONTWAIT)
			if !isErrWouldblock(err) {
				t.Fatal(err)
			}
			for ntot := 0; ntot < len(wms); {
				n, err := cc.SendMsgs(wms[ntot:], syscall.MSG_DONTWAIT)
				if err == nil {
					ntot += n
				} else if !isErrWouldblock(err) {
					t.Fatal(err)
				}
			}
			for ntot := 0; ntot < len(rms); {
				n, err := cc.RecvMsgs(rms[ntot:], syscall.MSG_DONTWAIT)
				if err == nil {
					ntot += n
				} else if !isErrWouldblock(err) {
					t.Fatal(err)
				}
			}
			nn := 0
			for i := 0; i < len(rms); i++ {
				nn += rms[i].N
			}
			if !bytes.Equal(b[:nn], data) {
				t.Fatalf("got %#v; want %#v", b[:nn], data)
			}
		})
	}
}
