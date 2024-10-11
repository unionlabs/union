package lb

import (
	"context"
	"math"
	"testing"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/sd"
)

func TestRandom(t *testing.T) {
	var (
		n          = 7
		endpoints  = make([]endpoint.Endpoint, n)
		counts     = make([]int, n)
		seed       = int64(12345)
		iterations = 1000000
		want       = iterations / n
		tolerance  = want / 100 // 1%
	)

	for i := 0; i < n; i++ {
		i0 := i
		endpoints[i] = func(context.Context, interface{}) (interface{}, error) { counts[i0]++; return struct{}{}, nil }
	}

	endpointer := sd.FixedEndpointer(endpoints)
	balancer := NewRandom(endpointer, seed)

	for i := 0; i < iterations; i++ {
		endpoint, _ := balancer.Endpoint()
		endpoint(context.Background(), struct{}{})
	}

	for i, have := range counts {
		delta := int(math.Abs(float64(want - have)))
		if delta > tolerance {
			t.Errorf("%d: want %d, have %d, delta %d > %d tolerance", i, want, have, delta, tolerance)
		}
	}
}

func TestRandomNoEndpoints(t *testing.T) {
	endpointer := sd.FixedEndpointer{}
	balancer := NewRandom(endpointer, 1415926)
	_, err := balancer.Endpoint()
	if want, have := ErrNoEndpoints, err; want != have {
		t.Errorf("want %v, have %v", want, have)
	}

}
