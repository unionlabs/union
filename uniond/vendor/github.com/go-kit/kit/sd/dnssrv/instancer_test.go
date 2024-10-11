package dnssrv

import (
	"net"
	"sync/atomic"
	"testing"
	"time"

	"github.com/go-kit/kit/sd"
	"github.com/go-kit/log"
)

var _ sd.Instancer = (*Instancer)(nil) // API check

func TestRefresh(t *testing.T) {
	name := "some.service.internal"

	ticker := time.NewTicker(time.Second)
	ticker.Stop()
	tickc := make(chan time.Time)
	ticker.C = tickc

	var lookups uint64
	records := []*net.SRV{}
	lookup := func(service, proto, name string) (string, []*net.SRV, error) {
		t.Logf("lookup(%q, %q, %q)", service, proto, name)
		atomic.AddUint64(&lookups, 1)
		return "cname", records, nil
	}

	instancer := NewInstancerDetailed(name, ticker, lookup, log.NewNopLogger())
	defer instancer.Stop()

	// First lookup, empty
	state := instancer.cache.State()
	if state.Err != nil {
		t.Error(state.Err)
	}
	if want, have := 0, len(state.Instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
	if want, have := uint64(1), atomic.LoadUint64(&lookups); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	// Load some records and lookup again
	records = []*net.SRV{
		{Target: "1.0.0.1", Port: 1001},
		{Target: "1.0.0.2", Port: 1002},
		{Target: "1.0.0.3", Port: 1003},
	}
	tickc <- time.Now()

	// There is a race condition where the instancer.State call below
	// invokes the cache before it is updated by the tick above.
	// TODO(pb): solve by running the read through the loop goroutine.
	time.Sleep(100 * time.Millisecond)

	state = instancer.cache.State()
	if state.Err != nil {
		t.Error(state.Err)
	}
	if want, have := 3, len(state.Instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
	if want, have := uint64(2), atomic.LoadUint64(&lookups); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}

func TestIssue892(t *testing.T) {
	ticker := time.NewTicker(time.Second)
	ticker.Stop()
	tickc := make(chan time.Time)
	ticker.C = tickc

	records := []*net.SRV{
		{Target: "1.0.0.1", Port: 80},
		{Target: "1.0.0.2", Port: 0},
		{Target: "1.0.0.3", Port: 80},
	}

	lookup := func(service, proto, name string) (string, []*net.SRV, error) {
		return "cname", records, nil
	}

	instancer := NewInstancerDetailed("name", ticker, lookup, log.NewNopLogger())
	defer instancer.Stop()

	tickc <- time.Now()
	time.Sleep(100 * time.Millisecond)

	if want, have := ErrPortZero, instancer.cache.State().Err; want != have {
		t.Fatalf("want %v, have %v", want, have)
	}
}

type nopCloser struct{}

func (nopCloser) Close() error { return nil }
