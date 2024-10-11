//go:build go1.7
// +build go1.7

package session

import (
	"bytes"
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"reflect"
	"strconv"
	"strings"
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/credentials"
	"github.com/aws/aws-sdk-go/aws/defaults"
	"github.com/aws/aws-sdk-go/aws/endpoints"
	"github.com/aws/aws-sdk-go/aws/request"
	"github.com/aws/aws-sdk-go/service/s3"
)

func TestNewDefaultSession(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	s := New(&aws.Config{Region: aws.String("region")})

	if e, a := "region", *s.Config.Region; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := http.DefaultClient, s.Config.HTTPClient; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if s.Config.Logger == nil {
		t.Errorf("expect not nil")
	}
	if e, a := aws.LogOff, *s.Config.LogLevel; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
}

func TestNew_WithCustomCreds(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	customCreds := credentials.NewStaticCredentials("AKID", "SECRET", "TOKEN")
	s := New(&aws.Config{Credentials: customCreds})

	if e, a := customCreds, s.Config.Credentials; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
}

type mockLogger struct {
	*bytes.Buffer
}

func (w mockLogger) Log(args ...interface{}) {
	fmt.Fprintln(w, args...)
}

func TestNew_WithSessionLoadError(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	os.Setenv("AWS_SDK_LOAD_CONFIG", "1")
	os.Setenv("AWS_CONFIG_FILE", testConfigFilename)
	os.Setenv("AWS_PROFILE", "assume_role_invalid_source_profile")

	logger := bytes.Buffer{}
	s := New(&aws.Config{
		Region: aws.String("us-west-2"),
		Logger: &mockLogger{&logger},
	})

	if s == nil {
		t.Errorf("expect not nil")
	}

	svc := s3.New(s)
	_, err := svc.ListBuckets(&s3.ListBucketsInput{})

	if err == nil {
		t.Errorf("expect not nil")
	}
	if e, a := "ERROR: failed to create session with AWS_SDK_LOAD_CONFIG enabled", logger.String(); !strings.Contains(a, e) {
		t.Errorf("expect %v, to be in %v", e, a)
	}

	expectErr := SharedConfigAssumeRoleError{
		RoleARN:       "assume_role_invalid_source_profile_role_arn",
		SourceProfile: "profile_not_exists",
	}
	if e, a := expectErr.Error(), err.Error(); !strings.Contains(a, e) {
		t.Errorf("expect %v, to be in %v", e, a)
	}
}

func TestSessionCopy(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	os.Setenv("AWS_REGION", "orig_region")

	s := Session{
		Config:   defaults.Config(),
		Handlers: defaults.Handlers(),
	}

	newSess := s.Copy(&aws.Config{Region: aws.String("new_region")})

	if e, a := "orig_region", *s.Config.Region; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "new_region", *newSess.Config.Region; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
}

func TestSessionClientConfig(t *testing.T) {
	s, err := NewSession(&aws.Config{
		Credentials: credentials.AnonymousCredentials,
		Region:      aws.String("orig_region"),
		EndpointResolver: endpoints.ResolverFunc(
			func(service, region string, opts ...func(*endpoints.Options)) (endpoints.ResolvedEndpoint, error) {
				if e, a := "mock-service", service; e != a {
					t.Errorf("expect %q service, got %q", e, a)
				}
				if e, a := "other-region", region; e != a {
					t.Errorf("expect %q region, got %q", e, a)
				}
				return endpoints.ResolvedEndpoint{
					URL:           "https://" + service + "." + region + ".amazonaws.com",
					SigningRegion: region,
				}, nil
			},
		),
	})
	if err != nil {
		t.Errorf("expect nil, %v", err)
	}

	cfg := s.ClientConfig("mock-service", &aws.Config{Region: aws.String("other-region")})

	if e, a := "https://mock-service.other-region.amazonaws.com", cfg.Endpoint; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "other-region", cfg.SigningRegion; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "other-region", *cfg.Config.Region; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
}

