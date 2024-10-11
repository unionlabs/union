// Copyright 2015 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package config

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/http/httptest"
	"net/url"
	"os"
	"path/filepath"
	"reflect"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v2"
)

const (
	TLSCAChainPath        = "testdata/tls-ca-chain.pem"
	ServerCertificatePath = "testdata/server.crt"
	ServerKeyPath         = "testdata/server.key"
	ClientCertificatePath = "testdata/client.crt"
	ClientKeyNoPassPath   = "testdata/client.key"
	InvalidCA             = "testdata/client.key"
	WrongClientCertPath   = "testdata/self-signed-client.crt"
	WrongClientKeyPath    = "testdata/self-signed-client.key"
	EmptyFile             = "testdata/empty"
	MissingCA             = "missing/ca.crt"
	MissingCert           = "missing/cert.crt"
	MissingKey            = "missing/secret.key"

	ExpectedMessage                   = "I'm here to serve you!!!"
	ExpectedError                     = "expected error"
	AuthorizationCredentials          = "theanswertothegreatquestionoflifetheuniverseandeverythingisfortytwo"
	AuthorizationCredentialsFile      = "testdata/bearer.token"
	AuthorizationType                 = "APIKEY"
	BearerToken                       = AuthorizationCredentials
	BearerTokenFile                   = AuthorizationCredentialsFile
	MissingBearerTokenFile            = "missing/bearer.token"
	ExpectedBearer                    = "Bearer " + BearerToken
	ExpectedAuthenticationCredentials = AuthorizationType + " " + BearerToken
	ExpectedUsername                  = "arthurdent"
	ExpectedPassword                  = "42"
	ExpectedAccessToken               = "12345"
)

var invalidHTTPClientConfigs = []struct {
	httpClientConfigFile string
	errMsg               string
}{
	{
		httpClientConfigFile: "testdata/http.conf.bearer-token-and-file-set.bad.yml",
		errMsg:               "at most one of bearer_token & bearer_token_file must be configured",
	},
	{
		httpClientConfigFile: "testdata/http.conf.empty.bad.yml",
		errMsg:               "at most one of basic_auth, oauth2, bearer_token & bearer_token_file must be configured",
	},
	{
		httpClientConfigFile: "testdata/http.conf.basic-auth.too-much.bad.yaml",
		errMsg:               "at most one of basic_auth password, password_file & password_ref must be configured",
	},
	{
		httpClientConfigFile: "testdata/http.conf.basic-auth.bad-username.yaml",
		errMsg:               "at most one of basic_auth username, username_file & username_ref must be configured",
	},
	{
		httpClientConfigFile: "testdata/http.conf.mix-bearer-and-creds.bad.yaml",
		errMsg:               "authorization is not compatible with bearer_token & bearer_token_file",
	},
	{
		httpClientConfigFile: "testdata/http.conf.auth-creds-and-file-set.too-much.bad.yaml",
		errMsg:               "at most one of authorization credentials & credentials_file must be configured",
	},
	{
		httpClientConfigFile: "testdata/http.conf.basic-auth-and-auth-creds.too-much.bad.yaml",
		errMsg:               "at most one of basic_auth, oauth2 & authorization must be configured",
	},
	{
		httpClientConfigFile: "testdata/http.conf.basic-auth-and-oauth2.too-much.bad.yaml",
		errMsg:               "at most one of basic_auth, oauth2 & authorization must be configured",
	},
	{
		httpClientConfigFile: "testdata/http.conf.auth-creds-no-basic.bad.yaml",
		errMsg:               `authorization type cannot be set to "basic", use "basic_auth" instead`,
	},
	{
		httpClientConfigFile: "testdata/http.conf.oauth2-secret-and-file-set.bad.yml",
		errMsg:               "at most one of oauth2 client_secret, client_secret_file & client_secret_ref must be configured",
	},
	{
		httpClientConfigFile: "testdata/http.conf.oauth2-no-client-id.bad.yaml",
		errMsg:               "oauth2 client_id must be configured",
	},
	{
		httpClientConfigFile: "testdata/http.conf.oauth2-no-token-url.bad.yaml",
		errMsg:               "oauth2 token_url must be configured",
	},
	{
		httpClientConfigFile: "testdata/http.conf.proxy-from-env.bad.yaml",
		errMsg:               "if proxy_from_environment is configured, proxy_url must not be configured",
	},
	{
		httpClientConfigFile: "testdata/http.conf.no-proxy.bad.yaml",
		errMsg:               "if proxy_from_environment is configured, no_proxy must not be configured",
	},
	{
		httpClientConfigFile: "testdata/http.conf.no-proxy-without-proxy-url.bad.yaml",
		errMsg:               "if no_proxy is configured, proxy_url must also be configured",
	},
	{
		httpClientConfigFile: "testdata/http.conf.headers-reserved.bad.yaml",
		errMsg:               `setting header "User-Agent" is not allowed`,
	},
}

func newTestServer(handler func(w http.ResponseWriter, r *http.Request)) (*httptest.Server, error) {
	testServer := httptest.NewUnstartedServer(http.HandlerFunc(handler))

	tlsCAChain, err := os.ReadFile(TLSCAChainPath)
	if err != nil {
		return nil, fmt.Errorf("Can't read %s", TLSCAChainPath)
	}
	serverCertificate, err := tls.LoadX509KeyPair(ServerCertificatePath, ServerKeyPath)
	if err != nil {
		return nil, fmt.Errorf("Can't load X509 key pair %s - %s", ServerCertificatePath, ServerKeyPath)
	}

	rootCAs := x509.NewCertPool()
	rootCAs.AppendCertsFromPEM(tlsCAChain)

	testServer.TLS = &tls.Config{
		Certificates: make([]tls.Certificate, 1),
		RootCAs:      rootCAs,
		ClientAuth:   tls.RequireAndVerifyClientCert,
		ClientCAs:    rootCAs,
	}
	testServer.TLS.Certificates[0] = serverCertificate

	testServer.StartTLS()

	return testServer, nil
}

