//go:build windows

package winio

import (
	"context"
	"errors"
	"fmt"
	"io"
	"math/rand"
	"strings"
	"testing"
	"time"

	"golang.org/x/sys/windows"

	"github.com/Microsoft/go-winio/internal/socket"
	"github.com/Microsoft/go-winio/pkg/guid"
)

const testStr = "test"

func randHvsockAddr() *HvsockAddr {
	p := rand.Uint32() //nolint:gosec // used for testing
	return &HvsockAddr{
		VMID:      HvsockGUIDLoopback(),
		ServiceID: VsockServiceID(p),
	}
}

func serverListen(u testUtil) (l *HvsockListener, a *HvsockAddr) {
	var err error
	for i := 0; i < 3; i++ {
		a = randHvsockAddr()
		l, err = ListenHvsock(a)
		if errors.Is(err, windows.WSAEADDRINUSE) {
			u.T.Logf("address collision %v", a)
			continue
		}
		break
	}
	u.Must(err, "could not listen")
	u.T.Cleanup(func() {
		if l != nil {
			u.Must(l.Close(), "Hyper-V socket listener close")
		}
	})

	return l, a
}

func clientServer(u testUtil) (cl, sv *HvsockConn, _ *HvsockAddr) {
	l, addr := serverListen(u)
	ch := u.Go(func() error {
		conn, err := l.Accept()
		if err != nil {
			return fmt.Errorf("listener accept: %w", err)
		}
		sv = conn.(*HvsockConn)
		if err := l.Close(); err != nil {
			return err
		}
		l = nil
		return nil
	})

	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	cl, err := Dial(ctx, addr)
	u.Must(err, "could not dial")
	u.T.Cleanup(func() {
		if cl != nil {
			u.Must(cl.Close(), "client close")
		}
	})

	u.WaitErr(ch, time.Second)
	u.T.Cleanup(func() {
		if sv != nil {
			u.Must(sv.Close(), "server close")
		}
	})
	return cl, sv, addr
}

func TestHvSockConstants(t *testing.T) {
	tests := []struct {
		name string
		want string
		give guid.GUID
	}{
		{"wildcard", "00000000-0000-0000-0000-000000000000", HvsockGUIDWildcard()},
		{"broadcast", "ffffffff-ffff-ffff-ffff-ffffffffffff", HvsockGUIDBroadcast()},
		{"loopback", "e0e16197-dd56-4a10-9195-5ee7a155a838", HvsockGUIDLoopback()},
		{"children", "90db8b89-0d35-4f79-8ce9-49ea0ac8b7cd", HvsockGUIDChildren()},
		{"parent", "a42e7cda-d03f-480c-9cc2-a4de20abb878", HvsockGUIDParent()},
		{"silohost", "36bd0c5c-7276-4223-88ba-7d03b654c568", HvsockGUIDSiloHost()},
		{"vsock template", "00000000-facb-11e6-bd58-64006a7986d3", hvsockVsockServiceTemplate()},
	}
	for _, tt := range tests {
		if tt.give.String() != tt.want {
			t.Errorf("%s give: %v; want: %s", tt.name, tt.give, tt.want)
		}
	}
}

func TestHvSockListenerAddresses(t *testing.T) {
	u := newUtil(t)
	l, addr := serverListen(u)

	la := (l.Addr()).(*HvsockAddr)
	u.Assert(*la == *addr, fmt.Sprintf("give: %v; want: %v", la, addr))

	ra := rawHvsockAddr{}
	sa := HvsockAddr{}
	u.Must(socket.GetSockName(windows.Handle(l.sock.handle), &ra))
	sa.fromRaw(&ra)
	u.Assert(sa == *addr, fmt.Sprintf("listener local addr give: %v; want: %v", sa, addr))
}

