//go:build go1.7
// +build go1.7

package session

import (
	"fmt"
	"io/ioutil"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"reflect"
	"runtime"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/credentials"
	"github.com/aws/aws-sdk-go/aws/credentials/stscreds"
	"github.com/aws/aws-sdk-go/aws/defaults"
	"github.com/aws/aws-sdk-go/aws/endpoints"
	"github.com/aws/aws-sdk-go/aws/request"
	"github.com/aws/aws-sdk-go/internal/sdktesting"
	"github.com/aws/aws-sdk-go/internal/shareddefaults"
	"github.com/aws/aws-sdk-go/private/protocol"
	"github.com/aws/aws-sdk-go/service/sts"
)

func newEc2MetadataServer(key, secret string, closeAfterGetCreds bool) *httptest.Server {
	var server *httptest.Server
	server = httptest.NewServer(http.HandlerFunc(
		func(w http.ResponseWriter, r *http.Request) {
			if r.URL.Path == "/latest/meta-data/iam/security-credentials/RoleName" {
				w.Write([]byte(fmt.Sprintf(ec2MetadataResponse, key, secret)))

				if closeAfterGetCreds {
					go server.Close()
				}
			} else if r.URL.Path == "/latest/meta-data/iam/security-credentials/" {
				w.Write([]byte("RoleName"))
			} else {
				w.Write([]byte(""))
			}
		}))

	return server
}

func setupCredentialsEndpoints(t *testing.T) (endpoints.Resolver, func()) {
	origECSEndpoint := shareddefaults.ECSContainerCredentialsURI

	ecsMetadataServer := httptest.NewServer(http.HandlerFunc(
		func(w http.ResponseWriter, r *http.Request) {
			if r.URL.Path == "/ECS" {
				w.Write([]byte(ecsResponse))
			} else {
				w.Write([]byte(""))
			}
		}))
	shareddefaults.ECSContainerCredentialsURI = ecsMetadataServer.URL

	ec2MetadataServer := newEc2MetadataServer("ec2_key", "ec2_secret", false)

	stsServer := httptest.NewServer(http.HandlerFunc(
		func(w http.ResponseWriter, r *http.Request) {
			if err := r.ParseForm(); err != nil {
				w.WriteHeader(500)
				return
			}

			form := r.Form

			switch form.Get("Action") {
			case "AssumeRole":
				w.Write([]byte(fmt.Sprintf(
					assumeRoleRespMsg,
					time.Now().
						Add(15*time.Minute).
						Format(protocol.ISO8601TimeFormat))))
				return
			case "AssumeRoleWithWebIdentity":
				w.Write([]byte(fmt.Sprintf(assumeRoleWithWebIdentityResponse,
					time.Now().
						Add(15*time.Minute).
						Format(protocol.ISO8601TimeFormat))))
				return
			default:
				w.WriteHeader(404)
				return
			}
		}))

	ssoServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte(fmt.Sprintf(
			getRoleCredentialsResponse,
			time.Now().
				Add(15*time.Minute).
				UnixNano()/int64(time.Millisecond))))
	}))

	resolver := endpoints.ResolverFunc(
		func(service, region string, opts ...func(*endpoints.Options)) (endpoints.ResolvedEndpoint, error) {
			switch service {
			case "ec2metadata":
				return endpoints.ResolvedEndpoint{
					URL: ec2MetadataServer.URL,
				}, nil
			case "sts":
				return endpoints.ResolvedEndpoint{
					URL: stsServer.URL,
				}, nil
			case "portal.sso":
				return endpoints.ResolvedEndpoint{
					URL: ssoServer.URL,
				}, nil
			default:
				return endpoints.ResolvedEndpoint{},
					fmt.Errorf("unknown service endpoint, %s", service)
			}
		})

	return resolver, func() {
		shareddefaults.ECSContainerCredentialsURI = origECSEndpoint
		ecsMetadataServer.Close()
		ec2MetadataServer.Close()
		ssoServer.Close()
		stsServer.Close()
	}
}

