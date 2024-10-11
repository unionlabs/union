package lb

import (
	"sync/atomic"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/sd"
)

// NewRoundRobin returns a load balancer that returns services in sequence.
func NewRoundRobin(s sd.Endpointer) Balancer {
	return &roundRobin{
		s: s,
		c: 0,
	}
}

type roundRobin struct {
	s sd.Endpointer
	c uint64
}

func (rr *roundRobin) Endpoint() (endpoint.Endpoint, error) {
	endpoints, err := rr.s.Endpoints()
	if err != nil {
		return nil, err
	}
	if len(endpoints) <= 0 {
		return nil, ErrNoEndpoints
	}
	old := atomic.AddUint64(&rr.c, 1) - 1
	idx := old % uint64(len(endpoints))
	return endpoints[idx], nil
}
