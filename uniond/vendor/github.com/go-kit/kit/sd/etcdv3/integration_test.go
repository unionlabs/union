//go:build flaky_integration
// +build flaky_integration

package etcdv3

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

func runIntegration(settings integrationSettings, client Client, service Service, t *testing.T) {
	// Verify test data is initially empty.
	entries, err := client.GetEntries(settings.key)
	if err != nil {
		t.Fatalf("GetEntries(%q): expected no error, got one: %v", settings.key, err)
	}
	if len(entries) > 0 {
		t.Fatalf("GetEntries(%q): expected no instance entries, got %d", settings.key, len(entries))
	}
	t.Logf("GetEntries(%q): %v (OK)", settings.key, entries)

	// Instantiate a new Registrar, passing in test data.
	registrar := NewRegistrar(
		client,
		service,
		log.With(log.NewLogfmtLogger(os.Stderr), "component", "registrar"),
	)

	// Register our instance.
	registrar.Register()
	t.Log("Registered")

	// Retrieve entries from etcd manually.
	entries, err = client.GetEntries(settings.key)
	if err != nil {
		t.Fatalf("client.GetEntries(%q): %v", settings.key, err)
	}
	if want, have := 1, len(entries); want != have {
		t.Fatalf("client.GetEntries(%q): want %d, have %d", settings.key, want, have)
	}
	if want, have := settings.value, entries[0]; want != have {
		t.Fatalf("want %q, have %q", want, have)
	}

	instancer, err := NewInstancer(
		client,
		settings.prefix,
		log.With(log.NewLogfmtLogger(os.Stderr), "component", "instancer"),
	)
	if err != nil {
		t.Fatalf("NewInstancer: %v", err)
	}
	t.Log("Constructed Instancer OK")
	defer instancer.Stop()

	endpointer := sd.NewEndpointer(
		instancer,
		func(string) (endpoint.Endpoint, io.Closer, error) { return endpoint.Nop, nil, nil },
		log.With(log.NewLogfmtLogger(os.Stderr), "component", "instancer"),
	)
	t.Log("Constructed Endpointer OK")
	defer endpointer.Close()

	if !within(time.Second, func() bool {
		endpoints, err := endpointer.Endpoints()
		return err == nil && len(endpoints) == 1
	}) {
		t.Fatal("Endpointer didn't see Register in time")
	}
	t.Log("Endpointer saw Register OK")

	// Deregister first instance of test data.
	registrar.Deregister()
	t.Log("Deregistered")

	// Check it was deregistered.
	if !within(time.Second, func() bool {
		endpoints, err := endpointer.Endpoints()
		t.Logf("Checking Deregister: len(endpoints) = %d, err = %v", len(endpoints), err)
		return err == nil && len(endpoints) == 0
	}) {
		t.Fatalf("Endpointer didn't see Deregister in time")
	}

	// Verify test data no longer exists in etcd.
	entries, err = client.GetEntries(settings.key)
	if err != nil {
		t.Fatalf("GetEntries(%q): expected no error, got one: %v", settings.key, err)
	}
	if len(entries) > 0 {
		t.Fatalf("GetEntries(%q): expected no entries, got %v", settings.key, entries)
	}
	t.Logf("GetEntries(%q): %v (OK)", settings.key, entries)
}

type integrationSettings struct {
	addr     string
	prefix   string
	instance string
	key      string
	value    string
}

func testIntegrationSettings(t *testing.T) integrationSettings {
	var settings integrationSettings

	settings.addr = os.Getenv("ETCD_ADDR")
	if settings.addr == "" {
		t.Skip("ETCD_ADDR not set; skipping integration test")
	}

	settings.prefix = "/services/foosvc/" // known at compile time
	settings.instance = "1.2.3.4:8080"    // taken from runtime or platform, somehow
	settings.key = settings.prefix + settings.instance
	settings.value = "http://" + settings.instance // based on our transport

	return settings
}

// Package sd/etcd provides a wrapper around the etcd key/value store. This
// example assumes the user has an instance of etcd installed and running
// locally on port 2379.
func TestIntegration(t *testing.T) {
	settings := testIntegrationSettings(t)
	client, err := NewClient(context.Background(), []string{settings.addr}, ClientOptions{
		DialTimeout:   2 * time.Second,
		DialKeepAlive: 2 * time.Second,
	})
	if err != nil {
		t.Fatalf("NewClient(%q): %v", settings.addr, err)
	}

	service := Service{
		Key:   settings.key,
		Value: settings.value,
	}

	runIntegration(settings, client, service, t)
}

func TestIntegrationTTL(t *testing.T) {
	settings := testIntegrationSettings(t)
	client, err := NewClient(context.Background(), []string{settings.addr}, ClientOptions{
		DialTimeout:   2 * time.Second,
		DialKeepAlive: 2 * time.Second,
	})
	if err != nil {
		t.Fatalf("NewClient(%q): %v", settings.addr, err)
	}

	service := Service{
		Key:   settings.key,
		Value: settings.value,
		TTL:   NewTTLOption(time.Second*3, time.Second*10),
	}
	defer client.Deregister(service)

	runIntegration(settings, client, service, t)
}

func TestIntegrationRegistrarOnly(t *testing.T) {
	settings := testIntegrationSettings(t)
	client, err := NewClient(context.Background(), []string{settings.addr}, ClientOptions{
		DialTimeout:   2 * time.Second,
		DialKeepAlive: 2 * time.Second,
	})
	if err != nil {
		t.Fatalf("NewClient(%q): %v", settings.addr, err)
	}

	service := Service{
		Key:   settings.key,
		Value: settings.value,
		TTL:   NewTTLOption(time.Second*3, time.Second*10),
	}
	defer client.Deregister(service)

	// Verify test data is initially empty.
	entries, err := client.GetEntries(settings.key)
	if err != nil {
		t.Fatalf("GetEntries(%q): expected no error, got one: %v", settings.key, err)
	}
	if len(entries) > 0 {
		t.Fatalf("GetEntries(%q): expected no instance entries, got %d", settings.key, len(entries))
	}
	t.Logf("GetEntries(%q): %v (OK)", settings.key, entries)

	// Instantiate a new Registrar, passing in test data.
	registrar := NewRegistrar(
		client,
		service,
		log.With(log.NewLogfmtLogger(os.Stderr), "component", "registrar"),
	)

	// Register our instance.
	registrar.Register()
	t.Log("Registered")

	// Deregister our instance. (so we test registrar only scenario)
	registrar.Deregister()
	t.Log("Deregistered")

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
