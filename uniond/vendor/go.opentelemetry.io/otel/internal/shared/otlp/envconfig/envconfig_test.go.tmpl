// Code created by gotmpl. DO NOT MODIFY.
// source: internal/shared/otlp/envconfig/envconfig_test.go.tmpl

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package envconfig

import (
	"crypto/tls"
	"crypto/x509"
	"errors"
	"net/url"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

const WeakKey = `
-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIEbrSPmnlSOXvVzxCyv+VR3a0HDeUTvOcqrdssZ2k4gFoAoGCCqGSM49
AwEHoUQDQgAEDMTfv75J315C3K9faptS9iythKOMEeV/Eep73nWX531YAkmmwBSB
2dXRD/brsgLnfG57WEpxZuY7dPRbxu33BA==
-----END EC PRIVATE KEY-----
`

const WeakCertificate = `
-----BEGIN CERTIFICATE-----
MIIBjjCCATWgAwIBAgIUKQSMC66MUw+kPp954ZYOcyKAQDswCgYIKoZIzj0EAwIw
EjEQMA4GA1UECgwHb3RlbC1nbzAeFw0yMjEwMTkwMDA5MTlaFw0yMzEwMTkwMDA5
MTlaMBIxEDAOBgNVBAoMB290ZWwtZ28wWTATBgcqhkjOPQIBBggqhkjOPQMBBwNC
AAQMxN+/vknfXkLcr19qm1L2LK2Eo4wR5X8R6nvedZfnfVgCSabAFIHZ1dEP9uuy
Aud8bntYSnFm5jt09FvG7fcEo2kwZzAdBgNVHQ4EFgQUicGuhnTTkYLZwofXMNLK
SHFeCWgwHwYDVR0jBBgwFoAUicGuhnTTkYLZwofXMNLKSHFeCWgwDwYDVR0TAQH/
BAUwAwEB/zAUBgNVHREEDTALgglsb2NhbGhvc3QwCgYIKoZIzj0EAwIDRwAwRAIg
Lfma8FnnxeSOi6223AsFfYwsNZ2RderNsQrS0PjEHb0CIBkrWacqARUAu7uT4cGu
jVcIxYQqhId5L8p/mAv2PWZS
-----END CERTIFICATE-----
`

type testOption struct {
	TestString   string
	TestBool     bool
	TestDuration time.Duration
	TestHeaders  map[string]string
	TestURL      *url.URL
	TestTLS      *tls.Config
}

func TestEnvConfig(t *testing.T) {
	parsedURL, err := url.Parse("https://example.com")
	assert.NoError(t, err)

	options := []testOption{}
	for _, testcase := range []struct {
		name            string
		reader          EnvOptionsReader
		configs         []ConfigFn
		expectedOptions []testOption
	}{
		{
			name: "with no namespace and a matching key",
			reader: EnvOptionsReader{
				GetEnv: func(n string) string {
					if n == "HELLO" {
						return "world"
					}
					return ""
				},
			},
			configs: []ConfigFn{
				WithString("HELLO", func(v string) {
					options = append(options, testOption{TestString: v})
				}),
			},
			expectedOptions: []testOption{
				{
					TestString: "world",
				},
			},
		},
		{
			name: "with no namespace and a non-matching key",
			reader: EnvOptionsReader{
				GetEnv: func(n string) string {
					if n == "HELLO" {
						return "world"
					}
					return ""
				},
			},
			configs: []ConfigFn{
				WithString("HOLA", func(v string) {
					options = append(options, testOption{TestString: v})
				}),
			},
			expectedOptions: []testOption{},
		},
		{
			name: "with a namespace and a matching key",
			reader: EnvOptionsReader{
				Namespace: "MY_NAMESPACE",
				GetEnv: func(n string) string {
					if n == "MY_NAMESPACE_HELLO" {
						return "world"
					}
					return ""
				},
			},
			configs: []ConfigFn{
				WithString("HELLO", func(v string) {
					options = append(options, testOption{TestString: v})
				}),
			},
			expectedOptions: []testOption{
				{
					TestString: "world",
				},
			},
		},
		{
			name: "with no namespace and a non-matching key",
			reader: EnvOptionsReader{
				Namespace: "MY_NAMESPACE",
				GetEnv: func(n string) string {
					if n == "HELLO" {
						return "world"
					}
					return ""
				},
			},
			configs: []ConfigFn{
				WithString("HELLO", func(v string) {
					options = append(options, testOption{TestString: v})
				}),
			},
			expectedOptions: []testOption{},
		},
		{
			name: "with a bool config",
			reader: EnvOptionsReader{
				GetEnv: func(n string) string {
					if n == "HELLO" {
						return "true"
					} else if n == "WORLD" {
						return "false"
					}
					return ""
				},
			},
			configs: []ConfigFn{
				WithBool("HELLO", func(b bool) {
					options = append(options, testOption{TestBool: b})
				}),
				WithBool("WORLD", func(b bool) {
					options = append(options, testOption{TestBool: b})
				}),
			},
			expectedOptions: []testOption{
				{
					TestBool: true,
				},
				{
					TestBool: false,
				},
			},
		},
		{
			name: "with an invalid bool config",
			reader: EnvOptionsReader{
				GetEnv: func(n string) string {
					if n == "HELLO" {
						return "world"
					}
					return ""
				},
			},
			configs: []ConfigFn{
				WithBool("HELLO", func(b bool) {
					options = append(options, testOption{TestBool: b})
				}),
			},
			expectedOptions: []testOption{
				{
					TestBool: false,
				},
			},
		},
		{
			name: "with a duration config",
			reader: EnvOptionsReader{
				GetEnv: func(n string) string {
					if n == "HELLO" {
						return "60"
					}
					return ""
				},
			},
			configs: []ConfigFn{
				WithDuration("HELLO", func(v time.Duration) {
					options = append(options, testOption{TestDuration: v})
				}),
			},
			expectedOptions: []testOption{
				{
					TestDuration: 60_000_000, // 60 milliseconds
				},
			},
		},
		{
			name: "with an invalid duration config",
			reader: EnvOptionsReader{
				GetEnv: func(n string) string {
					if n == "HELLO" {
						return "world"
					}
					return ""
				},
			},
			configs: []ConfigFn{
				WithDuration("HELLO", func(v time.Duration) {
					options = append(options, testOption{TestDuration: v})
				}),
			},
			expectedOptions: []testOption{},
		},
		{
			name: "with headers",
			reader: EnvOptionsReader{
				GetEnv: func(n string) string {
					if n == "HELLO" {
						return "userId=42,userName=alice"
					}
					return ""
				},
			},
			configs: []ConfigFn{
				WithHeaders("HELLO", func(v map[string]string) {
					options = append(options, testOption{TestHeaders: v})
				}),
			},
			expectedOptions: []testOption{
				{
					TestHeaders: map[string]string{
						"userId":   "42",
						"userName": "alice",
					},
				},
			},
		},
		{
			name: "with invalid headers",
			reader: EnvOptionsReader{
				GetEnv: func(n string) string {
					if n == "HELLO" {
						return "world"
					}
					return ""
				},
			},
			configs: []ConfigFn{
				WithHeaders("HELLO", func(v map[string]string) {
					options = append(options, testOption{TestHeaders: v})
				}),
			},
			expectedOptions: []testOption{
				{
					TestHeaders: map[string]string{},
				},
			},
		},
		{
			name: "with URL",
			reader: EnvOptionsReader{
				GetEnv: func(n string) string {
					if n == "HELLO" {
						return "https://example.com"
					}
					return ""
				},
			},
			configs: []ConfigFn{
				WithURL("HELLO", func(v *url.URL) {
					options = append(options, testOption{TestURL: v})
				}),
			},
			expectedOptions: []testOption{
				{
					TestURL: parsedURL,
				},
			},
		},
		{
			name: "with invalid URL",
			reader: EnvOptionsReader{
				GetEnv: func(n string) string {
					if n == "HELLO" {
						return "i nvalid://url"
					}
					return ""
				},
			},
			configs: []ConfigFn{
				WithURL("HELLO", func(v *url.URL) {
					options = append(options, testOption{TestURL: v})
				}),
			},
			expectedOptions: []testOption{},
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			testcase.reader.Apply(testcase.configs...)
			assert.Equal(t, testcase.expectedOptions, options)
			options = []testOption{}
		})
	}
}

