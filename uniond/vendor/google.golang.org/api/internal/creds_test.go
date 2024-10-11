// Copyright 2017 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package internal

import (
	"context"
	"os"
	"testing"

	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"
)

func TestDefaultServiceAccount(t *testing.T) {
	ctx := context.Background()

	// Load a valid JSON file. No way to really test the contents; we just
	// verify that there is no error.
	ds := &DialSettings{
		CredentialsFile: "testdata/service-account.json",
		DefaultScopes:   []string{"foo"},
	}
	if _, err := Creds(ctx, ds); err != nil {
		t.Errorf("got %v, wanted no error", err)
	}

	// Load valid JSON. No way to really test the contents; we just
	// verify that there is no error.
	ds = &DialSettings{
		CredentialsJSON: []byte(validServiceAccountJSON),
		DefaultScopes:   []string{"foo"},
	}
	if _, err := Creds(ctx, ds); err != nil {
		t.Errorf("got %v, wanted no error", err)
	}
}

func TestJWTWithAudience(t *testing.T) {
	ctx := context.Background()

	// Load a valid JSON file. No way to really test the contents; we just
	// verify that there is no error.
	ds := &DialSettings{CredentialsFile: "testdata/service-account.json", Audiences: []string{"foo"}}
	if _, err := Creds(ctx, ds); err != nil {
		t.Errorf("got %v, wanted no error", err)
	}

	// Load valid JSON. No way to really test the contents; we just
	// verify that there is no error.
	ds = &DialSettings{CredentialsJSON: []byte(validServiceAccountJSON), Audiences: []string{"foo"}}
	if _, err := Creds(ctx, ds); err != nil {
		t.Errorf("got %v, wanted no error", err)
	}
}

func TestJWTWithScope(t *testing.T) {
	ctx := context.Background()

	// Load a valid JSON file. No way to really test the contents; we just
	// verify that there is no error.
	ds := &DialSettings{
		CredentialsFile:    "testdata/service-account.json",
		Scopes:             []string{"foo"},
		EnableJwtWithScope: true,
	}
	if _, err := Creds(ctx, ds); err != nil {
		t.Errorf("got %v, wanted no error", err)
	}

	// Load valid JSON. No way to really test the contents; we just
	// verify that there is no error.
	ds = &DialSettings{
		CredentialsJSON:    []byte(validServiceAccountJSON),
		Scopes:             []string{"foo"},
		EnableJwtWithScope: true,
	}
	if _, err := Creds(ctx, ds); err != nil {
		t.Errorf("got %v, wanted no error", err)
	}
}

func TestJWTWithScopeAndUniverseDomain(t *testing.T) {
	ctx := context.Background()

	// Load a valid JSON file. No way to really test the contents; we just
	// verify that there is no error.
	ds := &DialSettings{
		CredentialsFile:    "testdata/service-account.json",
		Scopes:             []string{"foo"},
		EnableJwtWithScope: true,
		UniverseDomain:     "example.com",
	}
	if _, err := Creds(ctx, ds); err != nil {
		t.Errorf("got %v, wanted no error", err)
	}

	// Load valid JSON. No way to really test the contents; we just
	// verify that there is no error.
	ds = &DialSettings{
		CredentialsJSON:    []byte(validServiceAccountJSON),
		Scopes:             []string{"foo"},
		EnableJwtWithScope: true,
		UniverseDomain:     "example.com",
	}
	if _, err := Creds(ctx, ds); err != nil {
		t.Errorf("got %v, wanted no error", err)
	}
}

func TestJWTWithDefaultScopes(t *testing.T) {
	ctx := context.Background()

	// Load a valid JSON file. No way to really test the contents; we just
	// verify that there is no error.
	ds := &DialSettings{
		CredentialsFile:    "testdata/service-account.json",
		DefaultScopes:      []string{"foo"},
		EnableJwtWithScope: true,
	}
	if _, err := Creds(ctx, ds); err != nil {
		t.Errorf("got %v, wanted no error", err)
	}

	// Load valid JSON. No way to really test the contents; we just
	// verify that there is no error.
	ds = &DialSettings{
		CredentialsJSON:    []byte(validServiceAccountJSON),
		DefaultScopes:      []string{"foo"},
		EnableJwtWithScope: true,
	}
	if _, err := Creds(ctx, ds); err != nil {
		t.Errorf("got %v, wanted no error", err)
	}
}

func TestJWTWithDefaultAudience(t *testing.T) {
	ctx := context.Background()

	// Load a valid JSON file. No way to really test the contents; we just
	// verify that there is no error.
	ds := &DialSettings{
		CredentialsFile: "testdata/service-account.json",
		DefaultAudience: "foo",
	}
	if _, err := Creds(ctx, ds); err != nil {
		t.Errorf("got %v, wanted no error", err)
	}

	// Load valid JSON. No way to really test the contents; we just
	// verify that there is no error.
	ds = &DialSettings{
		CredentialsJSON: []byte(validServiceAccountJSON),
		DefaultAudience: "foo",
	}
	if _, err := Creds(ctx, ds); err != nil {
		t.Errorf("got %v, wanted no error", err)
	}
}

