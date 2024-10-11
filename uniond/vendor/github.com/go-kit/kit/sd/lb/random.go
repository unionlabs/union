package lb

import (
	"math/rand"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/sd"
)

// NewRandom returns a load balancer that selects services randomly.
func NewRandom(s sd.Endpointer, seed int64) Balancer {
	return &random{
		s: s,
		r: rand.New(rand.NewSource(seed)),
	}
}

type random struct {
	s sd.Endpointer
	r *rand.Rand
}

func (r *random) Endpoint() (endpoint.Endpoint, error) {
	endpoints, err := r.s.Endpoints()
	if err != nil {
		return nil, err
	}
	if len(endpoints) <= 0 {
		return nil, ErrNoEndpoints
	}
	return endpoints[r.r.Intn(len(endpoints))], nil
}
