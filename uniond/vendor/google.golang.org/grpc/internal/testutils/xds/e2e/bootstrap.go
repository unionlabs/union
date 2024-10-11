/*
 *
 * Copyright 2020 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package e2e

import (
	"encoding/json"
	"fmt"
	"os"
	"path"
	"testing"

	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/testdata"
)

// DefaultFileWatcherConfig is a helper function to create a default certificate
// provider plugin configuration. The test is expected to have setup the files
// appropriately before this configuration is used to instantiate providers.
func DefaultFileWatcherConfig(certPath, keyPath, caPath string) json.RawMessage {
	return json.RawMessage(fmt.Sprintf(`{
			"plugin_name": "file_watcher",
			"config": {
				"certificate_file": %q,
				"private_key_file": %q,
				"ca_certificate_file": %q,
				"refresh_interval": "600s"
			}
		}`, certPath, keyPath, caPath))
}

// DefaultBootstrapContents creates a default bootstrap configuration with the
// given node ID and server URI. It also creates certificate provider
// configuration and sets the listener resource name template to be used on the
// server side.
func DefaultBootstrapContents(t *testing.T, nodeID, serverURI string) []byte {
	t.Helper()

	// Create a directory to hold certs and key files used on the server side.
	serverDir, err := createTmpDirWithCerts("testServerSideXDS*", "x509/server1_cert.pem", "x509/server1_key.pem", "x509/client_ca_cert.pem")
	if err != nil {
		t.Fatalf("Failed to create bootstrap configuration: %v", err)
	}

	// Create a directory to hold certs and key files used on the client side.
	clientDir, err := createTmpDirWithCerts("testClientSideXDS*", "x509/client1_cert.pem", "x509/client1_key.pem", "x509/server_ca_cert.pem")
	if err != nil {
		t.Fatalf("Failed to create bootstrap configuration: %v", err)
	}

	// Create certificate providers section of the bootstrap config with entries
	// for both the client and server sides.
	cpc := map[string]json.RawMessage{
		ServerSideCertProviderInstance: DefaultFileWatcherConfig(path.Join(serverDir, certFile), path.Join(serverDir, keyFile), path.Join(serverDir, rootFile)),
		ClientSideCertProviderInstance: DefaultFileWatcherConfig(path.Join(clientDir, certFile), path.Join(clientDir, keyFile), path.Join(clientDir, rootFile)),
	}

	// Create the bootstrap configuration.
	bs, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
		Servers: []byte(fmt.Sprintf(`[{
			"server_uri": "passthrough:///%s",
			"channel_creds": [{"type": "insecure"}]
		}]`, serverURI)),
		Node:                               []byte(fmt.Sprintf(`{"id": "%s"}`, nodeID)),
		CertificateProviders:               cpc,
		ServerListenerResourceNameTemplate: ServerListenerResourceNameTemplate,
	})
	if err != nil {
		t.Fatalf("Failed to create bootstrap configuration: %v", err)
	}
	return bs
}

const (
	// Names of files inside tempdir, for certprovider plugin to watch.
	certFile = "cert.pem"
	keyFile  = "key.pem"
	rootFile = "ca.pem"
)

func createTmpFile(src, dst string) error {
	data, err := os.ReadFile(src)
	if err != nil {
		return fmt.Errorf("os.ReadFile(%q) failed: %v", src, err)
	}
	if err := os.WriteFile(dst, data, os.ModePerm); err != nil {
		return fmt.Errorf("os.WriteFile(%q) failed: %v", dst, err)
	}
	return nil
}

// createTmpDirWithCerts creates a temporary directory under the system default
// tempDir with the given dirPattern. It also reads from certSrc, keySrc and
// rootSrc files and creates appropriate files under the newly create tempDir.
// Returns the path of the created tempDir if successful, and an error
// otherwise.
func createTmpDirWithCerts(dirPattern, certSrc, keySrc, rootSrc string) (string, error) {
	// Create a temp directory. Passing an empty string for the first argument
	// uses the system temp directory.
	dir, err := os.MkdirTemp("", dirPattern)
	if err != nil {
		return "", fmt.Errorf("os.MkdirTemp() failed: %v", err)
	}

	if err := createTmpFile(testdata.Path(certSrc), path.Join(dir, certFile)); err != nil {
		return "", err
	}
	if err := createTmpFile(testdata.Path(keySrc), path.Join(dir, keyFile)); err != nil {
		return "", err
	}
	if err := createTmpFile(testdata.Path(rootSrc), path.Join(dir, rootFile)); err != nil {
		return "", err
	}
	return dir, nil
}
