package etcdv3

import (
	"context"
	"testing"
	"time"

	"google.golang.org/grpc"
)

const (
	// irrelevantEndpoint is an address which does not exists.
	irrelevantEndpoint = "http://irrelevant:12345"
)

func TestNewClient(t *testing.T) {
	client, err := NewClient(
		context.Background(),
		[]string{irrelevantEndpoint},
		ClientOptions{
			DialTimeout:   3 * time.Second,
			DialKeepAlive: 3 * time.Second,
		},
	)
	if err != nil {
		t.Fatalf("unexpected error creating client: %v", err)
	}
	if client == nil {
		t.Fatal("expected new Client, got nil")
	}
}

func TestClientOptions(t *testing.T) {
	client, err := NewClient(
		context.Background(),
		[]string{},
		ClientOptions{
			Cert:          "",
			Key:           "",
			CACert:        "",
			DialTimeout:   3 * time.Second,
			DialKeepAlive: 3 * time.Second,
		},
	)
	if err == nil {
		t.Errorf("expected error: %v", err)
	}
	if client != nil {
		t.Fatalf("expected client to be nil on failure")
	}

	_, err = NewClient(
		context.Background(),
		[]string{irrelevantEndpoint},
		ClientOptions{
			Cert:          "does-not-exist.crt",
			Key:           "does-not-exist.key",
			CACert:        "does-not-exist.CACert",
			DialTimeout:   3 * time.Second,
			DialKeepAlive: 3 * time.Second,
		},
	)
	if err == nil {
		t.Errorf("expected error: %v", err)
	}

	client, err = NewClient(
		context.Background(),
		[]string{irrelevantEndpoint},
		ClientOptions{
			DialOptions: []grpc.DialOption{grpc.WithBlock()},
		},
	)
	if err == nil {
		t.Errorf("expected connection should fail")
	}
	if client != nil {
		t.Errorf("expected client to be nil on failure")
	}
}
