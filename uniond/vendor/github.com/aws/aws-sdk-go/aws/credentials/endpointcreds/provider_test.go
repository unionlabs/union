package endpointcreds_test

import (
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/aws/aws-sdk-go/aws/awserr"
	"github.com/aws/aws-sdk-go/aws/credentials/endpointcreds"
	"github.com/aws/aws-sdk-go/awstesting/unit"
)

func TestRetrieveRefreshableCredentials(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if e, a := "/path/to/endpoint", r.URL.Path; e != a {
			t.Errorf("expect %v, got %v", e, a)
		}
		if e, a := "application/json", r.Header.Get("Accept"); e != a {
			t.Errorf("expect %v, got %v", e, a)
		}
		if e, a := "else", r.URL.Query().Get("something"); e != a {
			t.Errorf("expect %v, got %v", e, a)
		}

		encoder := json.NewEncoder(w)
		err := encoder.Encode(map[string]interface{}{
			"AccessKeyID":     "AKID",
			"SecretAccessKey": "SECRET",
			"Token":           "TOKEN",
			"Expiration":      time.Now().Add(1 * time.Hour),
		})

		if err != nil {
			fmt.Println("failed to write out creds", err)
		}
	}))
	defer server.Close()

	client := endpointcreds.NewProviderClient(*unit.Session.Config,
		unit.Session.Handlers,
		server.URL+"/path/to/endpoint?something=else",
	)
	creds, err := client.Retrieve()

	if err != nil {
		t.Errorf("expect no error, got %v", err)
	}

	if e, a := "AKID", creds.AccessKeyID; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "SECRET", creds.SecretAccessKey; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "TOKEN", creds.SessionToken; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if client.IsExpired() {
		t.Errorf("expect not expired, was")
	}

	client.(*endpointcreds.Provider).CurrentTime = func() time.Time {
		return time.Now().Add(2 * time.Hour)
	}

	if !client.IsExpired() {
		t.Errorf("expect expired, wasn't")
	}
}

func TestRetrieveStaticCredentials(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		encoder := json.NewEncoder(w)
		err := encoder.Encode(map[string]interface{}{
			"AccessKeyID":     "AKID",
			"SecretAccessKey": "SECRET",
		})

		if err != nil {
			fmt.Println("failed to write out creds", err)
		}
	}))
	defer server.Close()

	client := endpointcreds.NewProviderClient(*unit.Session.Config, unit.Session.Handlers, server.URL)
	creds, err := client.Retrieve()

	if err != nil {
		t.Errorf("expect no error, got %v", err)
	}

	if e, a := "AKID", creds.AccessKeyID; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "SECRET", creds.SecretAccessKey; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if v := creds.SessionToken; len(v) != 0 {
		t.Errorf("Expect no SessionToken, got %#v", v)
	}
	if client.IsExpired() {
		t.Errorf("expect not expired, was")
	}
}

func TestFailedRetrieveCredentials(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(400)
		encoder := json.NewEncoder(w)
		err := encoder.Encode(map[string]interface{}{
			"Code":    "Error",
			"Message": "Message",
		})

		if err != nil {
			fmt.Println("failed to write error", err)
		}
	}))
	defer server.Close()

	client := endpointcreds.NewProviderClient(*unit.Session.Config, unit.Session.Handlers, server.URL)
	creds, err := client.Retrieve()

	if err == nil {
		t.Errorf("expect error, got none")
	}
	aerr := err.(awserr.Error)

	if e, a := "CredentialsEndpointError", aerr.Code(); e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "failed to load credentials", aerr.Message(); e != a {
		t.Errorf("expect %v, got %v", e, a)
	}

	aerr = aerr.OrigErr().(awserr.Error)
	if e, a := "Error", aerr.Code(); e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "Message", aerr.Message(); e != a {
		t.Errorf("expect %v, got %v", e, a)
	}

	if v := creds.AccessKeyID; len(v) != 0 {
		t.Errorf("expect empty, got %#v", v)
	}
	if v := creds.SecretAccessKey; len(v) != 0 {
		t.Errorf("expect empty, got %#v", v)
	}
	if v := creds.SessionToken; len(v) != 0 {
		t.Errorf("expect empty, got %#v", v)
	}
	if !client.IsExpired() {
		t.Errorf("expect expired, wasn't")
	}
}

func TestAuthorizationToken(t *testing.T) {
	cases := map[string]struct {
		ExpectPath        string
		ServerPath        string
		AuthToken         string
		AuthTokenProvider endpointcreds.AuthTokenProvider
		ExpectAuthToken   string
		ExpectError       bool
	}{
		"AuthToken": {
			ExpectPath:      "/path/to/endpoint",
			ServerPath:      "/path/to/endpoint?something=else",
			AuthToken:       "Basic abc123",
			ExpectAuthToken: "Basic abc123",
		},
		"AuthFileToken": {
			ExpectPath: "/path/to/endpoint",
			ServerPath: "/path/to/endpoint?something=else",
			AuthToken:  "Basic abc123",
			AuthTokenProvider: endpointcreds.TokenProviderFunc(func() (string, error) {
				return "Hello %20world", nil
			}),
			ExpectAuthToken: "Hello %20world",
		},
		"RetrieveFileTokenError": {
			ExpectPath: "/path/to/endpoint",
			ServerPath: "/path/to/endpoint?something=else",
			AuthToken:  "Basic abc123",
			AuthTokenProvider: endpointcreds.TokenProviderFunc(func() (string, error) {
				return "", fmt.Errorf("test error")
			}),
			ExpectAuthToken: "Hello %20world",
			ExpectError:     true,
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				if e, a := c.ExpectPath, r.URL.Path; e != a {
					t.Errorf("expect %v, got %v", e, a)
				}
				if e, a := "application/json", r.Header.Get("Accept"); e != a {
					t.Errorf("expect %v, got %v", e, a)
				}
				if e, a := c.ExpectAuthToken, r.Header.Get("Authorization"); e != a {
					t.Fatalf("expect %v, got %v", e, a)
				}

				encoder := json.NewEncoder(w)
				err := encoder.Encode(map[string]interface{}{
					"AccessKeyID":     "AKID",
					"SecretAccessKey": "SECRET",
					"Token":           "TOKEN",
					"Expiration":      time.Now().Add(1 * time.Hour),
				})

				if err != nil {
					fmt.Println("failed to write out creds", err)
				}
			}))
			defer server.Close()

			client := endpointcreds.NewProviderClient(*unit.Session.Config,
				unit.Session.Handlers,
				server.URL+c.ServerPath,
				func(p *endpointcreds.Provider) {
					p.AuthorizationToken = c.AuthToken
					p.AuthorizationTokenProvider = c.AuthTokenProvider
				},
			)
			creds, err := client.Retrieve()

			if err != nil && !c.ExpectError {
				t.Errorf("expect no error, got %v", err)
			} else if err == nil && c.ExpectError {
				t.Errorf("expect error, got nil")
			}

			if c.ExpectError {
				return
			}

			if e, a := "AKID", creds.AccessKeyID; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}
			if e, a := "SECRET", creds.SecretAccessKey; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}
			if e, a := "TOKEN", creds.SessionToken; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}
			if client.IsExpired() {
				t.Errorf("expect not expired, was")
			}

			client.(*endpointcreds.Provider).CurrentTime = func() time.Time {
				return time.Now().Add(2 * time.Hour)
			}

			if !client.IsExpired() {
				t.Errorf("expect expired, wasn't")
			}
		})
	}
}
