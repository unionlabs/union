package eureka

import (
	"errors"
	"fmt"
	"reflect"
	"sync"
	"time"

	"github.com/go-kit/log"
	"github.com/hudl/fargo"
)

type testConnection struct {
	mu        sync.RWMutex
	instances []*fargo.Instance

	errApplication error
	errHeartbeat   error
	errRegister    error
	errDeregister  error
}

var (
	errTest       = errors.New("kaboom")
	errNotFound   = &fargoUnsuccessfulHTTPResponse{statusCode: 404, messagePrefix: "not found"}
	loggerTest    = log.NewNopLogger()
	appNameTest   = "go-kit"
	instanceTest1 = &fargo.Instance{
		HostName:         "serveregistrar1.acme.org",
		Port:             8080,
		App:              appNameTest,
		IPAddr:           "192.168.0.1",
		VipAddress:       "192.168.0.1",
		SecureVipAddress: "192.168.0.1",
		HealthCheckUrl:   "http://serveregistrar1.acme.org:8080/healthz",
		StatusPageUrl:    "http://serveregistrar1.acme.org:8080/status",
		HomePageUrl:      "http://serveregistrar1.acme.org:8080/",
		Status:           fargo.UP,
		DataCenterInfo:   fargo.DataCenterInfo{Name: fargo.MyOwn},
		LeaseInfo:        fargo.LeaseInfo{RenewalIntervalInSecs: 1},
	}
	instanceTest2 = &fargo.Instance{
		HostName:         "serveregistrar2.acme.org",
		Port:             8080,
		App:              appNameTest,
		IPAddr:           "192.168.0.2",
		VipAddress:       "192.168.0.2",
		SecureVipAddress: "192.168.0.2",
		HealthCheckUrl:   "http://serveregistrar2.acme.org:8080/healthz",
		StatusPageUrl:    "http://serveregistrar2.acme.org:8080/status",
		HomePageUrl:      "http://serveregistrar2.acme.org:8080/",
		Status:           fargo.UP,
		DataCenterInfo:   fargo.DataCenterInfo{Name: fargo.MyOwn},
	}
)

var _ fargoConnection = (*testConnection)(nil)

func (c *testConnection) RegisterInstance(i *fargo.Instance) error {
	if c.errRegister != nil {
		return c.errRegister
	}
	c.mu.Lock()
	defer c.mu.Unlock()
	for _, instance := range c.instances {
		if reflect.DeepEqual(*instance, *i) {
			return errors.New("already registered")
		}
	}
	c.instances = append(c.instances, i)
	return nil
}

func (c *testConnection) HeartBeatInstance(i *fargo.Instance) error {
	return c.errHeartbeat
}

func (c *testConnection) DeregisterInstance(i *fargo.Instance) error {
	if c.errDeregister != nil {
		return c.errDeregister
	}
	c.mu.Lock()
	defer c.mu.Unlock()
	remaining := make([]*fargo.Instance, 0, len(c.instances))
	for _, instance := range c.instances {
		if reflect.DeepEqual(*instance, *i) {
			continue
		}
		remaining = append(remaining, instance)
	}
	if len(remaining) == len(c.instances) {
		return errors.New("not registered")
	}
	c.instances = remaining
	return nil
}

func (c *testConnection) ReregisterInstance(ins *fargo.Instance) error {
	return nil
}

func (c *testConnection) instancesForApplication(name string) []*fargo.Instance {
	c.mu.RLock()
	defer c.mu.RUnlock()
	instances := make([]*fargo.Instance, 0, len(c.instances))
	for _, i := range c.instances {
		if i.App == name {
			instances = append(instances, i)
		}
	}
	return instances
}

func (c *testConnection) GetApp(name string) (*fargo.Application, error) {
	if err := c.errApplication; err != nil {
		return nil, err
	}
	instances := c.instancesForApplication(name)
	if len(instances) == 0 {
		return nil, fmt.Errorf("application not found for name=%s", name)
	}
	return &fargo.Application{Name: name, Instances: instances}, nil
}

func (c *testConnection) ScheduleAppUpdates(name string, await bool, done <-chan struct{}) <-chan fargo.AppUpdate {
	updatec := make(chan fargo.AppUpdate, 1)
	send := func() {
		app, err := c.GetApp(name)
		select {
		case updatec <- fargo.AppUpdate{App: app, Err: err}:
		default:
		}
	}

	if await {
		send()
	}
	go func() {
		ticker := time.NewTicker(100 * time.Millisecond)
		for {
			select {
			case <-ticker.C:
				send()
			case <-done:
				ticker.Stop()
				return
			}
		}
	}()
	return updatec
}
