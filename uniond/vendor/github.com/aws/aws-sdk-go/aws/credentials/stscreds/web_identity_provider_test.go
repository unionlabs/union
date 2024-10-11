//go:build go1.7
// +build go1.7

package stscreds

import (
	"fmt"
	"net/http"
	"reflect"
	"strings"
	"testing"
	"time"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/client"
	"github.com/aws/aws-sdk-go/aws/credentials"
	"github.com/aws/aws-sdk-go/aws/request"
	"github.com/aws/aws-sdk-go/service/sts"
	"github.com/aws/aws-sdk-go/service/sts/stsiface"
)

func TestWebIdentityProviderRetrieve(t *testing.T) {
	cases := map[string]struct {
		roleARN           string
		tokenPath         string
		sessionName       string
		newClient         func(t *testing.T) stsiface.STSAPI
		duration          time.Duration
		expectedError     string
		expectedCredValue credentials.Value
	}{
		"session name case": {
			roleARN:     "arn01234567890123456789",
			tokenPath:   "testdata/token.jwt",
			sessionName: "foo",
			newClient: func(t *testing.T) stsiface.STSAPI {
				return mockAssumeRoleWithWebIdentityClient{
					t: t,
					doRequest: func(t *testing.T, input *sts.AssumeRoleWithWebIdentityInput) (
						*sts.AssumeRoleWithWebIdentityOutput, error,
					) {
						if e, a := "foo", *input.RoleSessionName; e != a {
							t.Errorf("expected %v, but received %v", e, a)
						}
						if input.DurationSeconds != nil {
							t.Errorf("expect no duration, got %v", *input.DurationSeconds)
						}

						return &sts.AssumeRoleWithWebIdentityOutput{
							Credentials: &sts.Credentials{
								Expiration:      aws.Time(time.Now()),
								AccessKeyId:     aws.String("access-key-id"),
								SecretAccessKey: aws.String("secret-access-key"),
								SessionToken:    aws.String("session-token"),
							},
						}, nil
					},
				}
			},
			expectedCredValue: credentials.Value{
				AccessKeyID:     "access-key-id",
				SecretAccessKey: "secret-access-key",
				SessionToken:    "session-token",
				ProviderName:    WebIdentityProviderName,
			},
		},
		"with duration": {
			roleARN:     "arn01234567890123456789",
			tokenPath:   "testdata/token.jwt",
			sessionName: "foo",
			duration:    15 * time.Minute,
			newClient: func(t *testing.T) stsiface.STSAPI {
				return mockAssumeRoleWithWebIdentityClient{
					t: t,
					doRequest: func(t *testing.T, input *sts.AssumeRoleWithWebIdentityInput) (
						*sts.AssumeRoleWithWebIdentityOutput, error,
					) {
						if e, a := int64((15*time.Minute)/time.Second), *input.DurationSeconds; e != a {
							t.Errorf("expect %v duration, got %v", e, a)
						}

						return &sts.AssumeRoleWithWebIdentityOutput{
							Credentials: &sts.Credentials{
								Expiration:      aws.Time(time.Now()),
								AccessKeyId:     aws.String("access-key-id"),
								SecretAccessKey: aws.String("secret-access-key"),
								SessionToken:    aws.String("session-token"),
							},
						}, nil
					},
				}
			},
			expectedCredValue: credentials.Value{
				AccessKeyID:     "access-key-id",
				SecretAccessKey: "secret-access-key",
				SessionToken:    "session-token",
				ProviderName:    WebIdentityProviderName,
			},
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			p := NewWebIdentityRoleProvider(c.newClient(t), c.roleARN, c.sessionName, c.tokenPath)
			p.Duration = c.duration

			credValue, err := p.Retrieve()
			if len(c.expectedError) != 0 {
				if err == nil {
					t.Fatalf("expect error, got none")
				}
				if e, a := c.expectedError, err.Error(); !strings.Contains(a, e) {
					t.Fatalf("expect error to contain %v, got %v", e, a)
				}
				return
			}
			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}

			if e, a := c.expectedCredValue, credValue; !reflect.DeepEqual(e, a) {
				t.Errorf("expected %v, but received %v", e, a)
			}
		})
	}
}

type mockAssumeRoleWithWebIdentityClient struct {
	stsiface.STSAPI

	t         *testing.T
	doRequest func(*testing.T, *sts.AssumeRoleWithWebIdentityInput) (*sts.AssumeRoleWithWebIdentityOutput, error)
}

func (c mockAssumeRoleWithWebIdentityClient) AssumeRoleWithWebIdentityRequest(input *sts.AssumeRoleWithWebIdentityInput) (
	*request.Request, *sts.AssumeRoleWithWebIdentityOutput,
) {
	output, err := c.doRequest(c.t, input)

	req := &request.Request{
		HTTPRequest: &http.Request{},
		Retryer:     client.DefaultRetryer{},
	}
	req.Handlers.Send.PushBack(func(r *request.Request) {
		r.HTTPResponse = &http.Response{}
		r.Data = output
		r.Error = err

		var found bool
		for _, retryCode := range req.RetryErrorCodes {
			if retryCode == sts.ErrCodeInvalidIdentityTokenException {
				found = true
				break
			}
		}
		if !found {
			c.t.Errorf("expect ErrCodeInvalidIdentityTokenException error code to be retry-able")
		}
	})

	return req, output
}

func TestNewWebIdentityRoleProviderWithOptions(t *testing.T) {
	const roleARN = "a-role-arn"
	const roleSessionName = "a-session-name"

	cases := map[string]struct {
		options []func(*WebIdentityRoleProvider)
		expect  WebIdentityRoleProvider
	}{
		"no options": {
			expect: WebIdentityRoleProvider{
				client:          stubClient{},
				tokenFetcher:    stubTokenFetcher{},
				roleARN:         roleARN,
				roleSessionName: roleSessionName,
			},
		},
		"with options": {
			options: []func(*WebIdentityRoleProvider){
				func(o *WebIdentityRoleProvider) {
					o.Duration = 10 * time.Minute
					o.ExpiryWindow = time.Minute
				},
			},
			expect: WebIdentityRoleProvider{
				client:          stubClient{},
				tokenFetcher:    stubTokenFetcher{},
				roleARN:         roleARN,
				roleSessionName: roleSessionName,
				Duration:        10 * time.Minute,
				ExpiryWindow:    time.Minute,
			},
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {

			p := NewWebIdentityRoleProviderWithOptions(
				stubClient{}, roleARN, roleSessionName,
				stubTokenFetcher{}, c.options...)

			if !reflect.DeepEqual(c.expect, *p) {
				t.Errorf("expect:\n%v\nactual:\n%v", c.expect, *p)
			}
		})
	}
}

type stubClient struct {
	stsiface.STSAPI
}
type stubTokenFetcher struct{}

func (stubTokenFetcher) FetchToken(credentials.Context) ([]byte, error) {
	return nil, fmt.Errorf("stubTokenFetcher should not be called")
}