func TestNewSession_ResolveEndpointError(t *testing.T) {
	logger := mockLogger{Buffer: bytes.NewBuffer(nil)}
	sess, err := NewSession(defaults.Config(), &aws.Config{
		Region: aws.String(""),
		Logger: logger,
		EndpointResolver: endpoints.ResolverFunc(
			func(service, region string, opts ...func(*endpoints.Options)) (endpoints.ResolvedEndpoint, error) {
				return endpoints.ResolvedEndpoint{}, fmt.Errorf("mock error")
			},
		),
	})
	if err != nil {
		t.Fatalf("expect no error got %v", err)
	}

	cfg := sess.ClientConfig("mock service")

	var r request.Request
	cfg.Handlers.Validate.Run(&r)

	if r.Error == nil {
		t.Fatalf("expect validation error, got none")
	}

	if e, a := aws.ErrMissingRegion.Error(), r.Error.Error(); !strings.Contains(a, e) {
		t.Errorf("expect %v validation error, got %v", e, a)
	}

	if v := logger.Buffer.String(); len(v) != 0 {
		t.Errorf("expect nothing logged, got %s", v)
	}
}

func TestNewSession_NoCredentials(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	s, err := NewSession()
	if err != nil {
		t.Errorf("expect nil, %v", err)
	}

	if s.Config.Credentials == nil {
		t.Errorf("expect not nil")
	}
	if e, a := credentials.AnonymousCredentials, s.Config.Credentials; e == a {
		t.Errorf("expect different credentials, %v", e)
	}
}

func TestNewSessionWithOptions_OverrideProfile(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	os.Setenv("AWS_SDK_LOAD_CONFIG", "1")
	os.Setenv("AWS_SHARED_CREDENTIALS_FILE", testConfigFilename)
	os.Setenv("AWS_PROFILE", "other_profile")

	s, err := NewSessionWithOptions(Options{
		Profile: "full_profile",
	})
	if err != nil {
		t.Errorf("expect nil, %v", err)
	}

	if e, a := "full_profile_region", *s.Config.Region; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}

	creds, err := s.Config.Credentials.Get()
	if err != nil {
		t.Errorf("expect nil, %v", err)
	}
	if e, a := "full_profile_akid", creds.AccessKeyID; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "full_profile_secret", creds.SecretAccessKey; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if v := creds.SessionToken; len(v) != 0 {
		t.Errorf("expect empty, got %v", v)
	}
	if e, a := "SharedConfigCredentials", creds.ProviderName; !strings.Contains(a, e) {
		t.Errorf("expect %v, to be in %v", e, a)
	}
}

func TestNewSessionWithOptions_OverrideSharedConfigEnable(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	os.Setenv("AWS_SDK_LOAD_CONFIG", "0")
	os.Setenv("AWS_SHARED_CREDENTIALS_FILE", testConfigFilename)
	os.Setenv("AWS_PROFILE", "full_profile")

	s, err := NewSessionWithOptions(Options{
		SharedConfigState: SharedConfigEnable,
	})
	if err != nil {
		t.Errorf("expect nil, %v", err)
	}

	if e, a := "full_profile_region", *s.Config.Region; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}

	creds, err := s.Config.Credentials.Get()
	if err != nil {
		t.Errorf("expect nil, %v", err)
	}
	if e, a := "full_profile_akid", creds.AccessKeyID; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "full_profile_secret", creds.SecretAccessKey; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if v := creds.SessionToken; len(v) != 0 {
		t.Errorf("expect empty, got %v", v)
	}
	if e, a := "SharedConfigCredentials", creds.ProviderName; !strings.Contains(a, e) {
		t.Errorf("expect %v, to be in %v", e, a)
	}
}