func TestHvSockAddresses(t *testing.T) {
	u := newUtil(t)
	cl, sv, addr := clientServer(u)

	sra := (sv.RemoteAddr()).(*HvsockAddr)
	sla := (sv.LocalAddr()).(*HvsockAddr)
	cra := (cl.RemoteAddr()).(*HvsockAddr)
	cla := (cl.LocalAddr()).(*HvsockAddr)

	t.Run("Info", func(t *testing.T) {
		tests := []struct {
			name string
			give *HvsockAddr
			want HvsockAddr
		}{
			{"client local", cla, HvsockAddr{HvsockGUIDChildren(), sra.ServiceID}},
			{"client remote", cra, *addr},
			{"server local", sla, HvsockAddr{HvsockGUIDChildren(), addr.ServiceID}},
			{"server remote", sra, HvsockAddr{HvsockGUIDLoopback(), cla.ServiceID}},
		}
		for _, tt := range tests {
			if *tt.give != tt.want {
				t.Errorf("%s address give: %v; want: %v", tt.name, tt.give, tt.want)
			}
		}
	})

	t.Run("OSinfo", func(t *testing.T) {
		u := newUtil(t)
		ra := rawHvsockAddr{}
		sa := HvsockAddr{}

		localTests := []struct {
			name     string
			giveSock *win32File
			wantAddr HvsockAddr
		}{
			{"client", cl.sock, HvsockAddr{HvsockGUIDChildren(), cla.ServiceID}},
			// The server sockets local address seems arbitrary, so skip this test
			// see comment in `(*HvsockListener) Accept()` for more info
			// {"server", sv.sock, _sla},
		}
		for _, tt := range localTests {
			u.Must(socket.GetSockName(windows.Handle(tt.giveSock.handle), &ra))
			sa.fromRaw(&ra)
			if sa != tt.wantAddr {
				t.Errorf("%s local addr give: %v; want: %v", tt.name, sa, tt.wantAddr)
			}
		}

		remoteTests := []struct {
			name     string
			giveConn *HvsockConn
		}{
			{"client", cl},
			{"server", sv},
		}
		for _, tt := range remoteTests {
			u.Must(socket.GetPeerName(windows.Handle(tt.giveConn.sock.handle), &ra))
			sa.fromRaw(&ra)
			if sa != tt.giveConn.remote {
				t.Errorf("%s remote addr give: %v; want: %v", tt.name, sa, tt.giveConn.remote)
			}
		}
	})
}

func TestHvSockReadWrite(t *testing.T) {
	u := newUtil(t)
	l, addr := serverListen(u)
	tests := []struct {
		req, rsp string
	}{
		{"hello ", "world!"},
		{"ping", "pong"},
	}

	// a sync.WaitGroup doesnt offer a channel to use in a select with a timeout
	// could use an errgroup.Group, but for now dual channels work fine
	svCh := u.Go(func() error {
		c, err := l.Accept()
		if err != nil {
			return fmt.Errorf("listener accept: %w", err)
		}
		defer c.Close()

		b := make([]byte, 64)
		for _, tt := range tests {
			n, err := c.Read(b)
			if err != nil {
				return fmt.Errorf("server rx: %w", err)
			}

			r := string(b[:n])
			if r != tt.req {
				return fmt.Errorf("server rx error: got %q; wanted %q", r, tt.req)
			}
			if _, err = c.Write([]byte(tt.rsp)); err != nil {
				return fmt.Errorf("server tx error, could not send %q: %w", tt.rsp, err)
			}
		}
		n, err := c.Read(b)
		if n != 0 {
			return errors.New("server did not get EOF")
		}
		if !errors.Is(err, io.EOF) {
			return fmt.Errorf("server did not get EOF: %w", err)
		}
		return nil
	})

	clCh := u.Go(func() error {
		cl, err := Dial(context.Background(), addr)
		if err != nil {
			return fmt.Errorf("client dial: %w", err)
		}
		defer cl.Close()

		b := make([]byte, 64)
		for _, tt := range tests {
			_, err := cl.Write([]byte(tt.req))
			if err != nil {
				return fmt.Errorf("client tx error, could not send %q: %w", tt.req, err)
			}

			n, err := cl.Read(b)
			if err != nil {
				return fmt.Errorf("client tx: %w", err)
			}

			r := string(b[:n])
			if r != tt.rsp {
				return fmt.Errorf("client rx error: got %q; wanted %q", b[:n], tt.rsp)
			}
		}
		return cl.CloseWrite()
	})

	u.WaitErr(svCh, 15*time.Second, "server")
	u.WaitErr(clCh, 15*time.Second, "client")
}

