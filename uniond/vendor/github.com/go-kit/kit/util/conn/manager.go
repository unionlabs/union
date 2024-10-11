package conn

import (
	"errors"
	"math/rand"
	"net"
	"time"

	"github.com/go-kit/log"
)

// Dialer imitates net.Dial. Dialer is assumed to yield connections that are
// safe for use by multiple concurrent goroutines.
type Dialer func(network, address string) (net.Conn, error)

// AfterFunc imitates time.After.
type AfterFunc func(time.Duration) <-chan time.Time

// Manager manages a net.Conn.
//
// Clients provide a way to create the connection with a Dialer, network, and
// address. Clients should Take the connection when they want to use it, and Put
// back whatever error they receive from its use. When a non-nil error is Put,
// the connection is invalidated, and a new connection is established.
// Connection failures are retried after an exponential backoff.
type Manager struct {
	dialer  Dialer
	network string
	address string
	after   AfterFunc
	logger  log.Logger

	takec chan net.Conn
	putc  chan error
}

// NewManager returns a connection manager using the passed Dialer, network, and
// address. The AfterFunc is used to control exponential backoff and retries.
// The logger is used to log errors; pass a log.NopLogger if you don't care to
// receive them. For normal use, prefer NewDefaultManager.
func NewManager(d Dialer, network, address string, after AfterFunc, logger log.Logger) *Manager {
	m := &Manager{
		dialer:  d,
		network: network,
		address: address,
		after:   after,
		logger:  logger,

		takec: make(chan net.Conn),
		putc:  make(chan error),
	}
	go m.loop()
	return m
}

// NewDefaultManager is a helper constructor, suitable for most normal use in
// real (non-test) code. It uses the real net.Dial and time.After functions.
func NewDefaultManager(network, address string, logger log.Logger) *Manager {
	return NewManager(net.Dial, network, address, time.After, logger)
}

// Take yields the current connection. It may be nil.
func (m *Manager) Take() net.Conn {
	return <-m.takec
}

// Put accepts an error that came from a previously yielded connection. If the
// error is non-nil, the manager will invalidate the current connection and try
// to reconnect, with exponential backoff. Putting a nil error is a no-op.
func (m *Manager) Put(err error) {
	m.putc <- err
}

// Write writes the passed data to the connection in a single Take/Put cycle.
func (m *Manager) Write(b []byte) (int, error) {
	conn := m.Take()
	if conn == nil {
		return 0, ErrConnectionUnavailable
	}
	n, err := conn.Write(b)
	defer m.Put(err)
	return n, err
}

func (m *Manager) loop() {
	var (
		conn       = dial(m.dialer, m.network, m.address, m.logger) // may block slightly
		connc      = make(chan net.Conn, 1)
		reconnectc <-chan time.Time // initially nil
		backoff    = time.Second
	)

	// If the initial dial fails, we need to trigger a reconnect via the loop
	// body, below. If we did this in a goroutine, we would race on the conn
	// variable. So we use a buffered chan instead.
	connc <- conn

	for {
		select {
		case <-reconnectc:
			reconnectc = nil // one-shot
			go func() { connc <- dial(m.dialer, m.network, m.address, m.logger) }()

		case conn = <-connc:
			if conn == nil {
				// didn't work
				backoff = Exponential(backoff) // wait longer
				reconnectc = m.after(backoff)  // try again
			} else {
				// worked!
				backoff = time.Second // reset wait time
				reconnectc = nil      // no retry necessary
			}

		case m.takec <- conn:

		case err := <-m.putc:
			if err != nil && conn != nil {
				m.logger.Log("err", err)
				conn.Close()
				conn = nil                            // connection is bad
				reconnectc = m.after(time.Nanosecond) // trigger immediately
			}
		}
	}
}

func dial(d Dialer, network, address string, logger log.Logger) net.Conn {
	conn, err := d(network, address)
	if err != nil {
		logger.Log("err", err)
		conn = nil // just to be sure
	}
	return conn
}

// Exponential takes a duration and returns another one that is twice as long, +/- 50%. It is
// used to provide backoff for operations that may fail and should avoid thundering herds.
// See https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/ for rationale
func Exponential(d time.Duration) time.Duration {
	d *= 2
	jitter := rand.Float64() + 0.5
	d = time.Duration(int64(float64(d.Nanoseconds()) * jitter))
	if d > time.Minute {
		d = time.Minute
	}
	return d

}

// ErrConnectionUnavailable is returned by the Manager's Write method when the
// manager cannot yield a good connection.
var ErrConnectionUnavailable = errors.New("connection unavailable")
