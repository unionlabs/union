//go:build flaky_integration
// +build flaky_integration

package etcd

import (
	"context"
	"io"
	"os"
	"testing"
	"time"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/sd"
	"github.com/go-kit/log"
)

// Package sd/etcd provides a wrapper around the etcd key/value store. This
// example assumes the user has an instance of etcd installed and running
// locally on port 2379.
func TestIntegration(t *testing.T) {
	addr := os.Getenv("ETCD_ADDR")
	if addr == "" {
		t.Skip("ETCD_ADDR not set; skipping integration test")
	}

	var (
		prefix   = "/services/foosvc/" // known at compile time
		instance = "1.2.3.4:8080"      // taken from runtime or platform, somehow
		key      = prefix + instance
		value    = "http://" + instance // based on our transport
	)

	client, err := NewClient(context.Background(), []string{addr}, ClientOptions{
		DialTimeout:             2 * time.Second,
		DialKeepAlive:           2 * time.Second,
		HeaderTimeoutPerRequest: 2 * time.Second,
	})
	if err != nil {
		t.Fatalf("NewClient(%q): %v", addr, err)
	}

	// Verify test data is initially empty.
	entries, err := client.GetEntries(key)
	if err == nil {
		t.Fatalf("GetEntries(%q): expected error, got none", key)
	}
	t.Logf("GetEntries(%q): %v (OK)", key, err)

	// Instantiate a new Registrar, passing in test data.
	registrar := NewRegistrar(client, Service{
		Key:   key,
		Value: value,
	}, log.With(log.NewLogfmtLogger(os.Stderr), "component", "registrar"))

	// Register our instance.
	registrar.Register()
	t.Logf("Registered")

	// Retrieve entries from etcd manually.
	entries, err = client.GetEntries(key)
	if err != nil {
		t.Fatalf("client.GetEntries(%q): %v", key, err)
	}
	if want, have := 1, len(entries); want != have {
		t.Fatalf("client.GetEntries(%q): want %d, have %d", key, want, have)
	}
	if want, have := value, entries[0]; want != have {
		t.Fatalf("want %q, have %q", want, have)
	}

	instancer, err := NewInstancer(
		client,
		prefix,
		log.With(log.NewLogfmtLogger(os.Stderr), "component", "instancer"),
	)
	if err != nil {
		t.Fatalf("NewInstancer: %v", err)
	}
	endpointer := sd.NewEndpointer(
		instancer,
		func(string) (endpoint.Endpoint, io.Closer, error) { return endpoint.Nop, nil, nil },
		log.With(log.NewLogfmtLogger(os.Stderr), "component", "instancer"),
	)
	t.Logf("Constructed Endpointer OK")

	if !within(time.Second, func() bool {
		endpoints, err := endpointer.Endpoints()
		return err == nil && len(endpoints) == 1
	}) {
		t.Fatalf("Endpointer didn't see Register in time")
	}
	t.Logf("Endpointer saw Register OK")

	// Deregister first instance of test data.
	registrar.Deregister()
	t.Logf("Deregistered")

	// Check it was deregistered.
	if !within(time.Second, func() bool {
		endpoints, err := endpointer.Endpoints()
		t.Logf("Checking Deregister: len(endpoints) = %d, err = %v", len(endpoints), err)
		return err == nil && len(endpoints) == 0
	}) {
		t.Fatalf("Endpointer didn't see Deregister in time")
	}

	// Verify test data no longer exists in etcd.
	_, err = client.GetEntries(key)
	if err == nil {
		t.Fatalf("GetEntries(%q): expected error, got none", key)
	}
	t.Logf("GetEntries(%q): %v (OK)", key, err)
}

func within(d time.Duration, f func() bool) bool {
	deadline := time.Now().Add(d)
	for time.Now().Before(deadline) {
		if f() {
			return true
		}
		time.Sleep(d / 10)
	}
	return false
}
