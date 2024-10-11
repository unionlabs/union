// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MPL-2.0

package plugin

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/hashicorp/go-hclog"
	"github.com/hashicorp/go-plugin/internal/cmdrunner"
	"github.com/hashicorp/go-plugin/runner"
)

func TestClient(t *testing.T) {
	process := helperProcess("mock")
	logger := &trackingLogger{Logger: hclog.Default()}
	c := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
		Logger:          logger,
	})
	defer c.Kill()

	// Test that it parses the proper address
	addr, err := c.Start()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	if addr.Network() != "tcp" {
		t.Fatalf("bad: %#v", addr)
	}

	if addr.String() != ":1234" {
		t.Fatalf("bad: %#v", addr)
	}

	// Test that it exits properly if killed
	c.Kill()

	// Test that it knows it is exited
	if !c.Exited() {
		t.Fatal("should say client has exited")
	}

	// this test isn't expected to get a client
	if !c.killed() {
		t.Fatal("Client should have failed")
	}

	// One error for connection refused, one for plugin exited.
	assertLines(t, logger.errorLogs, 2)
}

// This tests a bug where Kill would start
func TestClient_killStart(t *testing.T) {
	// Create a temporary dir to store the result file
	td := t.TempDir()
	defer os.RemoveAll(td)

	// Start the client
	path := filepath.Join(td, "booted")
	process := helperProcess("bad-version", path)
	c := NewClient(&ClientConfig{Cmd: process, HandshakeConfig: testHandshake})
	defer c.Kill()

	// Verify our path doesn't exist
	if _, err := os.Stat(path); err == nil || !os.IsNotExist(err) {
		t.Fatalf("bad: %s", err)
	}

	// Test that it parses the proper address
	if _, err := c.Start(); err == nil {
		t.Fatal("expected error")
	}

	// Verify we started
	if _, err := os.Stat(path); err != nil {
		t.Fatalf("bad: %s", err)
	}
	if err := os.Remove(path); err != nil {
		t.Fatalf("bad: %s", err)
	}

	// Test that Kill does nothing really
	c.Kill()

	// Test that it knows it is exited
	if !c.Exited() {
		t.Fatal("should say client has exited")
	}

	if !c.killed() {
		t.Fatal("process should have failed")
	}

	// Verify our path doesn't exist
	if _, err := os.Stat(path); err == nil || !os.IsNotExist(err) {
		t.Fatalf("bad: %s", err)
	}
}

func TestClient_testCleanup(t *testing.T) {
	t.Parallel()

	// Create a path that the helper process will write on cleanup
	path := filepath.Join(t.TempDir(), "output")

	// Test the cleanup
	process := helperProcess("cleanup", path)
	logger := &trackingLogger{Logger: hclog.Default()}
	c := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
		Logger:          logger,
	})

	// Grab the client so the process starts
	if _, err := c.Client(); err != nil {
		c.Kill()
		t.Fatalf("err: %s", err)
	}

	// Kill it gracefully
	c.Kill()

	// Test for the file
	if _, err := os.Stat(path); err != nil {
		t.Fatalf("err: %s", err)
	}

	assertLines(t, logger.errorLogs, 0)
}

func TestClient_noStdoutScannerRace(t *testing.T) {
	t.Parallel()

	process := helperProcess("test-grpc")
	logger := &trackingLogger{Logger: hclog.Default()}
	c := NewClient(&ClientConfig{
		RunnerFunc: func(l hclog.Logger, cmd *exec.Cmd, tmpDir string) (runner.Runner, error) {
			process.Env = append(process.Env, cmd.Env...)
			concreteRunner, err := cmdrunner.NewCmdRunner(l, process)
			if err != nil {
				return nil, err
			}
			// Inject a delay before calling .Read() method on the command's
			// stdout reader. This ensures that if there is a race between the
			// stdout scanner loop reading stdout and runner.Wait() closing
			// stdout, .Wait() will win and trigger a scanner error in the logs.
			return &delayedStdoutCmdRunner{concreteRunner}, nil
		},
		HandshakeConfig:  testHandshake,
		Plugins:          testGRPCPluginMap,
		AllowedProtocols: []Protocol{ProtocolGRPC},
		Logger:           logger,
	})

	// Grab the client so the process starts
	if _, err := c.Client(); err != nil {
		c.Kill()
		t.Fatalf("err: %s", err)
	}

	// Kill it gracefully
	c.Kill()

	assertLines(t, logger.errorLogs, 0)
}

type delayedStdoutCmdRunner struct {
	*cmdrunner.CmdRunner
}

