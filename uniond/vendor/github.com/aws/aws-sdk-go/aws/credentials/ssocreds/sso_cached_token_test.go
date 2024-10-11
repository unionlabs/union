//go:build go1.9
// +build go1.9

package ssocreds

import (
	"io/ioutil"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
	"time"
)

func TestStandardSSOCacheTokenFilepath(t *testing.T) {
	origHomeDur := resolvedOsUserHomeDir
	defer func() {
		resolvedOsUserHomeDir = origHomeDur
	}()

	cases := map[string]struct {
		key            string
		osUserHomeDir  func() string
		expectFilename string
		expectErr      string
	}{
		"success": {
			key: "https://example.awsapps.com/start",
			osUserHomeDir: func() string {
				return os.TempDir()
			},
			expectFilename: filepath.Join(os.TempDir(), ".aws", "sso", "cache",
				"e8be5486177c5b5392bd9aa76563515b29358e6e.json"),
		},
		"failure": {
			key: "https://example.awsapps.com/start",
			osUserHomeDir: func() string {
				return ""
			},
			expectErr: "some error",
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			resolvedOsUserHomeDir = c.osUserHomeDir

			actual, err := StandardCachedTokenFilepath(c.key)
			if c.expectErr != "" {
				if err == nil {
					t.Fatalf("expect error, got none")
				}
				return
			}
			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}

			if e, a := c.expectFilename, actual; e != a {
				t.Errorf("expect %v filename, got %v", e, a)
			}
		})
	}
}

func TestLoadCachedToken(t *testing.T) {
	cases := map[string]struct {
		filename    string
		expectToken cachedToken
		expectErr   string
	}{
		"file not found": {
			filename:  filepath.Join("testdata", "does_not_exist.json"),
			expectErr: "failed to read cached SSO token file",
		},
		"invalid json": {
			filename:  filepath.Join("testdata", "invalid_json.json"),
			expectErr: "failed to parse cached SSO token file",
		},
		"missing accessToken": {
			filename:  filepath.Join("testdata", "missing_accessToken.json"),
			expectErr: "must contain accessToken and expiresAt fields",
		},
		"missing expiresAt": {
			filename:  filepath.Join("testdata", "missing_expiresAt.json"),
			expectErr: "must contain accessToken and expiresAt fields",
		},
		"standard token": {
			filename: filepath.Join("testdata", "valid_token.json"),
			expectToken: cachedToken{
				tokenKnownFields: tokenKnownFields{
					AccessToken:  "dGhpcyBpcyBub3QgYSByZWFsIHZhbHVl",
					ExpiresAt:    (*rfc3339)(Time(time.Date(2044, 4, 4, 7, 0, 1, 0, time.UTC))),
					ClientID:     "client id",
					ClientSecret: "client secret",
					RefreshToken: "refresh token",
				},
				UnknownFields: map[string]interface{}{
					"unknownField":          "some value",
					"registrationExpiresAt": "2044-04-04T07:00:01Z",
					"region":                "region",
					"startURL":              "start URL",
				},
			},
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			actualToken, err := loadCachedToken(c.filename)
			if c.expectErr != "" {
				if err == nil {
					t.Fatalf("expect %v error, got none", c.expectErr)
				}
				if e, a := c.expectErr, err.Error(); !strings.Contains(a, e) {
					t.Fatalf("expect %v error, got %v", e, a)
				}
				return
			}
			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}

			if !reflect.DeepEqual(c.expectToken, actualToken) {
				t.Errorf("expect token file %v but got actual %v", c.expectToken, actualToken)
			}
		})
	}
}

func TestStoreCachedToken(t *testing.T) {
	tempDir, err := ioutil.TempDir(os.TempDir(), "aws-sdk-go-"+t.Name())
	if err != nil {
		t.Fatalf("failed to create temporary test directory, %v", err)
	}
	defer func() {
		if err := os.RemoveAll(tempDir); err != nil {
			t.Errorf("failed to cleanup temporary test directory, %v", err)
		}
	}()

	cases := map[string]struct {
		token    cachedToken
		filename string
		fileMode os.FileMode
	}{
		"standard token": {
			filename: filepath.Join(tempDir, "token_file.json"),
			fileMode: 0600,
			token: cachedToken{
				tokenKnownFields: tokenKnownFields{
					AccessToken:  "dGhpcyBpcyBub3QgYSByZWFsIHZhbHVl",
					ExpiresAt:    (*rfc3339)(Time(time.Date(2044, 4, 4, 7, 0, 1, 0, time.UTC))),
					ClientID:     "client id",
					ClientSecret: "client secret",
					RefreshToken: "refresh token",
				},
				UnknownFields: map[string]interface{}{
					"unknownField":          "some value",
					"registrationExpiresAt": "2044-04-04T07:00:01Z",
					"region":                "region",
					"startURL":              "start URL",
				},
			},
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			err := storeCachedToken(c.filename, c.token, c.fileMode)
			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}

			actual, err := loadCachedToken(c.filename)
			if err != nil {
				t.Fatalf("failed to load stored token, %v", err)
			}

			if !reflect.DeepEqual(c.token, actual) {
				t.Errorf("expect token file %v but got actual %v", c.token, actual)
			}
		})
	}
}

// Time returns a pointer value for the time.Time value passed in.
func Time(v time.Time) *time.Time {
	return &v
}
