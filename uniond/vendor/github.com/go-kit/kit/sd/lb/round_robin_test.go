package lb

import (
	"context"
	"reflect"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/sd"
)

func TestRoundRobin(t *testing.T) {
	var (
		counts    = []int{0, 0, 0}
		endpoints = []endpoint.Endpoint{
			func(context.Context, interface{}) (interface{}, error) { counts[0]++; return struct{}{}, nil },
			func(context.Context, interface{}) (interface{}, error) { counts[1]++; return struct{}{}, nil },
			func(context.Context, interface{}) (interface{}, error) { counts[2]++; return struct{}{}, nil },
		}
	)

	endpointer := sd.FixedEndpointer(endpoints)
	balancer := NewRoundRobin(endpointer)

	for i, want := range [][]int{
		{1, 0, 0},
		{1, 1, 0},
		{1, 1, 1},
		{2, 1, 1},
		{2, 2, 1},
		{2, 2, 2},
		{3, 2, 2},
	} {
		endpoint, err := balancer.Endpoint()
		if err != nil {
			t.Fatal(err)
		}
		endpoint(context.Background(), struct{}{})
		if have := counts; !reflect.DeepEqual(want, have) {
			t.Fatalf("%d: want %v, have %v", i, want, have)
		}
	}
}

func TestRoundRobinNoEndpoints(t *testing.T) {
	endpointer := sd.FixedEndpointer{}
	balancer := NewRoundRobin(endpointer)
	_, err := balancer.Endpoint()
	if want, have := ErrNoEndpoints, err; want != have {
		t.Errorf("want %v, have %v", want, have)
	}
}

func TestRoundRobinNoRace(t *testing.T) {
	balancer := NewRoundRobin(sd.FixedEndpointer([]endpoint.Endpoint{
		endpoint.Nop,
		endpoint.Nop,
		endpoint.Nop,
		endpoint.Nop,
		endpoint.Nop,
	}))

	var (
		n     = 100
		done  = make(chan struct{})
		wg    sync.WaitGroup
		count uint64
	)

	wg.Add(n)

	for i := 0; i < n; i++ {
		go func() {
			defer wg.Done()
			for {
				select {
				case <-done:
					return
				default:
					_, _ = balancer.Endpoint()
					atomic.AddUint64(&count, 1)
				}
			}
		}()
	}

	time.Sleep(time.Second)
	close(done)
	wg.Wait()

	t.Logf("made %d calls", atomic.LoadUint64(&count))
}