func TestNewSessionWithOptions_OverrideSharedConfigDisable(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	os.Setenv("AWS_SDK_LOAD_CONFIG", "1")
	os.Setenv("AWS_SHARED_CREDENTIALS_FILE", testConfigFilename)
	os.Setenv("AWS_PROFILE", "full_profile")

	s, err := NewSessionWithOptions(Options{
		SharedConfigState: SharedConfigDisable,
	})
	if err != nil {
		t.Errorf("expect nil, %v", err)
	}

	if v := *s.Config.Region; len(v) != 0 {
		t.Errorf("expect empty, got %v", v)
	}

	creds, err := s.Config.Credentials.Get()
	if err != nil {
		t.Errorf("expect nil, %v", err)
	}
	if e, a := "full_profile_akid", creds.AccessKeyID; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "full_profile_secret", creds.SecretAccessKey; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if v := creds.SessionToken; len(v) != 0 {
		t.Errorf("expect empty, got %v", v)
	}
	if e, a := "SharedConfigCredentials", creds.ProviderName; !strings.Contains(a, e) {
		t.Errorf("expect %v, to be in %v", e, a)
	}
}

func TestNewSessionWithOptions_OverrideSharedConfigFiles(t *testing.T) {
	restoreEnvFn := initSessionTestEnv()
	defer restoreEnvFn()

	os.Setenv("AWS_SDK_LOAD_CONFIG", "1")
	os.Setenv("AWS_SHARED_CREDENTIALS_FILE", testConfigFilename)
	os.Setenv("AWS_PROFILE", "config_file_load_order")

	s, err := NewSessionWithOptions(Options{
		SharedConfigFiles: []string{testConfigOtherFilename},
	})
	if err != nil {
		t.Errorf("expect nil, %v", err)
	}

	if e, a := "shared_config_other_region", *s.Config.Region; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}

	creds, err := s.Config.Credentials.Get()
	if err != nil {
		t.Errorf("expect nil, %v", err)
	}
	if e, a := "shared_config_other_akid", creds.AccessKeyID; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "shared_config_other_secret", creds.SecretAccessKey; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if v := creds.SessionToken; len(v) != 0 {
		t.Errorf("expect empty, got %v", v)
	}
	if e, a := "SharedConfigCredentials", creds.ProviderName; !strings.Contains(a, e) {
		t.Errorf("expect %v, to be in %v", e, a)
	}
}

func TestNewSessionWithOptions_Overrides(t *testing.T) {
	cases := map[string]struct {
		InEnvs    map[string]string
		InProfile string
		OutRegion string
		OutCreds  credentials.Value
	}{
		"env profile with opt profile": {
			InEnvs: map[string]string{
				"AWS_SDK_LOAD_CONFIG":         "0",
				"AWS_SHARED_CREDENTIALS_FILE": testConfigFilename,
				"AWS_PROFILE":                 "other_profile",
			},
			InProfile: "full_profile",
			OutRegion: "full_profile_region",
			OutCreds: credentials.Value{
				AccessKeyID:     "full_profile_akid",
				SecretAccessKey: "full_profile_secret",
				ProviderName:    "SharedConfigCredentials",
			},
		},
		"env creds with env profile": {
			InEnvs: map[string]string{
				"AWS_SDK_LOAD_CONFIG":         "0",
				"AWS_SHARED_CREDENTIALS_FILE": testConfigFilename,
				"AWS_REGION":                  "env_region",
				"AWS_ACCESS_KEY":              "env_akid",
				"AWS_SECRET_ACCESS_KEY":       "env_secret",
				"AWS_PROFILE":                 "other_profile",
			},
			OutRegion: "env_region",
			OutCreds: credentials.Value{
				AccessKeyID:     "env_akid",
				SecretAccessKey: "env_secret",
				ProviderName:    "EnvConfigCredentials",
			},
		},
		"env creds with opt profile": {
			InEnvs: map[string]string{
				"AWS_SDK_LOAD_CONFIG":         "0",
				"AWS_SHARED_CREDENTIALS_FILE": testConfigFilename,
				"AWS_REGION":                  "env_region",
				"AWS_ACCESS_KEY":              "env_akid",
				"AWS_SECRET_ACCESS_KEY":       "env_secret",
				"AWS_PROFILE":                 "other_profile",
			},
			InProfile: "full_profile",
			OutRegion: "env_region",
			OutCreds: credentials.Value{
				AccessKeyID:     "full_profile_akid",
				SecretAccessKey: "full_profile_secret",
				ProviderName:    "SharedConfigCredentials",
			},
		},
		"cfg and cred file with opt profile": {
			InEnvs: map[string]string{
				"AWS_SDK_LOAD_CONFIG":         "0",
				"AWS_SHARED_CREDENTIALS_FILE": testConfigFilename,
				"AWS_CONFIG_FILE":             testConfigOtherFilename,
				"AWS_PROFILE":                 "other_profile",
			},
			InProfile: "config_file_load_order",
			OutRegion: "shared_config_region",
			OutCreds: credentials.Value{
				AccessKeyID:     "shared_config_akid",
				SecretAccessKey: "shared_config_secret",
				ProviderName:    "SharedConfigCredentials",
			},
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			restoreEnvFn := initSessionTestEnv()
			defer restoreEnvFn()

			for k, v := range c.InEnvs {
				os.Setenv(k, v)
			}

			s, err := NewSessionWithOptions(Options{
				Profile:           c.InProfile,
				SharedConfigState: SharedConfigEnable,
			})
			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}

			creds, err := s.Config.Credentials.Get()
			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}
			if e, a := c.OutRegion, *s.Config.Region; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}
			if e, a := c.OutCreds.AccessKeyID, creds.AccessKeyID; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}
			if e, a := c.OutCreds.SecretAccessKey, creds.SecretAccessKey; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}
			if e, a := c.OutCreds.SessionToken, creds.SessionToken; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}
			if e, a := c.OutCreds.ProviderName, creds.ProviderName; !strings.Contains(a, e) {
				t.Errorf("expect %v, to be in %v", e, a)
			}
		})
	}
}