func TestSharedConfigCredentialSource(t *testing.T) {
	const configFileForWindows = "testdata/credential_source_config_for_windows"
	const configFile = "testdata/credential_source_config"

	cases := []struct {
		name                   string
		profile                string
		sessOptProfile         string
		sessOptEC2IMDSEndpoint string
		expectedError          error
		expectedAccessKey      string
		expectedSecretKey      string
		expectedSessionToken   string
		expectedChain          []string
		init                   func() (func(), error)
		dependentOnOS          bool
	}{
		{
			name:          "credential source and source profile",
			profile:       "invalid_source_and_credential_source",
			expectedError: ErrSharedConfigSourceCollision,
			init: func() (func(), error) {
				os.Setenv("AWS_ACCESS_KEY", "access_key")
				os.Setenv("AWS_SECRET_KEY", "secret_key")
				return func() {}, nil
			},
		},
		{
			name:                 "env var credential source",
			sessOptProfile:       "env_var_credential_source",
			expectedAccessKey:    "AKID",
			expectedSecretKey:    "SECRET",
			expectedSessionToken: "SESSION_TOKEN",
			expectedChain: []string{
				"assume_role_w_creds_role_arn_env",
			},
			init: func() (func(), error) {
				os.Setenv("AWS_ACCESS_KEY", "access_key")
				os.Setenv("AWS_SECRET_KEY", "secret_key")
				return func() {}, nil
			},
		},
		{
			name:    "ec2metadata credential source",
			profile: "ec2metadata",
			expectedChain: []string{
				"assume_role_w_creds_role_arn_ec2",
			},
			expectedAccessKey:    "AKID",
			expectedSecretKey:    "SECRET",
			expectedSessionToken: "SESSION_TOKEN",
		},
		{
			name:                 "ec2metadata custom EC2 IMDS endpoint, env var",
			profile:              "not-exists-profile",
			expectedAccessKey:    "ec2_custom_key",
			expectedSecretKey:    "ec2_custom_secret",
			expectedSessionToken: "token",
			init: func() (func(), error) {
				altServer := newEc2MetadataServer("ec2_custom_key", "ec2_custom_secret", true)
				os.Setenv("AWS_EC2_METADATA_SERVICE_ENDPOINT", altServer.URL)
				return func() {}, nil
			},
		},
		{
			name:                 "ecs container credential source",
			profile:              "ecscontainer",
			expectedAccessKey:    "AKID",
			expectedSecretKey:    "SECRET",
			expectedSessionToken: "SESSION_TOKEN",
			expectedChain: []string{
				"assume_role_w_creds_role_arn_ecs",
			},
			init: func() (func(), error) {
				os.Setenv("AWS_CONTAINER_CREDENTIALS_RELATIVE_URI", "/ECS")
				return func() {}, nil
			},
		},
		{
			name:                 "chained assume role with env creds",
			profile:              "chained_assume_role",
			expectedAccessKey:    "AKID",
			expectedSecretKey:    "SECRET",
			expectedSessionToken: "SESSION_TOKEN",
			expectedChain: []string{
				"assume_role_w_creds_role_arn_chain",
				"assume_role_w_creds_role_arn_ec2",
			},
		},
		{
			name:              "credential process with no ARN set",
			profile:           "cred_proc_no_arn_set",
			dependentOnOS:     true,
			expectedAccessKey: "cred_proc_akid",
			expectedSecretKey: "cred_proc_secret",
		},
		{
			name:                 "credential process with ARN set",
			profile:              "cred_proc_arn_set",
			dependentOnOS:        true,
			expectedAccessKey:    "AKID",
			expectedSecretKey:    "SECRET",
			expectedSessionToken: "SESSION_TOKEN",
			expectedChain: []string{
				"assume_role_w_creds_proc_role_arn",
			},
		},
		{
			name:                 "chained assume role with credential process",
			profile:              "chained_cred_proc",
			dependentOnOS:        true,
			expectedAccessKey:    "AKID",
			expectedSecretKey:    "SECRET",
			expectedSessionToken: "SESSION_TOKEN",
			expectedChain: []string{
				"assume_role_w_creds_proc_source_prof",
			},
		},
		{
			name:                 "sso credentials",
			profile:              "sso_creds",
			expectedAccessKey:    "SSO_AKID",
			expectedSecretKey:    "SSO_SECRET_KEY",
			expectedSessionToken: "SSO_SESSION_TOKEN",
			init: func() (func(), error) {
				return ssoTestSetup()
			},
		},
		{
			name:                 "chained assume role with sso credentials",
			profile:              "source_sso_creds",
			expectedAccessKey:    "AKID",
			expectedSecretKey:    "SECRET",
			expectedSessionToken: "SESSION_TOKEN",
			expectedChain: []string{
				"source_sso_creds_arn",
			},
			init: func() (func(), error) {
				return ssoTestSetup()
			},
		},
		{
			name:                 "chained assume role with sso and static credentials",
			profile:              "assume_sso_and_static",
			expectedAccessKey:    "AKID",
			expectedSecretKey:    "SECRET",
			expectedSessionToken: "SESSION_TOKEN",
			expectedChain: []string{
				"assume_sso_and_static_arn",
			},
		},
		{
			name:          "invalid sso configuration",
			profile:       "sso_invalid",
			expectedError: fmt.Errorf("profile \"sso_invalid\" is configured to use SSO but is missing required configuration: sso_region, sso_start_url"),
		},
		{
			name:              "environment credentials with invalid sso",
			profile:           "sso_invalid",
			expectedAccessKey: "access_key",
			expectedSecretKey: "secret_key",
			init: func() (func(), error) {
				os.Setenv("AWS_ACCESS_KEY", "access_key")
				os.Setenv("AWS_SECRET_KEY", "secret_key")
				return func() {}, nil
			},
		},
		{
			name:                 "sso mixed with credential process provider",
			profile:              "sso_mixed_credproc",
			expectedAccessKey:    "SSO_AKID",
			expectedSecretKey:    "SSO_SECRET_KEY",
			expectedSessionToken: "SSO_SESSION_TOKEN",
			init: func() (func(), error) {
				return ssoTestSetup()
			},
		},
		{
			name:                 "sso mixed with web identity token provider",
			profile:              "sso_mixed_webident",
			expectedAccessKey:    "WEB_IDENTITY_AKID",
			expectedSecretKey:    "WEB_IDENTITY_SECRET",
			expectedSessionToken: "WEB_IDENTITY_SESSION_TOKEN",
		},
	}

	for i, c := range cases {
		t.Run(strconv.Itoa(i)+"_"+c.name, func(t *testing.T) {
			restoreEnvFn := sdktesting.StashEnv()
			defer restoreEnvFn()

			if c.dependentOnOS && runtime.GOOS == "windows" {
				os.Setenv("AWS_CONFIG_FILE", configFileForWindows)
			} else {
				os.Setenv("AWS_CONFIG_FILE", configFile)
			}

			os.Setenv("AWS_REGION", "us-east-1")
			os.Setenv("AWS_SDK_LOAD_CONFIG", "1")
			if len(c.profile) != 0 {
				os.Setenv("AWS_PROFILE", c.profile)
			}

			endpointResolver, cleanupFn := setupCredentialsEndpoints(t)
			defer cleanupFn()

			if c.init != nil {
				cleanup, err := c.init()
				if err != nil {
					t.Fatalf("expect no error, got %v", err)
				}
				defer cleanup()
			}

			var credChain []string
			handlers := defaults.Handlers()
			handlers.Sign.PushBack(func(r *request.Request) {
				if r.Config.Credentials == credentials.AnonymousCredentials {
					return
				}
				params := r.Params.(*sts.AssumeRoleInput)
				credChain = append(credChain, *params.RoleArn)
			})

			sess, err := NewSessionWithOptions(Options{
				Profile: c.sessOptProfile,
				Config: aws.Config{
					Logger:           t,
					EndpointResolver: endpointResolver,
				},
				Handlers:        handlers,
				EC2IMDSEndpoint: c.sessOptEC2IMDSEndpoint,
			})

			if c.expectedError != nil {
				var errStr string
				if err != nil {
					errStr = err.Error()
				}
				if e, a := c.expectedError.Error(), errStr; !strings.Contains(a, e) {
					t.Fatalf("expected %v, but received %v", e, a)
				}
			}

			if c.expectedError != nil {
				return
			}

			creds, err := sess.Config.Credentials.Get()
			if err != nil {
				t.Fatalf("expected no error, but received %v", err)
			}

			if e, a := c.expectedChain, credChain; !reflect.DeepEqual(e, a) {
				t.Errorf("expected %v, but received %v", e, a)
			}

			if e, a := c.expectedAccessKey, creds.AccessKeyID; e != a {
				t.Errorf("expected %v, but received %v", e, a)
			}

			if e, a := c.expectedSecretKey, creds.SecretAccessKey; e != a {
				t.Errorf("expected %v, but received %v", e, a)
			}

			if e, a := c.expectedSessionToken, creds.SessionToken; e != a {
				t.Errorf("expected %v, but received %v", e, a)
			}
		})
	}
}

