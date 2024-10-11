//go:build go1.7
// +build go1.7

package session

import (
	"fmt"
	"path/filepath"
	"reflect"
	"strconv"
	"strings"
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/credentials"
	"github.com/aws/aws-sdk-go/aws/endpoints"
	"github.com/aws/aws-sdk-go/internal/ini"
)

var (
	testConfigFilename      = filepath.Join("testdata", "shared_config")
	testConfigOtherFilename = filepath.Join("testdata", "shared_config_other")
)

func TestLoadSharedConfig(t *testing.T) {
	cases := []struct {
		Filenames []string
		Profile   string
		Expected  sharedConfig
		Err       error
	}{
		{
			Filenames: []string{"file_not_exists"},
			Profile:   "default",
			Expected: sharedConfig{
				Profile: "default",
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Expected: sharedConfig{
				Profile: "default",
				Region:  "default_region",
			},
		},
		{
			Filenames: []string{testConfigOtherFilename, testConfigFilename},
			Profile:   "config_file_load_order",
			Expected: sharedConfig{
				Profile: "config_file_load_order",
				Region:  "shared_config_region",
				Creds: credentials.Value{
					AccessKeyID:     "shared_config_akid",
					SecretAccessKey: "shared_config_secret",
					ProviderName:    fmt.Sprintf("SharedConfigCredentials: %s", testConfigFilename),
				},
			},
		},
		{
			Filenames: []string{testConfigFilename, testConfigOtherFilename},
			Profile:   "config_file_load_order",
			Expected: sharedConfig{
				Profile: "config_file_load_order",
				Region:  "shared_config_other_region",
				Creds: credentials.Value{
					AccessKeyID:     "shared_config_other_akid",
					SecretAccessKey: "shared_config_other_secret",
					ProviderName:    fmt.Sprintf("SharedConfigCredentials: %s", testConfigOtherFilename),
				},
			},
		},
		{
			Filenames: []string{testConfigOtherFilename, testConfigFilename},
			Profile:   "assume_role",
			Expected: sharedConfig{
				Profile:           "assume_role",
				RoleARN:           "assume_role_role_arn",
				SourceProfileName: "complete_creds",
				SourceProfile: &sharedConfig{
					Profile: "complete_creds",
					Creds: credentials.Value{
						AccessKeyID:     "complete_creds_akid",
						SecretAccessKey: "complete_creds_secret",
						ProviderName:    fmt.Sprintf("SharedConfigCredentials: %s", testConfigFilename),
					},
				},
			},
		},
		{
			Filenames: []string{testConfigOtherFilename, testConfigFilename},
			Profile:   "assume_role_invalid_source_profile",
			Expected: sharedConfig{
				Profile:           "assume_role_invalid_source_profile",
				RoleARN:           "assume_role_invalid_source_profile_role_arn",
				SourceProfileName: "profile_not_exists",
			},
			Err: SharedConfigAssumeRoleError{
				RoleARN:       "assume_role_invalid_source_profile_role_arn",
				SourceProfile: "profile_not_exists",
			},
		},
		{
			Filenames: []string{testConfigOtherFilename, testConfigFilename},
			Profile:   "assume_role_w_creds",
			Expected: sharedConfig{
				Profile:           "assume_role_w_creds",
				RoleARN:           "assume_role_w_creds_role_arn",
				ExternalID:        "1234",
				RoleSessionName:   "assume_role_w_creds_session_name",
				SourceProfileName: "assume_role_w_creds",
				SourceProfile: &sharedConfig{
					Profile: "assume_role_w_creds",
					Creds: credentials.Value{
						AccessKeyID:     "assume_role_w_creds_akid",
						SecretAccessKey: "assume_role_w_creds_secret",
						ProviderName:    fmt.Sprintf("SharedConfigCredentials: %s", testConfigFilename),
					},
				},
			},
		},
		{
			Filenames: []string{testConfigOtherFilename, testConfigFilename},
			Profile:   "assume_role_wo_creds",
			Expected: sharedConfig{
				Profile:           "assume_role_wo_creds",
				RoleARN:           "assume_role_wo_creds_role_arn",
				SourceProfileName: "assume_role_wo_creds",
			},
			Err: SharedConfigAssumeRoleError{
				RoleARN:       "assume_role_wo_creds_role_arn",
				SourceProfile: "assume_role_wo_creds",
			},
		},
		{
			Filenames: []string{filepath.Join("testdata", "shared_config_invalid_ini")},
			Profile:   "profile_name",
			Err:       SharedConfigLoadError{Filename: filepath.Join("testdata", "shared_config_invalid_ini")},
		},
		{
			Filenames: []string{testConfigOtherFilename, testConfigFilename},
			Profile:   "assume_role_with_credential_source",
			Expected: sharedConfig{
				Profile:          "assume_role_with_credential_source",
				RoleARN:          "assume_role_with_credential_source_role_arn",
				CredentialSource: credSourceEc2Metadata,
			},
		},
		{
			Filenames: []string{testConfigOtherFilename, testConfigFilename},
			Profile:   "multiple_assume_role",
			Expected: sharedConfig{
				Profile:           "multiple_assume_role",
				RoleARN:           "multiple_assume_role_role_arn",
				SourceProfileName: "assume_role",
				SourceProfile: &sharedConfig{
					Profile:           "assume_role",
					RoleARN:           "assume_role_role_arn",
					SourceProfileName: "complete_creds",
					SourceProfile: &sharedConfig{
						Profile: "complete_creds",
						Creds: credentials.Value{
							AccessKeyID:     "complete_creds_akid",
							SecretAccessKey: "complete_creds_secret",
							ProviderName:    fmt.Sprintf("SharedConfigCredentials: %s", testConfigFilename),
						},
					},
				},
			},
		},
		{
			Filenames: []string{testConfigOtherFilename, testConfigFilename},
			Profile:   "multiple_assume_role_with_credential_source",
			Expected: sharedConfig{
				Profile:           "multiple_assume_role_with_credential_source",
				RoleARN:           "multiple_assume_role_with_credential_source_role_arn",
				SourceProfileName: "assume_role_with_credential_source",
				SourceProfile: &sharedConfig{
					Profile:          "assume_role_with_credential_source",
					RoleARN:          "assume_role_with_credential_source_role_arn",
					CredentialSource: credSourceEc2Metadata,
				},
			},
		},
		{
			Filenames: []string{testConfigOtherFilename, testConfigFilename},
			Profile:   "multiple_assume_role_with_credential_source2",
			Expected: sharedConfig{
				Profile:           "multiple_assume_role_with_credential_source2",
				RoleARN:           "multiple_assume_role_with_credential_source2_role_arn",
				SourceProfileName: "multiple_assume_role_with_credential_source",
				SourceProfile: &sharedConfig{
					Profile:           "multiple_assume_role_with_credential_source",
					RoleARN:           "multiple_assume_role_with_credential_source_role_arn",
					SourceProfileName: "assume_role_with_credential_source",
					SourceProfile: &sharedConfig{
						Profile:          "assume_role_with_credential_source",
						RoleARN:          "assume_role_with_credential_source_role_arn",
						CredentialSource: credSourceEc2Metadata,
					},
				},
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "with_sts_regional",
			Expected: sharedConfig{
				Profile:             "with_sts_regional",
				STSRegionalEndpoint: endpoints.RegionalSTSEndpoint,
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "with_s3_us_east_1_regional",
			Expected: sharedConfig{
				Profile:                   "with_s3_us_east_1_regional",
				S3UsEast1RegionalEndpoint: endpoints.RegionalS3UsEast1Endpoint,
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "sso_creds",
			Expected: sharedConfig{
				Profile:      "sso_creds",
				SSOAccountID: "012345678901",
				SSORegion:    "us-west-2",
				SSORoleName:  "TestRole",
				SSOStartURL:  "https://127.0.0.1/start",
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "source_sso_creds",
			Expected: sharedConfig{
				Profile:           "source_sso_creds",
				RoleARN:           "source_sso_creds_arn",
				SourceProfileName: "sso_creds",
				SourceProfile: &sharedConfig{
					Profile:      "sso_creds",
					SSOAccountID: "012345678901",
					SSORegion:    "us-west-2",
					SSORoleName:  "TestRole",
					SSOStartURL:  "https://127.0.0.1/start",
				},
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "sso_and_static",
			Expected: sharedConfig{
				Profile: "sso_and_static",
				Creds: credentials.Value{
					AccessKeyID:     "sso_and_static_akid",
					SecretAccessKey: "sso_and_static_secret",
					SessionToken:    "sso_and_static_token",
					ProviderName:    fmt.Sprintf("SharedConfigCredentials: %s", testConfigFilename),
				},
				SSOAccountID: "012345678901",
				SSORegion:    "us-west-2",
				SSORoleName:  "TestRole",
				SSOStartURL:  "https://THIS_SHOULD_NOT_BE_IN_TESTDATA_CACHE/start",
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "source_sso_and_assume",
			Expected: sharedConfig{
				Profile:           "source_sso_and_assume",
				RoleARN:           "source_sso_and_assume_arn",
				SourceProfileName: "sso_and_assume",
				SourceProfile: &sharedConfig{
					Profile:           "sso_and_assume",
					RoleARN:           "sso_with_assume_role_arn",
					SourceProfileName: "multiple_assume_role_with_credential_source",
					SourceProfile: &sharedConfig{
						Profile:           "multiple_assume_role_with_credential_source",
						RoleARN:           "multiple_assume_role_with_credential_source_role_arn",
						SourceProfileName: "assume_role_with_credential_source",
						SourceProfile: &sharedConfig{
							Profile:          "assume_role_with_credential_source",
							RoleARN:          "assume_role_with_credential_source_role_arn",
							CredentialSource: credSourceEc2Metadata,
						},
					},
				},
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "sso_mixed_credproc",
			Expected: sharedConfig{
				Profile:           "sso_mixed_credproc",
				SSOAccountID:      "012345678901",
				SSORegion:         "us-west-2",
				SSORoleName:       "TestRole",
				SSOStartURL:       "https://127.0.0.1/start",
				CredentialProcess: "/path/to/process",
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "EC2MetadataServiceEndpoint",
			Expected: sharedConfig{
				Profile:         "EC2MetadataServiceEndpoint",
				EC2IMDSEndpoint: "http://endpoint.localhost",
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "EC2MetadataServiceEndpointModeIPv6",
			Expected: sharedConfig{
				Profile:             "EC2MetadataServiceEndpointModeIPv6",
				EC2IMDSEndpointMode: endpoints.EC2IMDSEndpointModeStateIPv6,
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "EC2MetadataServiceEndpointModeIPv4",
			Expected: sharedConfig{
				Profile:             "EC2MetadataServiceEndpointModeIPv4",
				EC2IMDSEndpointMode: endpoints.EC2IMDSEndpointModeStateIPv4,
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "EC2MetadataServiceEndpointModeUnknown",
			Expected: sharedConfig{
				Profile: "EC2MetadataServiceEndpointModeUnknown",
			},
			Err: fmt.Errorf("failed to load ec2_metadata_service_endpoint_mode from shared config"),
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "EC2MetadataServiceEndpointAndModeMixed",
			Expected: sharedConfig{
				Profile:             "EC2MetadataServiceEndpointAndModeMixed",
				EC2IMDSEndpoint:     "http://endpoint.localhost",
				EC2IMDSEndpointMode: endpoints.EC2IMDSEndpointModeStateIPv6,
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "UseDualStackEndpointEnabled",
			Expected: sharedConfig{
				Profile:              "UseDualStackEndpointEnabled",
				Region:               "us-west-2",
				UseDualStackEndpoint: endpoints.DualStackEndpointStateEnabled,
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "UseDualStackEndpointDisabled",
			Expected: sharedConfig{
				Profile:              "UseDualStackEndpointDisabled",
				Region:               "us-west-2",
				UseDualStackEndpoint: endpoints.DualStackEndpointStateDisabled,
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "UseDualStackEndpointInvalid",
			Expected: sharedConfig{
				Profile:              "UseDualStackEndpointInvalid",
				Region:               "us-west-2",
				UseDualStackEndpoint: endpoints.DualStackEndpointStateDisabled,
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "UseFIPSEndpointEnabled",
			Expected: sharedConfig{
				Profile:         "UseFIPSEndpointEnabled",
				Region:          "us-west-2",
				UseFIPSEndpoint: endpoints.FIPSEndpointStateEnabled,
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "UseFIPSEndpointDisabled",
			Expected: sharedConfig{
				Profile:         "UseFIPSEndpointDisabled",
				Region:          "us-west-2",
				UseFIPSEndpoint: endpoints.FIPSEndpointStateDisabled,
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "UseFIPSEndpointInvalid",
			Expected: sharedConfig{
				Profile:         "UseFIPSEndpointInvalid",
				Region:          "us-west-2",
				UseFIPSEndpoint: endpoints.FIPSEndpointStateDisabled,
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "sso-session-success",
			Expected: sharedConfig{
				Profile:        "sso-session-success",
				Region:         "us-east-1",
				SSOAccountID:   "123456789012",
				SSORoleName:    "testRole",
				SSOSessionName: "sso-session-success-dev",
				SSOSession: &ssoSession{
					Name:        "sso-session-success-dev",
					SSORegion:   "us-east-1",
					SSOStartURL: "https://d-123456789a.awsapps.com/start",
				},
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "sso-session-not-exist",
			Err:       fmt.Errorf("failed to find SSO session section, sso-session-lost"),
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "ec2-metadata-v1-disabled-false",
			Expected: sharedConfig{
				Profile:           "ec2-metadata-v1-disabled-false",
				EC2IMDSv1Disabled: aws.Bool(false),
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "ec2-metadata-v1-disabled-true",
			Expected: sharedConfig{
				Profile:           "ec2-metadata-v1-disabled-true",
				EC2IMDSv1Disabled: aws.Bool(true),
			},
		},
		{
			Filenames: []string{testConfigFilename},
			Profile:   "ec2-metadata-v1-disabled-invalid",
			Expected: sharedConfig{
				Profile:           "ec2-metadata-v1-disabled-invalid",
				EC2IMDSv1Disabled: aws.Bool(false),
			},
		},
	}

	for i, c := range cases {
		t.Run(strconv.Itoa(i)+"_"+c.Profile, func(t *testing.T) {
			cfg, err := loadSharedConfig(c.Profile, c.Filenames, true)
			if c.Err != nil {
				if err == nil {
					t.Fatalf("expect error, got none")
				}
				if e, a := c.Err.Error(), err.Error(); !strings.Contains(a, e) {
					t.Errorf("expect %v, to be in %v", e, a)
				}
				return
			}

			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}
			if e, a := c.Expected, cfg; !reflect.DeepEqual(e, a) {
				t.Errorf("expect %v, got %v", e, a)
			}
		})
	}
}

func TestLoadSharedConfigFromFile(t *testing.T) {
	filename := testConfigFilename
	f, err := ini.OpenFile(filename)
	if err != nil {
		t.Fatalf("failed to load test config file, %s, %v", filename, err)
	}
	iniFile := sharedConfigFile{IniData: f, Filename: filename}

	cases := []struct {
		Profile  string
		Expected sharedConfig
		Err      error
	}{
		{
			Profile:  "default",
			Expected: sharedConfig{Region: "default_region"},
		},
		{
			Profile:  "alt_profile_name",
			Expected: sharedConfig{Region: "alt_profile_name_region"},
		},
		{
			Profile:  "short_profile_name_first",
			Expected: sharedConfig{Region: "short_profile_name_first_short"},
		},
		{
			Profile:  "partial_creds",
			Expected: sharedConfig{},
		},
		{
			Profile: "complete_creds",
			Expected: sharedConfig{
				Creds: credentials.Value{
					AccessKeyID:     "complete_creds_akid",
					SecretAccessKey: "complete_creds_secret",
					ProviderName:    fmt.Sprintf("SharedConfigCredentials: %s", testConfigFilename),
				},
			},
		},
		{
			Profile: "complete_creds_with_token",
			Expected: sharedConfig{
				Creds: credentials.Value{
					AccessKeyID:     "complete_creds_with_token_akid",
					SecretAccessKey: "complete_creds_with_token_secret",
					SessionToken:    "complete_creds_with_token_token",
					ProviderName:    fmt.Sprintf("SharedConfigCredentials: %s", testConfigFilename),
				},
			},
		},
		{
			Profile: "full_profile",
			Expected: sharedConfig{
				Creds: credentials.Value{
					AccessKeyID:     "full_profile_akid",
					SecretAccessKey: "full_profile_secret",
					ProviderName:    fmt.Sprintf("SharedConfigCredentials: %s", testConfigFilename),
				},
				Region: "full_profile_region",
			},
		},
		{
			Profile: "partial_assume_role",
			Expected: sharedConfig{
				RoleARN: "partial_assume_role_role_arn",
			},
		},
		{
			Profile: "assume_role",
			Expected: sharedConfig{
				RoleARN:           "assume_role_role_arn",
				SourceProfileName: "complete_creds",
			},
		},
		{
			Profile: "assume_role_w_mfa",
			Expected: sharedConfig{
				RoleARN:           "assume_role_role_arn",
				SourceProfileName: "complete_creds",
				MFASerial:         "0123456789",
			},
		},
		{
			Profile: "does_not_exists",
			Err:     SharedConfigProfileNotExistsError{Profile: "does_not_exists"},
		},
		{
			Profile: "valid_arn_region",
			Expected: sharedConfig{
				S3UseARNRegion: true,
			},
		},
		{
			Profile: "sso-session-success",
			Expected: sharedConfig{
				Region:         "us-east-1",
				SSOAccountID:   "123456789012",
				SSORoleName:    "testRole",
				SSOSessionName: "sso-session-success-dev",
			},
		},
	}

	for i, c := range cases {
		t.Run(strconv.Itoa(i)+"_"+c.Profile, func(t *testing.T) {
			cfg := sharedConfig{}

			err := cfg.setFromIniFile(c.Profile, iniFile, true)
			if c.Err != nil {
				if err == nil {
					t.Fatalf("expect error, got none")
				}
				if e, a := c.Err.Error(), err.Error(); !strings.Contains(a, e) {
					t.Errorf("expect %v, to be in %v", e, a)
				}
				return
			}

			if err != nil {
				t.Errorf("expect no error, got %v", err)
			}
			if e, a := c.Expected, cfg; !reflect.DeepEqual(e, a) {
				t.Errorf("expect %v, got %v", e, a)
			}
		})
	}
}

func TestLoadSharedConfigIniFiles(t *testing.T) {
	cases := []struct {
		Filenames []string
		Expected  []sharedConfigFile
	}{
		{
			Filenames: []string{"not_exists", testConfigFilename},
			Expected: []sharedConfigFile{
				{Filename: testConfigFilename},
			},
		},
		{
			Filenames: []string{testConfigFilename, testConfigOtherFilename},
			Expected: []sharedConfigFile{
				{Filename: testConfigFilename},
				{Filename: testConfigOtherFilename},
			},
		},
	}

	for i, c := range cases {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			files, err := loadSharedConfigIniFiles(c.Filenames)
			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}
			if e, a := len(c.Expected), len(files); e != a {
				t.Errorf("expect %v, got %v", e, a)
			}

			for i, expectedFile := range c.Expected {
				if e, a := expectedFile.Filename, files[i].Filename; e != a {
					t.Errorf("expect %v, got %v", e, a)
				}
			}
		})
	}
}