func TestHvSockReadTooSmall(t *testing.T) {
	u := newUtil(t)
	s := "this is a really long string that hopefully takes up more than 16 bytes ..."
	l, addr := serverListen(u)

	svCh := u.Go(func() error {
		c, err := l.Accept()
		if err != nil {
			return fmt.Errorf("listener accept: %w", err)
		}
		defer c.Close()

		b := make([]byte, 16)
		ss := ""
		for {
			n, err := c.Read(b)
			if errors.Is(err, io.EOF) {
				break
			}
			if err != nil {
				return fmt.Errorf("server rx: %w", err)
			}
			ss += string(b[:n])
		}

		if ss != s {
			return fmt.Errorf("got %q, wanted: %q", ss, s)
		}
		return nil
	})

	clCh := u.Go(func() error {
		cl, err := Dial(context.Background(), addr)
		if err != nil {
			return fmt.Errorf("client dial: %w", err)
		}
		defer cl.Close()

		if _, err = cl.Write([]byte(s)); err != nil {
			return fmt.Errorf("client tx error, could not send: %w", err)
		}
		return nil
	})

	u.WaitErr(svCh, 15*time.Second, "server")
	u.WaitErr(clCh, 15*time.Second, "client")
}

func TestHvSockCloseReadWriteListener(t *testing.T) {
	u := newUtil(t)
	l, addr := serverListen(u)

	ch := make(chan struct{})
	svCh := u.Go(func() error {
		defer close(ch)
		c, err := l.Accept()
		if err != nil {
			return fmt.Errorf("listener accept: %w", err)
		}
		defer c.Close()

		hv := c.(*HvsockConn)
		//
		// test CloseWrite()
		//
		n, err := c.Write([]byte(testStr))
		if err != nil {
			return fmt.Errorf("server tx: %w", err)
		}
		if n != len(testStr) {
			return fmt.Errorf("server wrote %d bytes, wanted %d", n, len(testStr))
		}

		if err := hv.CloseWrite(); err != nil {
			return fmt.Errorf("server close write: %w", err)
		}

		if _, err = c.Write([]byte(testStr)); !errors.Is(err, windows.WSAESHUTDOWN) {
			return fmt.Errorf("server did not shutdown writes: %w", err)
		}
		// safe to call multiple times
		if err := hv.CloseWrite(); err != nil {
			return fmt.Errorf("server second close write: %w", err)
		}

		//
		// test CloseRead()
		//
		b := make([]byte, 256)
		n, err = c.Read(b)
		if err != nil {
			return fmt.Errorf("server read: %w", err)
		}
		if n != len(testStr) {
			return fmt.Errorf("server read %d bytes, wanted %d", n, len(testStr))
		}
		if string(b[:n]) != testStr {
			return fmt.Errorf("server got %q; wanted %q", b[:n], testStr)
		}
		if err := hv.CloseRead(); err != nil {
			return fmt.Errorf("server close read: %w", err)
		}

		ch <- struct{}{}

		// signal the client to send more info
		// if it was sent before, the read would succeed if the data was buffered prior
		_, err = c.Read(b)
		if !errors.Is(err, windows.WSAESHUTDOWN) {
			return fmt.Errorf("server did not shutdown reads: %w", err)
		}
		// safe to call multiple times
		if err := hv.CloseRead(); err != nil {
			return fmt.Errorf("server second close read: %w", err)
		}

		c.Close()
		if err := hv.CloseWrite(); !errors.Is(err, socket.ErrSocketClosed) {
			return fmt.Errorf("server close write: %w", err)
		}
		if err := hv.CloseRead(); !errors.Is(err, socket.ErrSocketClosed) {
			return fmt.Errorf("server close read: %w", err)
		}
		return nil
	})

	cl, err := Dial(context.Background(), addr)
	u.Must(err, "could not dial")
	defer cl.Close()

	b := make([]byte, 256)
	n, err := cl.Read(b)
	u.Must(err, "client read")
	u.Assert(n == len(testStr), fmt.Sprintf("client read %d bytes, wanted %d", n, len(testStr)))
	u.Assert(string(b[:n]) == testStr, fmt.Sprintf("client got %q; wanted %q", b[:n], testStr))

	n, err = cl.Read(b)
	u.Assert(n == 0, "client did not get EOF")
	u.Is(err, io.EOF, "client did not get EOF")

	n, err = cl.Write([]byte(testStr))
	u.Must(err, "client write")
	u.Assert(n == len(testStr), fmt.Sprintf("client wrote %d bytes, wanted %d", n, len(testStr)))

	u.Wait(ch, time.Second)

	// this should succeed
	_, err = cl.Write([]byte("test2"))
	u.Must(err, "client write")
	u.WaitErr(svCh, time.Second, "server")
}