func TestNewSession_EnvCredsWithInvalidConfigFile(t *testing.T) {
	cases := map[string]struct {
		AccessKey, SecretKey string
		Profile              string
		Options              Options
		ExpectCreds          credentials.Value
		Err                  string
	}{
		"no options": {
			Err: "SharedConfigLoadError",
		},
		"env only": {
			AccessKey: "env_akid",
			SecretKey: "env_secret",
			ExpectCreds: credentials.Value{
				AccessKeyID:     "env_akid",
				SecretAccessKey: "env_secret",
				ProviderName:    "EnvConfigCredentials",
			},
		},
		"static credentials only": {
			Options: Options{
				Config: aws.Config{
					Credentials: credentials.NewStaticCredentials(
						"AKID", "SECRET", ""),
				},
			},
			ExpectCreds: credentials.Value{
				AccessKeyID:     "AKID",
				SecretAccessKey: "SECRET",
				ProviderName:    "StaticProvider",
			},
		},
		"env profile and env": {
			AccessKey: "env_akid",
			SecretKey: "env_secret",
			Profile:   "env_profile",
			Err:       "SharedConfigLoadError",
		},
		"opt profile and env": {
			AccessKey: "env_akid",
			SecretKey: "env_secret",
			Options: Options{
				Profile: "someProfile",
			},
			Err: "SharedConfigLoadError",
		},
		"cfg enabled": {
			AccessKey: "env_akid",
			SecretKey: "env_secret",
			Options: Options{
				SharedConfigState: SharedConfigEnable,
			},
			Err: "SharedConfigLoadError",
		},
	}

	var cfgFile = filepath.Join("testdata", "shared_config_invalid_ini")

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			restoreEnvFn := initSessionTestEnv()
			defer restoreEnvFn()

			if v := c.AccessKey; len(v) != 0 {
				os.Setenv("AWS_ACCESS_KEY", v)
			}
			if v := c.SecretKey; len(v) != 0 {
				os.Setenv("AWS_SECRET_ACCESS_KEY", v)
			}
			if v := c.Profile; len(v) != 0 {
				os.Setenv("AWS_PROFILE", v)
			}

			opts := c.Options
			opts.SharedConfigFiles = []string{cfgFile}
			s, err := NewSessionWithOptions(opts)
			if len(c.Err) != 0 {
				if err == nil {
					t.Fatalf("expect session error, got none")
				}
				if e, a := c.Err, err.Error(); !strings.Contains(a, e) {
					t.Fatalf("expect session error to contain %q, got %v", e, a)
				}
				return
			}

			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}

			creds, err := s.Config.Credentials.Get()
			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}
			if e, a := c.ExpectCreds.AccessKeyID, creds.AccessKeyID; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}
			if e, a := c.ExpectCreds.SecretAccessKey, creds.SecretAccessKey; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}
			if e, a := c.ExpectCreds.ProviderName, creds.ProviderName; !strings.Contains(a, e) {
				t.Errorf("expect %v, to be in %v", e, a)
			}
		})
	}
}