func TestOAuth(t *testing.T) {
	ctx := context.Background()

	// Load a valid JSON file. No way to really test the contents; we just
	// verify that there is no error.
	ds := &DialSettings{CredentialsFile: "testdata/service-account.json", Scopes: []string{"foo"}}
	if _, err := Creds(ctx, ds); err != nil {
		t.Errorf("got %v, wanted no error", err)
	}

	// Load valid JSON. No way to really test the contents; we just
	// verify that there is no error.
	ds = &DialSettings{CredentialsJSON: []byte(validServiceAccountJSON), Scopes: []string{"foo"}}
	if _, err := Creds(ctx, ds); err != nil {
		t.Errorf("got %v, wanted no error", err)
	}
}

const validServiceAccountJSON = `{
  "type": "service_account",
  "project_id": "dumba-504",
  "private_key_id": "adsfsdd",
  "private_key": "-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDczcmZh9zhcDjd\nc+a1bcNu06QA+PGwjAZygTy9ays8qTLi4J8zWSjxgb18ZgYnv1gwVenmSBRuj+kg\nG03NPOxGmPrC/fTnZVBZpQRwBccBrPnRMvEM94egVrHKvPgqbifkyn2VR+ouWZvv\nwigR2PbjcvrSNkPE0QpLnRV0stilgCIYXR17lFrOPoiMra3N//1J0IPIFl3qZHxB\nsiejdi7zUiqLkqjYrNRHKulAGcJjqdCsNjAdjxgbRHgTjXSyuJh8bdKBgAMhetKj\nvU2OM431a9MQq77q/kvuJuCDRe6WqHs7JEFICUymTSSauANXowTUz63jfoSMMwmL\nBNcoePAZAgMBAAECggEAFxpkJe+YjbERjiBOqzybJok3/48MsOGR2iDKI3KncT8t\n7x28IqIJXe2qjy3YpoeHmXUf1mPD2YauyJh6xUcC3OcsU8NBQQXxiJOy2WrNVhBM\nilZXcPkkZIM1sqYfnEbu3ypNxhUifGuNXKKW0Tk/qfBRrLWXVSxfNKawxEdjUjua\ncknCwOBuZYkp8tTO5py5+RcoxHjAoNGaajep3yvNTIZ0WOLSjpAxLh0XCL5PRqKe\nfOrxL7ZY5Xl+yhc7/9PnVcdVOyUrry6I3byx8Yu46USLamNivZPk4xCiCe0k5OO1\nnXiU7qSLky4iiSzEd8o+0j/G8gMPZ9CF944kF60QIQKBgQDygfFrPjdYT5tpuPy+\nfpAZVnYWqLkvQHty0jmAqHucYRYVd1zpzY8zXW1JPXWSwGMSqB/Nz82v6oUw/Ovm\nRJ4+hvvUqZtUM1KJ10RUUWZDDLKoUgHp96IHarytdVy6kXZ0F2QNzW/VFTuzdKaK\n53c7Zc7iFK+4B/6XfyAumU0PSQKBgQDpFrdeQSoT4jXw/een7Hj3686cb9fkLEIf\na/pOOlqfGlJf7+NfqZpGBj5XxLGIJX80FFRtWTTAdgWrBmP0Nyvh48yd0KLALvyh\nmmqX/tBkkP41ASRMD+fWYh0AMhH6LmgrZtSUPKy0NvLIosH0qSbKGLIJEcXx3Pm4\nS1+eH0xKUQKBgA1hXhCsviEBQ3Hx4wAfu5OqUZmudYlF5YnQT5vpr+hQ8wb8LwQ3\nc09COGVyHqqaMt00qYyRiqfKKM8rJVjvMEwC5qI1OXzL2CIC3qJIW3wXl0PyQmjG\nYQpHuWFYuGUS4ZZGNB8O1rzLDyA3r3i6jLmaRG/09D0TM9joCr6HdtkBAoGBAIi6\n1p3nw/MeA1520uligiOMpAqIYTBr9e3QvWgeOwKRwjic09hN+T2SdAewTiP7Ov8l\n3dC3P4aWtQR6HzAnHQrJQkJhHNd3uKJjnpvC0iPsGfKl1ND5k5niu/hdZsZHarvq\n+lBqtzSP9yNStkv63dI3YliHoIIcijBdpp1u5qXBAoGBAOLrmvUKnx4NLEcauQ0e\naHndQ/6y4ie6knn1iJsJdYNJnYh9RKqDPTgpi8DbE2eb5JNkBQl0nSBMl74+MhVl\nMKBPVprkv7p3BdxoanpsncY14TUnzWIngkH21Rk0gqE3t/iJ7xnCTSv8qv3yYDj3\nL54zu6Y9GbjLgn6BtfhLHG4v\n-----END PRIVATE KEY-----\n",
  "client_email": "dumba-504@appspot.gserviceaccount.com",
  "client_id": "111",
  "auth_uri": "https://accounts.google.com/o/oauth2/auth",
  "token_uri": "https://accounts.google.com/o/oauth2/token",
  "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
  "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/dumba-504%40appspot.gserviceaccount.com"
}`

