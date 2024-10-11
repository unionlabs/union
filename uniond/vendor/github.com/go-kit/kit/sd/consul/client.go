package consul

import (
	consul "github.com/hashicorp/consul/api"
)

// Client is a wrapper around the Consul API.
type Client interface {
	// Register a service with the local agent.
	Register(r *consul.AgentServiceRegistration) error

	// Deregister a service with the local agent.
	Deregister(r *consul.AgentServiceRegistration) error

	// Service
	Service(service, tag string, passingOnly bool, queryOpts *consul.QueryOptions) ([]*consul.ServiceEntry, *consul.QueryMeta, error)
}

type client struct {
	consul *consul.Client
}

// NewClient returns an implementation of the Client interface, wrapping a
// concrete Consul client.
func NewClient(c *consul.Client) Client {
	return &client{consul: c}
}

func (c *client) Register(r *consul.AgentServiceRegistration) error {
	return c.consul.Agent().ServiceRegister(r)
}

func (c *client) Deregister(r *consul.AgentServiceRegistration) error {
	return c.consul.Agent().ServiceDeregister(r.ID)
}

func (c *client) Service(service, tag string, passingOnly bool, queryOpts *consul.QueryOptions) ([]*consul.ServiceEntry, *consul.QueryMeta, error) {
	return c.consul.Health().Service(service, tag, passingOnly, queryOpts)
}
