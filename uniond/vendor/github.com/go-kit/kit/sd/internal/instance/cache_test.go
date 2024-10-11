package instance

import (
	"context"
	"fmt"
	"io"
	"reflect"
	"testing"
	"time"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/sd"
	"github.com/go-kit/log"
)

var _ sd.Instancer = (*Cache)(nil) // API check

// The test verifies the following:
//   registering causes initial notification of the current state
//   instances are sorted
//   different update causes new notification
//   identical notifications cause no updates
//   no updates after de-registering
func TestCache(t *testing.T) {
	e1 := sd.Event{Instances: []string{"y", "x"}} // not sorted
	e2 := sd.Event{Instances: []string{"c", "a", "b"}}

	cache := NewCache()
	if want, have := 0, len(cache.State().Instances); want != have {
		t.Fatalf("want %v instances, have %v", want, have)
	}

	cache.Update(e1) // sets initial state
	if want, have := 2, len(cache.State().Instances); want != have {
		t.Fatalf("want %v instances, have %v", want, have)
	}

	r1 := make(chan sd.Event)
	go cache.Register(r1)
	expectUpdate(t, r1, []string{"x", "y"})

	go cache.Update(e2) // different set
	expectUpdate(t, r1, []string{"a", "b", "c"})

	cache.Deregister(r1)
	close(r1)
}

func expectUpdate(t *testing.T, r chan sd.Event, expect []string) {
	select {
	case e := <-r:
		if want, have := expect, e.Instances; !reflect.DeepEqual(want, have) {
			t.Fatalf("want: %v, have: %v", want, have)
		}
	case <-time.After(time.Second):
		t.Fatalf("did not receive expected update %v", expect)
	}
}

func TestRegistry(t *testing.T) {
	reg := make(registry)
	c1 := make(chan sd.Event, 1)
	c2 := make(chan sd.Event, 1)
	reg.register(c1)
	reg.register(c2)

	// validate that both channels receive the update
	reg.broadcast(sd.Event{Instances: []string{"x", "y"}})
	if want, have := []string{"x", "y"}, (<-c1).Instances; !reflect.DeepEqual(want, have) {
		t.Fatalf("want: %v, have: %v", want, have)
	}
	if want, have := []string{"x", "y"}, (<-c2).Instances; !reflect.DeepEqual(want, have) {
		t.Fatalf("want: %v, have: %v", want, have)
	}

	reg.deregister(c1)
	reg.deregister(c2)
	close(c1)
	close(c2)
	// if deregister didn't work, broadcast would panic on closed channels
	reg.broadcast(sd.Event{Instances: []string{"x", "y"}})
}

// This test is meant to be run with the race detector enabled: -race.
// It ensures that every registered observer receives a copy
// of sd.Event.Instances because observers can directly modify the field.
// For example, endpointCache calls sort.Strings() on sd.Event.Instances.
func TestDataRace(t *testing.T) {
	instances := make([]string, 0)
	// the number of iterations here maters because we need sort.Strings to
	// perform a Swap in doPivot -> medianOfThree to cause a data race.
	for i := 1; i < 1000; i++ {
		instances = append(instances, fmt.Sprintf("%v", i))
	}
	e1 := sd.Event{Instances: instances}
	cache := NewCache()
	cache.Update(e1)
	nullEndpoint := func(_ context.Context, _ interface{}) (interface{}, error) {
		return nil, nil
	}
	nullFactory := func(instance string) (endpoint.Endpoint, io.Closer, error) {
		return nullEndpoint, nil, nil
	}
	logger := log.Logger(log.LoggerFunc(func(keyvals ...interface{}) error {
		return nil
	}))

	sd.NewEndpointer(cache, nullFactory, logger)
	sd.NewEndpointer(cache, nullFactory, logger)
}
