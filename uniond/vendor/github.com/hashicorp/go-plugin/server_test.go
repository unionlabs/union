// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MPL-2.0

package plugin

import (
	"bytes"
	"context"
	"log"
	"net"
	"os"
	"path"
	"runtime"
	"strings"
	"testing"
	"time"

	hclog "github.com/hashicorp/go-hclog"
)

func TestServer_testMode(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	ch := make(chan *ReattachConfig, 1)
	closeCh := make(chan struct{})
	go Serve(&ServeConfig{
		HandshakeConfig: testHandshake,
		Plugins:         testGRPCPluginMap,
		GRPCServer:      DefaultGRPCServer,
		Test: &ServeTestConfig{
			Context:          ctx,
			ReattachConfigCh: ch,
			CloseCh:          closeCh,
		},
	})

	// We should get a config
	var config *ReattachConfig
	select {
	case config = <-ch:
	case <-time.After(2000 * time.Millisecond):
		t.Fatal("should've received reattach")
	}
	if config == nil {
		t.Fatal("config should not be nil")
	}

	// Check that the reattach config includes the negotiated protocol version
	if config.ProtocolVersion != int(testHandshake.ProtocolVersion) {
		t.Fatalf("wrong protocol version in reattach config. got %d, expected %d", config.ProtocolVersion, testHandshake.ProtocolVersion)
	}

	// Connect!
	c := NewClient(&ClientConfig{
		Cmd:              nil,
		HandshakeConfig:  testHandshake,
		Plugins:          testGRPCPluginMap,
		Reattach:         config,
		AllowedProtocols: []Protocol{ProtocolGRPC},
	})
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	// Pinging should work
	if err := client.Ping(); err != nil {
		t.Fatalf("should not err: %s", err)
	}

	// Kill which should do nothing
	c.Kill()
	if err := client.Ping(); err != nil {
		t.Fatalf("should not err: %s", err)
	}

	// Canceling should cause an exit
	cancel()
	<-closeCh
	if err := client.Ping(); err == nil {
		t.Fatal("should error")
	}

	// Try logging, this should show out in tests. We have to manually verify.
	t.Logf("HELLO")
}

func TestServer_testMode_AutoMTLS(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	closeCh := make(chan struct{})
	go Serve(&ServeConfig{
		HandshakeConfig: testVersionedHandshake,
		VersionedPlugins: map[int]PluginSet{
			2: testGRPCPluginMap,
		},
		GRPCServer: DefaultGRPCServer,
		Logger:     hclog.NewNullLogger(),
		Test: &ServeTestConfig{
			Context:          ctx,
			ReattachConfigCh: nil,
			CloseCh:          closeCh,
		},
	})

	// Connect!
	process := helperProcess("test-mtls")
	c := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testVersionedHandshake,
		VersionedPlugins: map[int]PluginSet{
			2: testGRPCPluginMap,
		},
		AllowedProtocols: []Protocol{ProtocolGRPC},
		AutoMTLS:         true,
	})
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	// Pinging should work
	if err := client.Ping(); err != nil {
		t.Fatalf("should not err: %s", err)
	}

	// Grab the impl
	raw, err := client.Dispense("test")
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	tester, ok := raw.(testInterface)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	n := tester.Double(3)
	if n != 6 {
		t.Fatal("invalid response", n)
	}

	// ensure we can make use of bidirectional communication with AutoMTLS
	// enabled
	err = tester.Bidirectional()
	if err != nil {
		t.Fatal("invalid response", err)
	}

	c.Kill()
	// Canceling should cause an exit
	cancel()
	<-closeCh
}

func TestServer_RPC(t *testing.T) {
	closeCh := make(chan struct{})
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// make a server, but we don't need to attach to it
	ch := make(chan *ReattachConfig, 1)
	go Serve(&ServeConfig{
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
		Test: &ServeTestConfig{
			Context:          ctx,
			CloseCh:          closeCh,
			ReattachConfigCh: ch,
		},
	})

	// Wait for the server
	select {
	case cfg := <-ch:
		if cfg == nil {
			t.Fatal("attach config should not be nil")
		}
	case <-time.After(2000 * time.Millisecond):
		t.Fatal("should've received reattach")
	}

	cancel()
	<-closeCh
}

func TestRmListener_impl(t *testing.T) {
	var _ net.Listener = new(rmListener)
}

