package consul

import (
	"context"
	"errors"
	"io"
	"reflect"
	"testing"

	stdconsul "github.com/hashicorp/consul/api"

	"github.com/go-kit/kit/endpoint"
)

func TestClientRegistration(t *testing.T) {
	c := newTestClient(nil)

	services, _, err := c.Service(testRegistration.Name, "", true, &stdconsul.QueryOptions{})
	if err != nil {
		t.Error(err)
	}
	if want, have := 0, len(services); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	if err := c.Register(testRegistration); err != nil {
		t.Error(err)
	}

	if err := c.Register(testRegistration); err == nil {
		t.Errorf("want error, have %v", err)
	}

	services, _, err = c.Service(testRegistration.Name, "", true, &stdconsul.QueryOptions{})
	if err != nil {
		t.Error(err)
	}
	if want, have := 1, len(services); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	if err := c.Deregister(testRegistration); err != nil {
		t.Error(err)
	}

	if err := c.Deregister(testRegistration); err == nil {
		t.Errorf("want error, have %v", err)
	}

	services, _, err = c.Service(testRegistration.Name, "", true, &stdconsul.QueryOptions{})
	if err != nil {
		t.Error(err)
	}
	if want, have := 0, len(services); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}

type testClient struct {
	entries []*stdconsul.ServiceEntry
}

func newTestClient(entries []*stdconsul.ServiceEntry) *testClient {
	return &testClient{
		entries: entries,
	}
}

var _ Client = &testClient{}

func (c *testClient) Service(service, tag string, _ bool, opts *stdconsul.QueryOptions) ([]*stdconsul.ServiceEntry, *stdconsul.QueryMeta, error) {
	var results []*stdconsul.ServiceEntry

	for _, entry := range c.entries {
		if entry.Service.Service != service {
			continue
		}
		if tag != "" {
			tagMap := map[string]struct{}{}

			for _, t := range entry.Service.Tags {
				tagMap[t] = struct{}{}
			}

			if _, ok := tagMap[tag]; !ok {
				continue
			}
		}

		results = append(results, entry)
	}

	return results, &stdconsul.QueryMeta{LastIndex: opts.WaitIndex}, nil
}

func (c *testClient) Register(r *stdconsul.AgentServiceRegistration) error {
	toAdd := registration2entry(r)

	for _, entry := range c.entries {
		if reflect.DeepEqual(*entry, *toAdd) {
			return errors.New("duplicate")
		}
	}

	c.entries = append(c.entries, toAdd)
	return nil
}

func (c *testClient) Deregister(r *stdconsul.AgentServiceRegistration) error {
	toDelete := registration2entry(r)

	var newEntries []*stdconsul.ServiceEntry
	for _, entry := range c.entries {
		if reflect.DeepEqual(*entry, *toDelete) {
			continue
		}
		newEntries = append(newEntries, entry)
	}
	if len(newEntries) == len(c.entries) {
		return errors.New("not found")
	}

	c.entries = newEntries
	return nil
}

func registration2entry(r *stdconsul.AgentServiceRegistration) *stdconsul.ServiceEntry {
	return &stdconsul.ServiceEntry{
		Node: &stdconsul.Node{
			Node:    "some-node",
			Address: r.Address,
		},
		Service: &stdconsul.AgentService{
			ID:      r.ID,
			Service: r.Name,
			Tags:    r.Tags,
			Port:    r.Port,
			Address: r.Address,
		},
		// Checks ignored
	}
}

func testFactory(instance string) (endpoint.Endpoint, io.Closer, error) {
	return func(context.Context, interface{}) (interface{}, error) {
		return instance, nil
	}, nil, nil
}

var testRegistration = &stdconsul.AgentServiceRegistration{
	ID:      "my-id",
	Name:    "my-name",
	Tags:    []string{"my-tag-1", "my-tag-2"},
	Port:    12345,
	Address: "my-address",
}