func TestNewClientFromConfig(t *testing.T) {
	newClientValidConfig := []struct {
		clientConfig HTTPClientConfig
		handler      func(w http.ResponseWriter, r *http.Request)
	}{
		{
			clientConfig: HTTPClientConfig{
				TLSConfig: TLSConfig{
					CAFile:             "",
					CertFile:           ClientCertificatePath,
					KeyFile:            ClientKeyNoPassPath,
					ServerName:         "",
					InsecureSkipVerify: true,
				},
			},
			handler: func(w http.ResponseWriter, r *http.Request) {
				fmt.Fprint(w, ExpectedMessage)
			},
		},
		{
			clientConfig: HTTPClientConfig{
				TLSConfig: TLSConfig{
					CAFile:             TLSCAChainPath,
					CertFile:           ClientCertificatePath,
					KeyFile:            ClientKeyNoPassPath,
					ServerName:         "",
					InsecureSkipVerify: false,
				},
			},
			handler: func(w http.ResponseWriter, r *http.Request) {
				fmt.Fprint(w, ExpectedMessage)
			},
		},
		{
			clientConfig: HTTPClientConfig{
				BearerToken: BearerToken,
				TLSConfig: TLSConfig{
					CAFile:             TLSCAChainPath,
					CertFile:           ClientCertificatePath,
					KeyFile:            ClientKeyNoPassPath,
					ServerName:         "",
					InsecureSkipVerify: false,
				},
			},
			handler: func(w http.ResponseWriter, r *http.Request) {
				bearer := r.Header.Get("Authorization")
				if bearer != ExpectedBearer {
					fmt.Fprintf(w, "The expected Bearer Authorization (%s) differs from the obtained Bearer Authorization (%s)",
						ExpectedBearer, bearer)
				} else {
					fmt.Fprint(w, ExpectedMessage)
				}
			},
		},
		{
			clientConfig: HTTPClientConfig{
				BearerTokenFile: BearerTokenFile,
				TLSConfig: TLSConfig{
					CAFile:             TLSCAChainPath,
					CertFile:           ClientCertificatePath,
					KeyFile:            ClientKeyNoPassPath,
					ServerName:         "",
					InsecureSkipVerify: false,
				},
			},
			handler: func(w http.ResponseWriter, r *http.Request) {
				bearer := r.Header.Get("Authorization")
				if bearer != ExpectedBearer {
					fmt.Fprintf(w, "The expected Bearer Authorization (%s) differs from the obtained Bearer Authorization (%s)",
						ExpectedBearer, bearer)
				} else {
					fmt.Fprint(w, ExpectedMessage)
				}
			},
		},
		{
			clientConfig: HTTPClientConfig{
				Authorization: &Authorization{Credentials: BearerToken},
				TLSConfig: TLSConfig{
					CAFile:             TLSCAChainPath,
					CertFile:           ClientCertificatePath,
					KeyFile:            ClientKeyNoPassPath,
					ServerName:         "",
					InsecureSkipVerify: false,
				},
			},
			handler: func(w http.ResponseWriter, r *http.Request) {
				bearer := r.Header.Get("Authorization")
				if bearer != ExpectedBearer {
					fmt.Fprintf(w, "The expected Bearer Authorization (%s) differs from the obtained Bearer Authorization (%s)",
						ExpectedBearer, bearer)
				} else {
					fmt.Fprint(w, ExpectedMessage)
				}
			},
		},
		{
			clientConfig: HTTPClientConfig{
				Authorization: &Authorization{CredentialsFile: AuthorizationCredentialsFile, Type: AuthorizationType},
				TLSConfig: TLSConfig{
					CAFile:             TLSCAChainPath,
					CertFile:           ClientCertificatePath,
					KeyFile:            ClientKeyNoPassPath,
					ServerName:         "",
					InsecureSkipVerify: false,
				},
			},
			handler: func(w http.ResponseWriter, r *http.Request) {
				bearer := r.Header.Get("Authorization")
				if bearer != ExpectedAuthenticationCredentials {
					fmt.Fprintf(w, "The expected Bearer Authorization (%s) differs from the obtained Bearer Authorization (%s)",
						ExpectedAuthenticationCredentials, bearer)
				} else {
					fmt.Fprint(w, ExpectedMessage)
				}
			},
		},
		{
			clientConfig: HTTPClientConfig{
				Authorization: &Authorization{
					Type: AuthorizationType,
				},
				TLSConfig: TLSConfig{
					CAFile:             TLSCAChainPath,
					CertFile:           ClientCertificatePath,
					KeyFile:            ClientKeyNoPassPath,
					ServerName:         "",
					InsecureSkipVerify: false,
				},
			},
			handler: func(w http.ResponseWriter, r *http.Request) {
				bearer := r.Header.Get("Authorization")
				if strings.TrimSpace(bearer) != AuthorizationType {
					fmt.Fprintf(w, "The expected Bearer Authorization (%s) differs from the obtained Bearer Authorization (%s)",
						AuthorizationType, bearer)
				} else {
					fmt.Fprint(w, ExpectedMessage)
				}
			},
		},
		{
			clientConfig: HTTPClientConfig{
				Authorization: &Authorization{
					Credentials: AuthorizationCredentials,
					Type:        AuthorizationType,
				},
				TLSConfig: TLSConfig{
					CAFile:             TLSCAChainPath,
					CertFile:           ClientCertificatePath,
					KeyFile:            ClientKeyNoPassPath,
					ServerName:         "",
					InsecureSkipVerify: false,
				},
			},
			handler: func(w http.ResponseWriter, r *http.Request) {
				bearer := r.Header.Get("Authorization")
				if bearer != ExpectedAuthenticationCredentials {
					fmt.Fprintf(w, "The expected Bearer Authorization (%s) differs from the obtained Bearer Authorization (%s)",
						ExpectedAuthenticationCredentials, bearer)
				} else {
					fmt.Fprint(w, ExpectedMessage)
				}
			},
		},
		{
			clientConfig: HTTPClientConfig{
				Authorization: &Authorization{
					CredentialsFile: BearerTokenFile,
				},
				TLSConfig: TLSConfig{
					CAFile:             TLSCAChainPath,
					CertFile:           ClientCertificatePath,
					KeyFile:            ClientKeyNoPassPath,
					ServerName:         "",
					InsecureSkipVerify: false,
				},
			},
			handler: func(w http.ResponseWriter, r *http.Request) {
				bearer := r.Header.Get("Authorization")
				if bearer != ExpectedBearer {
					fmt.Fprintf(w, "The expected Bearer Authorization (%s) differs from the obtained Bearer Authorization (%s)",
						ExpectedBearer, bearer)
				} else {
					fmt.Fprint(w, ExpectedMessage)
				}
			},
		},
		{
			clientConfig: HTTPClientConfig{
				BasicAuth: &BasicAuth{
					Username: ExpectedUsername,
					Password: ExpectedPassword,
				},
				TLSConfig: TLSConfig{
					CAFile:             TLSCAChainPath,
					CertFile:           ClientCertificatePath,
					KeyFile:            ClientKeyNoPassPath,
					ServerName:         "",
					InsecureSkipVerify: false,
				},
			},
			handler: func(w http.ResponseWriter, r *http.Request) {
				username, password, ok := r.BasicAuth()
				if !ok {
					fmt.Fprintf(w, "The Authorization header wasn't set")
				} else if ExpectedUsername != username {
					fmt.Fprintf(w, "The expected username (%s) differs from the obtained username (%s).", ExpectedUsername, username)
				} else if ExpectedPassword != password {
					fmt.Fprintf(w, "The expected password (%s) differs from the obtained password (%s).", ExpectedPassword, password)
				} else {
					fmt.Fprint(w, ExpectedMessage)
				}
			},
		},
		{
			clientConfig: HTTPClientConfig{
				FollowRedirects: true,
				TLSConfig: TLSConfig{
					CAFile:             TLSCAChainPath,
					CertFile:           ClientCertificatePath,
					KeyFile:            ClientKeyNoPassPath,
					ServerName:         "",
					InsecureSkipVerify: false,
				},
			},
			handler: func(w http.ResponseWriter, r *http.Request) {
				switch r.URL.Path {
				case "/redirected":
					fmt.Fprint(w, ExpectedMessage)
				default:
					w.Header().Set("Location", "/redirected")
					w.WriteHeader(http.StatusFound)
					fmt.Fprint(w, "It should follow the redirect.")
				}
			},
		},
		{
			clientConfig: HTTPClientConfig{
				FollowRedirects: false,
				TLSConfig: TLSConfig{
					CAFile:             TLSCAChainPath,
					CertFile:           ClientCertificatePath,
					KeyFile:            ClientKeyNoPassPath,
					ServerName:         "",
					InsecureSkipVerify: false,
				},
			},
			handler: func(w http.ResponseWriter, r *http.Request) {
				switch r.URL.Path {
				case "/redirected":
					fmt.Fprint(w, "The redirection was followed.")
				default:
					w.Header().Set("Location", "/redirected")
					w.WriteHeader(http.StatusFound)
					fmt.Fprint(w, ExpectedMessage)
				}
			},
		},
		{
			clientConfig: HTTPClientConfig{
				OAuth2: &OAuth2{
					ClientID: "ExpectedUsername",
					TLSConfig: TLSConfig{
						CAFile:             TLSCAChainPath,
						CertFile:           ClientCertificatePath,
						KeyFile:            ClientKeyNoPassPath,
						ServerName:         "",
						InsecureSkipVerify: false,
					},
				},
				TLSConfig: TLSConfig{
					CAFile:             TLSCAChainPath,
					CertFile:           ClientCertificatePath,
					KeyFile:            ClientKeyNoPassPath,
					ServerName:         "",
					InsecureSkipVerify: false,
				},
			},
			handler: func(w http.ResponseWriter, r *http.Request) {
				switch r.URL.Path {
				case "/token":
					res, _ := json.Marshal(oauth2TestServerResponse{
						AccessToken: ExpectedAccessToken,
						TokenType:   "Bearer",
					})
					w.Header().Add("Content-Type", "application/json")
					_, _ = w.Write(res)

				default:
					authorization := r.Header.Get("Authorization")
					if authorization != "Bearer "+ExpectedAccessToken {
						fmt.Fprintf(w, "Expected Authorization header %q, got %q", "Bearer "+ExpectedAccessToken, authorization)
					} else {
						fmt.Fprint(w, ExpectedMessage)
					}
				}
			},
		},
		{
			clientConfig: HTTPClientConfig{
				OAuth2: &OAuth2{
					ClientID:     "ExpectedUsername",
					ClientSecret: "ExpectedPassword",
					TLSConfig: TLSConfig{
						CAFile:             TLSCAChainPath,
						CertFile:           ClientCertificatePath,
						KeyFile:            ClientKeyNoPassPath,
						ServerName:         "",
						InsecureSkipVerify: false,
					},
				},
				TLSConfig: TLSConfig{
					CAFile:             TLSCAChainPath,
					CertFile:           ClientCertificatePath,
					KeyFile:            ClientKeyNoPassPath,
					ServerName:         "",
					InsecureSkipVerify: false,
				},
			},
			handler: func(w http.ResponseWriter, r *http.Request) {
				switch r.URL.Path {
				case "/token":
					res, _ := json.Marshal(oauth2TestServerResponse{
						AccessToken: ExpectedAccessToken,
						TokenType:   "Bearer",
					})
					w.Header().Add("Content-Type", "application/json")
					_, _ = w.Write(res)

				default:
					authorization := r.Header.Get("Authorization")
					if authorization != "Bearer "+ExpectedAccessToken {
						fmt.Fprintf(w, "Expected Authorization header %q, got %q", "Bearer "+ExpectedAccessToken, authorization)
					} else {
						fmt.Fprint(w, ExpectedMessage)
					}
				}
			},
		},
	}

	for _, validConfig := range newClientValidConfig {
		t.Run("", func(t *testing.T) {
			testServer, err := newTestServer(validConfig.handler)
			if err != nil {
				t.Fatal(err.Error())
			}
			defer testServer.Close()

			if validConfig.clientConfig.OAuth2 != nil {
				// We don't have access to the test server's URL when configuring the test cases,
				// so it has to be specified here.
				validConfig.clientConfig.OAuth2.TokenURL = testServer.URL + "/token"
			}

			err = validConfig.clientConfig.Validate()
			if err != nil {
				t.Fatal(err.Error())
			}
			client, err := NewClientFromConfig(validConfig.clientConfig, "test")
			if err != nil {
				t.Errorf("Can't create a client from this config: %+v", validConfig.clientConfig)
				return
			}

			response, err := client.Get(testServer.URL)
			if err != nil {
				t.Errorf("Can't connect to the test server using this config: %+v: %v", validConfig.clientConfig, err)
				return
			}

			message, err := io.ReadAll(response.Body)
			response.Body.Close()
			if err != nil {
				t.Errorf("Can't read the server response body using this config: %+v", validConfig.clientConfig)
				return
			}

			trimMessage := strings.TrimSpace(string(message))
			if ExpectedMessage != trimMessage {
				t.Errorf("The expected message (%s) differs from the obtained message (%s) using this config: %+v",
					ExpectedMessage, trimMessage, validConfig.clientConfig)
			}
		})
	}
}