func (m *delayedStdoutCmdRunner) Stdout() io.ReadCloser {
	return &delayedReader{m.CmdRunner.Stdout()}
}

type delayedReader struct {
	io.ReadCloser
}

func (d *delayedReader) Read(p []byte) (n int, err error) {
	time.Sleep(100 * time.Millisecond)
	return d.ReadCloser.Read(p)
}

func TestClient_testInterface(t *testing.T) {
	process := helperProcess("test-interface")
	c := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
	})
	defer c.Kill()

	// Grab the RPC client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	// Grab the impl
	raw, err := client.Dispense("test")
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	impl, ok := raw.(testInterface)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	result := impl.Double(21)
	if result != 42 {
		t.Fatalf("bad: %#v", result)
	}

	// Kill it
	c.Kill()

	// Test that it knows it is exited
	if !c.Exited() {
		t.Fatal("should say client has exited")
	}

	if c.killed() {
		t.Fatal("process failed to exit gracefully")
	}
}

func TestClient_grpc_servercrash(t *testing.T) {
	process := helperProcess("test-grpc")
	c := NewClient(&ClientConfig{
		Cmd:              process,
		HandshakeConfig:  testHandshake,
		Plugins:          testGRPCPluginMap,
		AllowedProtocols: []Protocol{ProtocolGRPC},
	})
	defer c.Kill()

	if _, err := c.Start(); err != nil {
		t.Fatalf("err: %s", err)
	}

	if v := c.Protocol(); v != ProtocolGRPC {
		t.Fatalf("bad: %s", v)
	}

	// Grab the RPC client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	// Grab the impl
	raw, err := client.Dispense("test")
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	_, ok := raw.(testInterface)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	c.runner.Kill(context.Background())

	select {
	case <-c.doneCtx.Done():
	case <-time.After(time.Second * 2):
		t.Fatal("Context was not closed")
	}
}

func TestClient_grpc(t *testing.T) {
	process := helperProcess("test-grpc")
	c := NewClient(&ClientConfig{
		Cmd:              process,
		HandshakeConfig:  testHandshake,
		Plugins:          testGRPCPluginMap,
		AllowedProtocols: []Protocol{ProtocolGRPC},
	})
	defer c.Kill()

	if _, err := c.Start(); err != nil {
		t.Fatalf("err: %s", err)
	}

	if v := c.Protocol(); v != ProtocolGRPC {
		t.Fatalf("bad: %s", v)
	}

	// Grab the RPC client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	// Grab the impl
	raw, err := client.Dispense("test")
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	impl, ok := raw.(testInterface)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	result := impl.Double(21)
	if result != 42 {
		t.Fatalf("bad: %#v", result)
	}

	// Kill it
	c.Kill()

	// Test that it knows it is exited
	if !c.Exited() {
		t.Fatal("should say client has exited")
	}

	if c.killed() {
		t.Fatal("process failed to exit gracefully")
	}
}

func TestClient_grpcNotAllowed(t *testing.T) {
	process := helperProcess("test-grpc")
	c := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
	})
	defer c.Kill()

	if _, err := c.Start(); err == nil {
		t.Fatal("should error")
	}
}

func TestClient_grpcSyncStdio(t *testing.T) {
	for name, tc := range map[string]struct {
		useRunnerFunc bool
	}{
		"default":        {false},
		"use RunnerFunc": {true},
	} {
		t.Run(name, func(t *testing.T) {
			testClient_grpcSyncStdio(t, tc.useRunnerFunc)
		})
	}
}