func TestHvSockCloseReadWriteDial(t *testing.T) {
	u := newUtil(t)
	l, addr := serverListen(u)

	ch := make(chan struct{})
	clCh := u.Go(func() error {
		defer close(ch)
		c, err := l.Accept()
		if err != nil {
			return fmt.Errorf("listener accept: %w", err)
		}
		defer c.Close()

		b := make([]byte, 256)
		n, err := c.Read(b)
		if err != nil {
			return fmt.Errorf("server read: %w", err)
		}
		if string(b[:n]) != testStr {
			return fmt.Errorf("server got %q; wanted %q", b[:n], testStr)
		}

		n, err = c.Read(b)
		if n != 0 {
			return fmt.Errorf("server did not get EOF")
		}
		if !errors.Is(err, io.EOF) {
			return errors.New("server did not get EOF")
		}

		_, err = c.Write([]byte(testStr))
		if err != nil {
			return fmt.Errorf("server tx: %w", err)
		}

		ch <- struct{}{}

		_, err = c.Write([]byte(testStr))
		if err != nil {
			return fmt.Errorf("server tx: %w", err)
		}
		return c.Close()
	})

	cl, err := Dial(context.Background(), addr)
	u.Must(err, "could not dial")
	defer cl.Close()

	//
	// test CloseWrite()
	//
	_, err = cl.Write([]byte(testStr))
	u.Must(err, "client write")
	u.Must(cl.CloseWrite(), "client close write")

	_, err = cl.Write([]byte(testStr))
	u.Is(err, windows.WSAESHUTDOWN, "client did not shutdown writes")

	// safe to call multiple times
	u.Must(cl.CloseWrite(), "client second close write")

	//
	// test CloseRead()
	//
	b := make([]byte, 256)
	n, err := cl.Read(b)
	u.Must(err, "client read")
	u.Assert(string(b[:n]) == testStr, fmt.Sprintf("client got %q; wanted %q", b[:n], testStr))
	u.Must(cl.CloseRead(), "client close read")

	u.Wait(ch, time.Millisecond)

	// signal the client to send more info
	// if it was sent before, the read would succeed if the data was buffered prior
	_, err = cl.Read(b)
	u.Is(err, windows.WSAESHUTDOWN, "client did not shutdown reads")

	// safe to call multiple times
	u.Must(cl.CloseRead(), "client second close write")

	l.Close()
	cl.Close()

	wantErr := socket.ErrSocketClosed
	u.Is(cl.CloseWrite(), wantErr, "client close write")
	u.Is(cl.CloseRead(), wantErr, "client close read")
	u.WaitErr(clCh, time.Second, "client")
}

