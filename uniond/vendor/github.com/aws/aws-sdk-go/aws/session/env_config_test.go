//go:build go1.7
// +build go1.7

package session

import (
	"os"
	"reflect"
	"strconv"
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/credentials"
	"github.com/aws/aws-sdk-go/aws/endpoints"
	"github.com/aws/aws-sdk-go/awstesting"
	"github.com/aws/aws-sdk-go/internal/sdktesting"
	"github.com/aws/aws-sdk-go/internal/shareddefaults"
)

func TestLoadEnvConfig_Creds(t *testing.T) {
	cases := []struct {
		Env map[string]string
		Val credentials.Value
	}{
		{
			Env: map[string]string{
				"AWS_ACCESS_KEY": "AKID",
			},
			Val: credentials.Value{},
		},
		{
			Env: map[string]string{
				"AWS_ACCESS_KEY_ID": "AKID",
			},
			Val: credentials.Value{},
		},
		{
			Env: map[string]string{
				"AWS_SECRET_KEY": "SECRET",
			},
			Val: credentials.Value{},
		},
		{
			Env: map[string]string{
				"AWS_SECRET_ACCESS_KEY": "SECRET",
			},
			Val: credentials.Value{},
		},
		{
			Env: map[string]string{
				"AWS_ACCESS_KEY_ID":     "AKID",
				"AWS_SECRET_ACCESS_KEY": "SECRET",
			},
			Val: credentials.Value{
				AccessKeyID: "AKID", SecretAccessKey: "SECRET",
				ProviderName: "EnvConfigCredentials",
			},
		},
		{
			Env: map[string]string{
				"AWS_ACCESS_KEY": "AKID",
				"AWS_SECRET_KEY": "SECRET",
			},
			Val: credentials.Value{
				AccessKeyID: "AKID", SecretAccessKey: "SECRET",
				ProviderName: "EnvConfigCredentials",
			},
		},
		{
			Env: map[string]string{
				"AWS_ACCESS_KEY":    "AKID",
				"AWS_SECRET_KEY":    "SECRET",
				"AWS_SESSION_TOKEN": "TOKEN",
			},
			Val: credentials.Value{
				AccessKeyID: "AKID", SecretAccessKey: "SECRET", SessionToken: "TOKEN",
				ProviderName: "EnvConfigCredentials",
			},
		},
	}

	for i, c := range cases {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			restoreEnvFn := sdktesting.StashEnv()
			defer restoreEnvFn()
			for k, v := range c.Env {
				os.Setenv(k, v)
			}

			cfg, err := loadEnvConfig()
			if err != nil {
				t.Fatalf("failed to load env config, %v", err)
			}
			if !reflect.DeepEqual(c.Val, cfg.Creds) {
				t.Errorf("expect credentials to match.\n%s",
					awstesting.SprintExpectActual(c.Val, cfg.Creds))
			}
		})

	}
}