func TestWithTLSConfig(t *testing.T) {
	pool, err := createCertPool([]byte(WeakCertificate))
	assert.NoError(t, err)

	reader := EnvOptionsReader{
		GetEnv: func(n string) string {
			if n == "CERTIFICATE" {
				return "/path/cert.pem"
			}
			return ""
		},
		ReadFile: func(p string) ([]byte, error) {
			if p == "/path/cert.pem" {
				return []byte(WeakCertificate), nil
			}
			return []byte{}, nil
		},
	}

	var option testOption
	reader.Apply(
		WithCertPool("CERTIFICATE", func(cp *x509.CertPool) {
			option = testOption{TestTLS: &tls.Config{RootCAs: cp}}
		}),
	)

	// nolint:staticcheck // ignoring tlsCert.RootCAs.Subjects is deprecated ERR because cert does not come from SystemCertPool.
	assert.Equal(t, pool.Subjects(), option.TestTLS.RootCAs.Subjects())
}

func TestWithClientCert(t *testing.T) {
	cert, err := tls.X509KeyPair([]byte(WeakCertificate), []byte(WeakKey))
	assert.NoError(t, err)

	reader := EnvOptionsReader{
		GetEnv: func(n string) string {
			switch n {
			case "CLIENT_CERTIFICATE":
				return "/path/tls.crt"
			case "CLIENT_KEY":
				return "/path/tls.key"
			}
			return ""
		},
		ReadFile: func(n string) ([]byte, error) {
			switch n {
			case "/path/tls.crt":
				return []byte(WeakCertificate), nil
			case "/path/tls.key":
				return []byte(WeakKey), nil
			}
			return []byte{}, nil
		},
	}

	var option testOption
	reader.Apply(
		WithClientCert("CLIENT_CERTIFICATE", "CLIENT_KEY", func(c tls.Certificate) {
			option = testOption{TestTLS: &tls.Config{Certificates: []tls.Certificate{c}}}
		}),
	)
	assert.Equal(t, cert, option.TestTLS.Certificates[0])

	reader.ReadFile = func(s string) ([]byte, error) { return nil, errors.New("oops") }
	option.TestTLS = nil
	reader.Apply(
		WithClientCert("CLIENT_CERTIFICATE", "CLIENT_KEY", func(c tls.Certificate) {
			option = testOption{TestTLS: &tls.Config{Certificates: []tls.Certificate{c}}}
		}),
	)
	assert.Nil(t, option.TestTLS)

	reader.GetEnv = func(s string) string { return "" }
	option.TestTLS = nil
	reader.Apply(
		WithClientCert("CLIENT_CERTIFICATE", "CLIENT_KEY", func(c tls.Certificate) {
			option = testOption{TestTLS: &tls.Config{Certificates: []tls.Certificate{c}}}
		}),
	)
	assert.Nil(t, option.TestTLS)
}