func TestHvSockDialNoTimeout(t *testing.T) {
	u := newUtil(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	ch := u.Go(func() error {
		addr := randHvsockAddr()
		cl, err := Dial(ctx, addr)
		if err == nil {
			cl.Close()
		}
		if !errors.Is(err, windows.WSAECONNREFUSED) {
			return err
		}
		return nil
	})

	// connections usually take about ~500µs
	u.WaitErr(ch, 2*time.Millisecond, "dial did not time out")
}

func TestHvSockDialDeadline(t *testing.T) {
	u := newUtil(t)
	d := &HvsockDialer{}
	d.Deadline = time.Now().Add(50 * time.Microsecond)
	d.Retries = 1
	// we need the wait time to be long enough for the deadline goroutine to run first and signal
	// timeout
	d.RetryWait = 100 * time.Millisecond
	addr := randHvsockAddr()
	cl, err := d.Dial(context.Background(), addr)
	if err == nil {
		cl.Close()
		t.Fatalf("dial should not have finished")
	}
	u.Is(err, context.DeadlineExceeded, "dial did not exceed deadline")
}

func TestHvSockDialContext(t *testing.T) {
	u := newUtil(t)
	ctx, cancel := context.WithCancel(context.Background())
	time.AfterFunc(50*time.Microsecond, cancel)

	d := &HvsockDialer{}
	d.Retries = 1
	d.RetryWait = 100 * time.Millisecond
	addr := randHvsockAddr()
	cl, err := d.Dial(ctx, addr)
	if err == nil {
		cl.Close()
		t.Fatalf("dial should not have finished")
	}
	u.Is(err, context.Canceled, "dial was not canceled")
}

func TestHvSockAcceptClose(t *testing.T) {
	u := newUtil(t)
	l, _ := serverListen(u)
	go func() {
		time.Sleep(50 * time.Millisecond)
		l.Close()
	}()

	c, err := l.Accept()
	if err == nil {
		c.Close()
		t.Fatal("listener should not have accepted anything")
	}
	u.Is(err, ErrFileClosed)
}

//
// helpers
//

type testUtil struct {
	T testing.TB
}

func newUtil(t testing.TB) testUtil {
	return testUtil{
		T: t,
	}
}

// Go launches f in a go routine and returns a channel that can be monitored for the result.
// ch is closed after f completes.
//
// Intended for use with [testUtil.WaitErr].
func (*testUtil) Go(f func() error) chan error {
	ch := make(chan error)
	go func() {
		defer close(ch)
		ch <- f()
	}()
	return ch
}

func (u testUtil) Wait(ch <-chan struct{}, d time.Duration, msgs ...string) {
	t := time.NewTimer(d)
	defer t.Stop()
	select {
	case <-ch:
	case <-t.C:
		u.T.Helper()
		u.T.Fatalf(msgJoin(msgs, "timed out after %v"), d)
	}
}

func (u testUtil) WaitErr(ch <-chan error, d time.Duration, msgs ...string) {
	t := time.NewTimer(d)
	defer t.Stop()
	select {
	case err := <-ch:
		if err != nil {
			u.T.Helper()
			u.T.Fatalf(msgJoin(msgs, "%v"), err)
		}
	case <-t.C:
		u.T.Helper()
		u.T.Fatalf(msgJoin(msgs, "timed out after %v"), d)
	}
}

func (u testUtil) Assert(b bool, msgs ...string) {
	if b {
		return
	}
	u.T.Helper()
	u.T.Fatalf(msgJoin(msgs, "failed assertion"))
}

func (u testUtil) Is(err, target error, msgs ...string) {
	if errors.Is(err, target) {
		return
	}
	u.T.Helper()
	u.T.Fatalf(msgJoin(msgs, "got error %q; wanted %q"), err, target)
}

func (u testUtil) Must(err error, msgs ...string) {
	if err == nil {
		return
	}
	u.T.Helper()
	u.T.Fatalf(msgJoin(msgs, "%v"), err)
}

// Check stops execution if testing failed in another go-routine.
func (u testUtil) Check() {
	if u.T.Failed() {
		u.T.FailNow()
	}
}

func msgJoin(pre []string, s string) string {
	return strings.Join(append(pre, s), ": ")
}
