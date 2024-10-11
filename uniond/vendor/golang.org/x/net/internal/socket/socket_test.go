// Copyright 2017 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build aix || darwin || dragonfly || freebsd || linux || netbsd || openbsd || solaris || windows || zos

package socket_test

import (
	"bytes"
	"fmt"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"syscall"
	"testing"

	"golang.org/x/net/internal/socket"
	"golang.org/x/net/nettest"
)

func TestSocket(t *testing.T) {
	t.Run("Option", func(t *testing.T) {
		testSocketOption(t, &socket.Option{Level: syscall.SOL_SOCKET, Name: syscall.SO_RCVBUF, Len: 4})
	})
}

func testSocketOption(t *testing.T, so *socket.Option) {
	c, err := nettest.NewLocalPacketListener("udp")
	if err != nil {
		t.Skipf("not supported on %s/%s: %v", runtime.GOOS, runtime.GOARCH, err)
	}
	defer c.Close()
	cc, err := socket.NewConn(c.(net.Conn))
	if err != nil {
		t.Fatal(err)
	}
	const N = 2048
	if err := so.SetInt(cc, N); err != nil {
		t.Fatal(err)
	}
	n, err := so.GetInt(cc)
	if err != nil {
		t.Fatal(err)
	}
	if n < N {
		t.Fatalf("got %d; want greater than or equal to %d", n, N)
	}
}

type mockControl struct {
	Level int
	Type  int
	Data  []byte
}

func TestControlMessage(t *testing.T) {
	switch runtime.GOOS {
	case "windows":
		t.Skipf("not supported on %s", runtime.GOOS)
	}

	for _, tt := range []struct {
		cs []mockControl
	}{
		{
			[]mockControl{
				{Level: 1, Type: 1},
			},
		},
		{
			[]mockControl{
				{Level: 2, Type: 2, Data: []byte{0xfe}},
			},
		},
		{
			[]mockControl{
				{Level: 3, Type: 3, Data: []byte{0xfe, 0xff, 0xff, 0xfe}},
			},
		},
		{
			[]mockControl{
				{Level: 4, Type: 4, Data: []byte{0xfe, 0xff, 0xff, 0xfe, 0xfe, 0xff, 0xff, 0xfe}},
			},
		},
		{
			[]mockControl{
				{Level: 4, Type: 4, Data: []byte{0xfe, 0xff, 0xff, 0xfe, 0xfe, 0xff, 0xff, 0xfe}},
				{Level: 2, Type: 2, Data: []byte{0xfe}},
			},
		},
	} {
		var w []byte
		var tailPadLen int
		mm := socket.NewControlMessage([]int{0})
		for i, c := range tt.cs {
			m := socket.NewControlMessage([]int{len(c.Data)})
			l := len(m) - len(mm)
			if i == len(tt.cs)-1 && l > len(c.Data) {
				tailPadLen = l - len(c.Data)
			}
			w = append(w, m...)
		}

		var err error
		ww := make([]byte, len(w))
		copy(ww, w)
		m := socket.ControlMessage(ww)
		for _, c := range tt.cs {
			if err = m.MarshalHeader(c.Level, c.Type, len(c.Data)); err != nil {
				t.Fatalf("(%v).MarshalHeader() = %v", tt.cs, err)
			}
			copy(m.Data(len(c.Data)), c.Data)
			m = m.Next(len(c.Data))
		}
		m = socket.ControlMessage(w)
		for _, c := range tt.cs {
			m, err = m.Marshal(c.Level, c.Type, c.Data)
			if err != nil {
				t.Fatalf("(%v).Marshal() = %v", tt.cs, err)
			}
		}
		if !bytes.Equal(ww, w) {
			t.Fatalf("got %#v; want %#v", ww, w)
		}

		ws := [][]byte{w}
		if tailPadLen > 0 {
			// Test a message with no tail padding.
			nopad := w[:len(w)-tailPadLen]
			ws = append(ws, [][]byte{nopad}...)
		}
		for _, w := range ws {
			ms, err := socket.ControlMessage(w).Parse()
			if err != nil {
				t.Fatalf("(%v).Parse() = %v", tt.cs, err)
			}
			for i, m := range ms {
				lvl, typ, dataLen, err := m.ParseHeader()
				if err != nil {
					t.Fatalf("(%v).ParseHeader() = %v", tt.cs, err)
				}
				if lvl != tt.cs[i].Level || typ != tt.cs[i].Type || dataLen != len(tt.cs[i].Data) {
					t.Fatalf("%v: got %d, %d, %d; want %d, %d, %d", tt.cs[i], lvl, typ, dataLen, tt.cs[i].Level, tt.cs[i].Type, len(tt.cs[i].Data))
				}
			}
		}
	}
}