func testClient_grpcSyncStdio(t *testing.T, useRunnerFunc bool) {
	var syncOut, syncErr safeBuffer

	process := helperProcess("test-grpc")
	cfg := &ClientConfig{
		HandshakeConfig:  testHandshake,
		Plugins:          testGRPCPluginMap,
		AllowedProtocols: []Protocol{ProtocolGRPC},
		SyncStdout:       &syncOut,
		SyncStderr:       &syncErr,
	}

	if useRunnerFunc {
		cfg.RunnerFunc = func(l hclog.Logger, cmd *exec.Cmd, _ string) (runner.Runner, error) {
			process.Env = append(process.Env, cmd.Env...)
			return cmdrunner.NewCmdRunner(l, process)
		}
	} else {
		cfg.Cmd = process
	}
	c := NewClient(cfg)
	defer c.Kill()

	if _, err := c.Start(); err != nil {
		t.Fatalf("err: %s", err)
	}

	if v := c.Protocol(); v != ProtocolGRPC {
		t.Fatalf("bad: %s", v)
	}

	// Grab the RPC client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	// Grab the impl
	raw, err := client.Dispense("test")
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	impl, ok := raw.(testInterface)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	// Check reattach config is sensible.
	reattach := c.ReattachConfig()
	if useRunnerFunc {
		if reattach.Pid != 0 {
			t.Fatal(reattach.Pid)
		}
	} else {
		if reattach.Pid == 0 {
			t.Fatal(reattach.Pid)
		}
	}

	// Print the data
	stdout := []byte("hello\nworld!")
	stderr := []byte("and some error\n messages!")
	impl.PrintStdio(stdout, stderr)

	// Wait for it to be copied
	for syncOut.String() == "" || syncErr.String() == "" {
		time.Sleep(10 * time.Millisecond)
	}

	// We should get the data
	if syncOut.String() != string(stdout) {
		t.Fatalf("stdout didn't match: %s", syncOut.String())
	}
	if syncErr.String() != string(stderr) {
		t.Fatalf("stderr didn't match: %s", syncErr.String())
	}
}

func TestClient_cmdAndReattach(t *testing.T) {
	config := &ClientConfig{
		Cmd:      helperProcess("start-timeout"),
		Reattach: &ReattachConfig{},
	}

	c := NewClient(config)
	defer c.Kill()

	_, err := c.Start()
	if err == nil {
		t.Fatal("err should not be nil")
	}
}

func TestClient_reattach(t *testing.T) {
	process := helperProcess("test-interface")
	c := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
	})
	defer c.Kill()

	// Grab the RPC client
	_, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	// Get the reattach configuration
	reattach := c.ReattachConfig()

	// Create a new client
	c = NewClient(&ClientConfig{
		Reattach:        reattach,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
	})

	// Grab the RPC client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	// Grab the impl
	raw, err := client.Dispense("test")
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	impl, ok := raw.(testInterface)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	result := impl.Double(21)
	if result != 42 {
		t.Fatalf("bad: %#v", result)
	}

	// Kill it
	c.Kill()

	// Test that it knows it is exited
	if !c.Exited() {
		t.Fatal("should say client has exited")
	}

	if c.killed() {
		t.Fatal("process failed to exit gracefully")
	}
}

func TestClient_reattachNoProtocol(t *testing.T) {
	process := helperProcess("test-interface")
	c := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
	})
	defer c.Kill()

	// Grab the RPC client
	_, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	// Get the reattach configuration
	reattach := c.ReattachConfig()
	reattach.Protocol = ""

	// Create a new client
	c = NewClient(&ClientConfig{
		Reattach:        reattach,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
	})

	// Grab the RPC client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	// Grab the impl
	raw, err := client.Dispense("test")
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	impl, ok := raw.(testInterface)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	result := impl.Double(21)
	if result != 42 {
		t.Fatalf("bad: %#v", result)
	}

	// Kill it
	c.Kill()

	// Test that it knows it is exited
	if !c.Exited() {
		t.Fatal("should say client has exited")
	}

	if c.killed() {
		t.Fatal("process failed to exit gracefully")
	}
}

func TestClient_reattachGRPC(t *testing.T) {
	for name, tc := range map[string]struct {
		useReattachFunc bool
	}{
		"default":          {false},
		"use ReattachFunc": {true},
	} {
		t.Run(name, func(t *testing.T) {
			testClient_reattachGRPC(t, tc.useReattachFunc)
		})
	}
}

func testClient_reattachGRPC(t *testing.T, useReattachFunc bool) {
	process := helperProcess("test-grpc")
	c := NewClient(&ClientConfig{
		Cmd:              process,
		HandshakeConfig:  testHandshake,
		Plugins:          testGRPCPluginMap,
		AllowedProtocols: []Protocol{ProtocolGRPC},
	})
	defer c.Kill()

	// Grab the RPC client
	_, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	// Get the reattach configuration
	reattach := c.ReattachConfig()

	if useReattachFunc {
		pid := reattach.Pid
		reattach.Pid = 0
		reattach.ReattachFunc = cmdrunner.ReattachFunc(pid, reattach.Addr)
	}

	// Create a new client
	c = NewClient(&ClientConfig{
		Reattach:         reattach,
		HandshakeConfig:  testHandshake,
		Plugins:          testGRPCPluginMap,
		AllowedProtocols: []Protocol{ProtocolGRPC},
	})

	// Grab the RPC client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	// Grab the impl
	raw, err := client.Dispense("test")
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	impl, ok := raw.(testInterface)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	result := impl.Double(21)
	if result != 42 {
		t.Fatalf("bad: %#v", result)
	}

	// Kill it
	c.Kill()

	// Test that it knows it is exited
	if !c.Exited() {
		t.Fatal("should say client has exited")
	}

	if c.killed() {
		t.Fatal("process failed to exit gracefully")
	}
}

