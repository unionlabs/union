// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package main

import (
	"bufio"
	"bytes"
	"context"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
	"testing"
)

func init() {
	// We reexec the test binary with CMD_INTEROP_MAIN=1 to run main.
	if os.Getenv("CMD_INTEROP_MAIN") == "1" {
		main()
		os.Exit(0)
	}
}

var (
	tryExecOnce sync.Once
	tryExecErr  error
)

// needsExec skips the test if we can't use exec.Command.
func needsExec(t *testing.T) {
	tryExecOnce.Do(func() {
		cmd := exec.Command(os.Args[0], "-test.list=^$")
		cmd.Env = []string{}
		tryExecErr = cmd.Run()
	})
	if tryExecErr != nil {
		t.Skipf("skipping test: cannot exec subprocess: %v", tryExecErr)
	}
}

type interopTest struct {
	donec chan struct{}
	addr  string
	cmd   *exec.Cmd
}

func run(ctx context.Context, t *testing.T, name, testcase string, args []string) *interopTest {
	needsExec(t)
	ctx, cancel := context.WithCancel(ctx)
	cmd := exec.CommandContext(ctx, os.Args[0], args...)
	out, err := cmd.StderrPipe()
	if err != nil {
		t.Fatal(err)
	}
	cmd.Stdout = cmd.Stderr
	cmd.Env = []string{
		"CMD_INTEROP_MAIN=1",
		"TESTCASE=" + testcase,
	}
	t.Logf("run %v: %v", name, args)
	err = cmd.Start()
	if err != nil {
		t.Fatal(err)
	}

	addrc := make(chan string, 1)
	donec := make(chan struct{})
	go func() {
		defer close(addrc)
		defer close(donec)
		defer t.Logf("%v done", name)
		s := bufio.NewScanner(out)
		for s.Scan() {
			line := s.Text()
			t.Logf("%v: %v", name, line)
			_, addr, ok := strings.Cut(line, "listening on ")
			if ok {
				select {
				case addrc <- addr:
				default:
				}
			}
		}
	}()

	t.Cleanup(func() {
		cancel()
		<-donec
	})

	addr, ok := <-addrc
	if !ok {
		t.Fatal(cmd.Wait())
	}
	_, port, _ := net.SplitHostPort(addr)
	addr = net.JoinHostPort("localhost", port)

	iop := &interopTest{
		cmd:   cmd,
		donec: donec,
		addr:  addr,
	}
	return iop
}

func (iop *interopTest) wait() {
	<-iop.donec
}

func TestTransfer(t *testing.T) {
	ctx := context.Background()
	src := t.TempDir()
	dst := t.TempDir()
	certs := t.TempDir()
	certFile := filepath.Join(certs, "cert.pem")
	keyFile := filepath.Join(certs, "key.pem")
	sourceName := "source"
	content := []byte("hello, world\n")

	os.WriteFile(certFile, localhostCert, 0600)
	os.WriteFile(keyFile, localhostKey, 0600)
	os.WriteFile(filepath.Join(src, sourceName), content, 0600)

	srv := run(ctx, t, "server", "transfer", []string{
		"-listen", "localhost:0",
		"-cert", filepath.Join(certs, "cert.pem"),
		"-key", filepath.Join(certs, "key.pem"),
		"-root", src,
	})
	cli := run(ctx, t, "client", "transfer", []string{
		"-output", dst, "https://" + srv.addr + "/" + sourceName,
	})
	cli.wait()

	got, err := os.ReadFile(filepath.Join(dst, "source"))
	if err != nil {
		t.Fatalf("reading downloaded file: %v", err)
	}
	if !bytes.Equal(got, content) {
		t.Fatalf("got downloaded file: %q, want %q", string(got), string(content))
	}
}

// localhostCert is a PEM-encoded TLS cert with SAN IPs
// "127.0.0.1" and "[::1]", expiring at Jan 29 16:00:00 2084 GMT.
// generated from src/crypto/tls:
// go run generate_cert.go  --ecdsa-curve P256 --host 127.0.0.1,::1,example.com --ca --start-date "Jan 1 00:00:00 1970" --duration=1000000h
var localhostCert = []byte(`-----BEGIN CERTIFICATE-----
MIIBrDCCAVKgAwIBAgIPCvPhO+Hfv+NW76kWxULUMAoGCCqGSM49BAMCMBIxEDAO
BgNVBAoTB0FjbWUgQ28wIBcNNzAwMTAxMDAwMDAwWhgPMjA4NDAxMjkxNjAwMDBa
MBIxEDAOBgNVBAoTB0FjbWUgQ28wWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAARh
WRF8p8X9scgW7JjqAwI9nYV8jtkdhqAXG9gyEgnaFNN5Ze9l3Tp1R9yCDBMNsGms
PyfMPe5Jrha/LmjgR1G9o4GIMIGFMA4GA1UdDwEB/wQEAwIChDATBgNVHSUEDDAK
BggrBgEFBQcDATAPBgNVHRMBAf8EBTADAQH/MB0GA1UdDgQWBBSOJri/wLQxq6oC
Y6ZImms/STbTljAuBgNVHREEJzAlggtleGFtcGxlLmNvbYcEfwAAAYcQAAAAAAAA
AAAAAAAAAAAAATAKBggqhkjOPQQDAgNIADBFAiBUguxsW6TGhixBAdORmVNnkx40
HjkKwncMSDbUaeL9jQIhAJwQ8zV9JpQvYpsiDuMmqCuW35XXil3cQ6Drz82c+fvE
-----END CERTIFICATE-----`)

// localhostKey is the private key for localhostCert.
var localhostKey = []byte(testingKey(`-----BEGIN TESTING KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgY1B1eL/Bbwf/MDcs
rnvvWhFNr1aGmJJR59PdCN9lVVqhRANCAARhWRF8p8X9scgW7JjqAwI9nYV8jtkd
hqAXG9gyEgnaFNN5Ze9l3Tp1R9yCDBMNsGmsPyfMPe5Jrha/LmjgR1G9
-----END TESTING KEY-----`))

// testingKey helps keep security scanners from getting excited about a private key in this file.
func testingKey(s string) string { return strings.ReplaceAll(s, "TESTING KEY", "PRIVATE KEY") }
