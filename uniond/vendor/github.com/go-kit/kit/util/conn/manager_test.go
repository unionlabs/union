package conn

import (
	"errors"
	"net"
	"sync/atomic"
	"testing"
	"time"

	"github.com/go-kit/log"
)

func TestManager(t *testing.T) {
	var (
		tickc    = make(chan time.Time)
		after    = func(time.Duration) <-chan time.Time { return tickc }
		dialconn = &mockConn{}
		dialerr  = error(nil)
		dialer   = func(string, string) (net.Conn, error) { return dialconn, dialerr }
		mgr      = NewManager(dialer, "netw", "addr", after, log.NewNopLogger())
	)

	// First conn should be fine.
	conn := mgr.Take()
	if conn == nil {
		t.Fatal("nil conn")
	}

	// Write and check it went through.
	if _, err := conn.Write([]byte{1, 2, 3}); err != nil {
		t.Fatal(err)
	}
	if want, have := uint64(3), atomic.LoadUint64(&dialconn.wr); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	// Put an error to kill the conn.
	mgr.Put(errors.New("should kill the connection"))

	// First takes should fail.
	for i := 0; i < 10; i++ {
		if conn = mgr.Take(); conn != nil {
			t.Fatalf("iteration %d: want nil conn, got real conn", i)
		}
	}

	// Trigger the reconnect.
	tickc <- time.Now()

	// The dial should eventually succeed and yield a good conn.
	if !within(100*time.Millisecond, func() bool {
		conn = mgr.Take()
		return conn != nil
	}) {
		t.Fatal("conn remained nil")
	}

	// Write and check it went through.
	if _, err := conn.Write([]byte{4, 5}); err != nil {
		t.Fatal(err)
	}
	if want, have := uint64(5), atomic.LoadUint64(&dialconn.wr); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	// Dial starts failing.
	dialconn, dialerr = nil, errors.New("oh noes")
	mgr.Put(errors.New("trigger that reconnect y'all"))
	if conn = mgr.Take(); conn != nil {
		t.Fatalf("want nil conn, got real conn")
	}

	// As many reconnects as they want.
	go func() {
		done := time.After(100 * time.Millisecond)
		for {
			select {
			case tickc <- time.Now():
			case <-done:
				return
			}
		}
	}()

	// The dial should never succeed.
	if within(100*time.Millisecond, func() bool {
		conn = mgr.Take()
		return conn != nil
	}) {
		t.Fatal("eventually got a good conn, despite failing dialer")
	}
}

func TestIssue292(t *testing.T) {
	// The util/conn.Manager won't attempt to reconnect to the provided endpoint
	// if the endpoint is initially unavailable (e.g. dial tcp :8080:
	// getsockopt: connection refused). If the endpoint is up when
	// conn.NewManager is called and then goes down/up, it reconnects just fine.

	var (
		tickc    = make(chan time.Time)
		after    = func(time.Duration) <-chan time.Time { return tickc }
		dialconn = net.Conn(nil)
		dialerr  = errors.New("fail")
		dialer   = func(string, string) (net.Conn, error) { return dialconn, dialerr }
		mgr      = NewManager(dialer, "netw", "addr", after, log.NewNopLogger())
	)

	if conn := mgr.Take(); conn != nil {
		t.Fatal("first Take should have yielded nil conn, but didn't")
	}

	dialconn, dialerr = &mockConn{}, nil
	select {
	case tickc <- time.Now():
	case <-time.After(time.Second):
		t.Fatal("manager isn't listening for a tick, despite a failed dial")
	}

	if !within(time.Second, func() bool {
		return mgr.Take() != nil
	}) {
		t.Fatal("second Take should have yielded good conn, but didn't")
	}
}

type mockConn struct {
	rd, wr uint64
}

func (c *mockConn) Read(b []byte) (n int, err error) {
	atomic.AddUint64(&c.rd, uint64(len(b)))
	return len(b), nil
}

func (c *mockConn) Write(b []byte) (n int, err error) {
	atomic.AddUint64(&c.wr, uint64(len(b)))
	return len(b), nil
}

func (c *mockConn) Close() error                       { return nil }
func (c *mockConn) LocalAddr() net.Addr                { return nil }
func (c *mockConn) RemoteAddr() net.Addr               { return nil }
func (c *mockConn) SetDeadline(t time.Time) error      { return nil }
func (c *mockConn) SetReadDeadline(t time.Time) error  { return nil }
func (c *mockConn) SetWriteDeadline(t time.Time) error { return nil }

func within(d time.Duration, f func() bool) bool {
	deadline := time.Now().Add(d)
	for {
		if time.Now().After(deadline) {
			return false
		}
		if f() {
			return true
		}
		time.Sleep(d / 10)
	}
}