func TestRmListener(t *testing.T) {
	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	tf, err := os.CreateTemp("", "plugin")
	if err != nil {
		t.Fatalf("err: %s", err)
	}
	path := tf.Name()

	// Close the file
	if err := tf.Close(); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Create the listener and test close
	rmL := newDeleteFileListener(l, path)
	if err := rmL.Close(); err != nil {
		t.Fatalf("err: %s", err)
	}

	// File should be goe
	if _, err := os.Stat(path); err == nil || !os.IsNotExist(err) {
		t.Fatalf("err: %s", err)
	}
}

func TestProtocolSelection_no_server(t *testing.T) {
	conf := &ServeConfig{
		HandshakeConfig: testVersionedHandshake,
		VersionedPlugins: map[int]PluginSet{
			2: testGRPCPluginMap,
		},
		GRPCServer:  DefaultGRPCServer,
		TLSProvider: helperTLSProvider,
	}

	_, protocol, _ := protocolVersion(conf)
	if protocol != ProtocolGRPC {
		t.Fatalf("bad protocol %s", protocol)
	}

	conf = &ServeConfig{
		HandshakeConfig: testVersionedHandshake,
		VersionedPlugins: map[int]PluginSet{
			2: testGRPCPluginMap,
		},
		TLSProvider: helperTLSProvider,
	}

	_, protocol, _ = protocolVersion(conf)
	if protocol != ProtocolNetRPC {
		t.Fatalf("bad protocol %s", protocol)
	}
}

func TestServer_testStdLogger(t *testing.T) {
	closeCh := make(chan struct{})
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	var logOut bytes.Buffer

	hclogger := hclog.New(&hclog.LoggerOptions{
		Name:       "test",
		Level:      hclog.Trace,
		Output:     &logOut,
		JSONFormat: true,
	})

	// Wrap the hclog.Logger to use it from the default std library logger
	// (and restore the original logger)
	defer func() {
		log.SetOutput(os.Stderr)
		log.SetFlags(log.LstdFlags)
		log.SetPrefix(log.Prefix())
	}()
	log.SetOutput(hclogger.StandardWriter(&hclog.StandardLoggerOptions{InferLevels: true}))
	log.SetFlags(0)
	log.SetPrefix("")

	// make a server, but we don't need to attach to it
	ch := make(chan *ReattachConfig, 1)
	go Serve(&ServeConfig{
		HandshakeConfig: testHandshake,
		Plugins:         testGRPCPluginMap,
		GRPCServer:      DefaultGRPCServer,
		Logger:          hclog.NewNullLogger(),
		Test: &ServeTestConfig{
			Context:          ctx,
			CloseCh:          closeCh,
			ReattachConfigCh: ch,
		},
	})

	// Wait for the server
	select {
	case cfg := <-ch:
		if cfg == nil {
			t.Fatal("attach config should not be nil")
		}
	case <-time.After(2000 * time.Millisecond):
		t.Fatal("should've received reattach")
	}

	log.Println("[DEBUG] test log")
	// shut down the server so there's no race on the buffer
	cancel()
	<-closeCh

	if !strings.Contains(logOut.String(), "test log") {
		t.Fatalf("expected: %q\ngot: %q", "test log", logOut.String())
	}
}

func TestUnixSocketDir(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("go-plugin doesn't support unix sockets on Windows")
	}

	tmpDir := t.TempDir()
	t.Setenv(EnvUnixSocketDir, tmpDir)

	closeCh := make(chan struct{})
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// make a server, but we don't need to attach to it
	ch := make(chan *ReattachConfig, 1)
	go Serve(&ServeConfig{
		HandshakeConfig: testHandshake,
		Plugins:         testGRPCPluginMap,
		GRPCServer:      DefaultGRPCServer,
		Logger:          hclog.NewNullLogger(),
		Test: &ServeTestConfig{
			Context:          ctx,
			CloseCh:          closeCh,
			ReattachConfigCh: ch,
		},
	})

	// Wait for the server
	var cfg *ReattachConfig
	select {
	case cfg = <-ch:
		if cfg == nil {
			t.Fatal("attach config should not be nil")
		}
	case <-time.After(2000 * time.Millisecond):
		t.Fatal("should've received reattach")
	}

	actualDir := path.Clean(path.Dir(cfg.Addr.String()))
	expectedDir := path.Clean(tmpDir)
	if actualDir != expectedDir {
		t.Fatalf("Expected socket in dir: %s, but was in %s", expectedDir, actualDir)
	}

	cancel()
	<-closeCh
}
