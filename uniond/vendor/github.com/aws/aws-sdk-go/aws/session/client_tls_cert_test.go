//go:build go1.9
// +build go1.9

package session

import (
	"crypto/x509"
	"io"
	"net/http"
	"os"
	"strings"
	"testing"
	"time"

	"github.com/aws/aws-sdk-go/awstesting"
)

func TestNewSession_WithClientTLSCert(t *testing.T) {
	type testCase struct {
		// Params
		setup     func(certFilename, keyFilename string) (Options, func(), error)
		ExpectErr string
	}

	cases := map[string]testCase{
		"env": {
			setup: func(certFilename, keyFilename string) (Options, func(), error) {
				os.Setenv(useClientTLSCert[0], certFilename)
				os.Setenv(useClientTLSKey[0], keyFilename)
				return Options{}, func() {}, nil
			},
		},
		"env file not found": {
			setup: func(certFilename, keyFilename string) (Options, func(), error) {
				os.Setenv(useClientTLSCert[0], "some-cert-file-not-exists")
				os.Setenv(useClientTLSKey[0], "some-key-file-not-exists")
				return Options{}, func() {}, nil
			},
			ExpectErr: "LoadClientTLSCertError",
		},
		"env cert file only": {
			setup: func(certFilename, keyFilename string) (Options, func(), error) {
				os.Setenv(useClientTLSCert[0], certFilename)
				return Options{}, func() {}, nil
			},
			ExpectErr: "must both be provided",
		},
		"env key file only": {
			setup: func(certFilename, keyFilename string) (Options, func(), error) {
				os.Setenv(useClientTLSKey[0], keyFilename)
				return Options{}, func() {}, nil
			},
			ExpectErr: "must both be provided",
		},

		"session options": {
			setup: func(certFilename, keyFilename string) (Options, func(), error) {
				certFile, err := os.Open(certFilename)
				if err != nil {
					return Options{}, nil, err
				}
				keyFile, err := os.Open(keyFilename)
				if err != nil {
					return Options{}, nil, err
				}

				return Options{
						ClientTLSCert: certFile,
						ClientTLSKey:  keyFile,
					}, func() {
						certFile.Close()
						keyFile.Close()
					}, nil
			},
		},
		"session cert load error": {
			setup: func(certFilename, keyFilename string) (Options, func(), error) {
				certFile, err := os.Open(certFilename)
				if err != nil {
					return Options{}, nil, err
				}
				keyFile, err := os.Open(keyFilename)
				if err != nil {
					return Options{}, nil, err
				}

				stat, _ := certFile.Stat()
				return Options{
						ClientTLSCert: io.LimitReader(certFile, stat.Size()/2),
						ClientTLSKey:  keyFile,
					}, func() {
						certFile.Close()
						keyFile.Close()
					}, nil
			},
			ExpectErr: "unable to load x509 key pair",
		},
		"session key load error": {
			setup: func(certFilename, keyFilename string) (Options, func(), error) {
				certFile, err := os.Open(certFilename)
				if err != nil {
					return Options{}, nil, err
				}
				keyFile, err := os.Open(keyFilename)
				if err != nil {
					return Options{}, nil, err
				}

				stat, _ := keyFile.Stat()
				return Options{
						ClientTLSCert: certFile,
						ClientTLSKey:  io.LimitReader(keyFile, stat.Size()/2),
					}, func() {
						certFile.Close()
						keyFile.Close()
					}, nil
			},
			ExpectErr: "unable to load x509 key pair",
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			// Asserts
			restoreEnvFn := initSessionTestEnv()
			defer restoreEnvFn()

			certFilename, keyFilename, err := awstesting.CreateClientTLSCertFiles()
			if err != nil {
				t.Fatalf("failed to create client certificate files, %v", err)
			}
			defer func() {
				if err := awstesting.CleanupTLSBundleFiles(certFilename, keyFilename); err != nil {
					t.Errorf("failed to cleanup client TLS cert files, %v", err)
				}
			}()

			opts, cleanup, err := c.setup(certFilename, keyFilename)
			if err != nil {
				t.Fatalf("test case failed setup, %v", err)
			}
			if cleanup != nil {
				defer cleanup()
			}

			server, err := awstesting.NewTLSClientCertServer(http.HandlerFunc(
				func(w http.ResponseWriter, r *http.Request) {
					w.WriteHeader(200)
				}))
			if err != nil {
				t.Fatalf("failed to load session, %v", err)
			}
			server.StartTLS()
			defer server.Close()

			// Give server change to start
			time.Sleep(time.Second)

			// Load SDK session with options configured.
			sess, err := NewSessionWithOptions(opts)
			if len(c.ExpectErr) != 0 {
				if err == nil {
					t.Fatalf("expect error, got none")
				}
				if e, a := c.ExpectErr, err.Error(); !strings.Contains(a, e) {
					t.Fatalf("expect error to contain %v, got %v", e, a)
				}
				return
			}
			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}

			// Clients need to add ca bundle for test service.
			p := x509.NewCertPool()
			p.AddCert(server.Certificate())
			client := sess.Config.HTTPClient
			client.Transport.(*http.Transport).TLSClientConfig.RootCAs = p

			// Send request
			req, _ := http.NewRequest("GET", server.URL, nil)
			resp, err := client.Do(req)
			if err != nil {
				t.Fatalf("failed to send request, %v", err)
			}

			if e, a := 200, resp.StatusCode; e != a {
				t.Errorf("expect %v status code, got %v", e, a)
			}
		})
	}
}