func TestSession_RegionalEndpoints(t *testing.T) {
	cases := map[string]struct {
		Env    map[string]string
		Config aws.Config

		ExpectErr       string
		ExpectSTS       endpoints.STSRegionalEndpoint
		ExpectS3UsEast1 endpoints.S3UsEast1RegionalEndpoint
	}{
		"default": {
			ExpectSTS:       endpoints.LegacySTSEndpoint,
			ExpectS3UsEast1: endpoints.LegacyS3UsEast1Endpoint,
		},
		"enable regional": {
			Config: aws.Config{
				STSRegionalEndpoint:       endpoints.RegionalSTSEndpoint,
				S3UsEast1RegionalEndpoint: endpoints.RegionalS3UsEast1Endpoint,
			},
			ExpectSTS:       endpoints.RegionalSTSEndpoint,
			ExpectS3UsEast1: endpoints.RegionalS3UsEast1Endpoint,
		},
		"sts env enable": {
			Env: map[string]string{
				"AWS_STS_REGIONAL_ENDPOINTS": "regional",
			},
			ExpectSTS:       endpoints.RegionalSTSEndpoint,
			ExpectS3UsEast1: endpoints.LegacyS3UsEast1Endpoint,
		},
		"sts us-east-1 env merge enable": {
			Env: map[string]string{
				"AWS_STS_REGIONAL_ENDPOINTS": "legacy",
			},
			Config: aws.Config{
				STSRegionalEndpoint: endpoints.RegionalSTSEndpoint,
			},
			ExpectSTS:       endpoints.RegionalSTSEndpoint,
			ExpectS3UsEast1: endpoints.LegacyS3UsEast1Endpoint,
		},
		"s3 us-east-1 env enable": {
			Env: map[string]string{
				"AWS_S3_US_EAST_1_REGIONAL_ENDPOINT": "regional",
			},
			ExpectSTS:       endpoints.LegacySTSEndpoint,
			ExpectS3UsEast1: endpoints.RegionalS3UsEast1Endpoint,
		},
		"s3 us-east-1 env merge enable": {
			Env: map[string]string{
				"AWS_S3_US_EAST_1_REGIONAL_ENDPOINT": "legacy",
			},
			Config: aws.Config{
				S3UsEast1RegionalEndpoint: endpoints.RegionalS3UsEast1Endpoint,
			},
			ExpectSTS:       endpoints.LegacySTSEndpoint,
			ExpectS3UsEast1: endpoints.RegionalS3UsEast1Endpoint,
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			restoreEnvFn := initSessionTestEnv()
			defer restoreEnvFn()

			for k, v := range c.Env {
				os.Setenv(k, v)
			}

			s, err := NewSession(&c.Config)
			if len(c.ExpectErr) != 0 {
				if err == nil {
					t.Fatalf("expect session error, got none")
				}
				if e, a := c.ExpectErr, err.Error(); !strings.Contains(a, e) {
					t.Fatalf("expect session error to contain %q, got %v", e, a)
				}
				return
			}

			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}

			if e, a := c.ExpectSTS, s.Config.STSRegionalEndpoint; e != a {
				t.Errorf("expect %v STSRegionalEndpoint, got %v", e, a)
			}

			if e, a := c.ExpectS3UsEast1, s.Config.S3UsEast1RegionalEndpoint; e != a {
				t.Errorf("expect %v S3UsEast1RegionalEndpoint, got %v", e, a)
			}

			// Asserts
		})
	}
}