func TestClient_reattachNotFound(t *testing.T) {
	// Find a bad pid
	var pid int = 5000
	for i := pid; i < 32000; i++ {
		if _, err := os.FindProcess(i); err != nil {
			pid = i
			break
		}
	}

	// Addr that won't work
	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("err: %s", err)
	}
	addr := l.Addr()
	l.Close()

	// Reattach
	c := NewClient(&ClientConfig{
		Reattach: &ReattachConfig{
			Addr: addr,
			Pid:  pid,
		},
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
	})

	if _, err := c.Start(); err == nil {
		t.Fatal("should error")
	} else if err != ErrProcessNotFound {
		t.Fatalf("err: %s", err)
	}
}

func TestClientStart_badVersion(t *testing.T) {
	config := &ClientConfig{
		Cmd:             helperProcess("bad-version"),
		StartTimeout:    50 * time.Millisecond,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
	}

	c := NewClient(config)
	defer c.Kill()

	_, err := c.Start()
	if err == nil {
		t.Fatal("err should not be nil")
	}
}

func TestClientStart_badNegotiatedVersion(t *testing.T) {
	config := &ClientConfig{
		Cmd:          helperProcess("test-versioned-plugins"),
		StartTimeout: 50 * time.Millisecond,
		// test-versioned-plugins only has version 2
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
	}

	c := NewClient(config)
	defer c.Kill()

	_, err := c.Start()
	if err == nil {
		t.Fatal("err should not be nil")
	}
	fmt.Println(err)
}

func TestClient_Start_Timeout(t *testing.T) {
	config := &ClientConfig{
		Cmd:             helperProcess("start-timeout"),
		StartTimeout:    50 * time.Millisecond,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
	}

	c := NewClient(config)
	defer c.Kill()

	_, err := c.Start()
	if err == nil {
		t.Fatal("err should not be nil")
	}
}

func TestClient_Stderr(t *testing.T) {
	stderr := new(bytes.Buffer)
	process := helperProcess("stderr")
	c := NewClient(&ClientConfig{
		Cmd:             process,
		Stderr:          stderr,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
	})
	defer c.Kill()

	if _, err := c.Start(); err != nil {
		t.Fatalf("err: %s", err)
	}

	for !c.Exited() {
		time.Sleep(10 * time.Millisecond)
	}

	if c.killed() {
		t.Fatal("process failed to exit gracefully")
	}

	if !strings.Contains(stderr.String(), "HELLO\n") {
		t.Fatalf("bad log data: '%s'", stderr.String())
	}

	if !strings.Contains(stderr.String(), "WORLD\n") {
		t.Fatalf("bad log data: '%s'", stderr.String())
	}
}

func TestClient_StderrJSON(t *testing.T) {
	stderr := new(bytes.Buffer)
	process := helperProcess("stderr-json")

	var logBuf bytes.Buffer
	mutex := new(sync.Mutex)
	// Custom hclog.Logger
	testLogger := hclog.New(&hclog.LoggerOptions{
		Name:   "test-logger",
		Level:  hclog.Trace,
		Output: &logBuf,
		Mutex:  mutex,
	})

	c := NewClient(&ClientConfig{
		Cmd:             process,
		Stderr:          stderr,
		HandshakeConfig: testHandshake,
		Logger:          testLogger,
		Plugins:         testPluginMap,
	})
	defer c.Kill()

	if _, err := c.Start(); err != nil {
		t.Fatalf("err: %s", err)
	}

	for !c.Exited() {
		time.Sleep(10 * time.Millisecond)
	}

	if c.killed() {
		t.Fatal("process failed to exit gracefully")
	}

	logOut := logBuf.String()

	if !strings.Contains(logOut, "[\"HELLO\"]\n") {
		t.Fatalf("missing json list: '%s'", logOut)
	}

	if !strings.Contains(logOut, "12345\n") {
		t.Fatalf("missing line with raw number: '%s'", logOut)
	}

	if !strings.Contains(logOut, "{\"a\":1}") {
		t.Fatalf("missing json object: '%s'", logOut)
	}
}

