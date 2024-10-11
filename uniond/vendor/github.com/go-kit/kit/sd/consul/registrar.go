package consul

import (
	"fmt"

	stdconsul "github.com/hashicorp/consul/api"

	"github.com/go-kit/log"
)

// Registrar registers service instance liveness information to Consul.
type Registrar struct {
	client       Client
	registration *stdconsul.AgentServiceRegistration
	logger       log.Logger
}

// NewRegistrar returns a Consul Registrar acting on the provided catalog
// registration.
func NewRegistrar(client Client, r *stdconsul.AgentServiceRegistration, logger log.Logger) *Registrar {
	return &Registrar{
		client:       client,
		registration: r,
		logger:       log.With(logger, "service", r.Name, "tags", fmt.Sprint(r.Tags), "address", r.Address),
	}
}

// Register implements sd.Registrar interface.
func (p *Registrar) Register() {
	if err := p.client.Register(p.registration); err != nil {
		p.logger.Log("err", err)
	} else {
		p.logger.Log("action", "register")
	}
}

// Deregister implements sd.Registrar interface.
func (p *Registrar) Deregister() {
	if err := p.client.Deregister(p.registration); err != nil {
		p.logger.Log("err", err)
	} else {
		p.logger.Log("action", "deregister")
	}
}