const ecsResponse = `{
	  "Code": "Success",
	  "Type": "AWS-HMAC",
	  "AccessKeyId" : "ecs-access-key",
	  "SecretAccessKey" : "ecs-secret-key",
	  "Token" : "token",
	  "Expiration" : "2100-01-01T00:00:00Z",
	  "LastUpdated" : "2009-11-23T00:00:00Z"
	}`

const ec2MetadataResponse = `{
	  "Code": "Success",
	  "Type": "AWS-HMAC",
	  "AccessKeyId" : "%s",
	  "SecretAccessKey" : "%s",
	  "Token" : "token",
	  "Expiration" : "2100-01-01T00:00:00Z",
	  "LastUpdated" : "2009-11-23T00:00:00Z"
	}`

const assumeRoleRespMsg = `
<AssumeRoleResponse xmlns="https://sts.amazonaws.com/doc/2011-06-15/">
  <AssumeRoleResult>
    <AssumedRoleUser>
      <Arn>arn:aws:sts::account_id:assumed-role/role/session_name</Arn>
      <AssumedRoleId>AKID:session_name</AssumedRoleId>
    </AssumedRoleUser>
    <Credentials>
      <AccessKeyId>AKID</AccessKeyId>
      <SecretAccessKey>SECRET</SecretAccessKey>
      <SessionToken>SESSION_TOKEN</SessionToken>
      <Expiration>%s</Expiration>
    </Credentials>
  </AssumeRoleResult>
  <ResponseMetadata>
    <RequestId>request-id</RequestId>
  </ResponseMetadata>
</AssumeRoleResponse>
`