func TestClient_textLogLevel(t *testing.T) {
	stderr := new(bytes.Buffer)
	process := helperProcess("level-warn-text")

	var logBuf bytes.Buffer
	mutex := new(sync.Mutex)
	// Custom hclog.Logger
	testLogger := hclog.New(&hclog.LoggerOptions{
		Name:   "test-logger",
		Level:  hclog.Warn,
		Output: &logBuf,
		Mutex:  mutex,
	})

	c := NewClient(&ClientConfig{
		Cmd:             process,
		Stderr:          stderr,
		HandshakeConfig: testHandshake,
		Logger:          testLogger,
		Plugins:         testPluginMap,
	})
	defer c.Kill()

	if _, err := c.Start(); err != nil {
		t.Fatalf("err: %s", err)
	}

	for !c.Exited() {
		time.Sleep(10 * time.Millisecond)
	}

	if c.killed() {
		t.Fatal("process failed to exit gracefully")
	}

	logOut := logBuf.String()

	if !strings.Contains(logOut, "test line 98765") {
		log.Fatalf("test string not found in log: %q\n", logOut)
	}
}

func TestClient_Stdin(t *testing.T) {
	// Overwrite stdin for this test with a temporary file
	tf, err := os.CreateTemp("", "terraform")
	if err != nil {
		t.Fatalf("err: %s", err)
	}
	defer os.Remove(tf.Name())
	defer tf.Close()

	if _, err = tf.WriteString("hello"); err != nil {
		t.Fatalf("error: %s", err)
	}

	if err = tf.Sync(); err != nil {
		t.Fatalf("error: %s", err)
	}

	if _, err = tf.Seek(0, 0); err != nil {
		t.Fatalf("error: %s", err)
	}

	oldStdin := os.Stdin
	defer func() { os.Stdin = oldStdin }()
	os.Stdin = tf

	process := helperProcess("stdin")
	c := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
	})
	defer c.Kill()

	_, err = c.Start()
	if err != nil {
		t.Fatalf("error: %s", err)
	}

	for {
		if c.Exited() {
			break
		}

		time.Sleep(50 * time.Millisecond)
	}

	if !process.ProcessState.Success() {
		t.Fatal("process didn't exit cleanly")
	}
}

func TestClient_SkipHostEnv(t *testing.T) {
	for _, tc := range []struct {
		helper string
		skip   bool
	}{
		{"test-skip-host-env-true", true},
		{"test-skip-host-env-false", false},
	} {
		t.Run(tc.helper, func(t *testing.T) {
			process := helperProcess(tc.helper)
			// Set env in the host process, which we'll look for in the plugin.
			t.Setenv("PLUGIN_TEST_SKIP_HOST_ENV", "foo")
			c := NewClient(&ClientConfig{
				Cmd:             process,
				HandshakeConfig: testHandshake,
				Plugins:         testPluginMap,
				SkipHostEnv:     tc.skip,
			})
			defer c.Kill()

			_, err := c.Start()
			if err != nil {
				t.Fatalf("error: %s", err)
			}

			for {
				if c.Exited() {
					break
				}

				time.Sleep(50 * time.Millisecond)
			}

			if !process.ProcessState.Success() {
				t.Fatal("process didn't exit cleanly")
			}
		})
	}
}

func TestClient_RequestGRPCMultiplexing_UnsupportedByPlugin(t *testing.T) {
	for _, name := range []string{
		"mux-grpc-with-old-plugin",
		"mux-grpc-with-unsupported-plugin",
	} {
		t.Run(name, func(t *testing.T) {
			process := helperProcess(name)
			c := NewClient(&ClientConfig{
				Cmd:                 process,
				HandshakeConfig:     testHandshake,
				Plugins:             testGRPCPluginMap,
				AllowedProtocols:    []Protocol{ProtocolGRPC},
				GRPCBrokerMultiplex: true,
			})
			defer c.Kill()

			_, err := c.Start()
			if err == nil {
				t.Fatal("expected error")
			}

			if !errors.Is(err, ErrGRPCBrokerMuxNotSupported) {
				t.Fatalf("expected %s, but got %s", ErrGRPCBrokerMuxNotSupported, err)
			}
		})
	}
}