func TestProxyConfiguration(t *testing.T) {
	testcases := map[string]struct {
		testFn  string
		loader  func(string) (*HTTPClientConfig, []byte, error)
		isValid bool
	}{
		"good yaml": {
			testFn:  "testdata/http.conf.proxy-headers.good.yml",
			loader:  LoadHTTPConfigFile,
			isValid: true,
		},
		"bad yaml": {
			testFn:  "testdata/http.conf.proxy-headers.bad.yml",
			loader:  LoadHTTPConfigFile,
			isValid: false,
		},
		"good json": {
			testFn:  "testdata/http.conf.proxy-headers.good.json",
			loader:  loadHTTPConfigJSONFile,
			isValid: true,
		},
		"bad json": {
			testFn:  "testdata/http.conf.proxy-headers.bad.json",
			loader:  loadHTTPConfigJSONFile,
			isValid: false,
		},
	}

	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			_, _, err := tc.loader(tc.testFn)
			if tc.isValid {
				if err != nil {
					t.Fatalf("Error validating %s: %s", tc.testFn, err)
				}
			} else {
				if err == nil {
					t.Fatalf("Expecting error validating %s but got %s", tc.testFn, err)
				}
			}
		})
	}
}

func TestNewClientFromInvalidConfig(t *testing.T) {
	newClientInvalidConfig := []struct {
		clientConfig HTTPClientConfig
		errorMsg     string
	}{
		{
			clientConfig: HTTPClientConfig{
				TLSConfig: TLSConfig{
					CAFile:             MissingCA,
					InsecureSkipVerify: true,
				},
			},
			errorMsg: fmt.Sprintf("unable to read CA cert: unable to read file %s", MissingCA),
		},
		{
			clientConfig: HTTPClientConfig{
				TLSConfig: TLSConfig{
					CAFile:             InvalidCA,
					InsecureSkipVerify: true,
				},
			},
			errorMsg: fmt.Sprintf("unable to use specified CA cert file %s", InvalidCA),
		},
	}

	for _, invalidConfig := range newClientInvalidConfig {
		client, err := NewClientFromConfig(invalidConfig.clientConfig, "test")
		if client != nil {
			t.Errorf("A client instance was returned instead of nil using this config: %+v", invalidConfig.clientConfig)
		}
		if err == nil {
			t.Errorf("No error was returned using this config: %+v", invalidConfig.clientConfig)
		}
		if !strings.Contains(err.Error(), invalidConfig.errorMsg) {
			t.Errorf("Expected error %q does not contain %q", err.Error(), invalidConfig.errorMsg)
		}
	}
}

func TestCustomDialContextFunc(t *testing.T) {
	dialFn := func(_ context.Context, _, _ string) (net.Conn, error) {
		return nil, errors.New(ExpectedError)
	}

	cfg := HTTPClientConfig{}
	client, err := NewClientFromConfig(cfg, "test", WithDialContextFunc(dialFn))
	if err != nil {
		t.Fatalf("Can't create a client from this config: %+v", cfg)
	}

	_, err = client.Get("http://localhost")
	if err == nil || !strings.Contains(err.Error(), ExpectedError) {
		t.Errorf("Expected error %q but got %q", ExpectedError, err)
	}
}

func TestCustomIdleConnTimeout(t *testing.T) {
	timeout := time.Second * 5

	cfg := HTTPClientConfig{}
	rt, err := NewRoundTripperFromConfig(cfg, "test", WithIdleConnTimeout(timeout))
	if err != nil {
		t.Fatalf("Can't create a round-tripper from this config: %+v", cfg)
	}

	transport, ok := rt.(*http.Transport)
	if !ok {
		t.Fatalf("Unexpected transport: %+v", transport)
	}

	if transport.IdleConnTimeout != timeout {
		t.Fatalf("Unexpected idle connection timeout: %+v", timeout)
	}
}

func TestMissingBearerAuthFile(t *testing.T) {
	cfg := HTTPClientConfig{
		BearerTokenFile: MissingBearerTokenFile,
		TLSConfig: TLSConfig{
			CAFile:             TLSCAChainPath,
			CertFile:           ClientCertificatePath,
			KeyFile:            ClientKeyNoPassPath,
			ServerName:         "",
			InsecureSkipVerify: false,
		},
	}
	handler := func(w http.ResponseWriter, r *http.Request) {
		bearer := r.Header.Get("Authorization")
		if bearer != ExpectedBearer {
			fmt.Fprintf(w, "The expected Bearer Authorization (%s) differs from the obtained Bearer Authorization (%s)",
				ExpectedBearer, bearer)
		} else {
			fmt.Fprint(w, ExpectedMessage)
		}
	}

	testServer, err := newTestServer(handler)
	if err != nil {
		t.Fatal(err.Error())
	}
	defer testServer.Close()

	client, err := NewClientFromConfig(cfg, "test")
	if err != nil {
		t.Fatal(err)
	}

	_, err = client.Get(testServer.URL)
	if err == nil {
		t.Fatal("No error is returned here")
	}

	if !strings.Contains(err.Error(), "unable to read authorization credentials: unable to read file missing/bearer.token: open missing/bearer.token: no such file or directory") {
		t.Fatal("wrong error message being returned")
	}
}

func TestBearerAuthRoundTripper(t *testing.T) {
	const (
		newBearerToken = "goodbyeandthankyouforthefish"
	)

	fakeRoundTripper := NewRoundTripCheckRequest(func(req *http.Request) {
		bearer := req.Header.Get("Authorization")
		if bearer != ExpectedBearer {
			t.Errorf("The expected Bearer Authorization (%s) differs from the obtained Bearer Authorization (%s)",
				ExpectedBearer, bearer)
		}
	}, nil, nil)

	// Normal flow.
	bearerAuthRoundTripper := NewAuthorizationCredentialsRoundTripper("Bearer", NewInlineSecret(BearerToken), fakeRoundTripper)
	request, _ := http.NewRequest("GET", "/hitchhiker", nil)
	request.Header.Set("User-Agent", "Douglas Adams mind")
	_, err := bearerAuthRoundTripper.RoundTrip(request)
	if err != nil {
		t.Errorf("unexpected error while executing RoundTrip: %s", err.Error())
	}

	// Should honor already Authorization header set.
	bearerAuthRoundTripperShouldNotModifyExistingAuthorization := NewAuthorizationCredentialsRoundTripper("Bearer", NewInlineSecret(newBearerToken), fakeRoundTripper)
	request, _ = http.NewRequest("GET", "/hitchhiker", nil)
	request.Header.Set("Authorization", ExpectedBearer)
	_, err = bearerAuthRoundTripperShouldNotModifyExistingAuthorization.RoundTrip(request)
	if err != nil {
		t.Errorf("unexpected error while executing RoundTrip: %s", err.Error())
	}
}

func TestBearerAuthFileRoundTripper(t *testing.T) {
	fakeRoundTripper := NewRoundTripCheckRequest(func(req *http.Request) {
		bearer := req.Header.Get("Authorization")
		if bearer != ExpectedBearer {
			t.Errorf("The expected Bearer Authorization (%s) differs from the obtained Bearer Authorization (%s)",
				ExpectedBearer, bearer)
		}
	}, nil, nil)

	// Normal flow.
	bearerAuthRoundTripper := NewAuthorizationCredentialsRoundTripper("Bearer", &FileSecret{file: BearerTokenFile}, fakeRoundTripper)
	request, _ := http.NewRequest("GET", "/hitchhiker", nil)
	request.Header.Set("User-Agent", "Douglas Adams mind")
	_, err := bearerAuthRoundTripper.RoundTrip(request)
	if err != nil {
		t.Errorf("unexpected error while executing RoundTrip: %s", err.Error())
	}

	// Should honor already Authorization header set.
	bearerAuthRoundTripperShouldNotModifyExistingAuthorization := NewAuthorizationCredentialsRoundTripper("Bearer", &FileSecret{file: MissingBearerTokenFile}, fakeRoundTripper)
	request, _ = http.NewRequest("GET", "/hitchhiker", nil)
	request.Header.Set("Authorization", ExpectedBearer)
	_, err = bearerAuthRoundTripperShouldNotModifyExistingAuthorization.RoundTrip(request)
	if err != nil {
		t.Errorf("unexpected error while executing RoundTrip: %s", err.Error())
	}
}

func TestTLSConfig(t *testing.T) {
	configTLSConfig := TLSConfig{
		CAFile:             TLSCAChainPath,
		CertFile:           ClientCertificatePath,
		KeyFile:            ClientKeyNoPassPath,
		ServerName:         "localhost",
		InsecureSkipVerify: false,
	}

	tlsCAChain, err := os.ReadFile(TLSCAChainPath)
	if err != nil {
		t.Fatalf("Can't read the CA certificate chain (%s)",
			TLSCAChainPath)
	}
	rootCAs := x509.NewCertPool()
	rootCAs.AppendCertsFromPEM(tlsCAChain)

	expectedTLSConfig := &tls.Config{
		RootCAs:            rootCAs,
		ServerName:         configTLSConfig.ServerName,
		InsecureSkipVerify: configTLSConfig.InsecureSkipVerify,
	}

	tlsConfig, err := NewTLSConfig(&configTLSConfig)
	if err != nil {
		t.Fatalf("Can't create a new TLS Config from a configuration (%s).", err)
	}

	clientCertificate, err := tls.LoadX509KeyPair(ClientCertificatePath, ClientKeyNoPassPath)
	if err != nil {
		t.Fatalf("Can't load the client key pair ('%s' and '%s'). Reason: %s",
			ClientCertificatePath, ClientKeyNoPassPath, err)
	}
	cert, err := tlsConfig.GetClientCertificate(nil)
	if err != nil {
		t.Fatalf("unexpected error returned by tlsConfig.GetClientCertificate(): %s", err)
	}
	if !reflect.DeepEqual(cert, &clientCertificate) {
		t.Fatalf("Unexpected client certificate result: \n\n%+v\n expected\n\n%+v", cert, clientCertificate)
	}

	// tlsConfig.rootCAs.LazyCerts contains functions getCert() in go 1.16, which are
	// never equal. Compare the Subjects instead.
	//nolint:staticcheck // Ignore SA1019. (*CertPool).Subjects is deprecated because it may not include the system certs but it isn't the case here.
	if !reflect.DeepEqual(tlsConfig.RootCAs.Subjects(), expectedTLSConfig.RootCAs.Subjects()) {
		t.Fatalf("Unexpected RootCAs result: \n\n%+v\n expected\n\n%+v", tlsConfig.RootCAs.Subjects(), expectedTLSConfig.RootCAs.Subjects())
	}
	tlsConfig.RootCAs = nil
	expectedTLSConfig.RootCAs = nil

	// Non-nil functions are never equal.
	tlsConfig.GetClientCertificate = nil

	if !reflect.DeepEqual(tlsConfig, expectedTLSConfig) {
		t.Fatalf("Unexpected TLS Config result: \n\n%+v\n expected\n\n%+v", tlsConfig, expectedTLSConfig)
	}
}