var assumeRoleWithWebIdentityResponse = `<AssumeRoleWithWebIdentityResponse xmlns="https://sts.amazonaws.com/doc/2011-06-15/">
  <AssumeRoleWithWebIdentityResult>
    <SubjectFromWebIdentityToken>amzn1.account.AF6RHO7KZU5XRVQJGXK6HB56KR2A</SubjectFromWebIdentityToken>
    <Audience>client.5498841531868486423.1548@apps.example.com</Audience>
    <AssumedRoleUser>
      <Arn>arn:aws:sts::123456789012:assumed-role/FederatedWebIdentityRole/app1</Arn>
      <AssumedRoleId>AROACLKWSDQRAOEXAMPLE:app1</AssumedRoleId>
    </AssumedRoleUser>
    <Credentials>
      <AccessKeyId>WEB_IDENTITY_AKID</AccessKeyId>
      <SecretAccessKey>WEB_IDENTITY_SECRET</SecretAccessKey>
      <SessionToken>WEB_IDENTITY_SESSION_TOKEN</SessionToken>
      <Expiration>%s</Expiration>
    </Credentials>
    <Provider>www.amazon.com</Provider>
  </AssumeRoleWithWebIdentityResult>
  <ResponseMetadata>
    <RequestId>request-id</RequestId>
  </ResponseMetadata>
</AssumeRoleWithWebIdentityResponse>
`

const getRoleCredentialsResponse = `{
  "roleCredentials": {
    "accessKeyId": "SSO_AKID",
    "secretAccessKey": "SSO_SECRET_KEY",
    "sessionToken": "SSO_SESSION_TOKEN",
    "expiration": %d
  }
}`

const ssoTokenCacheFile = `{
  "accessToken": "ssoAccessToken",
  "expiresAt": "%s"
}`