func TestUDP(t *testing.T) {
	switch runtime.GOOS {
	case "windows":
		t.Skipf("not supported on %s", runtime.GOOS)
	}

	c, err := nettest.NewLocalPacketListener("udp")
	if err != nil {
		t.Skipf("not supported on %s/%s: %v", runtime.GOOS, runtime.GOARCH, err)
	}
	defer c.Close()
	// test that wrapped connections work with NewConn too
	type wrappedConn struct{ *net.UDPConn }
	cc, err := socket.NewConn(&wrappedConn{c.(*net.UDPConn)})
	if err != nil {
		t.Fatal(err)
	}

	// create a dialed connection talking (only) to c/cc
	cDialed, err := net.Dial("udp", c.LocalAddr().String())
	if err != nil {
		t.Fatal(err)
	}
	ccDialed, err := socket.NewConn(cDialed)
	if err != nil {
		t.Fatal(err)
	}

	const data = "HELLO-R-U-THERE"
	messageTests := []struct {
		name string
		conn *socket.Conn
		dest net.Addr
	}{
		{
			name: "Message",
			conn: cc,
			dest: c.LocalAddr(),
		},
		{
			name: "Message-dialed",
			conn: ccDialed,
			dest: nil,
		},
	}
	for _, tt := range messageTests {
		t.Run(tt.name, func(t *testing.T) {
			wm := socket.Message{
				Buffers: bytes.SplitAfter([]byte(data), []byte("-")),
				Addr:    tt.dest,
			}
			if err := tt.conn.SendMsg(&wm, 0); err != nil {
				t.Fatal(err)
			}
			b := make([]byte, 32)
			rm := socket.Message{
				Buffers: [][]byte{b[:1], b[1:3], b[3:7], b[7:11], b[11:]},
			}
			if err := cc.RecvMsg(&rm, 0); err != nil {
				t.Fatal(err)
			}
			received := string(b[:rm.N])
			if received != data {
				t.Fatalf("Roundtrip SendMsg/RecvMsg got %q; want %q", received, data)
			}
		})
	}

	switch runtime.GOOS {
	case "android", "linux":
		messagesTests := []struct {
			name string
			conn *socket.Conn
			dest net.Addr
		}{
			{
				name: "Messages",
				conn: cc,
				dest: c.LocalAddr(),
			},
			{
				name: "Messages-dialed",
				conn: ccDialed,
				dest: nil,
			},
		}
		for _, tt := range messagesTests {
			t.Run(tt.name, func(t *testing.T) {
				wmbs := bytes.SplitAfter([]byte(data), []byte("-"))
				wms := []socket.Message{
					{Buffers: wmbs[:1], Addr: tt.dest},
					{Buffers: wmbs[1:], Addr: tt.dest},
				}
				n, err := tt.conn.SendMsgs(wms, 0)
				if err != nil {
					t.Fatal(err)
				}
				if n != len(wms) {
					t.Fatalf("SendMsgs(%#v) != %d; want %d", wms, n, len(wms))
				}
				rmbs := [][]byte{make([]byte, 32), make([]byte, 32)}
				rms := []socket.Message{
					{Buffers: [][]byte{rmbs[0]}},
					{Buffers: [][]byte{rmbs[1][:1], rmbs[1][1:3], rmbs[1][3:7], rmbs[1][7:11], rmbs[1][11:]}},
				}
				nrecv := 0
				for nrecv < len(rms) {
					n, err := cc.RecvMsgs(rms[nrecv:], 0)
					if err != nil {
						t.Fatal(err)
					}
					nrecv += n
				}
				received0, received1 := string(rmbs[0][:rms[0].N]), string(rmbs[1][:rms[1].N])
				assembled := received0 + received1
				assembledReordered := received1 + received0
				if assembled != data && assembledReordered != data {
					t.Fatalf("Roundtrip SendMsgs/RecvMsgs got %q / %q; want %q", assembled, assembledReordered, data)
				}
			})
		}
		t.Run("Messages-undialed-no-dst", func(t *testing.T) {
			// sending without destination address should fail.
			// This checks that the internally recycled buffers are reset correctly.
			data := []byte("HELLO-R-U-THERE")
			wmbs := bytes.SplitAfter(data, []byte("-"))
			wms := []socket.Message{
				{Buffers: wmbs[:1], Addr: nil},
				{Buffers: wmbs[1:], Addr: nil},
			}
			n, err := cc.SendMsgs(wms, 0)
			if n != 0 && err == nil {
				t.Fatal("expected error, destination address required")
			}
		})
	}

	// The behavior of transmission for zero byte paylaod depends
	// on each platform implementation. Some may transmit only
	// protocol header and options, other may transmit nothing.
	// We test only that SendMsg and SendMsgs will not crash with
	// empty buffers.
	wm := socket.Message{
		Buffers: [][]byte{{}},
		Addr:    c.LocalAddr(),
	}
	cc.SendMsg(&wm, 0)
	wms := []socket.Message{
		{Buffers: [][]byte{{}}, Addr: c.LocalAddr()},
	}
	cc.SendMsgs(wms, 0)
}