func TestSession_ClientConfig_ResolveEndpoint(t *testing.T) {
	cases := map[string]struct {
		Service        string
		Region         string
		Env            map[string]string
		Options        Options
		ExpectEndpoint string
	}{
		"IMDS custom endpoint from env": {
			Service: ec2MetadataServiceID,
			Region:  "ignored",
			Env: map[string]string{
				"AWS_EC2_METADATA_SERVICE_ENDPOINT": "http://example.aws",
			},
			ExpectEndpoint: "http://example.aws",
		},
		"IMDS custom endpoint from aws.Config": {
			Service: ec2MetadataServiceID,
			Region:  "ignored",
			Options: Options{
				EC2IMDSEndpoint: "http://example.aws",
			},
			ExpectEndpoint: "http://example.aws",
		},
		"IMDS custom endpoint from aws.Config and env": {
			Service: ec2MetadataServiceID,
			Region:  "ignored",
			Env: map[string]string{
				"AWS_EC2_METADATA_SERVICE_ENDPOINT": "http://wrong.example.aws",
			},
			Options: Options{
				EC2IMDSEndpoint: "http://correct.example.aws",
			},
			ExpectEndpoint: "http://correct.example.aws",
		},
		"IMDS custom endpoint from profile": {
			Service: ec2MetadataServiceID,
			Options: Options{
				SharedConfigState: SharedConfigEnable,
				SharedConfigFiles: []string{testConfigFilename},
				Profile:           "EC2MetadataServiceEndpoint",
			},
			Region:         "ignored",
			ExpectEndpoint: "http://endpoint.localhost",
		},
		"IMDS custom endpoint from profile and env": {
			Service: ec2MetadataServiceID,
			Options: Options{
				SharedConfigFiles: []string{testConfigFilename},
				Profile:           "EC2MetadataServiceEndpoint",
			},
			Env: map[string]string{
				"AWS_EC2_METADATA_SERVICE_ENDPOINT": "http://endpoint-env.localhost",
			},
			Region:         "ignored",
			ExpectEndpoint: "http://endpoint-env.localhost",
		},
		"IMDS IPv6 mode from profile": {
			Service: ec2MetadataServiceID,
			Options: Options{
				SharedConfigState: SharedConfigEnable,
				SharedConfigFiles: []string{testConfigFilename},
				Profile:           "EC2MetadataServiceEndpointModeIPv6",
			},
			Region:         "ignored",
			ExpectEndpoint: "http://[fd00:ec2::254]/latest",
		},
		"IMDS IPv4 mode from profile": {
			Service: ec2MetadataServiceID,
			Options: Options{
				SharedConfigFiles: []string{testConfigFilename},
				Profile:           "EC2MetadataServiceEndpointModeIPv4",
			},
			Region:         "ignored",
			ExpectEndpoint: "http://169.254.169.254/latest",
		},
		"IMDS IPv6 mode in env": {
			Service: ec2MetadataServiceID,
			Env: map[string]string{
				"AWS_EC2_METADATA_SERVICE_ENDPOINT_MODE": "IPv6",
			},
			Region:         "ignored",
			ExpectEndpoint: "http://[fd00:ec2::254]/latest",
		},
		"IMDS IPv4 mode in env": {
			Service: ec2MetadataServiceID,
			Env: map[string]string{
				"AWS_EC2_METADATA_SERVICE_ENDPOINT_MODE": "IPv4",
			},
			Region:         "ignored",
			ExpectEndpoint: "http://169.254.169.254/latest",
		},
		"IMDS mode in env and profile": {
			Service: ec2MetadataServiceID,
			Options: Options{
				SharedConfigFiles: []string{testConfigFilename},
				Profile:           "EC2MetadataServiceEndpointModeIPv4",
			},
			Env: map[string]string{
				"AWS_EC2_METADATA_SERVICE_ENDPOINT_MODE": "IPv6",
			},
			Region:         "ignored",
			ExpectEndpoint: "http://[fd00:ec2::254]/latest",
		},
		"IMDS mode and endpoint profile": {
			Service: ec2MetadataServiceID,
			Options: Options{
				SharedConfigState: SharedConfigEnable,
				SharedConfigFiles: []string{testConfigFilename},
				Profile:           "EC2MetadataServiceEndpointAndModeMixed",
			},
			Region:         "ignored",
			ExpectEndpoint: "http://endpoint.localhost",
		},
		"IMDS mode session option and env": {
			Service: ec2MetadataServiceID,
			Options: Options{
				EC2IMDSEndpointMode: endpoints.EC2IMDSEndpointModeStateIPv6,
			},
			Env: map[string]string{
				"AWS_EC2_METADATA_SERVICE_ENDPOINT_MODE": "IPv4",
			},
			Region:         "ignored",
			ExpectEndpoint: "http://[fd00:ec2::254]/latest",
		},
		"IMDS endpoint session option and env": {
			Service: ec2MetadataServiceID,
			Options: Options{
				EC2IMDSEndpoint: "http://endpoint.localhost",
			},
			Env: map[string]string{
				"AWS_EC2_METADATA_SERVICE_ENDPOINT": "http://endpoint-env.localhost",
			},
			Region:         "ignored",
			ExpectEndpoint: "http://endpoint.localhost",
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			restoreEnvFn := initSessionTestEnv()
			defer restoreEnvFn()

			for k, v := range c.Env {
				os.Setenv(k, v)
			}

			s, err := NewSessionWithOptions(c.Options)
			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}

			clientCfg := s.ClientConfig(c.Service, &aws.Config{
				Region: aws.String(c.Region),
			})

			if e, a := c.ExpectEndpoint, clientCfg.Endpoint; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}
		})
	}
}