func TestSessionAssumeRole(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	os.Setenv("AWS_REGION", "us-east-1")
	os.Setenv("AWS_SDK_LOAD_CONFIG", "1")
	os.Setenv("AWS_SHARED_CREDENTIALS_FILE", testConfigFilename)
	os.Setenv("AWS_PROFILE", "assume_role_w_creds")

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte(fmt.Sprintf(
			assumeRoleRespMsg,
			time.Now().Add(15*time.Minute).Format("2006-01-02T15:04:05Z"))))
	}))
	defer server.Close()

	s, err := NewSession(&aws.Config{
		Endpoint:   aws.String(server.URL),
		DisableSSL: aws.Bool(true),
	})
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	creds, err := s.Config.Credentials.Get()
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}
	if e, a := "AKID", creds.AccessKeyID; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "SECRET", creds.SecretAccessKey; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "SESSION_TOKEN", creds.SessionToken; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "AssumeRoleProvider", creds.ProviderName; !strings.Contains(a, e) {
		t.Errorf("expect %v, to be in %v", e, a)
	}
}

func TestSessionAssumeRole_WithMFA(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	os.Setenv("AWS_REGION", "us-east-1")
	os.Setenv("AWS_SDK_LOAD_CONFIG", "1")
	os.Setenv("AWS_SHARED_CREDENTIALS_FILE", testConfigFilename)
	os.Setenv("AWS_PROFILE", "assume_role_w_creds")

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if e, a := r.FormValue("SerialNumber"), "0123456789"; e != a {
			t.Errorf("expect %v, got %v", e, a)
		}
		if e, a := r.FormValue("TokenCode"), "tokencode"; e != a {
			t.Errorf("expect %v, got %v", e, a)
		}
		if e, a := "900", r.FormValue("DurationSeconds"); e != a {
			t.Errorf("expect %v, got %v", e, a)
		}

		w.Write([]byte(fmt.Sprintf(
			assumeRoleRespMsg,
			time.Now().Add(15*time.Minute).Format("2006-01-02T15:04:05Z"))))
	}))
	defer server.Close()

	customProviderCalled := false
	sess, err := NewSessionWithOptions(Options{
		Profile: "assume_role_w_mfa",
		Config: aws.Config{
			Region:     aws.String("us-east-1"),
			Endpoint:   aws.String(server.URL),
			DisableSSL: aws.Bool(true),
		},
		SharedConfigState: SharedConfigEnable,
		AssumeRoleTokenProvider: func() (string, error) {
			customProviderCalled = true

			return "tokencode", nil
		},
	})
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	creds, err := sess.Config.Credentials.Get()
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}
	if !customProviderCalled {
		t.Errorf("expect true")
	}

	if e, a := "AKID", creds.AccessKeyID; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "SECRET", creds.SecretAccessKey; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "SESSION_TOKEN", creds.SessionToken; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "AssumeRoleProvider", creds.ProviderName; !strings.Contains(a, e) {
		t.Errorf("expect %v, to be in %v", e, a)
	}
}

func TestSessionAssumeRole_WithMFA_NoTokenProvider(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	os.Setenv("AWS_REGION", "us-east-1")
	os.Setenv("AWS_SDK_LOAD_CONFIG", "1")
	os.Setenv("AWS_SHARED_CREDENTIALS_FILE", testConfigFilename)
	os.Setenv("AWS_PROFILE", "assume_role_w_creds")

	_, err := NewSessionWithOptions(Options{
		Profile:           "assume_role_w_mfa",
		SharedConfigState: SharedConfigEnable,
	})
	if e, a := (AssumeRoleTokenProviderNotSetError{}), err; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
}

func TestSessionAssumeRole_DisableSharedConfig(t *testing.T) {
	// Backwards compatibility with Shared config disabled
	// assume role should not be built into the config.
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	os.Setenv("AWS_SDK_LOAD_CONFIG", "0")
	os.Setenv("AWS_SHARED_CREDENTIALS_FILE", testConfigFilename)
	os.Setenv("AWS_PROFILE", "assume_role_w_creds")

	s, err := NewSession(&aws.Config{
		CredentialsChainVerboseErrors: aws.Bool(true),
	})
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	creds, err := s.Config.Credentials.Get()
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}
	if e, a := "assume_role_w_creds_akid", creds.AccessKeyID; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "assume_role_w_creds_secret", creds.SecretAccessKey; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "SharedConfigCredentials", creds.ProviderName; !strings.Contains(a, e) {
		t.Errorf("expect %v, to be in %v", e, a)
	}
}