func TestStringToHeader(t *testing.T) {
	tests := []struct {
		name  string
		value string
		want  map[string]string
	}{
		{
			name:  "simple test",
			value: "userId=alice",
			want:  map[string]string{"userId": "alice"},
		},
		{
			name:  "simple test with spaces",
			value: " userId = alice  ",
			want:  map[string]string{"userId": "alice"},
		},
		{
			name:  "simple header conforms to RFC 3986 spec",
			value: " userId = alice+test ",
			want:  map[string]string{"userId": "alice+test"},
		},
		{
			name:  "multiple headers encoded",
			value: "userId=alice,serverNode=DF%3A28,isProduction=false",
			want: map[string]string{
				"userId":       "alice",
				"serverNode":   "DF:28",
				"isProduction": "false",
			},
		},
		{
			name:  "multiple headers encoded per RFC 3986 spec",
			value: "userId=alice+test,serverNode=DF%3A28,isProduction=false,namespace=localhost/test",
			want: map[string]string{
				"userId":       "alice+test",
				"serverNode":   "DF:28",
				"isProduction": "false",
				"namespace":    "localhost/test",
			},
		},
		{
			name:  "invalid headers format",
			value: "userId:alice",
			want:  map[string]string{},
		},
		{
			name:  "invalid key",
			value: "%XX=missing,userId=alice",
			want: map[string]string{
				"userId": "alice",
			},
		},
		{
			name:  "invalid value",
			value: "missing=%XX,userId=alice",
			want: map[string]string{
				"userId": "alice",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.Equal(t, tt.want, stringToHeader(tt.value))
		})
	}
}
