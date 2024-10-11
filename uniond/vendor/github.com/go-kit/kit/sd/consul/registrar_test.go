package consul

import (
	"testing"

	stdconsul "github.com/hashicorp/consul/api"

	"github.com/go-kit/log"
)

func TestRegistrar(t *testing.T) {
	client := newTestClient([]*stdconsul.ServiceEntry{})
	p := NewRegistrar(client, testRegistration, log.NewNopLogger())
	if want, have := 0, len(client.entries); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	p.Register()
	if want, have := 1, len(client.entries); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	p.Deregister()
	if want, have := 0, len(client.entries); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}