func TestSessionAssumeRole_InvalidSourceProfile(t *testing.T) {
	// Backwards compatibility with Shared config disabled
	// assume role should not be built into the config.
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	os.Setenv("AWS_SDK_LOAD_CONFIG", "1")
	os.Setenv("AWS_SHARED_CREDENTIALS_FILE", testConfigFilename)
	os.Setenv("AWS_PROFILE", "assume_role_invalid_source_profile")

	s, err := NewSession()
	if err == nil {
		t.Fatalf("expect error, got none")
	}

	expectMsg := "SharedConfigAssumeRoleError: failed to load assume role"
	if e, a := expectMsg, err.Error(); !strings.Contains(a, e) {
		t.Errorf("expect %v, to be in %v", e, a)
	}
	if s != nil {
		t.Errorf("expect nil, %v", err)
	}
}

func TestSessionAssumeRole_ExtendedDuration(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	cases := []struct {
		profile          string
		optionDuration   time.Duration
		expectedDuration string
	}{
		{
			profile:          "assume_role_w_creds",
			expectedDuration: "900",
		},
		{
			profile:          "assume_role_w_creds",
			optionDuration:   30 * time.Minute,
			expectedDuration: "1800",
		},
		{
			profile:          "assume_role_w_creds_w_duration",
			expectedDuration: "1800",
		},
		{
			profile:          "assume_role_w_creds_w_invalid_duration",
			expectedDuration: "900",
		},
	}

	for _, tt := range cases {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if e, a := tt.expectedDuration, r.FormValue("DurationSeconds"); e != a {
				t.Errorf("expect %v, got %v", e, a)
			}

			w.Write([]byte(fmt.Sprintf(
				assumeRoleRespMsg,
				time.Now().Add(15*time.Minute).Format("2006-01-02T15:04:05Z"))))
		}))

		os.Setenv("AWS_REGION", "us-east-1")
		os.Setenv("AWS_SDK_LOAD_CONFIG", "1")
		os.Setenv("AWS_SHARED_CREDENTIALS_FILE", testConfigFilename)
		os.Setenv("AWS_PROFILE", "assume_role_w_creds")

		opts := Options{
			Profile: tt.profile,
			Config: aws.Config{
				Endpoint:   aws.String(server.URL),
				DisableSSL: aws.Bool(true),
			},
			SharedConfigState: SharedConfigEnable,
		}
		if tt.optionDuration != 0 {
			opts.AssumeRoleDuration = tt.optionDuration
		}

		s, err := NewSessionWithOptions(opts)
		if err != nil {
			server.Close()
			t.Fatalf("expect no error, got %v", err)
		}

		creds, err := s.Config.Credentials.Get()
		if err != nil {
			server.Close()
			t.Fatalf("expect no error, got %v", err)
		}

		if e, a := "AKID", creds.AccessKeyID; e != a {
			t.Errorf("expect %v, got %v", e, a)
		}
		if e, a := "SECRET", creds.SecretAccessKey; e != a {
			t.Errorf("expect %v, got %v", e, a)
		}
		if e, a := "SESSION_TOKEN", creds.SessionToken; e != a {
			t.Errorf("expect %v, got %v", e, a)
		}
		if e, a := "AssumeRoleProvider", creds.ProviderName; !strings.Contains(a, e) {
			t.Errorf("expect %v, to be in %v", e, a)
		}

		server.Close()
	}
}

