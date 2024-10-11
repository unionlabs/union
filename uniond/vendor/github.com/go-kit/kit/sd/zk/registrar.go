package zk

import "github.com/go-kit/log"

// Registrar registers service instance liveness information to ZooKeeper.
type Registrar struct {
	client  Client
	service Service
	logger  log.Logger
}

// Service holds the root path, service name and instance identifying data you
// want to publish to ZooKeeper.
type Service struct {
	Path string // discovery namespace, example: /myorganization/myplatform/
	Name string // service name, example: addscv
	Data []byte // instance data to store for discovery, example: 10.0.2.10:80
	node string // Client will record the ephemeral node name so we can deregister
}

// NewRegistrar returns a ZooKeeper Registrar acting on the provided catalog
// registration.
func NewRegistrar(client Client, service Service, logger log.Logger) *Registrar {
	return &Registrar{
		client:  client,
		service: service,
		logger: log.With(logger,
			"service", service.Name,
			"path", service.Path,
			"data", string(service.Data),
		),
	}
}

// Register implements sd.Registrar interface.
func (r *Registrar) Register() {
	if err := r.client.Register(&r.service); err != nil {
		r.logger.Log("err", err)
	} else {
		r.logger.Log("action", "register")
	}
}

// Deregister implements sd.Registrar interface.
func (r *Registrar) Deregister() {
	if err := r.client.Deregister(&r.service); err != nil {
		r.logger.Log("err", err)
	} else {
		r.logger.Log("action", "deregister")
	}
}
