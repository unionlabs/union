// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MPL-2.0

package plugin

import (
	"bytes"
	"io"
	"os"
	"sync"
	"testing"
	"time"

	hclog "github.com/hashicorp/go-hclog"
)

func TestClient_App(t *testing.T) {
	pluginLogger := hclog.New(&hclog.LoggerOptions{
		Level:      hclog.Trace,
		Output:     os.Stderr,
		JSONFormat: true,
	})

	testPlugin := &testInterfaceImpl{
		logger: pluginLogger,
	}

	client, _ := TestPluginRPCConn(t, map[string]Plugin{
		"test": &testInterfacePlugin{Impl: testPlugin},
	}, nil)
	defer client.Close()

	raw, err := client.Dispense("test")
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	impl, ok := raw.(testInterface)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	result := impl.Double(21)
	if result != 42 {
		t.Fatalf("bad: %#v", result)
	}
}

func TestClient_syncStreams(t *testing.T) {
	// Create streams for the server that we can talk to
	stdout_r, stdout_w := io.Pipe()
	stderr_r, stderr_w := io.Pipe()

	client, _ := TestPluginRPCConn(t, map[string]Plugin{}, &TestOptions{
		ServerStdout: stdout_r,
		ServerStderr: stderr_r,
	})

	// Start the data copying
	var stdout_out, stderr_out safeBuffer
	stdout := &safeBuffer{
		b: bytes.NewBufferString("stdouttest"),
	}
	stderr := &safeBuffer{
		b: bytes.NewBufferString("stderrtest"),
	}
	go client.SyncStreams(&stdout_out, &stderr_out)
	go io.Copy(stdout_w, stdout)
	go io.Copy(stderr_w, stderr)

	// Unfortunately I can't think of a better way to make sure all the
	// copies above go through so let's just exit.
	time.Sleep(100 * time.Millisecond)

	// Close everything, and lets test the result
	client.Close()
	stdout_w.Close()
	stderr_w.Close()

	if v := stdout_out.String(); v != "stdouttest" {
		t.Fatalf("bad: %q", v)
	}
	if v := stderr_out.String(); v != "stderrtest" {
		t.Fatalf("bad: %q", v)
	}
}

type safeBuffer struct {
	sync.Mutex
	b *bytes.Buffer
}

func (s *safeBuffer) Write(p []byte) (n int, err error) {
	s.Lock()
	defer s.Unlock()
	if s.b == nil {
		s.b = new(bytes.Buffer)
	}
	return s.b.Write(p)
}

func (s *safeBuffer) Read(p []byte) (n int, err error) {
	s.Lock()
	defer s.Unlock()
	if s.b == nil {
		s.b = new(bytes.Buffer)
	}
	return s.b.Read(p)
}

func (s *safeBuffer) String() string {
	s.Lock()
	defer s.Unlock()
	if s.b == nil {
		s.b = new(bytes.Buffer)
	}
	return s.b.String()
}
