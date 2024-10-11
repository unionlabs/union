//go:build go1.16
// +build go1.16

package ssocreds

import (
	"fmt"
	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/auth/bearer"
	"github.com/aws/aws-sdk-go/service/ssooidc"
	"io/ioutil"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
	"time"
)

func TestSSOTokenProvider(t *testing.T) {
	restoreTime := swapNowTime(time.Date(2021, 12, 21, 12, 21, 1, 0, time.UTC))
	defer restoreTime()

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
		setup         func() error
		postRetrieve  func() error
		client        CreateTokenAPIClient
		cacheFilePath string
		optFns        []func(*SSOTokenProviderOptions)

		expectToken bearer.Token
		expectErr   string
	}{
		"no cache file": {
			cacheFilePath: filepath.Join("testdata", "file_not_exists"),
			expectErr:     "failed to read cached SSO token file",
		},
		"invalid json cache file": {
			cacheFilePath: filepath.Join("testdata", "invalid_json.json"),
			expectErr:     "failed to parse cached SSO token file",
		},
		"missing accessToken": {
			cacheFilePath: filepath.Join("testdata", "missing_accessToken.json"),
			expectErr:     "must contain accessToken and expiresAt fields",
		},
		"missing expiresAt": {
			cacheFilePath: filepath.Join("testdata", "missing_expiresAt.json"),
			expectErr:     "must contain accessToken and expiresAt fields",
		},
		"expired no clientSecret": {
			cacheFilePath: filepath.Join("testdata", "missing_clientSecret.json"),
			expectErr:     "cached SSO token is expired, or not present",
		},
		"expired no clientId": {
			cacheFilePath: filepath.Join("testdata", "missing_clientId.json"),
			expectErr:     "cached SSO token is expired, or not present",
		},
		"expired no refreshToken": {
			cacheFilePath: filepath.Join("testdata", "missing_refreshToken.json"),
			expectErr:     "cached SSO token is expired, or not present",
		},
		"valid sso token": {
			cacheFilePath: filepath.Join("testdata", "valid_token.json"),
			expectToken: bearer.Token{
				Value:     "dGhpcyBpcyBub3QgYSByZWFsIHZhbHVl",
				CanExpire: true,
				Expires:   time.Date(2044, 4, 4, 7, 0, 1, 0, time.UTC),
			},
		},
		"refresh expired token": {
			setup: func() error {
				testFile, err := os.ReadFile(filepath.Join("testdata", "expired_token.json"))
				if err != nil {
					return err
				}

				return os.WriteFile(filepath.Join(tempDir, "expired_token.json"), testFile, 0600)
			},
			postRetrieve: func() error {
				actual, err := loadCachedToken(filepath.Join(tempDir, "expired_token.json"))
				if err != nil {
					return err

				}
				expect := cachedToken{
					tokenKnownFields: tokenKnownFields{
						AccessToken: "updated access token",
						ExpiresAt:   (*rfc3339)(aws.Time(time.Date(2021, 12, 21, 12, 31, 1, 0, time.UTC))),

						RefreshToken: "updated refresh token",
						ClientID:     "client id",
						ClientSecret: "client secret",
					},
					UnknownFields: map[string]interface{}{
						"unknownField": "some value",
					},
				}

				if !reflect.DeepEqual(expect, actual) {
					return fmt.Errorf("expect token file %v but got actual %v", expect, actual)
				}
				return nil
			},
			cacheFilePath: filepath.Join(tempDir, "expired_token.json"),
			client: &mockCreateTokenAPIClient{
				expectInput: &ssooidc.CreateTokenInput{
					ClientId:     aws.String("client id"),
					ClientSecret: aws.String("client secret"),
					RefreshToken: aws.String("refresh token"),
					GrantType:    aws.String("refresh_token"),
				},
				output: &ssooidc.CreateTokenOutput{
					AccessToken:  aws.String("updated access token"),
					ExpiresIn:    aws.Int64(600),
					RefreshToken: aws.String("updated refresh token"),
				},
			},
			expectToken: bearer.Token{
				Value:     "updated access token",
				CanExpire: true,
				Expires:   time.Date(2021, 12, 21, 12, 31, 1, 0, time.UTC),
			},
		},
		"fail refresh expired token": {
			setup: func() error {
				testFile, err := os.ReadFile(filepath.Join("testdata", "expired_token.json"))
				if err != nil {
					return err
				}
				return os.WriteFile(filepath.Join(tempDir, "expired_token.json"), testFile, 0600)
			},
			postRetrieve: func() error {
				actual, err := loadCachedToken(filepath.Join(tempDir, "expired_token.json"))
				if err != nil {
					return err

				}
				expect := cachedToken{
					tokenKnownFields: tokenKnownFields{
						AccessToken: "access token",
						ExpiresAt:   (*rfc3339)(aws.Time(time.Date(2021, 12, 21, 12, 21, 1, 0, time.UTC))),

						RefreshToken: "refresh token",
						ClientID:     "client id",
						ClientSecret: "client secret",
					},
				}

				if !reflect.DeepEqual(expect, actual) {
					return fmt.Errorf("expect token file %v but got actual %v", expect, actual)
				}
				return nil
			},
			cacheFilePath: filepath.Join(tempDir, "expired_token.json"),
			client: &mockCreateTokenAPIClient{
				err: fmt.Errorf("sky is falling"),
			},
			expectErr: "unable to refresh SSO token, sky is falling",
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			if c.setup != nil {
				if err := c.setup(); err != nil {
					t.Fatalf("failed to setup test, %v", err)
				}
			}
			provider := NewSSOTokenProvider(c.client, c.cacheFilePath, c.optFns...)

			token, err := provider.RetrieveBearerToken(aws.BackgroundContext())
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

			if !reflect.DeepEqual(c.expectToken, token) {
				t.Errorf("expect %v, got %v", c.expectToken, token)
			}

			if c.postRetrieve != nil {
				if err := c.postRetrieve(); err != nil {
					t.Fatalf("post retrieve failed, %v", err)
				}
			}
		})
	}
}

type mockCreateTokenAPIClient struct {
	expectInput *ssooidc.CreateTokenInput
	output      *ssooidc.CreateTokenOutput
	err         error
}

func (c *mockCreateTokenAPIClient) CreateToken(input *ssooidc.CreateTokenInput) (
	*ssooidc.CreateTokenOutput, error,
) {
	if c.expectInput != nil {
		if !reflect.DeepEqual(c.expectInput, input) {
			return nil, fmt.Errorf("expect token file %v but got actual %v", c.expectInput, input)
		}
	}

	return c.output, c.err
}