func TestTLSConfigEmpty(t *testing.T) {
	configTLSConfig := TLSConfig{
		InsecureSkipVerify: true,
	}

	expectedTLSConfig := &tls.Config{
		InsecureSkipVerify: configTLSConfig.InsecureSkipVerify,
	}

	tlsConfig, err := NewTLSConfig(&configTLSConfig)
	if err != nil {
		t.Fatalf("Can't create a new TLS Config from a configuration (%s).", err)
	}

	if !reflect.DeepEqual(tlsConfig, expectedTLSConfig) {
		t.Fatalf("Unexpected TLS Config result: \n\n%+v\n expected\n\n%+v", tlsConfig, expectedTLSConfig)
	}
}

func TestTLSConfigInvalidCA(t *testing.T) {
	invalidTLSConfig := []struct {
		configTLSConfig TLSConfig
		errorMessage    string
	}{
		{
			configTLSConfig: TLSConfig{
				CAFile:             MissingCA,
				CertFile:           "",
				KeyFile:            "",
				ServerName:         "",
				InsecureSkipVerify: false,
			},
			errorMessage: fmt.Sprintf("unable to read CA cert: unable to read file %s", MissingCA),
		},
		{
			configTLSConfig: TLSConfig{
				CAFile:             "",
				CertFile:           MissingCert,
				KeyFile:            ClientKeyNoPassPath,
				ServerName:         "",
				InsecureSkipVerify: false,
			},
			errorMessage: fmt.Sprintf("unable to read specified client cert: unable to read file %s", MissingCert),
		},
		{
			configTLSConfig: TLSConfig{
				CAFile:             "",
				CertFile:           ClientCertificatePath,
				KeyFile:            MissingKey,
				ServerName:         "",
				InsecureSkipVerify: false,
			},
			errorMessage: fmt.Sprintf("unable to read specified client key: unable to read file %s", MissingKey),
		},
		{
			configTLSConfig: TLSConfig{
				CAFile:             "",
				Cert:               readFile(t, ClientCertificatePath),
				CertFile:           ClientCertificatePath,
				KeyFile:            ClientKeyNoPassPath,
				ServerName:         "",
				InsecureSkipVerify: false,
			},
			errorMessage: "at most one of cert, cert_file & cert_ref must be configured",
		},
		{
			configTLSConfig: TLSConfig{
				CAFile:             "",
				CertFile:           ClientCertificatePath,
				Key:                Secret(readFile(t, ClientKeyNoPassPath)),
				KeyFile:            ClientKeyNoPassPath,
				ServerName:         "",
				InsecureSkipVerify: false,
			},
			errorMessage: "at most one of key and key_file must be configured",
		},
	}

	for _, anInvalididTLSConfig := range invalidTLSConfig {
		tlsConfig, err := NewTLSConfig(&anInvalididTLSConfig.configTLSConfig)
		if tlsConfig != nil && err == nil {
			t.Errorf("The TLS Config could be created even with this %+v", anInvalididTLSConfig.configTLSConfig)
			continue
		}
		if !strings.Contains(err.Error(), anInvalididTLSConfig.errorMessage) {
			t.Errorf("The expected error should contain %s, but got %s", anInvalididTLSConfig.errorMessage, err)
		}
	}
}

func TestBasicAuthNoPassword(t *testing.T) {
	cfg, _, err := LoadHTTPConfigFile("testdata/http.conf.basic-auth.no-password.yaml")
	if err != nil {
		t.Fatalf("Error loading HTTP client config: %v", err)
	}
	client, err := NewClientFromConfig(*cfg, "test")
	if err != nil {
		t.Fatalf("Error creating HTTP Client: %v", err)
	}

	rt, ok := client.Transport.(*basicAuthRoundTripper)
	if !ok {
		t.Fatalf("Error casting to basic auth transport, %v", client.Transport)
	}

	if username, _ := rt.username.Fetch(context.Background()); username != "user" {
		t.Errorf("Bad HTTP client username: %s", username)
	}
	if rt.password != nil {
		t.Errorf("Expected empty HTTP client password")
	}
}

func TestBasicAuthNoUsername(t *testing.T) {
	cfg, _, err := LoadHTTPConfigFile("testdata/http.conf.basic-auth.no-username.yaml")
	if err != nil {
		t.Fatalf("Error loading HTTP client config: %v", err)
	}
	client, err := NewClientFromConfig(*cfg, "test")
	if err != nil {
		t.Fatalf("Error creating HTTP Client: %v", err)
	}

	rt, ok := client.Transport.(*basicAuthRoundTripper)
	if !ok {
		t.Fatalf("Error casting to basic auth transport, %v", client.Transport)
	}

	if rt.username != nil {
		t.Errorf("Got unexpected username")
	}
	if password, _ := rt.password.Fetch(context.Background()); password != "secret" {
		t.Errorf("Unexpected HTTP client password: %s", password)
	}
}

func TestBasicAuthPasswordFile(t *testing.T) {
	cfg, _, err := LoadHTTPConfigFile("testdata/http.conf.basic-auth.good.yaml")
	if err != nil {
		t.Fatalf("Error loading HTTP client config: %v", err)
	}
	client, err := NewClientFromConfig(*cfg, "test")
	if err != nil {
		t.Fatalf("Error creating HTTP Client: %v", err)
	}

	rt, ok := client.Transport.(*basicAuthRoundTripper)
	if !ok {
		t.Fatalf("Error casting to basic auth transport, %v", client.Transport)
	}

	if username, _ := rt.username.Fetch(context.Background()); username != "user" {
		t.Errorf("Bad HTTP client username: %s", username)
	}
	if password, _ := rt.password.Fetch(context.Background()); password != "foobar" {
		t.Errorf("Bad HTTP client password: %s", password)
	}
}

type secretManager struct {
	data map[string]string
}

func (m *secretManager) Fetch(ctx context.Context, secretRef string) (string, error) {
	secretData, ok := m.data[secretRef]
	if !ok {
		return "", fmt.Errorf("unknown secret %s", secretRef)
	}
	return secretData, nil
}

func TestBasicAuthSecretManager(t *testing.T) {
	cfg, _, err := LoadHTTPConfigFile("testdata/http.conf.basic-auth.ref.yaml")
	if err != nil {
		t.Fatalf("Error loading HTTP client config: %v", err)
	}
	manager := secretManager{
		data: map[string]string{
			"admin": "user",
			"pass":  "foobar",
		},
	}
	client, err := NewClientFromConfig(*cfg, "test", WithSecretManager(&manager))
	if err != nil {
		t.Fatalf("Error creating HTTP Client: %v", err)
	}

	rt, ok := client.Transport.(*basicAuthRoundTripper)
	if !ok {
		t.Fatalf("Error casting to basic auth transport, %v", client.Transport)
	}

	if username, _ := rt.username.Fetch(context.Background()); username != "user" {
		t.Errorf("Bad HTTP client username: %s", username)
	}
	if password, _ := rt.password.Fetch(context.Background()); password != "foobar" {
		t.Errorf("Bad HTTP client password: %s", password)
	}
}

func TestBasicAuthSecretManagerNotFound(t *testing.T) {
	cfg, _, err := LoadHTTPConfigFile("testdata/http.conf.basic-auth.ref.yaml")
	if err != nil {
		t.Fatalf("Error loading HTTP client config: %v", err)
	}
	manager := secretManager{
		data: map[string]string{
			"admin1": "user",
			"foobar": "pass",
		},
	}
	client, err := NewClientFromConfig(*cfg, "test", WithSecretManager(&manager))
	if err != nil {
		t.Fatalf("Error creating HTTP Client: %v", err)
	}

	rt, ok := client.Transport.(*basicAuthRoundTripper)
	if !ok {
		t.Fatalf("Error casting to basic auth transport, %v", client.Transport)
	}

	if _, err := rt.username.Fetch(context.Background()); !strings.Contains(err.Error(), "unknown secret admin") {
		t.Errorf("Unexpected error message: %s", err)
	}
	if _, err := rt.password.Fetch(context.Background()); !strings.Contains(err.Error(), "unknown secret pass") {
		t.Errorf("Unexpected error message: %s", err)
	}
}

func TestBasicUsernameFile(t *testing.T) {
	cfg, _, err := LoadHTTPConfigFile("testdata/http.conf.basic-auth.username-file.good.yaml")
	if err != nil {
		t.Fatalf("Error loading HTTP client config: %v", err)
	}
	client, err := NewClientFromConfig(*cfg, "test")
	if err != nil {
		t.Fatalf("Error creating HTTP Client: %v", err)
	}

	rt, ok := client.Transport.(*basicAuthRoundTripper)
	if !ok {
		t.Fatalf("Error casting to basic auth transport, %v", client.Transport)
	}

	if username, _ := rt.username.Fetch(context.Background()); username != "testuser" {
		t.Errorf("Bad HTTP client username: %s", username)
	}
	if password, _ := rt.password.Fetch(context.Background()); password != "foobar" {
		t.Errorf("Bad HTTP client passwordFile: %s", password)
	}
}

