package zk

import (
	"github.com/go-zookeeper/zk"

	"github.com/go-kit/kit/sd"
	"github.com/go-kit/kit/sd/internal/instance"
	"github.com/go-kit/log"
)

// Instancer yield instances stored in a certain ZooKeeper path. Any kind of
// change in that path is watched and will update the subscribers.
type Instancer struct {
	cache  *instance.Cache
	client Client
	path   string
	logger log.Logger
	quitc  chan struct{}
}

// NewInstancer returns a ZooKeeper Instancer. ZooKeeper will start watching
// the given path for changes and update the Instancer endpoints.
func NewInstancer(c Client, path string, logger log.Logger) (*Instancer, error) {
	s := &Instancer{
		cache:  instance.NewCache(),
		client: c,
		path:   path,
		logger: logger,
		quitc:  make(chan struct{}),
	}

	err := s.client.CreateParentNodes(s.path)
	if err != nil {
		return nil, err
	}

	instances, eventc, err := s.client.GetEntries(s.path)
	if err != nil {
		logger.Log("path", s.path, "msg", "failed to retrieve entries", "err", err)
		// other implementations continue here, but we exit because we don't know if eventc is valid
		return nil, err
	}
	logger.Log("path", s.path, "instances", len(instances))
	s.cache.Update(sd.Event{Instances: instances})

	go s.loop(eventc)

	return s, nil
}

func (s *Instancer) loop(eventc <-chan zk.Event) {
	var (
		instances []string
		err       error
	)
	for {
		select {
		case <-eventc:
			// We received a path update notification. Call GetEntries to
			// retrieve child node data, and set a new watch, as ZK watches are
			// one-time triggers.
			instances, eventc, err = s.client.GetEntries(s.path)
			if err != nil {
				s.logger.Log("path", s.path, "msg", "failed to retrieve entries", "err", err)
				s.cache.Update(sd.Event{Err: err})
				continue
			}
			s.logger.Log("path", s.path, "instances", len(instances))
			s.cache.Update(sd.Event{Instances: instances})

		case <-s.quitc:
			return
		}
	}
}

// Stop terminates the Instancer.
func (s *Instancer) Stop() {
	close(s.quitc)
}

// Register implements Instancer.
func (s *Instancer) Register(ch chan<- sd.Event) {
	s.cache.Register(ch)
}

// Deregister implements Instancer.
func (s *Instancer) Deregister(ch chan<- sd.Event) {
	s.cache.Deregister(ch)
}

// state returns the current state of instance.Cache, only for testing
func (s *Instancer) state() sd.Event {
	return s.cache.State()
}