func TestLoadEnvConfig(t *testing.T) {
	restoreEnvFn := sdktesting.StashEnv()
	defer restoreEnvFn()

	cases := []struct {
		Env                 map[string]string
		UseSharedConfigCall bool
		Config              envConfig
		WantErr             bool
	}{
		0: {
			Env: map[string]string{
				"AWS_REGION":  "region",
				"AWS_PROFILE": "profile",
			},
			Config: envConfig{
				Region: "region", Profile: "profile",
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		1: {
			Env: map[string]string{
				"AWS_REGION":          "region",
				"AWS_DEFAULT_REGION":  "default_region",
				"AWS_PROFILE":         "profile",
				"AWS_DEFAULT_PROFILE": "default_profile",
			},
			Config: envConfig{
				Region: "region", Profile: "profile",
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		2: {
			Env: map[string]string{
				"AWS_REGION":          "region",
				"AWS_DEFAULT_REGION":  "default_region",
				"AWS_PROFILE":         "profile",
				"AWS_DEFAULT_PROFILE": "default_profile",
				"AWS_SDK_LOAD_CONFIG": "1",
			},
			Config: envConfig{
				Region: "region", Profile: "profile",
				EnableSharedConfig:    true,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		3: {
			Env: map[string]string{
				"AWS_DEFAULT_REGION":  "default_region",
				"AWS_DEFAULT_PROFILE": "default_profile",
			},
			Config: envConfig{
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		4: {
			Env: map[string]string{
				"AWS_DEFAULT_REGION":  "default_region",
				"AWS_DEFAULT_PROFILE": "default_profile",
				"AWS_SDK_LOAD_CONFIG": "1",
			},
			Config: envConfig{
				Region: "default_region", Profile: "default_profile",
				EnableSharedConfig:    true,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		5: {
			Env: map[string]string{
				"AWS_REGION":  "region",
				"AWS_PROFILE": "profile",
			},
			Config: envConfig{
				Region: "region", Profile: "profile",
				EnableSharedConfig:    true,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
			UseSharedConfigCall: true,
		},
		6: {
			Env: map[string]string{
				"AWS_REGION":          "region",
				"AWS_DEFAULT_REGION":  "default_region",
				"AWS_PROFILE":         "profile",
				"AWS_DEFAULT_PROFILE": "default_profile",
			},
			Config: envConfig{
				Region: "region", Profile: "profile",
				EnableSharedConfig:    true,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
			UseSharedConfigCall: true,
		},
		7: {
			Env: map[string]string{
				"AWS_REGION":          "region",
				"AWS_DEFAULT_REGION":  "default_region",
				"AWS_PROFILE":         "profile",
				"AWS_DEFAULT_PROFILE": "default_profile",
				"AWS_SDK_LOAD_CONFIG": "1",
			},
			Config: envConfig{
				Region: "region", Profile: "profile",
				EnableSharedConfig:    true,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
			UseSharedConfigCall: true,
		},
		8: {
			Env: map[string]string{
				"AWS_DEFAULT_REGION":  "default_region",
				"AWS_DEFAULT_PROFILE": "default_profile",
			},
			Config: envConfig{
				Region: "default_region", Profile: "default_profile",
				EnableSharedConfig:    true,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
			UseSharedConfigCall: true,
		},
		9: {
			Env: map[string]string{
				"AWS_DEFAULT_REGION":  "default_region",
				"AWS_DEFAULT_PROFILE": "default_profile",
				"AWS_SDK_LOAD_CONFIG": "1",
			},
			Config: envConfig{
				Region: "default_region", Profile: "default_profile",
				EnableSharedConfig:    true,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
			UseSharedConfigCall: true,
		},
		10: {
			Env: map[string]string{
				"AWS_CA_BUNDLE": "custom_ca_bundle",
			},
			Config: envConfig{
				CustomCABundle:        "custom_ca_bundle",
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		11: {
			Env: map[string]string{
				"AWS_CA_BUNDLE": "custom_ca_bundle",
			},
			Config: envConfig{
				CustomCABundle:        "custom_ca_bundle",
				EnableSharedConfig:    true,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
			UseSharedConfigCall: true,
		},
		12: {
			Env: map[string]string{
				"AWS_SDK_GO_CLIENT_TLS_CERT": "client_tls_cert",
			},
			Config: envConfig{
				ClientTLSCert:         "client_tls_cert",
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		13: {
			Env: map[string]string{
				"AWS_SDK_GO_CLIENT_TLS_CERT": "client_tls_cert",
			},
			Config: envConfig{
				ClientTLSCert:         "client_tls_cert",
				EnableSharedConfig:    true,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
			UseSharedConfigCall: true,
		},
		14: {
			Env: map[string]string{
				"AWS_SDK_GO_CLIENT_TLS_KEY": "client_tls_key",
			},
			Config: envConfig{
				ClientTLSKey:          "client_tls_key",
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		15: {
			Env: map[string]string{
				"AWS_SDK_GO_CLIENT_TLS_KEY": "client_tls_key",
			},
			Config: envConfig{
				ClientTLSKey:          "client_tls_key",
				EnableSharedConfig:    true,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
			UseSharedConfigCall: true,
		},
		16: {
			Env: map[string]string{
				"AWS_SHARED_CREDENTIALS_FILE": "/path/to/credentials/file",
				"AWS_CONFIG_FILE":             "/path/to/config/file",
			},
			Config: envConfig{
				SharedCredentialsFile: "/path/to/credentials/file",
				SharedConfigFile:      "/path/to/config/file",
			},
		},
		17: {
			Env: map[string]string{
				"AWS_STS_REGIONAL_ENDPOINTS": "regional",
			},
			Config: envConfig{
				STSRegionalEndpoint:   endpoints.RegionalSTSEndpoint,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		18: {
			Env: map[string]string{
				"AWS_S3_US_EAST_1_REGIONAL_ENDPOINT": "regional",
			},
			Config: envConfig{
				S3UsEast1RegionalEndpoint: endpoints.RegionalS3UsEast1Endpoint,
				SharedCredentialsFile:     shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:          shareddefaults.SharedConfigFilename(),
			},
		},
		19: {
			Env: map[string]string{
				"AWS_S3_USE_ARN_REGION": "true",
			},
			Config: envConfig{
				S3UseARNRegion:        true,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		20: {
			Env: map[string]string{
				"AWS_EC2_METADATA_SERVICE_ENDPOINT": "http://example.aws",
			},
			Config: envConfig{
				EC2IMDSEndpoint:       "http://example.aws",
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		21: {
			Env: map[string]string{
				"AWS_EC2_METADATA_SERVICE_ENDPOINT_MODE": "IPv6",
			},
			Config: envConfig{
				EC2IMDSEndpointMode:   endpoints.EC2IMDSEndpointModeStateIPv6,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		22: {
			Env: map[string]string{
				"AWS_EC2_METADATA_SERVICE_ENDPOINT_MODE": "IPv4",
			},
			Config: envConfig{
				EC2IMDSEndpointMode:   endpoints.EC2IMDSEndpointModeStateIPv4,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		23: {
			Env: map[string]string{
				"AWS_EC2_METADATA_SERVICE_ENDPOINT_MODE": "foobar",
			},
			WantErr: true,
		},
		24: {
			Env: map[string]string{
				"AWS_EC2_METADATA_SERVICE_ENDPOINT": "http://endpoint.localhost",
			},
			Config: envConfig{
				EC2IMDSEndpoint:       "http://endpoint.localhost",
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		25: {
			Env: map[string]string{
				"AWS_EC2_METADATA_SERVICE_ENDPOINT_MODE": "IPv6",
				"AWS_EC2_METADATA_SERVICE_ENDPOINT":      "http://endpoint.localhost",
			},
			Config: envConfig{
				EC2IMDSEndpoint:       "http://endpoint.localhost",
				EC2IMDSEndpointMode:   endpoints.EC2IMDSEndpointModeStateIPv6,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		26: {
			Env: map[string]string{
				"AWS_EC2_METADATA_DISABLED": "false",
			},
			Config: envConfig{
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		27: {
			Env: map[string]string{
				"AWS_USE_DUALSTACK_ENDPOINT": "true",
			},
			Config: envConfig{
				UseDualStackEndpoint:  endpoints.DualStackEndpointStateEnabled,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		28: {
			Env: map[string]string{
				"AWS_USE_DUALSTACK_ENDPOINT": "false",
			},
			Config: envConfig{
				UseDualStackEndpoint:  endpoints.DualStackEndpointStateDisabled,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		29: {
			Env: map[string]string{
				"AWS_USE_DUALSTACK_ENDPOINT": "invalid",
			},
			WantErr: true,
		},
		30: {
			Env: map[string]string{
				"AWS_USE_FIPS_ENDPOINT": "true",
			},
			Config: envConfig{
				UseFIPSEndpoint:       endpoints.FIPSEndpointStateEnabled,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		31: {
			Env: map[string]string{
				"AWS_USE_FIPS_ENDPOINT": "false",
			},
			Config: envConfig{
				UseFIPSEndpoint:       endpoints.FIPSEndpointStateDisabled,
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
		32: {
			Env: map[string]string{
				"AWS_USE_FIPS_ENDPOINT": "invalid",
			},
			WantErr: true,
		},
		33: {
			Env: map[string]string{
				"AWS_EC2_METADATA_V1_DISABLED": "fAlSe",
			},
			Config: envConfig{
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
				EC2IMDSv1Disabled:     aws.Bool(false),
			},
		},
		34: {
			Env: map[string]string{
				"AWS_EC2_METADATA_V1_DISABLED": "tRuE",
			},
			Config: envConfig{
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
				EC2IMDSv1Disabled:     aws.Bool(true),
			},
		},
		35: {
			Env: map[string]string{
				"AWS_EC2_METADATA_V1_DISABLED": "invalid",
			},
			Config: envConfig{
				SharedCredentialsFile: shareddefaults.SharedCredentialsFilename(),
				SharedConfigFile:      shareddefaults.SharedConfigFilename(),
			},
		},
	}

	for i, c := range cases {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			restoreEnvFn = sdktesting.StashEnv()
			defer restoreEnvFn()
			for k, v := range c.Env {
				os.Setenv(k, v)
			}

			var cfg envConfig
			var err error
			if c.UseSharedConfigCall {
				cfg, err = loadSharedEnvConfig()
				if (err != nil) != c.WantErr {
					t.Errorf("WantErr=%v, got err=%v", c.WantErr, err)
					return
				} else if c.WantErr {
					return
				}
			} else {
				cfg, err = loadEnvConfig()
				if (err != nil) != c.WantErr {
					t.Errorf("WantErr=%v, got err=%v", c.WantErr, err)
					return
				} else if c.WantErr {
					return
				}
			}

			if !reflect.DeepEqual(c.Config, cfg) {
				t.Errorf("expect config to match.\n%s",
					awstesting.SprintExpectActual(c.Config, cfg))
			}
		})
	}
}

func TestSetEnvValue(t *testing.T) {
	restoreEnvFn := sdktesting.StashEnv()
	defer restoreEnvFn()

	os.Setenv("empty_key", "")
	os.Setenv("second_key", "2")
	os.Setenv("third_key", "3")

	var dst string
	setFromEnvVal(&dst, []string{
		"empty_key", "first_key", "second_key", "third_key",
	})

	if e, a := "2", dst; e != a {
		t.Errorf("expect %s value from environment, got %s", e, a)
	}
}