func getCertificateBlobs(t *testing.T) map[string][]byte {
	files := []string{
		TLSCAChainPath,
		ClientCertificatePath,
		ClientKeyNoPassPath,
		ServerCertificatePath,
		ServerKeyPath,
		WrongClientCertPath,
		WrongClientKeyPath,
		EmptyFile,
	}
	bs := make(map[string][]byte, len(files)+1)
	for _, f := range files {
		b, err := os.ReadFile(f)
		if err != nil {
			t.Fatal(err)
		}
		bs[f] = b
	}

	return bs
}

func writeCertificate(bs map[string][]byte, src, dst string) {
	b, ok := bs[src]
	if !ok {
		panic(fmt.Sprintf("Couldn't find %q in bs", src))
	}
	if err := os.WriteFile(dst, b, 0o664); err != nil {
		panic(err)
	}
}

func TestTLSRoundTripper(t *testing.T) {
	bs := getCertificateBlobs(t)

	tmpDir, err := os.MkdirTemp("", "tlsroundtripper")
	if err != nil {
		t.Fatal("Failed to create tmp dir", err)
	}
	defer os.RemoveAll(tmpDir)

	ca, cert, key := filepath.Join(tmpDir, "ca"), filepath.Join(tmpDir, "cert"), filepath.Join(tmpDir, "key")

	handler := func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprint(w, ExpectedMessage)
	}
	testServer, err := newTestServer(handler)
	if err != nil {
		t.Fatal(err.Error())
	}
	defer testServer.Close()

	testCases := []struct {
		ca   string
		cert string
		key  string

		errMsg string
	}{
		{
			// Valid certs.
			ca:   TLSCAChainPath,
			cert: ClientCertificatePath,
			key:  ClientKeyNoPassPath,
		},
		{
			// CA not matching.
			ca:   ClientCertificatePath,
			cert: ClientCertificatePath,
			key:  ClientKeyNoPassPath,

			errMsg: "certificate signed by unknown authority",
		},
		{
			// Invalid client cert+key.
			ca:   TLSCAChainPath,
			cert: WrongClientCertPath,
			key:  WrongClientKeyPath,

			errMsg: "remote error: tls",
		},
		{
			// CA file empty
			ca:   EmptyFile,
			cert: ClientCertificatePath,
			key:  ClientKeyNoPassPath,

			errMsg: "unable to use specified CA cert",
		},
		{
			// cert file empty
			ca:   TLSCAChainPath,
			cert: EmptyFile,
			key:  ClientKeyNoPassPath,

			errMsg: "failed to find any PEM data in certificate input",
		},
		{
			// key file empty
			ca:   TLSCAChainPath,
			cert: ClientCertificatePath,
			key:  EmptyFile,

			errMsg: "failed to find any PEM data in key input",
		},
		{
			// Valid certs again.
			ca:   TLSCAChainPath,
			cert: ClientCertificatePath,
			key:  ClientKeyNoPassPath,
		},
	}

	cfg := HTTPClientConfig{
		TLSConfig: TLSConfig{
			CAFile:             ca,
			CertFile:           cert,
			KeyFile:            key,
			InsecureSkipVerify: false,
		},
	}

	var c *http.Client
	for i, tc := range testCases {
		tc := tc
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			writeCertificate(bs, tc.ca, ca)
			writeCertificate(bs, tc.cert, cert)
			writeCertificate(bs, tc.key, key)
			if c == nil {
				c, err = NewClientFromConfig(cfg, "test")
				if err != nil {
					t.Fatalf("Error creating HTTP Client: %v", err)
				}
			}

			req, err := http.NewRequest(http.MethodGet, testServer.URL, nil)
			if err != nil {
				t.Fatalf("Error creating HTTP request: %v", err)
			}
			r, err := c.Do(req)
			if len(tc.errMsg) > 0 {
				if err == nil {
					r.Body.Close()
					t.Fatalf("Could connect to the test server.")
				}
				if !strings.Contains(err.Error(), tc.errMsg) {
					t.Fatalf("Expected error message to contain %q, got %q", tc.errMsg, err)
				}
				return
			}

			if err != nil {
				t.Fatalf("Can't connect to the test server")
			}

			b, err := io.ReadAll(r.Body)
			r.Body.Close()
			if err != nil {
				t.Errorf("Can't read the server response body")
			}

			got := strings.TrimSpace(string(b))
			if ExpectedMessage != got {
				t.Errorf("The expected message %q differs from the obtained message %q", ExpectedMessage, got)
			}
		})
	}
}

func TestTLSRoundTripper_Inline(t *testing.T) {
	handler := func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprint(w, ExpectedMessage)
	}
	testServer, err := newTestServer(handler)
	if err != nil {
		t.Fatal(err.Error())
	}
	defer testServer.Close()

	testCases := []struct {
		caText, caFile     string
		certText, certFile string
		keyText, keyFile   string

		errMsg string
	}{
		{
			// File-based everything.
			caFile:   TLSCAChainPath,
			certFile: ClientCertificatePath,
			keyFile:  ClientKeyNoPassPath,
		},
		{
			// Inline CA.
			caText:   readFile(t, TLSCAChainPath),
			certFile: ClientCertificatePath,
			keyFile:  ClientKeyNoPassPath,
		},
		{
			// Inline cert.
			caFile:   TLSCAChainPath,
			certText: readFile(t, ClientCertificatePath),
			keyFile:  ClientKeyNoPassPath,
		},
		{
			// Inline key.
			caFile:   TLSCAChainPath,
			certFile: ClientCertificatePath,
			keyText:  readFile(t, ClientKeyNoPassPath),
		},
		{
			// Inline everything.
			caText:   readFile(t, TLSCAChainPath),
			certText: readFile(t, ClientCertificatePath),
			keyText:  readFile(t, ClientKeyNoPassPath),
		},

		{
			// Invalid inline CA.
			caText:   "badca",
			certText: readFile(t, ClientCertificatePath),
			keyText:  readFile(t, ClientKeyNoPassPath),

			errMsg: "unable to use specified CA cert inline",
		},
		{
			// Invalid cert.
			caText:   readFile(t, TLSCAChainPath),
			certText: "badcert",
			keyText:  readFile(t, ClientKeyNoPassPath),

			errMsg: "failed to find any PEM data in certificate input",
		},
		{
			// Invalid key.
			caText:   readFile(t, TLSCAChainPath),
			certText: readFile(t, ClientCertificatePath),
			keyText:  "badkey",

			errMsg: "failed to find any PEM data in key input",
		},
	}

	for i, tc := range testCases {
		tc := tc
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			cfg := HTTPClientConfig{
				TLSConfig: TLSConfig{
					CA:                 tc.caText,
					CAFile:             tc.caFile,
					Cert:               tc.certText,
					CertFile:           tc.certFile,
					Key:                Secret(tc.keyText),
					KeyFile:            tc.keyFile,
					InsecureSkipVerify: false,
				},
			}

			c, err := NewClientFromConfig(cfg, "test")
			if tc.errMsg != "" {
				if !strings.Contains(err.Error(), tc.errMsg) {
					t.Fatalf("Expected error message to contain %q, got %q", tc.errMsg, err)
				}
				return
			} else if err != nil {
				t.Fatalf("Error creating HTTP Client: %v", err)
			}

			req, err := http.NewRequest(http.MethodGet, testServer.URL, nil)
			if err != nil {
				t.Fatalf("Error creating HTTP request: %v", err)
			}
			r, err := c.Do(req)
			if err != nil {
				t.Fatalf("Can't connect to the test server")
			}

			b, err := io.ReadAll(r.Body)
			r.Body.Close()
			if err != nil {
				t.Errorf("Can't read the server response body")
			}

			got := strings.TrimSpace(string(b))
			if ExpectedMessage != got {
				t.Errorf("The expected message %q differs from the obtained message %q", ExpectedMessage, got)
			}
		})
	}
}

func TestTLSRoundTripperRaces(t *testing.T) {
	bs := getCertificateBlobs(t)

	tmpDir, err := os.MkdirTemp("", "tlsroundtripper")
	if err != nil {
		t.Fatal("Failed to create tmp dir", err)
	}
	defer os.RemoveAll(tmpDir)

	ca, cert, key := filepath.Join(tmpDir, "ca"), filepath.Join(tmpDir, "cert"), filepath.Join(tmpDir, "key")

	handler := func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprint(w, ExpectedMessage)
	}
	testServer, err := newTestServer(handler)
	if err != nil {
		t.Fatal(err.Error())
	}
	defer testServer.Close()

	cfg := HTTPClientConfig{
		TLSConfig: TLSConfig{
			CAFile:             ca,
			CertFile:           cert,
			KeyFile:            key,
			InsecureSkipVerify: false,
		},
	}

	var c *http.Client
	writeCertificate(bs, TLSCAChainPath, ca)
	writeCertificate(bs, ClientCertificatePath, cert)
	writeCertificate(bs, ClientKeyNoPassPath, key)
	c, err = NewClientFromConfig(cfg, "test")
	if err != nil {
		t.Fatalf("Error creating HTTP Client: %v", err)
	}

	var wg sync.WaitGroup
	ch := make(chan struct{})
	var total, ok int64
	// Spawn 10 Go routines polling the server concurrently.
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for {
				select {
				case <-ch:
					return
				default:
					atomic.AddInt64(&total, 1)
					r, err := c.Get(testServer.URL)
					if err == nil {
						r.Body.Close()
						atomic.AddInt64(&ok, 1)
					}
				}
			}
		}()
	}

	// Change the CA file every 10ms for 1 second.
	wg.Add(1)
	go func() {
		defer wg.Done()
		i := 0
		for {
			tick := time.NewTicker(10 * time.Millisecond)
			<-tick.C
			if i%2 == 0 {
				writeCertificate(bs, ClientCertificatePath, ca)
			} else {
				writeCertificate(bs, TLSCAChainPath, ca)
			}
			i++
			if i > 100 {
				close(ch)
				return
			}
		}
	}()

	wg.Wait()
	if ok == total {
		t.Fatalf("Expecting some requests to fail but got %d/%d successful requests", ok, total)
	}
}

