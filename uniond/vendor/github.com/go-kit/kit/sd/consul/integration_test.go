//go:build integration
// +build integration

package consul

import (
	"io"
	"os"
	"testing"
	"time"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/sd"
	"github.com/go-kit/log"
	stdconsul "github.com/hashicorp/consul/api"
)

func TestIntegration(t *testing.T) {
	consulAddr := os.Getenv("CONSUL_ADDR")
	if consulAddr == "" {
		t.Skip("CONSUL_ADDR not set; skipping integration test")
	}
	stdClient, err := stdconsul.NewClient(&stdconsul.Config{
		Address: consulAddr,
	})
	if err != nil {
		t.Fatal(err)
	}
	client := NewClient(stdClient)
	logger := log.NewLogfmtLogger(os.Stderr)

	// Produce a fake service registration.
	r := &stdconsul.AgentServiceRegistration{
		ID:                "my-service-ID",
		Name:              "my-service-name",
		Tags:              []string{"alpha", "beta"},
		Port:              12345,
		Address:           "my-address",
		EnableTagOverride: false,
		// skipping check(s)
	}

	// Build an Instancer on r.Name + r.Tags.
	factory := func(instance string) (endpoint.Endpoint, io.Closer, error) {
		t.Logf("factory invoked for %q", instance)
		return endpoint.Nop, nil, nil
	}
	instancer := NewInstancer(
		client,
		log.With(logger, "component", "instancer"),
		r.Name,
		r.Tags,
		true,
	)
	endpointer := sd.NewEndpointer(
		instancer,
		factory,
		log.With(logger, "component", "endpointer"),
	)

	time.Sleep(time.Second)

	// Before we publish, we should have no endpoints.
	endpoints, err := endpointer.Endpoints()
	if err != nil {
		t.Error(err)
	}
	if want, have := 0, len(endpoints); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	// Build a registrar for r.
	registrar := NewRegistrar(client, r, log.With(logger, "component", "registrar"))
	registrar.Register()
	defer registrar.Deregister()

	time.Sleep(time.Second)

	// Now we should have one active endpoints.
	endpoints, err = endpointer.Endpoints()
	if err != nil {
		t.Error(err)
	}
	if want, have := 1, len(endpoints); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}
