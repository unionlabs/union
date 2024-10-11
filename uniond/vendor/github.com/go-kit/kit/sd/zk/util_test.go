package zk

import (
	"errors"
	"fmt"
	"io"
	"sync"
	"time"

	"github.com/go-zookeeper/zk"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/sd"
	"github.com/go-kit/log"
)

var (
	path   = "/gokit.test/service.name"
	logger = log.NewNopLogger()
)

type fakeClient struct {
	mtx       sync.Mutex
	ch        chan zk.Event
	responses map[string]string
	result    bool
}

func newFakeClient() *fakeClient {
	return &fakeClient{
		ch:        make(chan zk.Event, 1),
		responses: make(map[string]string),
		result:    true,
	}
}

func (c *fakeClient) CreateParentNodes(path string) error {
	if path == "BadPath" {
		return errors.New("dummy error")
	}
	return nil
}

func (c *fakeClient) GetEntries(path string) ([]string, <-chan zk.Event, error) {
	c.mtx.Lock()
	defer c.mtx.Unlock()
	if c.result == false {
		c.result = true
		return []string{}, c.ch, errors.New("dummy error")
	}
	responses := []string{}
	for _, data := range c.responses {
		responses = append(responses, data)
	}
	return responses, c.ch, nil
}

func (c *fakeClient) AddService(node, data string) {
	c.mtx.Lock()
	defer c.mtx.Unlock()
	c.responses[node] = data
	c.ch <- zk.Event{}
}

func (c *fakeClient) RemoveService(node string) {
	c.mtx.Lock()
	defer c.mtx.Unlock()
	delete(c.responses, node)
	c.ch <- zk.Event{}
}

func (c *fakeClient) Register(s *Service) error {
	return nil
}

func (c *fakeClient) Deregister(s *Service) error {
	return nil
}

func (c *fakeClient) SendErrorOnWatch() {
	c.mtx.Lock()
	defer c.mtx.Unlock()
	c.result = false
	c.ch <- zk.Event{}
}

func (c *fakeClient) ErrorIsConsumedWithin(timeout time.Duration) error {
	t := time.After(timeout)
	for {
		select {
		case <-t:
			return fmt.Errorf("expected error not consumed after timeout %s", timeout)
		default:
			c.mtx.Lock()
			if c.result == false {
				c.mtx.Unlock()
				return nil
			}
			c.mtx.Unlock()
		}
	}
}

func (c *fakeClient) Stop() {}

func newFactory(fakeError string) sd.Factory {
	return func(instance string) (endpoint.Endpoint, io.Closer, error) {
		if fakeError == instance {
			return nil, nil, errors.New(fakeError)
		}
		return endpoint.Nop, nil, nil
	}
}

func asyncTest(timeout time.Duration, want int, s sd.Endpointer) (err error) {
	var endpoints []endpoint.Endpoint
	have := -1 // want can never be <0
	t := time.After(timeout)
	for {
		select {
		case <-t:
			return fmt.Errorf("want %d, have %d (timeout %s)", want, have, timeout.String())
		default:
			endpoints, err = s.Endpoints()
			have = len(endpoints)
			if err != nil || want == have {
				return
			}
			time.Sleep(timeout / 10)
		}
	}
}