func TestGetQuotaProject(t *testing.T) {
	ctx := context.Background()
	emptyCred, err := credentialsFromJSON(
		ctx,
		[]byte(validServiceAccountJSON),
		&DialSettings{
			Endpoint:      "foo.googleapis.com",
			DefaultScopes: []string{"foo"},
		})
	if err != nil {
		t.Fatalf("got %v, wanted no error", err)
	}
	quotaProjectJSON := []byte(`
{
	"type": "authorized_user",
	"quota_project_id": "foobar"
}`)

	quotaCred, err := credentialsFromJSON(
		ctx,
		[]byte(quotaProjectJSON),
		&DialSettings{
			Endpoint:      "foo.googleapis.com",
			DefaultScopes: []string{"foo"},
		})
	if err != nil {
		t.Fatalf("got %v, wanted no error", err)
	}

	tests := []struct {
		name      string
		cred      *google.Credentials
		clientOpt string
		env       string
		want      string
	}{
		{
			name: "empty all",
			cred: nil,
			want: "",
		},
		{
			name: "empty cred",
			cred: emptyCred,
			want: "",
		},
		{
			name: "from cred",
			cred: quotaCred,
			want: "foobar",
		},
		{
			name:      "from opt",
			cred:      quotaCred,
			clientOpt: "clientopt",
			want:      "clientopt",
		},
		{
			name: "from env",
			cred: quotaCred,
			env:  "envProject",
			want: "envProject",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			oldEnv := os.Getenv(quotaProjectEnvVar)
			if tt.env != "" {
				os.Setenv(quotaProjectEnvVar, tt.env)
			}
			if want, got := tt.want, GetQuotaProject(tt.cred, tt.clientOpt); want != got {
				t.Errorf("GetQuotaProject(%v, %q): want %q, got %q", tt.cred, tt.clientOpt, want, got)
			}
			os.Setenv(quotaProjectEnvVar, oldEnv)
		})
	}
}

func TestCredsWithCredentials(t *testing.T) {
	tests := []struct {
		name string
		ds   *DialSettings
		want string
	}{
		{
			name: "only normal opt",
			ds: &DialSettings{
				Credentials: &google.Credentials{
					TokenSource: oauth2.StaticTokenSource(&oauth2.Token{
						AccessToken: "normal",
					}),
				},
			},
			want: "normal",
		},
		{
			name: "normal and internal creds opt",
			ds: &DialSettings{
				Credentials: &google.Credentials{
					TokenSource: oauth2.StaticTokenSource(&oauth2.Token{
						AccessToken: "normal",
					}),
				},
				InternalCredentials: &google.Credentials{
					TokenSource: oauth2.StaticTokenSource(&oauth2.Token{
						AccessToken: "internal",
					}),
				},
			},
			want: "internal",
		},
		{
			name: "only internal creds opt",
			ds: &DialSettings{
				InternalCredentials: &google.Credentials{
					TokenSource: oauth2.StaticTokenSource(&oauth2.Token{
						AccessToken: "internal",
					}),
				},
			},
			want: "internal",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			creds, err := Creds(context.Background(), tc.ds)
			if err != nil {
				t.Fatalf("got %v, want nil error", err)
			}
			if tok, _ := creds.TokenSource.Token(); tok.AccessToken != tc.want {
				t.Fatalf("tok.AccessToken = %q, want %q", tok.AccessToken, tc.want)
			}
		})
	}
}

func TestIsSelfSignedJWTFlow(t *testing.T) {
	tests := []struct {
		name string
		ds   *DialSettings
		want bool
	}{
		{
			name: "EnableJwtWithScope true",
			ds: &DialSettings{
				CredentialsFile:    "testdata/service-account.json",
				Scopes:             []string{"foo"},
				EnableJwtWithScope: true,
			},
			want: true,
		},
		{
			name: "EnableJwtWithScope false",
			ds: &DialSettings{
				CredentialsFile:    "testdata/service-account.json",
				Scopes:             []string{"foo"},
				EnableJwtWithScope: false,
			},
			want: false,
		},
		{
			name: "UniverseDomain",
			ds: &DialSettings{
				CredentialsFile:    "testdata/service-account.json",
				Scopes:             []string{"foo"},
				EnableJwtWithScope: false,
				UniverseDomain:     "example.com",
			},
			want: true,
		},
		{
			name: "UniverseDomainUserAccount",
			ds: &DialSettings{
				CredentialsFile:    "testdata/user-account.json",
				Scopes:             []string{"foo"},
				EnableJwtWithScope: false,
				UniverseDomain:     "example.com",
			},
			want: false,
		},
	}

	for _, tc := range tests {

		bytes, err := os.ReadFile(tc.ds.CredentialsFile)
		if err != nil {
			t.Fatal(err)
		}
		isSSJ, err := isSelfSignedJWTFlow(bytes, tc.ds)
		if err != nil {
			t.Errorf("[%s]: got %v, wanted no error", tc.name, err)
		}
		if isSSJ != tc.want {
			t.Errorf("[%s]: got %t, wanted %t", tc.name, isSSJ, tc.want)
		}
	}
}