func TestClient_SecureConfig(t *testing.T) {
	// Test failure case
	secureConfig := &SecureConfig{
		Checksum: []byte{'1'},
		Hash:     sha256.New(),
	}
	process := helperProcess("test-interface")
	c := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
		SecureConfig:    secureConfig,
	})

	// Grab the RPC client, should error
	_, err := c.Client()
	c.Kill()
	if err != ErrChecksumsDoNotMatch {
		t.Fatalf("err should be %s, got %s", ErrChecksumsDoNotMatch, err)
	}

	// Get the checksum of the executable
	file, err := os.Open(os.Args[0])
	if err != nil {
		t.Fatal(err)
	}
	defer file.Close()

	hash := sha256.New()

	_, err = io.Copy(hash, file)
	if err != nil {
		t.Fatal(err)
	}

	sum := hash.Sum(nil)

	secureConfig = &SecureConfig{
		Checksum: sum,
		Hash:     sha256.New(),
	}

	process = helperProcess("test-interface")
	c = NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
		SecureConfig:    secureConfig,
	})
	defer c.Kill()

	// Grab the RPC client
	_, err = c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}
}

func TestClient_TLS(t *testing.T) {
	// Test failure case
	process := helperProcess("test-interface-tls")
	cBad := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
	})
	defer cBad.Kill()

	// Grab the RPC client
	clientBad, err := cBad.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	// Grab the impl
	raw, err := clientBad.Dispense("test")
	if err == nil {
		t.Fatal("expected error, got nil")
	}

	cBad.Kill()

	// Add TLS config to client
	tlsConfig, err := helperTLSProvider()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	process = helperProcess("test-interface-tls")
	c := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
		TLSConfig:       tlsConfig,
	})
	defer c.Kill()

	// Grab the RPC client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	// Grab the impl
	raw, err = client.Dispense("test")
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	impl, ok := raw.(testInterface)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	result := impl.Double(21)
	if result != 42 {
		t.Fatalf("bad: %#v", result)
	}

	// Kill it
	c.Kill()

	// Test that it knows it is exited
	if !c.Exited() {
		t.Fatal("should say client has exited")
	}

	if c.killed() {
		t.Fatal("process failed to exit gracefully")
	}
}

func TestClient_TLS_grpc(t *testing.T) {
	// Add TLS config to client
	tlsConfig, err := helperTLSProvider()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	process := helperProcess("test-grpc-tls")
	c := NewClient(&ClientConfig{
		Cmd:              process,
		HandshakeConfig:  testHandshake,
		Plugins:          testGRPCPluginMap,
		TLSConfig:        tlsConfig,
		AllowedProtocols: []Protocol{ProtocolGRPC},
	})
	defer c.Kill()

	// Grab the RPC client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	// Grab the impl
	raw, err := client.Dispense("test")
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	impl, ok := raw.(testInterface)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	result := impl.Double(21)
	if result != 42 {
		t.Fatalf("bad: %#v", result)
	}

	// Kill it
	c.Kill()

	if !c.Exited() {
		t.Fatal("should say client has exited")
	}

	if c.killed() {
		t.Fatal("process failed to exit gracefully")
	}
}

func TestClient_secureConfigAndReattach(t *testing.T) {
	config := &ClientConfig{
		SecureConfig: &SecureConfig{},
		Reattach:     &ReattachConfig{},
	}

	c := NewClient(config)
	defer c.Kill()

	_, err := c.Start()
	if err != ErrSecureConfigAndReattach {
		t.Fatalf("err should not be %s, got %s", ErrSecureConfigAndReattach, err)
	}
}

func TestClient_ping(t *testing.T) {
	process := helperProcess("test-interface")
	c := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testHandshake,
		Plugins:         testPluginMap,
	})
	defer c.Kill()

	// Get the client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	// Ping, should work
	if err := client.Ping(); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Kill it
	c.Kill()
	if err := client.Ping(); err == nil {
		t.Fatal("should error")
	}
}

func TestClient_wrongVersion(t *testing.T) {
	process := helperProcess("test-proto-upgraded-plugin")
	c := NewClient(&ClientConfig{
		Cmd:              process,
		HandshakeConfig:  testHandshake,
		Plugins:          testGRPCPluginMap,
		AllowedProtocols: []Protocol{ProtocolGRPC},
	})
	defer c.Kill()

	// Get the client
	_, err := c.Client()
	if err == nil {
		t.Fatal("expected incorrect protocol version server")
	}

}

func TestClient_legacyClient(t *testing.T) {
	process := helperProcess("test-proto-upgraded-plugin")
	c := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testVersionedHandshake,
		VersionedPlugins: map[int]PluginSet{
			1: testPluginMap,
		},
	})
	defer c.Kill()

	// Get the client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	if c.NegotiatedVersion() != 1 {
		t.Fatal("using incorrect version", c.NegotiatedVersion())
	}

	// Ping, should work
	if err := client.Ping(); err == nil {
		t.Fatal("expected error, should negotiate wrong plugin")
	}
}