func TestSessionAssumeRole_WithMFA_ExtendedDuration(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	os.Setenv("AWS_REGION", "us-east-1")
	os.Setenv("AWS_SDK_LOAD_CONFIG", "1")
	os.Setenv("AWS_SHARED_CREDENTIALS_FILE", testConfigFilename)
	os.Setenv("AWS_PROFILE", "assume_role_w_creds")

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if e, a := "0123456789", r.FormValue("SerialNumber"); e != a {
			t.Errorf("expect %v, got %v", e, a)
		}
		if e, a := "tokencode", r.FormValue("TokenCode"); e != a {
			t.Errorf("expect %v, got %v", e, a)
		}
		if e, a := "1800", r.FormValue("DurationSeconds"); e != a {
			t.Errorf("expect %v, got %v", e, a)
		}

		w.Write([]byte(fmt.Sprintf(
			assumeRoleRespMsg,
			time.Now().Add(30*time.Minute).Format("2006-01-02T15:04:05Z"))))
	}))
	defer server.Close()

	customProviderCalled := false
	sess, err := NewSessionWithOptions(Options{
		Profile: "assume_role_w_mfa",
		Config: aws.Config{
			Region:     aws.String("us-east-1"),
			Endpoint:   aws.String(server.URL),
			DisableSSL: aws.Bool(true),
		},
		SharedConfigState:  SharedConfigEnable,
		AssumeRoleDuration: 30 * time.Minute,
		AssumeRoleTokenProvider: func() (string, error) {
			customProviderCalled = true

			return "tokencode", nil
		},
	})
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	creds, err := sess.Config.Credentials.Get()
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}
	if !customProviderCalled {
		t.Errorf("expect true")
	}

	if e, a := "AKID", creds.AccessKeyID; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "SECRET", creds.SecretAccessKey; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "SESSION_TOKEN", creds.SessionToken; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "AssumeRoleProvider", creds.ProviderName; !strings.Contains(a, e) {
		t.Errorf("expect %v, to be in %v", e, a)
	}
}

func TestSessionAssumeRoleWithWebIdentity_Options(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	os.Setenv("AWS_REGION", "us-east-1")
	os.Setenv("AWS_ROLE_ARN", "web_identity_role_arn")
	os.Setenv("AWS_WEB_IDENTITY_TOKEN_FILE", "./testdata/wit.txt")

	endpointResolver, teardown := setupCredentialsEndpoints(t)
	defer teardown()

	optionsCalled := false

	sess, err := NewSessionWithOptions(Options{
		Config: aws.Config{
			EndpointResolver: endpointResolver,
		},
		CredentialsProviderOptions: &CredentialsProviderOptions{
			WebIdentityRoleProviderOptions: func(*stscreds.WebIdentityRoleProvider) {
				optionsCalled = true
			},
		},
	})
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	if !optionsCalled {
		t.Errorf("expect options func to be called")
	}

	creds, err := sess.Config.Credentials.Get()
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	if e, a := "WEB_IDENTITY_AKID", creds.AccessKeyID; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "WEB_IDENTITY_SECRET", creds.SecretAccessKey; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "WEB_IDENTITY_SESSION_TOKEN", creds.SessionToken; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := stscreds.WebIdentityProviderName, creds.ProviderName; e != a {
		t.Errorf("expect %v,got %v", e, a)
	}
}

func ssoTestSetup() (func(), error) {
	dir, err := ioutil.TempDir("", "sso-test")
	if err != nil {
		return nil, err
	}

	cacheDir := filepath.Join(dir, ".aws", "sso", "cache")
	err = os.MkdirAll(cacheDir, 0750)
	if err != nil {
		os.RemoveAll(dir)
		return nil, err
	}

	tokenFile, err := os.Create(filepath.Join(cacheDir, "eb5e43e71ce87dd92ec58903d76debd8ee42aefd.json"))
	if err != nil {
		os.RemoveAll(dir)
		return nil, err
	}
	defer tokenFile.Close()

	_, err = tokenFile.WriteString(fmt.Sprintf(ssoTokenCacheFile, time.Now().
		Add(15*time.Minute).
		Format(time.RFC3339)))
	if err != nil {
		os.RemoveAll(dir)
		return nil, err
	}

	if runtime.GOOS == "windows" {
		os.Setenv("USERPROFILE", dir)
	} else {
		os.Setenv("HOME", dir)
	}

	return func() {
		os.RemoveAll(dir)
	}, nil
}