func TestNormalizeRegion(t *testing.T) {
	cases := []struct {
		Config                 aws.Config
		ExpectedConfig         aws.Config
		ExpectedResolvedRegion string
	}{
		{
			Config: aws.Config{
				Region: aws.String("us-west-2"),
			},
			ExpectedConfig: aws.Config{
				Region: aws.String("us-west-2"),
			},
			ExpectedResolvedRegion: "",
		},
		{
			Config: aws.Config{
				Region:          aws.String("us-west-2"),
				UseFIPSEndpoint: endpoints.FIPSEndpointStateEnabled,
			},
			ExpectedConfig: aws.Config{
				Region:          aws.String("us-west-2"),
				UseFIPSEndpoint: endpoints.FIPSEndpointStateEnabled,
			},
			ExpectedResolvedRegion: "",
		},
		{
			Config: aws.Config{
				Region:          aws.String("fips-us-west-2"),
				UseFIPSEndpoint: endpoints.FIPSEndpointStateDisabled,
			},
			ExpectedConfig: aws.Config{
				Region:          aws.String("fips-us-west-2"),
				UseFIPSEndpoint: endpoints.FIPSEndpointStateEnabled,
			},
			ExpectedResolvedRegion: "us-west-2",
		},
		{
			Config: aws.Config{
				Region: aws.String("fips-us-west-2"),
			},
			ExpectedConfig: aws.Config{
				Region:          aws.String("fips-us-west-2"),
				UseFIPSEndpoint: endpoints.FIPSEndpointStateEnabled,
			},
			ExpectedResolvedRegion: "us-west-2",
		},
		{
			Config: aws.Config{
				Region: aws.String("us-west-2-fips"),
			},
			ExpectedConfig: aws.Config{
				Region:          aws.String("us-west-2-fips"),
				UseFIPSEndpoint: endpoints.FIPSEndpointStateEnabled,
			},
			ExpectedResolvedRegion: "us-west-2",
		},
		{
			Config: aws.Config{
				Region: aws.String("us-west-2-fips-thing"),
			},
			ExpectedConfig: aws.Config{
				Region:          aws.String("us-west-2-fips-thing"),
				UseFIPSEndpoint: endpoints.FIPSEndpointStateEnabled,
			},
			ExpectedResolvedRegion: "us-west-2-thing",
		},
	}

	for i, tt := range cases {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			resolved := normalizeRegion(&tt.Config)
			if !reflect.DeepEqual(tt.ExpectedConfig, tt.Config) {
				t.Errorf("expect %v, got %v", tt.ExpectedConfig, tt.Config)
			}
			if e, a := tt.ExpectedResolvedRegion, resolved; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}
		})
	}
}