func BenchmarkUDP(b *testing.B) {
	c, err := nettest.NewLocalPacketListener("udp")
	if err != nil {
		b.Skipf("not supported on %s/%s: %v", runtime.GOOS, runtime.GOARCH, err)
	}
	defer c.Close()
	cc, err := socket.NewConn(c.(net.Conn))
	if err != nil {
		b.Fatal(err)
	}
	data := []byte("HELLO-R-U-THERE")
	wm := socket.Message{
		Buffers: [][]byte{data},
		Addr:    c.LocalAddr(),
	}
	rm := socket.Message{
		Buffers: [][]byte{make([]byte, 128)},
		OOB:     make([]byte, 128),
	}

	for M := 1; M <= 1<<9; M = M << 1 {
		b.Run(fmt.Sprintf("Iter-%d", M), func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				for j := 0; j < M; j++ {
					if err := cc.SendMsg(&wm, 0); err != nil {
						b.Fatal(err)
					}
					if err := cc.RecvMsg(&rm, 0); err != nil {
						b.Fatal(err)
					}
				}
			}
		})
		switch runtime.GOOS {
		case "android", "linux":
			wms := make([]socket.Message, M)
			for i := range wms {
				wms[i].Buffers = [][]byte{data}
				wms[i].Addr = c.LocalAddr()
			}
			rms := make([]socket.Message, M)
			for i := range rms {
				rms[i].Buffers = [][]byte{make([]byte, 128)}
				rms[i].OOB = make([]byte, 128)
			}
			b.Run(fmt.Sprintf("Batch-%d", M), func(b *testing.B) {
				for i := 0; i < b.N; i++ {
					if _, err := cc.SendMsgs(wms, 0); err != nil {
						b.Fatal(err)
					}
					if _, err := cc.RecvMsgs(rms, 0); err != nil {
						b.Fatal(err)
					}
				}
			})
		}
	}
}

func TestRace(t *testing.T) {
	tests := []string{
		`
package main
import (
	"log"
	"net"

	"golang.org/x/net/ipv4"
)

var g byte

func main() {
	c, err := net.ListenPacket("udp", "127.0.0.1:0")
	if err != nil {
		log.Fatalf("ListenPacket: %v", err)
	}
	cc := ipv4.NewPacketConn(c)
	sync := make(chan bool)
	src := make([]byte, 100)
	dst := make([]byte, 100)
	go func() {
		if _, err := cc.WriteTo(src, nil, c.LocalAddr()); err != nil {
			log.Fatalf("WriteTo: %v", err)
		}
	}()
	go func() {
		if _, _, _, err := cc.ReadFrom(dst); err != nil {
			log.Fatalf("ReadFrom: %v", err)
		}
		sync <- true
	}()
	g = dst[0]
	<-sync
}
`,
		`
package main
import (
	"log"
	"net"

	"golang.org/x/net/ipv4"
)

func main() {
	c, err := net.ListenPacket("udp", "127.0.0.1:0")
	if err != nil {
		log.Fatalf("ListenPacket: %v", err)
	}
	cc := ipv4.NewPacketConn(c)
	sync := make(chan bool)
	src := make([]byte, 100)
	dst := make([]byte, 100)
	go func() {
		if _, err := cc.WriteTo(src, nil, c.LocalAddr()); err != nil {
			log.Fatalf("WriteTo: %v", err)
		}
		sync <- true
	}()
	src[0] = 0
	go func() {
		if _, _, _, err := cc.ReadFrom(dst); err != nil {
			log.Fatalf("ReadFrom: %v", err)
		}
	}()
	<-sync
}
`,
	}
	platforms := map[string]bool{
		"linux/amd64":   true,
		"linux/ppc64le": true,
		"linux/arm64":   true,
	}
	if !platforms[runtime.GOOS+"/"+runtime.GOARCH] {
		t.Skip("skipping test on non-race-enabled host.")
	}
	if runtime.Compiler == "gccgo" {
		t.Skip("skipping race test when built with gccgo")
	}
	dir, err := os.MkdirTemp("", "testrace")
	if err != nil {
		t.Fatalf("failed to create temp directory: %v", err)
	}
	defer os.RemoveAll(dir)
	goBinary := filepath.Join(runtime.GOROOT(), "bin", "go")
	t.Logf("%s version", goBinary)
	got, err := exec.Command(goBinary, "version").CombinedOutput()
	if len(got) > 0 {
		t.Logf("%s", got)
	}
	if err != nil {
		t.Fatalf("go version failed: %v", err)
	}
	for i, test := range tests {
		t.Run(fmt.Sprintf("test %d", i), func(t *testing.T) {
			src := filepath.Join(dir, fmt.Sprintf("test%d.go", i))
			if err := os.WriteFile(src, []byte(test), 0644); err != nil {
				t.Fatalf("failed to write file: %v", err)
			}
			t.Logf("%s run -race %s", goBinary, src)
			got, err := exec.Command(goBinary, "run", "-race", src).CombinedOutput()
			if len(got) > 0 {
				t.Logf("%s", got)
			}
			if strings.Contains(string(got), "-race requires cgo") {
				t.Log("CGO is not enabled so can't use -race")
			} else if !strings.Contains(string(got), "WARNING: DATA RACE") {
				t.Errorf("race not detected for test %d: err:%v", i, err)
			}
		})
	}
}
