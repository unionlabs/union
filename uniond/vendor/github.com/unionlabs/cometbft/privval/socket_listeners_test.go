package privval

import (
	"net"
	"os"
	"testing"
	"time"

	"github.com/cometbft/cometbft/crypto/ed25519"
)

// -------------------------------------------
// helper funcs

func newPrivKey() ed25519.PrivKey {
	return ed25519.GenPrivKey()
}

// -------------------------------------------
// tests

type listenerTestCase struct {
	description string // For test reporting purposes.
	listener    net.Listener
	dialer      SocketDialer
}

// testUnixAddr will attempt to obtain a platform-independent temporary file
// name for a Unix socket.
func testUnixAddr() (string, error) {
	f, err := os.CreateTemp("", "cometbft-privval-test-*")
	if err != nil {
		return "", err
	}
	addr := f.Name()
	f.Close()
	os.Remove(addr)
	return addr, nil
}

func tcpListenerTestCase(t *testing.T, timeoutAccept, timeoutReadWrite time.Duration) listenerTestCase {
	t.Helper()
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}

	tcpLn := NewTCPListener(ln, newPrivKey())
	TCPListenerTimeoutAccept(timeoutAccept)(tcpLn)
	TCPListenerTimeoutReadWrite(timeoutReadWrite)(tcpLn)
	return listenerTestCase{
		description: "TCP",
		listener:    tcpLn,
		dialer:      DialTCPFn(ln.Addr().String(), testTimeoutReadWrite, newPrivKey()),
	}
}

func unixListenerTestCase(t *testing.T, timeoutAccept, timeoutReadWrite time.Duration) listenerTestCase {
	t.Helper()
	addr, err := testUnixAddr()
	if err != nil {
		t.Fatal(err)
	}
	ln, err := net.Listen("unix", addr)
	if err != nil {
		t.Fatal(err)
	}

	unixLn := NewUnixListener(ln)
	UnixListenerTimeoutAccept(timeoutAccept)(unixLn)
	UnixListenerTimeoutReadWrite(timeoutReadWrite)(unixLn)
	return listenerTestCase{
		description: "Unix",
		listener:    unixLn,
		dialer:      DialUnixFn(addr),
	}
}

func listenerTestCases(t *testing.T, timeoutAccept, timeoutReadWrite time.Duration) []listenerTestCase {
	t.Helper()
	return []listenerTestCase{
		tcpListenerTestCase(t, timeoutAccept, timeoutReadWrite),
		unixListenerTestCase(t, timeoutAccept, timeoutReadWrite),
	}
}

func TestListenerTimeoutAccept(t *testing.T) {
	for _, tc := range listenerTestCases(t, time.Millisecond, time.Second) {
		_, err := tc.listener.Accept()
		opErr, ok := err.(*net.OpError)
		if !ok {
			t.Fatalf("for %s listener, have %v, want *net.OpError", tc.description, err)
		}

		if have, want := opErr.Op, "accept"; have != want {
			t.Errorf("for %s listener,  have %v, want %v", tc.description, have, want)
		}
	}
}

func TestListenerTimeoutReadWrite(t *testing.T) {
	const (
		// This needs to be long enough s.t. the Accept will definitely succeed:
		timeoutAccept = time.Second
		// This can be really short but in the TCP case, the accept can
		// also trigger a timeoutReadWrite. Hence, we need to give it some time.
		// Note: this controls how long this test actually runs.
		timeoutReadWrite = 10 * time.Millisecond
	)

	for _, tc := range listenerTestCases(t, timeoutAccept, timeoutReadWrite) {
		go func(dialer SocketDialer) {
			conn, err := dialer()
			if err != nil {
				panic(err)
			}
			// Add a delay before closing the connection
			time.Sleep(2 * timeoutReadWrite)
			conn.Close()
		}(tc.dialer)

		c, err := tc.listener.Accept()
		if err != nil {
			t.Fatal(err)
		}

		// this will timeout because we don't write anything:
		msg := make([]byte, 200)
		_, err = c.Read(msg)
		opErr, ok := err.(*net.OpError)
		if !ok {
			t.Fatalf("for %s listener, have %v, want *net.OpError", tc.description, err)
		}

		if have, want := opErr.Op, "read"; have != want {
			t.Errorf("for %s listener, have %v, want %v", tc.description, have, want)
		}

		if !opErr.Timeout() {
			t.Errorf("for %s listener, got unexpected error: have %v, want Timeout error", tc.description, opErr)
		}
	}
}