func TestHideHTTPClientConfigSecrets(t *testing.T) {
	c, _, err := LoadHTTPConfigFile("testdata/http.conf.good.yml")
	if err != nil {
		t.Fatalf("Error parsing %s: %s", "testdata/http.conf.good.yml", err)
	}

	// String method must not reveal authentication credentials.
	s := c.String()
	if strings.Contains(s, "mysecret") {
		t.Fatal("http client config's String method reveals authentication credentials.")
	}
}

func TestDefaultFollowRedirect(t *testing.T) {
	cfg, _, err := LoadHTTPConfigFile("testdata/http.conf.good.yml")
	if err != nil {
		t.Errorf("Error loading HTTP client config: %v", err)
	}
	if !cfg.FollowRedirects {
		t.Errorf("follow_redirects should be true")
	}
}

func TestValidateHTTPConfig(t *testing.T) {
	cfg, _, err := LoadHTTPConfigFile("testdata/http.conf.good.yml")
	if err != nil {
		t.Errorf("Error loading HTTP client config: %v", err)
	}
	err = cfg.Validate()
	if err != nil {
		t.Fatalf("Error validating %s: %s", "testdata/http.conf.good.yml", err)
	}
}

func TestInvalidHTTPConfigs(t *testing.T) {
	for _, ee := range invalidHTTPClientConfigs {
		_, _, err := LoadHTTPConfigFile(ee.httpClientConfigFile)
		if err == nil {
			t.Errorf("Expected error with config %q but got none", ee.httpClientConfigFile)
			continue
		}
		if !strings.Contains(err.Error(), ee.errMsg) {
			t.Errorf("Expected error for invalid HTTP client configuration to contain %q but got: %s", ee.errMsg, err)
		}
	}
}

type roundTrip struct {
	theResponse *http.Response
	theError    error
}

func (rt *roundTrip) RoundTrip(r *http.Request) (*http.Response, error) {
	return rt.theResponse, rt.theError
}

type roundTripCheckRequest struct {
	checkRequest func(*http.Request)
	roundTrip
}

func (rt *roundTripCheckRequest) RoundTrip(r *http.Request) (*http.Response, error) {
	rt.checkRequest(r)
	return rt.theResponse, rt.theError
}

// NewRoundTripCheckRequest creates a new instance of a type that implements http.RoundTripper,
// which before returning theResponse and theError, executes checkRequest against a http.Request.
func NewRoundTripCheckRequest(checkRequest func(*http.Request), theResponse *http.Response, theError error) http.RoundTripper {
	return &roundTripCheckRequest{
		checkRequest: checkRequest,
		roundTrip: roundTrip{
			theResponse: theResponse,
			theError:    theError,
		},
	}
}

type oauth2TestServerResponse struct {
	AccessToken string `json:"access_token"`
	TokenType   string `json:"token_type"`
}

type testOAuthServer struct {
	tokenTS *httptest.Server
	ts      *httptest.Server
}

// newTestOAuthServer returns a new test server with the expected base64 encoded client ID and secret.
func newTestOAuthServer(t testing.TB, expectedAuth *string) testOAuthServer {
	var previousAuth string
	tokenTS := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		auth := r.Header.Get("Authorization")
		if auth != *expectedAuth {
			t.Fatalf("bad auth, expected %s, got %s", *expectedAuth, auth)
		}
		if auth == previousAuth {
			t.Fatal("token endpoint called twice")
		}
		previousAuth = auth
		res, _ := json.Marshal(oauth2TestServerResponse{
			AccessToken: "12345",
			TokenType:   "Bearer",
		})
		w.Header().Add("Content-Type", "application/json")
		_, _ = w.Write(res)
	}))
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		auth := r.Header.Get("Authorization")
		if auth != "Bearer 12345" {
			t.Fatalf("bad auth, expected %s, got %s", "Bearer 12345", auth)
		}
		fmt.Fprintln(w, "Hello, client")
	}))
	return testOAuthServer{
		tokenTS: tokenTS,
		ts:      ts,
	}
}

func (s *testOAuthServer) url() string {
	return s.ts.URL
}

func (s *testOAuthServer) tokenURL() string {
	return s.tokenTS.URL
}

func (s *testOAuthServer) close() {
	s.tokenTS.Close()
	s.ts.Close()
}

func TestOAuth2(t *testing.T) {
	var expectedAuth string
	ts := newTestOAuthServer(t, &expectedAuth)
	defer ts.close()

	yamlConfig := fmt.Sprintf(`
client_id: 1
client_secret: 2
scopes:
 - A
 - B
token_url: %s
endpoint_params:
 hi: hello
`, ts.tokenURL())
	expectedConfig := OAuth2{
		ClientID:       "1",
		ClientSecret:   "2",
		Scopes:         []string{"A", "B"},
		EndpointParams: map[string]string{"hi": "hello"},
		TokenURL:       ts.tokenURL(),
	}

	var unmarshalledConfig OAuth2
	if err := yaml.Unmarshal([]byte(yamlConfig), &unmarshalledConfig); err != nil {
		t.Fatalf("Expected no error unmarshalling yaml, got %v", err)
	}
	if !reflect.DeepEqual(unmarshalledConfig, expectedConfig) {
		t.Fatalf("Got unmarshalled config %v, expected %v", unmarshalledConfig, expectedConfig)
	}

	secret := NewInlineSecret(string(expectedConfig.ClientSecret))
	rt := NewOAuth2RoundTripper(secret, &expectedConfig, http.DefaultTransport, &defaultHTTPClientOptions)

	client := http.Client{
		Transport: rt,
	}

	// Default secret.
	expectedAuth = "Basic MToy"
	resp, err := client.Get(ts.url())
	if err != nil {
		t.Fatal(err)
	}

	authorization := resp.Request.Header.Get("Authorization")
	if authorization != "Bearer 12345" {
		t.Fatalf("Expected authorization header to be 'Bearer', got '%s'", authorization)
	}

	// Making a second request with the same secret should not re-call the token API.
	_, err = client.Get(ts.url())
	if err != nil {
		t.Fatal(err)
	}

	// Empty secret.
	expectedAuth = "Basic MTo="
	expectedConfig.ClientSecret = ""
	resp, err = client.Get(ts.url())
	if err != nil {
		t.Fatal(err)
	}

	authorization = resp.Request.Header.Get("Authorization")
	if authorization != "Bearer 12345" {
		t.Fatalf("Expected authorization header to be 'Bearer 12345', got '%s'", authorization)
	}

	// Making a second request with the same secret should not re-call the token API.
	resp, err = client.Get(ts.url())
	if err != nil {
		t.Fatal(err)
	}

	// Update secret.
	expectedAuth = "Basic MToxMjM0NTY3"
	expectedConfig.ClientSecret = "1234567"
	_, err = client.Get(ts.url())
	if err != nil {
		t.Fatal(err)
	}

	// Making a second request with the same secret should not re-call the token API.
	_, err = client.Get(ts.url())
	if err != nil {
		t.Fatal(err)
	}

	authorization = resp.Request.Header.Get("Authorization")
	if authorization != "Bearer 12345" {
		t.Fatalf("Expected authorization header to be 'Bearer 12345', got '%s'", authorization)
	}
}

func TestOAuth2UserAgent(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Header.Get("User-Agent") != "myuseragent" {
			t.Fatalf("Expected User-Agent header in oauth request to be 'myuseragent', got '%s'", r.Header.Get("User-Agent"))
		}

		res, _ := json.Marshal(oauth2TestServerResponse{
			AccessToken: "12345",
			TokenType:   "Bearer",
		})
		w.Header().Add("Content-Type", "application/json")
		_, _ = w.Write(res)
	}))
	defer ts.Close()

	config := DefaultHTTPClientConfig
	config.OAuth2 = &OAuth2{
		ClientID:       "1",
		ClientSecret:   "2",
		Scopes:         []string{"A", "B"},
		EndpointParams: map[string]string{"hi": "hello"},
		TokenURL:       fmt.Sprintf("%s/token", ts.URL),
	}

	rt, err := NewRoundTripperFromConfig(config, "test_oauth2", WithUserAgent("myuseragent"))
	if err != nil {
		t.Fatal(err)
	}

	client := http.Client{
		Transport: rt,
	}
	resp, err := client.Get(ts.URL)
	if err != nil {
		t.Fatal(err)
	}

	authorization := resp.Request.Header.Get("Authorization")
	if authorization != "Bearer 12345" {
		t.Fatalf("Expected authorization header to be 'Bearer 12345', got '%s'", authorization)
	}
}

func TestHost(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Host != "localhost.localdomain" {
			t.Fatalf("Expected Host header in request to be 'localhost.localdomain', got '%s'", r.Host)
		}

		w.Header().Add("Content-Type", "application/json")
	}))
	defer ts.Close()

	config := DefaultHTTPClientConfig

	rt, err := NewRoundTripperFromConfig(config, "test_host", WithHost("localhost.localdomain"))
	if err != nil {
		t.Fatal(err)
	}

	client := http.Client{
		Transport: rt,
	}
	_, err = client.Get(ts.URL)
	if err != nil {
		t.Fatal(err)
	}
}

