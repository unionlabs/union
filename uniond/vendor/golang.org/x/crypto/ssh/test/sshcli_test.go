// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package test

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"testing"

	"golang.org/x/crypto/internal/testenv"
	"golang.org/x/crypto/ssh"
	"golang.org/x/crypto/ssh/testdata"
)

func sshClient(t *testing.T) string {
	if testing.Short() {
		t.Skip("Skipping test that executes OpenSSH in -short mode")
	}
	sshCLI := os.Getenv("SSH_CLI_PATH")
	if sshCLI == "" {
		sshCLI = "ssh"
	}
	var err error
	sshCLI, err = exec.LookPath(sshCLI)
	if err != nil {
		t.Skipf("Can't find an ssh(1) client to test against: %v", err)
	}
	return sshCLI
}

func TestSSHCLIAuth(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skipf("always fails on Windows, see #64403")
	}
	sshCLI := sshClient(t)
	dir := t.TempDir()
	keyPrivPath := filepath.Join(dir, "rsa")

	for fn, content := range map[string][]byte{
		keyPrivPath:                        testdata.PEMBytes["rsa"],
		keyPrivPath + ".pub":               ssh.MarshalAuthorizedKey(testPublicKeys["rsa"]),
		filepath.Join(dir, "rsa-cert.pub"): testdata.SSHCertificates["rsa-user-testcertificate"],
	} {
		if err := os.WriteFile(fn, content, 0600); err != nil {
			t.Fatalf("WriteFile(%q): %v", fn, err)
		}
	}

	certChecker := ssh.CertChecker{
		IsUserAuthority: func(k ssh.PublicKey) bool {
			return bytes.Equal(k.Marshal(), testPublicKeys["ca"].Marshal())
		},
		UserKeyFallback: func(conn ssh.ConnMetadata, key ssh.PublicKey) (*ssh.Permissions, error) {
			if conn.User() == "testpubkey" && bytes.Equal(key.Marshal(), testPublicKeys["rsa"].Marshal()) {
				return nil, nil
			}

			return nil, fmt.Errorf("pubkey for %q not acceptable", conn.User())
		},
	}

	config := &ssh.ServerConfig{
		PublicKeyCallback: certChecker.Authenticate,
	}
	config.AddHostKey(testSigners["rsa"])

	server, err := newTestServer(config)
	if err != nil {
		t.Fatalf("unable to start test server: %v", err)
	}
	defer server.Close()

	port, err := server.port()
	if err != nil {
		t.Fatalf("unable to get server port: %v", err)
	}

	// test public key authentication.
	cmd := testenv.Command(t, sshCLI, "-vvv", "-i", keyPrivPath, "-o", "StrictHostKeyChecking=no",
		"-p", port, "testpubkey@127.0.0.1", "true")
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("public key authentication failed, error: %v, command output %q", err, string(out))
	}
	// Test SSH user certificate authentication.
	// The username must match one of the principals included in the certificate.
	// The certificate "rsa-user-testcertificate" has "testcertificate" as principal.
	cmd = testenv.Command(t, sshCLI, "-vvv", "-i", keyPrivPath, "-o", "StrictHostKeyChecking=no",
		"-p", port, "testcertificate@127.0.0.1", "true")
	out, err = cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("user certificate authentication failed, error: %v, command output %q", err, string(out))
	}
}