func TestClient_legacyServer(t *testing.T) {
	// test using versioned plugins version when the server supports only
	// supports one
	process := helperProcess("test-proto-upgraded-client")
	c := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testVersionedHandshake,
		VersionedPlugins: map[int]PluginSet{
			2: testGRPCPluginMap,
		},
		AllowedProtocols: []Protocol{ProtocolGRPC},
	})
	defer c.Kill()

	// Get the client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	if c.NegotiatedVersion() != 2 {
		t.Fatal("using incorrect version", c.NegotiatedVersion())
	}

	// Ping, should work
	if err := client.Ping(); err == nil {
		t.Fatal("expected error, should negotiate wrong plugin")
	}
}

func TestClient_versionedClient(t *testing.T) {
	process := helperProcess("test-versioned-plugins")
	c := NewClient(&ClientConfig{
		Cmd:             process,
		HandshakeConfig: testVersionedHandshake,
		VersionedPlugins: map[int]PluginSet{
			2: testGRPCPluginMap,
		},
		AllowedProtocols: []Protocol{ProtocolGRPC},
	})
	defer c.Kill()

	if _, err := c.Start(); err != nil {
		t.Fatalf("err: %s", err)
	}

	if v := c.Protocol(); v != ProtocolGRPC {
		t.Fatalf("bad: %s", v)
	}

	// Grab the RPC client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	if c.NegotiatedVersion() != 2 {
		t.Fatal("using incorrect version", c.NegotiatedVersion())
	}

	// Grab the impl
	raw, err := client.Dispense("test")
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	_, ok := raw.(testInterface)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	c.runner.Kill(context.Background())

	select {
	case <-c.doneCtx.Done():
	case <-time.After(time.Second * 2):
		t.Fatal("Context was not closed")
	}
}

func TestClient_mtlsClient(t *testing.T) {
	process := helperProcess("test-mtls")
	c := NewClient(&ClientConfig{
		AutoMTLS:        true,
		Cmd:             process,
		HandshakeConfig: testVersionedHandshake,
		VersionedPlugins: map[int]PluginSet{
			2: testGRPCPluginMap,
		},
		AllowedProtocols: []Protocol{ProtocolGRPC},
	})
	defer c.Kill()

	if _, err := c.Start(); err != nil {
		t.Fatalf("err: %s", err)
	}

	if v := c.Protocol(); v != ProtocolGRPC {
		t.Fatalf("bad: %s", v)
	}

	// Grab the RPC client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	if c.NegotiatedVersion() != 2 {
		t.Fatal("using incorrect version", c.NegotiatedVersion())
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

	c.runner.Kill(context.Background())

	select {
	case <-c.doneCtx.Done():
	case <-time.After(time.Second * 2):
		t.Fatal("Context was not closed")
	}
}

func TestClient_mtlsNetRPCClient(t *testing.T) {
	process := helperProcess("test-interface-mtls")
	c := NewClient(&ClientConfig{
		AutoMTLS:         true,
		Cmd:              process,
		HandshakeConfig:  testVersionedHandshake,
		Plugins:          testPluginMap,
		AllowedProtocols: []Protocol{ProtocolNetRPC},
	})
	defer c.Kill()

	if _, err := c.Start(); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Grab the RPC client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
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

	c.runner.Kill(context.Background())

	select {
	case <-c.doneCtx.Done():
	case <-time.After(time.Second * 2):
		t.Fatal("Context was not closed")
	}
}

func TestClient_logger(t *testing.T) {
	t.Run("net/rpc", func(t *testing.T) { testClient_logger(t, "netrpc") })
	t.Run("grpc", func(t *testing.T) { testClient_logger(t, "grpc") })
}