func TestOAuth2WithFile(t *testing.T) {
	var expectedAuth string
	ts := newTestOAuthServer(t, &expectedAuth)
	defer ts.close()

	secretFile, err := os.CreateTemp("", "oauth2_secret")
	if err != nil {
		t.Fatal(err)
	}
	defer os.Remove(secretFile.Name())

	yamlConfig := fmt.Sprintf(`
client_id: 1
client_secret_file: %s
scopes:
 - A
 - B
token_url: %s
endpoint_params:
 hi: hello
`, secretFile.Name(), ts.tokenURL())
	expectedConfig := OAuth2{
		ClientID:         "1",
		ClientSecretFile: secretFile.Name(),
		Scopes:           []string{"A", "B"},
		EndpointParams:   map[string]string{"hi": "hello"},
		TokenURL:         ts.tokenURL(),
	}

	var unmarshalledConfig OAuth2
	err = yaml.Unmarshal([]byte(yamlConfig), &unmarshalledConfig)
	if err != nil {
		t.Fatalf("Expected no error unmarshalling yaml, got %v", err)
	}
	if !reflect.DeepEqual(unmarshalledConfig, expectedConfig) {
		t.Fatalf("Got unmarshalled config %v, expected %v", unmarshalledConfig, expectedConfig)
	}

	secret := NewFileSecret(expectedConfig.ClientSecretFile)
	rt := NewOAuth2RoundTripper(secret, &expectedConfig, http.DefaultTransport, &defaultHTTPClientOptions)

	client := http.Client{
		Transport: rt,
	}

	// Empty secret file.
	expectedAuth = "Basic MTo="
	resp, err := client.Get(ts.url())
	if err != nil {
		t.Fatal(err)
	}

	authorization := resp.Request.Header.Get("Authorization")
	if authorization != "Bearer 12345" {
		t.Fatalf("Expected authorization header to be 'Bearer', got '%s'", authorization)
	}

	// Making a second request with the same file content should not re-call the token API.
	_, err = client.Get(ts.url())
	if err != nil {
		t.Fatal(err)
	}

	// File populated.
	expectedAuth = "Basic MToxMjM0NTY="
	if _, err := secretFile.Write([]byte("123456")); err != nil {
		t.Fatal(err)
	}
	resp, err = client.Get(ts.url())
	if err != nil {
		t.Fatal(err)
	}

	authorization = resp.Request.Header.Get("Authorization")
	if authorization != "Bearer 12345" {
		t.Fatalf("Expected authorization header to be 'Bearer 12345', got '%s'", authorization)
	}

	// Making a second request with the same file content should not re-call the token API.
	resp, err = client.Get(ts.url())
	if err != nil {
		t.Fatal(err)
	}

	// Update file.
	expectedAuth = "Basic MToxMjM0NTY3"
	if _, err := secretFile.Write([]byte("7")); err != nil {
		t.Fatal(err)
	}
	_, err = client.Get(ts.url())
	if err != nil {
		t.Fatal(err)
	}

	// Making a second request with the same file content should not re-call the token API.
	_, err = client.Get(ts.url())
	if err != nil {
		t.Fatal(err)
	}

	authorization = resp.Request.Header.Get("Authorization")
	if authorization != "Bearer 12345" {
		t.Fatalf("Expected authorization header to be 'Bearer 12345', got '%s'", authorization)
	}
}

func TestMarshalURL(t *testing.T) {
	urlp, err := url.Parse("http://example.com/")
	if err != nil {
		t.Fatal(err)
	}
	u := &URL{urlp}

	c, err := json.Marshal(u)
	if err != nil {
		t.Fatal(err)
	}
	if string(c) != "\"http://example.com/\"" {
		t.Fatalf("URL not properly marshaled in JSON got '%s'", string(c))
	}

	c, err = yaml.Marshal(u)
	if err != nil {
		t.Fatal(err)
	}
	if string(c) != "http://example.com/\n" {
		t.Fatalf("URL not properly marshaled in YAML got '%s'", string(c))
	}
}

func TestMarshalURLWrapperWithNilValue(t *testing.T) {
	u := &URL{}

	c, err := json.Marshal(u)
	if err != nil {
		t.Fatal(err)
	}
	if string(c) != "null" {
		t.Fatalf("URL with nil value not properly marshaled into JSON, got %q", c)
	}

	c, err = yaml.Marshal(u)
	if err != nil {
		t.Fatal(err)
	}
	if string(c) != "null\n" {
		t.Fatalf("URL with nil value not properly marshaled into JSON, got %q", c)
	}
}

func TestUnmarshalNullURL(t *testing.T) {
	b := []byte(`null`)

	{
		var u URL
		err := json.Unmarshal(b, &u)
		if err != nil {
			t.Fatal(err)
		}
		if !isEmptyNonNilURL(u.URL) {
			t.Fatalf("`null` literal not properly unmarshaled from JSON as URL, got %#v", u.URL)
		}
	}

	{
		var u URL
		err := yaml.Unmarshal(b, &u)
		if err != nil {
			t.Fatal(err)
		}
		if u.URL != nil { // UnmarshalYAML is not called when parsing null literal.
			t.Fatalf("`null` literal not properly unmarshaled from YAML as URL, got %#v", u.URL)
		}
	}
}

func TestUnmarshalEmptyURL(t *testing.T) {
	b := []byte(`""`)

	{
		var u URL
		err := json.Unmarshal(b, &u)
		if err != nil {
			t.Fatal(err)
		}
		if !isEmptyNonNilURL(u.URL) {
			t.Fatalf("empty string not properly unmarshaled from JSON as URL, got %#v", u.URL)
		}
	}

	{
		var u URL
		err := yaml.Unmarshal(b, &u)
		if err != nil {
			t.Fatal(err)
		}
		if !isEmptyNonNilURL(u.URL) {
			t.Fatalf("empty string not properly unmarshaled from YAML as URL, got %#v", u.URL)
		}
	}
}

// checks if u equals to &url.URL{}
func isEmptyNonNilURL(u *url.URL) bool {
	return u != nil && *u == url.URL{}
}

func TestUnmarshalURL(t *testing.T) {
	b := []byte(`"http://example.com/a b"`)
	var u URL

	err := json.Unmarshal(b, &u)
	if err != nil {
		t.Fatal(err)
	}
	if u.String() != "http://example.com/a%20b" {
		t.Fatalf("URL not properly unmarshaled in JSON, got '%s'", u.String())
	}

	err = yaml.Unmarshal(b, &u)
	if err != nil {
		t.Fatal(err)
	}
	if u.String() != "http://example.com/a%20b" {
		t.Fatalf("URL not properly unmarshaled in YAML, got '%s'", u.String())
	}
}

func TestMarshalURLWithSecret(t *testing.T) {
	var u URL
	err := yaml.Unmarshal([]byte("http://foo:bar@example.com"), &u)
	if err != nil {
		t.Fatal(err)
	}

	b, err := yaml.Marshal(u)
	if err != nil {
		t.Fatal(err)
	}
	if strings.TrimSpace(string(b)) != "http://foo:xxxxx@example.com" {
		t.Fatalf("URL not properly marshaled in YAML, got '%s'", string(b))
	}
}

func TestHTTPClientConfig_Marshal(t *testing.T) {
	proxyURL, err := url.Parse("http://localhost:8080")
	require.NoError(t, err)

	t.Run("without HTTP headers", func(t *testing.T) {
		config := &HTTPClientConfig{
			ProxyConfig: ProxyConfig{
				ProxyURL: URL{proxyURL},
			},
		}

		t.Run("YAML", func(t *testing.T) {
			actualYAML, err := yaml.Marshal(config)
			require.NoError(t, err)
			require.YAMLEq(t, `
proxy_url: "http://localhost:8080"
follow_redirects: false
enable_http2: false
`, string(actualYAML))

			// Unmarshalling the YAML should get the same struct in input.
			actual := &HTTPClientConfig{}
			require.NoError(t, yaml.Unmarshal(actualYAML, actual))
			require.Equal(t, config, actual)
		})

		t.Run("JSON", func(t *testing.T) {
			actualJSON, err := json.Marshal(config)

			require.NoError(t, err)
			require.JSONEq(t, `{
				"proxy_url":"http://localhost:8080",
				"tls_config":{"insecure_skip_verify":false},
				"follow_redirects":false,
				"enable_http2":false
			}`, string(actualJSON))

			// Unmarshalling the JSON should get the same struct in input.
			actual := &HTTPClientConfig{}
			require.NoError(t, json.Unmarshal(actualJSON, actual))
			require.Equal(t, config, actual)
		})
	})

	t.Run("with HTTP headers", func(t *testing.T) {
		config := &HTTPClientConfig{
			ProxyConfig: ProxyConfig{
				ProxyURL: URL{proxyURL},
			},
			HTTPHeaders: &Headers{
				Headers: map[string]Header{
					"X-Test": {
						Values: []string{"Value-1", "Value-2"},
					},
				},
			},
		}

		actualYAML, err := yaml.Marshal(config)
		require.NoError(t, err)
		require.YAMLEq(t, `
proxy_url: "http://localhost:8080"
follow_redirects: false
enable_http2: false
http_headers:
  X-Test:
    values:
    - Value-1
    - Value-2
`, string(actualYAML))

		actualJSON, err := json.Marshal(config)
		require.NoError(t, err)
		require.JSONEq(t, `{
			"proxy_url":"http://localhost:8080",
			"tls_config":{"insecure_skip_verify":false},
			"follow_redirects":false,
			"enable_http2":false,
			"http_headers":{"X-Test":{"values":["Value-1","Value-2"]}}
		}`, string(actualJSON))
	})
}

func TestOAuth2Proxy(t *testing.T) {
	_, _, err := LoadHTTPConfigFile("testdata/http.conf.oauth2-proxy.good.yml")
	if err != nil {
		t.Errorf("Error loading OAuth2 client config: %v", err)
	}
}

func TestModifyTLSCertificates(t *testing.T) {
	bs := getCertificateBlobs(t)

	tmpDir, err := os.MkdirTemp("", "modifytlscertificates")
	if err != nil {
		t.Fatal("Failed to create tmp dir", err)
	}
	defer os.RemoveAll(tmpDir)
	ca, cert, key := filepath.Join(tmpDir, "ca"), filepath.Join(tmpDir, "cert"), filepath.Join(tmpDir, "key")

	handler := func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprint(w, ExpectedMessage)
	}
	testServer, err := newTestServer(handler)
	if err != nil {
		t.Fatal(err.Error())
	}
	defer testServer.Close()

	tests := []struct {
		ca   string
		cert string
		key  string

		errMsg string

		modification func()
	}{
		{
			ca:   ClientCertificatePath,
			cert: ClientCertificatePath,
			key:  ClientKeyNoPassPath,

			errMsg: "certificate signed by unknown authority",

			modification: func() { writeCertificate(bs, TLSCAChainPath, ca) },
		},
		{
			ca:   TLSCAChainPath,
			cert: WrongClientCertPath,
			key:  ClientKeyNoPassPath,

			errMsg: "private key does not match public key",

			modification: func() { writeCertificate(bs, ClientCertificatePath, cert) },
		},
		{
			ca:   TLSCAChainPath,
			cert: ClientCertificatePath,
			key:  WrongClientCertPath,

			errMsg: "found a certificate rather than a key in the PEM for the private key",

			modification: func() { writeCertificate(bs, ClientKeyNoPassPath, key) },
		},
	}

	cfg := HTTPClientConfig{
		TLSConfig: TLSConfig{
			CAFile:             ca,
			CertFile:           cert,
			KeyFile:            key,
			InsecureSkipVerify: false,
		},
	}

	var c *http.Client
	for i, tc := range tests {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			writeCertificate(bs, tc.ca, ca)
			writeCertificate(bs, tc.cert, cert)
			writeCertificate(bs, tc.key, key)
			if c == nil {
				c, err = NewClientFromConfig(cfg, "test")
				if err != nil {
					t.Fatalf("Error creating HTTP Client: %v", err)
				}
			}

			req, err := http.NewRequest(http.MethodGet, testServer.URL, nil)
			if err != nil {
				t.Fatalf("Error creating HTTP request: %v", err)
			}

			r, err := c.Do(req)
			if err == nil {
				r.Body.Close()
				t.Fatalf("Could connect to the test server.")
			}
			if !strings.Contains(err.Error(), tc.errMsg) {
				t.Fatalf("Expected error message to contain %q, got %q", tc.errMsg, err)
			}

			tc.modification()

			r, err = c.Do(req)
			if err != nil {
				t.Fatalf("Expected no error, got %q", err)
			}

			b, err := io.ReadAll(r.Body)
			r.Body.Close()
			if err != nil {
				t.Errorf("Can't read the server response body")
			}

			got := strings.TrimSpace(string(b))
			if ExpectedMessage != got {
				t.Errorf("The expected message %q differs from the obtained message %q", ExpectedMessage, got)
			}
		})
	}
}

// loadHTTPConfigJSON parses the JSON input s into a HTTPClientConfig.
func loadHTTPConfigJSON(buf []byte) (*HTTPClientConfig, error) {
	cfg := &HTTPClientConfig{}
	err := json.Unmarshal(buf, cfg)
	if err != nil {
		return nil, err
	}
	return cfg, nil
}

// loadHTTPConfigJSONFile parses the given JSON file into a HTTPClientConfig.
func loadHTTPConfigJSONFile(filename string) (*HTTPClientConfig, []byte, error) {
	content, err := os.ReadFile(filename)
	if err != nil {
		return nil, nil, err
	}
	cfg, err := loadHTTPConfigJSON(content)
	if err != nil {
		return nil, nil, err
	}
	return cfg, content, nil
}

func TestProxyConfig_Proxy(t *testing.T) {
	var proxyServer *httptest.Server

	defer func() {
		if proxyServer != nil {
			proxyServer.Close()
		}
	}()

	proxyServerHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintf(w, "Hello, %s", r.URL.Path)
	})

	proxyServer = httptest.NewServer(proxyServerHandler)

	testCases := []struct {
		name             string
		proxyConfig      string
		expectedProxyURL string
		targetURL        string
		proxyEnv         string
		noProxyEnv       string
	}{
		{
			name:             "proxy from environment",
			proxyConfig:      `proxy_from_environment: true`,
			expectedProxyURL: proxyServer.URL,
			proxyEnv:         proxyServer.URL,
			targetURL:        "http://prometheus.io/",
		},
		{
			name:             "proxy_from_environment with no_proxy",
			proxyConfig:      `proxy_from_environment: true`,
			expectedProxyURL: "",
			proxyEnv:         proxyServer.URL,
			noProxyEnv:       "prometheus.io",
			targetURL:        "http://prometheus.io/",
		},
		{
			name:             "proxy_from_environment and localhost",
			proxyConfig:      `proxy_from_environment: true`,
			expectedProxyURL: "",
			proxyEnv:         proxyServer.URL,
			targetURL:        "http://localhost/",
		},
		{
			name:             "valid proxy_url and localhost",
			proxyConfig:      fmt.Sprintf(`proxy_url: %s`, proxyServer.URL),
			expectedProxyURL: proxyServer.URL,
			targetURL:        "http://localhost/",
		},
		{
			name: "valid proxy_url and no_proxy and localhost",
			proxyConfig: fmt.Sprintf(`proxy_url: %s
no_proxy: prometheus.io`, proxyServer.URL),
			expectedProxyURL: "",
			targetURL:        "http://localhost/",
		},
		{
			name:             "valid proxy_url",
			proxyConfig:      fmt.Sprintf(`proxy_url: %s`, proxyServer.URL),
			expectedProxyURL: proxyServer.URL,
			targetURL:        "http://prometheus.io/",
		},
		{
			name: "valid proxy url and no_proxy",
			proxyConfig: fmt.Sprintf(`proxy_url: %s
no_proxy: prometheus.io`, proxyServer.URL),
			expectedProxyURL: "",
			targetURL:        "http://prometheus.io/",
		},
		{
			name: "valid proxy url and no_proxies",
			proxyConfig: fmt.Sprintf(`proxy_url: %s
no_proxy: promcon.io,prometheus.io,cncf.io`, proxyServer.URL),
			expectedProxyURL: "",
			targetURL:        "http://prometheus.io/",
		},
		{
			name: "valid proxy url and no_proxies that do not include target",
			proxyConfig: fmt.Sprintf(`proxy_url: %s
no_proxy: promcon.io,cncf.io`, proxyServer.URL),
			expectedProxyURL: proxyServer.URL,
			targetURL:        "http://prometheus.io/",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			if proxyServer != nil {
				defer proxyServer.Close()
			}

			var proxyConfig ProxyConfig

			err := yaml.Unmarshal([]byte(tc.proxyConfig), &proxyConfig)
			if err != nil {
				t.Errorf("failed to unmarshal proxy config: %v", err)
				return
			}

			if tc.proxyEnv != "" {
				currentProxy := os.Getenv("HTTP_PROXY")
				t.Cleanup(func() { os.Setenv("HTTP_PROXY", currentProxy) })
				os.Setenv("HTTP_PROXY", tc.proxyEnv)
			}

			if tc.noProxyEnv != "" {
				currentProxy := os.Getenv("NO_PROXY")
				t.Cleanup(func() { os.Setenv("NO_PROXY", currentProxy) })
				os.Setenv("NO_PROXY", tc.noProxyEnv)
			}

			req := httptest.NewRequest("GET", tc.targetURL, nil)

			proxyFunc := proxyConfig.Proxy()
			resultURL, err := proxyFunc(req)
			if err != nil {
				t.Fatalf("expected no error, but got: %v", err)
				return
			}
			if tc.expectedProxyURL == "" && resultURL != nil {
				t.Fatalf("expected no result URL, but got: %s", resultURL.String())
				return
			}
			if tc.expectedProxyURL != "" && resultURL == nil {
				t.Fatalf("expected result URL, but got nil")
				return
			}
			if tc.expectedProxyURL != "" && resultURL.String() != tc.expectedProxyURL {
				t.Fatalf("expected result URL: %s, but got: %s", tc.expectedProxyURL, resultURL.String())
			}
		})
	}
}

func readFile(t *testing.T, filename string) string {
	t.Helper()

	content, err := os.ReadFile(filename)
	if err != nil {
		t.Fatalf("Failed to read file %q: %s", filename, err)
	}

	return string(content)
}

func TestHeaders(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		for k, v := range map[string]string{
			"One":   "value1",
			"Two":   "value2",
			"Three": "value3",
		} {
			if r.Header.Get(k) != v {
				t.Errorf("expected %q, got %q", v, r.Header.Get(k))
			}
		}
		w.WriteHeader(http.StatusNoContent)
	}))
	t.Cleanup(ts.Close)

	cfg, _, err := LoadHTTPConfigFile("testdata/http.conf.headers.good.yaml")
	if err != nil {
		t.Fatalf("Error loading HTTP client config: %v", err)
	}
	client, err := NewClientFromConfig(*cfg, "test")
	if err != nil {
		t.Fatalf("Error creating HTTP Client: %v", err)
	}

	_, err = client.Get(ts.URL)
	if err != nil {
		t.Fatalf("can't fetch URL: %v", err)
	}
}

func TestMultipleHeaders(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		for k, v := range map[string][]string{
			"One":   {"value1a", "value1b", "value1c"},
			"Two":   {"value2a", "value2b", "value2c"},
			"Three": {"value3a", "value3b", "value3c"},
		} {
			if !reflect.DeepEqual(r.Header.Values(k), v) {
				t.Errorf("expected %v, got %v", v, r.Header.Values(k))
			}
		}
		w.WriteHeader(http.StatusNoContent)
	}))
	t.Cleanup(ts.Close)

	cfg, _, err := LoadHTTPConfigFile("testdata/http.conf.headers-multiple.good.yaml")
	if err != nil {
		t.Fatalf("Error loading HTTP client config: %v", err)
	}
	client, err := NewClientFromConfig(*cfg, "test")
	if err != nil {
		t.Fatalf("Error creating HTTP Client: %v", err)
	}

	_, err = client.Get(ts.URL)
	if err != nil {
		t.Fatalf("can't fetch URL: %v", err)
	}
}