func testClient_logger(t *testing.T, proto string) {
	var buffer bytes.Buffer
	mutex := new(sync.Mutex)
	stderr := io.MultiWriter(os.Stderr, &buffer)
	// Custom hclog.Logger
	clientLogger := hclog.New(&hclog.LoggerOptions{
		Name:   "test-logger",
		Level:  hclog.Trace,
		Output: stderr,
		Mutex:  mutex,
	})

	process := helperProcess("test-interface-logger-" + proto)
	c := NewClient(&ClientConfig{
		Cmd:              process,
		HandshakeConfig:  testHandshake,
		Plugins:          testGRPCPluginMap,
		Logger:           clientLogger,
		AllowedProtocols: []Protocol{ProtocolNetRPC, ProtocolGRPC},
	})
	defer c.Kill()

	// Grab the RPC client
	client, err := c.Client()
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	// Grab the impl
	raw, err := client.Dispense("test")
	if err != nil {
		t.Fatalf("err should be nil, got %s", err)
	}

	impl, ok := raw.(testInterface)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	{
		// Discard everything else, and capture the output we care about
		mutex.Lock()
		buffer.Reset()
		mutex.Unlock()
		impl.PrintKV("foo", "bar")
		time.Sleep(100 * time.Millisecond)
		mutex.Lock()
		line, err := buffer.ReadString('\n')
		mutex.Unlock()
		if err != nil {
			t.Fatal(err)
		}
		if !strings.Contains(line, "foo=bar") {
			t.Fatalf("bad: %q", line)
		}
	}

	{
		// Try an integer type
		mutex.Lock()
		buffer.Reset()
		mutex.Unlock()
		impl.PrintKV("foo", 12)
		time.Sleep(100 * time.Millisecond)
		mutex.Lock()
		line, err := buffer.ReadString('\n')
		mutex.Unlock()
		if err != nil {
			t.Fatal(err)
		}
		if !strings.Contains(line, "foo=12") {
			t.Fatalf("bad: %q", line)
		}
	}

	// Kill it
	c.Kill()

	// Test that it knows it is exited
	if !c.Exited() {
		t.Fatal("should say client has exited")
	}

	if c.killed() {
		t.Fatal("process failed to exit gracefully")
	}
}

// Test that we continue to consume stderr over long lines.
func TestClient_logStderr(t *testing.T) {
	stderr := bytes.Buffer{}
	c := NewClient(&ClientConfig{
		Stderr: &stderr,
		Cmd: &exec.Cmd{
			Path: "test",
		},
		PluginLogBufferSize: 32,
	})
	c.clientWaitGroup.Add(1)

	msg := `
this line is more than 32 bytes long
and this line is more than 32 bytes long
{"a": "b", "@level": "debug"}
this line is short
`

	reader := strings.NewReader(msg)

	c.pipesWaitGroup.Add(1)
	c.logStderr(c.config.Cmd.Path, reader)
	read := stderr.String()

	if read != msg {
		t.Fatalf("\nexpected output: %q\ngot output:      %q", msg, read)
	}
}

func TestClient_logStderrParseJSON(t *testing.T) {
	logBuf := bytes.Buffer{}
	c := NewClient(&ClientConfig{
		Stderr:              bytes.NewBuffer(nil),
		Cmd:                 &exec.Cmd{Path: "test"},
		PluginLogBufferSize: 64,
		Logger: hclog.New(&hclog.LoggerOptions{
			Name:       "test-logger",
			Level:      hclog.Trace,
			Output:     &logBuf,
			JSONFormat: true,
		}),
	})
	c.clientWaitGroup.Add(1)

	msg := `{"@message": "this is a message", "@level": "info"}
{"@message": "this is a large message that is more than 64 bytes long", "@level": "info"}`
	reader := strings.NewReader(msg)

	c.pipesWaitGroup.Add(1)
	c.logStderr(c.config.Cmd.Path, reader)
	logs := strings.Split(strings.TrimSpace(logBuf.String()), "\n")

	wants := []struct {
		wantLevel   string
		wantMessage string
	}{
		{"info", "this is a message"},
		{"debug", `{"@message": "this is a large message that is more than 64 bytes`},
		{"debug", ` long", "@level": "info"}`},
	}

	if len(logs) != len(wants) {
		t.Fatalf("expected %d logs, got %d", len(wants), len(logs))
	}

	for i, tt := range wants {
		l := make(map[string]interface{})
		if err := json.Unmarshal([]byte(logs[i]), &l); err != nil {
			t.Fatal(err)
		}

		if l["@level"] != tt.wantLevel {
			t.Fatalf("expected level %q, got %q", tt.wantLevel, l["@level"])
		}

		if l["@message"] != tt.wantMessage {
			t.Fatalf("expected message %q, got %q", tt.wantMessage, l["@message"])
		}
	}
}

type trackingLogger struct {
	hclog.Logger
	errorLogs []string
}

func (l *trackingLogger) Error(msg string, args ...interface{}) {
	l.errorLogs = append(l.errorLogs, fmt.Sprintf("%s: %v", msg, args))
	l.Logger.Error(msg, args...)
}

func assertLines(t *testing.T, lines []string, expected int) {
	t.Helper()
	if len(lines) != expected {
		t.Errorf("expected %d, got %d", expected, len(lines))
		for _, log := range lines {
			t.Error(log)
		}
	}
}
