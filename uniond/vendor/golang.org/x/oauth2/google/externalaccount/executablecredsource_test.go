// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package externalaccount

import (
	"context"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"
	"sort"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
)

type testEnvironment struct {
	envVars      map[string]string
	deadline     time.Time
	deadlineSet  bool
	byteResponse []byte
	jsonResponse *executableResponse
}

var executablesAllowed = map[string]string{
	"GOOGLE_EXTERNAL_ACCOUNT_ALLOW_EXECUTABLES": "1",
}

func (t *testEnvironment) existingEnv() []string {
	result := []string{}
	for k, v := range t.envVars {
		result = append(result, fmt.Sprintf("%v=%v", k, v))
	}
	return result
}

func (t *testEnvironment) getenv(key string) string {
	return t.envVars[key]
}

func (t *testEnvironment) run(ctx context.Context, command string, env []string) ([]byte, error) {
	t.deadline, t.deadlineSet = ctx.Deadline()
	if t.jsonResponse != nil {
		return json.Marshal(t.jsonResponse)
	}
	return t.byteResponse, nil
}

func (t *testEnvironment) getDeadline() (time.Time, bool) {
	return t.deadline, t.deadlineSet
}

func (t *testEnvironment) now() time.Time {
	return defaultTime
}

func Bool(b bool) *bool {
	return &b
}

func Int(i int) *int {
	return &i
}

var creationTests = []struct {
	name             string
	executableConfig ExecutableConfig
	expectedErr      error
	expectedTimeout  time.Duration
}{
	{
		name: "Basic Creation",
		executableConfig: ExecutableConfig{
			Command:       "blarg",
			TimeoutMillis: Int(50000),
		},
		expectedTimeout: 50000 * time.Millisecond,
	},
	{
		name: "Without Timeout",
		executableConfig: ExecutableConfig{
			Command: "blarg",
		},
		expectedTimeout: 30000 * time.Millisecond,
	},
	{
		name:             "Without Command",
		executableConfig: ExecutableConfig{},
		expectedErr:      commandMissingError(),
	},
	{
		name: "Timeout Too Low",
		executableConfig: ExecutableConfig{
			Command:       "blarg",
			TimeoutMillis: Int(4999),
		},
		expectedErr: timeoutRangeError(),
	},
	{
		name: "Timeout Lower Bound",
		executableConfig: ExecutableConfig{
			Command:       "blarg",
			TimeoutMillis: Int(5000),
		},
		expectedTimeout: 5000 * time.Millisecond,
	},
	{
		name: "Timeout Upper Bound",
		executableConfig: ExecutableConfig{
			Command:       "blarg",
			TimeoutMillis: Int(120000),
		},
		expectedTimeout: 120000 * time.Millisecond,
	},
	{
		name: "Timeout Too High",
		executableConfig: ExecutableConfig{
			Command:       "blarg",
			TimeoutMillis: Int(120001),
		},
		expectedErr: timeoutRangeError(),
	},
}

func TestCreateExecutableCredential(t *testing.T) {
	for _, tt := range creationTests {
		t.Run(tt.name, func(t *testing.T) {
			ecs, err := createExecutableCredential(context.Background(), &tt.executableConfig, nil)
			if tt.expectedErr != nil {
				if err == nil {
					t.Fatalf("Expected error but found none")
				}
				if got, want := err.Error(), tt.expectedErr.Error(); got != want {
					t.Errorf("Incorrect error received.\nReceived: %s\nExpected: %s", got, want)
				}
			} else if err != nil {
				ecJson := "{???}"
				if ecBytes, err2 := json.Marshal(tt.executableConfig); err2 != nil {
					ecJson = string(ecBytes)
				}

				t.Fatalf("CreateExecutableCredential with %v returned error: %v", ecJson, err)
			} else {
				if ecs.Command != "blarg" {
					t.Errorf("ecs.Command got %v but want %v", ecs.Command, "blarg")
				}
				if ecs.Timeout != tt.expectedTimeout {
					t.Errorf("ecs.Timeout got %v but want %v", ecs.Timeout, tt.expectedTimeout)
				}
				if ecs.credentialSourceType() != "executable" {
					t.Errorf("ecs.CredentialSourceType() got %s but want executable", ecs.credentialSourceType())
				}
			}
		})
	}
}

var getEnvironmentTests = []struct {
	name                string
	config              Config
	environment         testEnvironment
	expectedEnvironment []string
}{
	{
		name: "Minimal Executable Config",
		config: Config{
			Audience:         "//iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/pool/providers/oidc",
			SubjectTokenType: "urn:ietf:params:oauth:token-type:jwt",
			CredentialSource: &CredentialSource{
				Executable: &ExecutableConfig{
					Command: "blarg",
				},
			},
		},
		environment: testEnvironment{
			envVars: map[string]string{
				"A": "B",
			},
		},
		expectedEnvironment: []string{
			"A=B",
			"GOOGLE_EXTERNAL_ACCOUNT_AUDIENCE=//iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/pool/providers/oidc",
			"GOOGLE_EXTERNAL_ACCOUNT_TOKEN_TYPE=urn:ietf:params:oauth:token-type:jwt",
			"GOOGLE_EXTERNAL_ACCOUNT_INTERACTIVE=0",
		},
	},
	{
		name: "Full Impersonation URL",
		config: Config{
			Audience:                       "//iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/pool/providers/oidc",
			ServiceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/test@project.iam.gserviceaccount.com:generateAccessToken",
			SubjectTokenType:               "urn:ietf:params:oauth:token-type:jwt",
			CredentialSource: &CredentialSource{
				Executable: &ExecutableConfig{
					Command:    "blarg",
					OutputFile: "/path/to/generated/cached/credentials",
				},
			},
		},
		environment: testEnvironment{
			envVars: map[string]string{
				"A": "B",
			},
		},
		expectedEnvironment: []string{
			"A=B",
			"GOOGLE_EXTERNAL_ACCOUNT_AUDIENCE=//iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/pool/providers/oidc",
			"GOOGLE_EXTERNAL_ACCOUNT_TOKEN_TYPE=urn:ietf:params:oauth:token-type:jwt",
			"GOOGLE_EXTERNAL_ACCOUNT_IMPERSONATED_EMAIL=test@project.iam.gserviceaccount.com",
			"GOOGLE_EXTERNAL_ACCOUNT_INTERACTIVE=0",
			"GOOGLE_EXTERNAL_ACCOUNT_OUTPUT_FILE=/path/to/generated/cached/credentials",
		},
	},
	{
		name: "Impersonation Email",
		config: Config{
			Audience:                       "//iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/pool/providers/oidc",
			ServiceAccountImpersonationURL: "test@project.iam.gserviceaccount.com",
			SubjectTokenType:               "urn:ietf:params:oauth:token-type:jwt",
			CredentialSource: &CredentialSource{
				Executable: &ExecutableConfig{
					Command:    "blarg",
					OutputFile: "/path/to/generated/cached/credentials",
				},
			},
		},
		environment: testEnvironment{
			envVars: map[string]string{
				"A": "B",
			},
		},
		expectedEnvironment: []string{
			"A=B",
			"GOOGLE_EXTERNAL_ACCOUNT_AUDIENCE=//iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/pool/providers/oidc",
			"GOOGLE_EXTERNAL_ACCOUNT_TOKEN_TYPE=urn:ietf:params:oauth:token-type:jwt",
			"GOOGLE_EXTERNAL_ACCOUNT_INTERACTIVE=0",
			"GOOGLE_EXTERNAL_ACCOUNT_OUTPUT_FILE=/path/to/generated/cached/credentials",
		},
	},
}

func TestExecutableCredentialGetEnvironment(t *testing.T) {
	for _, tt := range getEnvironmentTests {
		t.Run(tt.name, func(t *testing.T) {
			config := tt.config

			ecs, err := createExecutableCredential(context.Background(), config.CredentialSource.Executable, &config)
			if err != nil {
				t.Fatalf("creation failed %v", err)
			}

			ecs.env = &tt.environment

			// This Transformer sorts a []string.
			sorter := cmp.Transformer("Sort", func(in []string) []string {
				out := append([]string(nil), in...) // Copy input to avoid mutating it
				sort.Strings(out)
				return out
			})

			if got, want := ecs.executableEnvironment(), tt.expectedEnvironment; !cmp.Equal(got, want, sorter) {
				t.Errorf("Incorrect environment received.\nReceived: %s\nExpected: %s", got, want)
			}
		})
	}
}

var failureTests = []struct {
	name            string
	testEnvironment testEnvironment
	noExecution     bool
	expectedErr     error
}{
	{
		name: "Environment Variable Not Set",
		testEnvironment: testEnvironment{
			byteResponse: []byte{},
		},
		noExecution: true,
		expectedErr: executablesDisallowedError(),
	},

	{
		name: "Invalid Token",
		testEnvironment: testEnvironment{
			envVars:      executablesAllowed,
			byteResponse: []byte("tokentokentoken"),
		},
		expectedErr: jsonParsingError(executableSource, "tokentokentoken"),
	},

	{
		name: "Version Field Missing",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success: Bool(true),
			},
		},
		expectedErr: missingFieldError(executableSource, "version"),
	},

	{
		name: "Success Field Missing",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Version: 1,
			},
		},
		expectedErr: missingFieldError(executableSource, "success"),
	},

	{
		name: "User defined error",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success: Bool(false),
				Version: 1,
				Code:    "404",
				Message: "Token Not Found",
			},
		},
		expectedErr: userDefinedError("404", "Token Not Found"),
	},

	{
		name: "User defined error without code",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success: Bool(false),
				Version: 1,
				Message: "Token Not Found",
			},
		},
		expectedErr: malformedFailureError(),
	},

	{
		name: "User defined error without message",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success: Bool(false),
				Version: 1,
				Code:    "404",
			},
		},
		expectedErr: malformedFailureError(),
	},

	{
		name: "User defined error without fields",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success: Bool(false),
				Version: 1,
			},
		},
		expectedErr: malformedFailureError(),
	},

	{
		name: "Newer Version",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success: Bool(true),
				Version: 2,
			},
		},
		expectedErr: unsupportedVersionError(executableSource, 2),
	},

	{
		name: "Missing Token Type",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix(),
			},
		},
		expectedErr: missingFieldError(executableSource, "token_type"),
	},

	{
		name: "Token Expired",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix() - 1,
				TokenType:      "urn:ietf:params:oauth:token-type:jwt",
			},
		},
		expectedErr: tokenExpiredError(),
	},

	{
		name: "Invalid Token Type",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix(),
				TokenType:      "urn:ietf:params:oauth:token-type:invalid",
			},
		},
		expectedErr: tokenTypeError(executableSource),
	},

	{
		name: "Missing JWT",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix(),
				TokenType:      "urn:ietf:params:oauth:token-type:jwt",
			},
		},
		expectedErr: missingFieldError(executableSource, "id_token"),
	},

	{
		name: "Missing ID Token",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix(),
				TokenType:      "urn:ietf:params:oauth:token-type:id_token",
			},
		},
		expectedErr: missingFieldError(executableSource, "id_token"),
	},

	{
		name: "Missing SAML Token",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix(),
				TokenType:      "urn:ietf:params:oauth:token-type:saml2",
			},
		},
		expectedErr: missingFieldError(executableSource, "saml_response"),
	},
}

func TestRetrieveExecutableSubjectTokenExecutableErrors(t *testing.T) {
	cs := CredentialSource{
		Executable: &ExecutableConfig{
			Command:       "blarg",
			TimeoutMillis: Int(5000),
		},
	}

	tfc := testFileConfig
	tfc.CredentialSource = &cs

	base, err := tfc.parse(context.Background())
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	ecs, ok := base.(executableCredentialSource)
	if !ok {
		t.Fatalf("Wrong credential type created.")
	}

	for _, tt := range failureTests {
		t.Run(tt.name, func(t *testing.T) {
			ecs.env = &tt.testEnvironment

			if _, err = ecs.subjectToken(); err == nil {
				t.Fatalf("Expected error but found none")
			} else if got, want := err.Error(), tt.expectedErr.Error(); got != want {
				t.Errorf("Incorrect error received.\nReceived: %s\nExpected: %s", got, want)
			}

			deadline, deadlineSet := tt.testEnvironment.getDeadline()
			if tt.noExecution {
				if deadlineSet {
					t.Errorf("Executable called when it should not have been")
				}
			} else {
				if !deadlineSet {
					t.Errorf("Command run without a deadline")
				} else if deadline != defaultTime.Add(5*time.Second) {
					t.Errorf("Command run with incorrect deadline")
				}
			}
		})
	}
}

var successTests = []struct {
	name            string
	testEnvironment testEnvironment
}{
	{
		name: "JWT",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix() + 3600,
				TokenType:      "urn:ietf:params:oauth:token-type:jwt",
				IdToken:        "tokentokentoken",
			},
		},
	},

	{
		name: "ID Token",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix() + 3600,
				TokenType:      "urn:ietf:params:oauth:token-type:id_token",
				IdToken:        "tokentokentoken",
			},
		},
	},

	{
		name: "SAML",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix() + 3600,
				TokenType:      "urn:ietf:params:oauth:token-type:saml2",
				SamlResponse:   "tokentokentoken",
			},
		},
	},

	{
		name: "Missing Expiration",
		testEnvironment: testEnvironment{
			envVars: executablesAllowed,
			jsonResponse: &executableResponse{
				Success:   Bool(true),
				Version:   1,
				TokenType: "urn:ietf:params:oauth:token-type:jwt",
				IdToken:   "tokentokentoken",
			},
		},
	},
}

func TestRetrieveExecutableSubjectTokenSuccesses(t *testing.T) {
	cs := CredentialSource{
		Executable: &ExecutableConfig{
			Command:       "blarg",
			TimeoutMillis: Int(5000),
		},
	}

	tfc := testFileConfig
	tfc.CredentialSource = &cs

	base, err := tfc.parse(context.Background())
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	ecs, ok := base.(executableCredentialSource)
	if !ok {
		t.Fatalf("Wrong credential type created.")
	}

	for _, tt := range successTests {
		t.Run(tt.name, func(t *testing.T) {
			ecs.env = &tt.testEnvironment

			out, err := ecs.subjectToken()
			if err != nil {
				t.Fatalf("retrieveSubjectToken() failed: %v", err)
			}

			deadline, deadlineSet := tt.testEnvironment.getDeadline()
			if !deadlineSet {
				t.Errorf("Command run without a deadline")
			} else if deadline != defaultTime.Add(5*time.Second) {
				t.Errorf("Command run with incorrect deadline")
			}

			if got, want := out, "tokentokentoken"; got != want {
				t.Errorf("Incorrect token received.\nReceived: %s\nExpected: %s", got, want)
			}
		})
	}
}

func TestRetrieveOutputFileSubjectTokenNotJSON(t *testing.T) {
	outputFile, err := ioutil.TempFile("testdata", "result.*.json")
	if err != nil {
		t.Fatalf("Tempfile failed: %v", err)
	}
	defer os.Remove(outputFile.Name())

	cs := CredentialSource{
		Executable: &ExecutableConfig{
			Command:       "blarg",
			TimeoutMillis: Int(5000),
			OutputFile:    outputFile.Name(),
		},
	}

	tfc := testFileConfig
	tfc.CredentialSource = &cs

	base, err := tfc.parse(context.Background())
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	ecs, ok := base.(executableCredentialSource)
	if !ok {
		t.Fatalf("Wrong credential type created.")
	}

	if _, err = outputFile.Write([]byte("tokentokentoken")); err != nil {
		t.Fatalf("error writing to file: %v", err)
	}

	te := testEnvironment{
		envVars:      executablesAllowed,
		byteResponse: []byte{},
	}
	ecs.env = &te

	if _, err = base.subjectToken(); err == nil {
		t.Fatalf("Expected error but found none")
	} else if got, want := err.Error(), jsonParsingError(outputFileSource, "tokentokentoken").Error(); got != want {
		t.Errorf("Incorrect error received.\nExpected: %s\nRecieved: %s", want, got)
	}

	_, deadlineSet := te.getDeadline()
	if deadlineSet {
		t.Errorf("Executable called when it should not have been")
	}
}

// These are errors in the output file that should be reported to the user.
// Most of these will help the developers debug their code.
var cacheFailureTests = []struct {
	name               string
	outputFileContents executableResponse
	expectedErr        error
}{
	{
		name: "Missing Version",
		outputFileContents: executableResponse{
			Success: Bool(true),
		},
		expectedErr: missingFieldError(outputFileSource, "version"),
	},

	{
		name: "Missing Success",
		outputFileContents: executableResponse{
			Version: 1,
		},
		expectedErr: missingFieldError(outputFileSource, "success"),
	},

	{
		name: "Newer Version",
		outputFileContents: executableResponse{
			Success: Bool(true),
			Version: 2,
		},
		expectedErr: unsupportedVersionError(outputFileSource, 2),
	},

	{
		name: "Missing Token Type",
		outputFileContents: executableResponse{
			Success:        Bool(true),
			Version:        1,
			ExpirationTime: defaultTime.Unix(),
		},
		expectedErr: missingFieldError(outputFileSource, "token_type"),
	},

	{
		name: "Missing Expiration",
		outputFileContents: executableResponse{
			Success:   Bool(true),
			Version:   1,
			TokenType: "urn:ietf:params:oauth:token-type:jwt",
		},
		expectedErr: missingFieldError(outputFileSource, "expiration_time"),
	},

	{
		name: "Invalid Token Type",
		outputFileContents: executableResponse{
			Success:        Bool(true),
			Version:        1,
			ExpirationTime: defaultTime.Unix(),
			TokenType:      "urn:ietf:params:oauth:token-type:invalid",
		},
		expectedErr: tokenTypeError(outputFileSource),
	},

	{
		name: "Missing JWT",
		outputFileContents: executableResponse{
			Success:        Bool(true),
			Version:        1,
			ExpirationTime: defaultTime.Unix() + 3600,
			TokenType:      "urn:ietf:params:oauth:token-type:jwt",
		},
		expectedErr: missingFieldError(outputFileSource, "id_token"),
	},

	{
		name: "Missing ID Token",
		outputFileContents: executableResponse{
			Success:        Bool(true),
			Version:        1,
			ExpirationTime: defaultTime.Unix() + 3600,
			TokenType:      "urn:ietf:params:oauth:token-type:id_token",
		},
		expectedErr: missingFieldError(outputFileSource, "id_token"),
	},

	{
		name: "Missing SAML",
		outputFileContents: executableResponse{
			Success:        Bool(true),
			Version:        1,
			ExpirationTime: defaultTime.Unix() + 3600,
			TokenType:      "urn:ietf:params:oauth:token-type:jwt",
		},
		expectedErr: missingFieldError(outputFileSource, "id_token"),
	},
}

func TestRetrieveOutputFileSubjectTokenFailureTests(t *testing.T) {
	for _, tt := range cacheFailureTests {
		t.Run(tt.name, func(t *testing.T) {
			outputFile, err := ioutil.TempFile("testdata", "result.*.json")
			if err != nil {
				t.Fatalf("Tempfile failed: %v", err)
			}
			defer os.Remove(outputFile.Name())

			cs := CredentialSource{
				Executable: &ExecutableConfig{
					Command:       "blarg",
					TimeoutMillis: Int(5000),
					OutputFile:    outputFile.Name(),
				},
			}

			tfc := testFileConfig
			tfc.CredentialSource = &cs

			base, err := tfc.parse(context.Background())
			if err != nil {
				t.Fatalf("parse() failed %v", err)
			}

			ecs, ok := base.(executableCredentialSource)
			if !ok {
				t.Fatalf("Wrong credential type created.")
			}
			te := testEnvironment{
				envVars:      executablesAllowed,
				byteResponse: []byte{},
			}
			ecs.env = &te
			if err = json.NewEncoder(outputFile).Encode(tt.outputFileContents); err != nil {
				t.Errorf("Error encoding to file: %v", err)
				return
			}
			if _, err = ecs.subjectToken(); err == nil {
				t.Errorf("Expected error but found none")
			} else if got, want := err.Error(), tt.expectedErr.Error(); got != want {
				t.Errorf("Incorrect error received.\nExpected: %s\nRecieved: %s", want, got)
			}

			if _, deadlineSet := te.getDeadline(); deadlineSet {
				t.Errorf("Executable called when it should not have been")
			}
		})
	}
}

// These tests should ignore the error in the output file, and check the executable.
var invalidCacheTests = []struct {
	name               string
	outputFileContents executableResponse
}{
	{
		name: "User Defined Error",
		outputFileContents: executableResponse{
			Success: Bool(false),
			Version: 1,
			Code:    "404",
			Message: "Token Not Found",
		},
	},

	{
		name: "User Defined Error without Code",
		outputFileContents: executableResponse{
			Success: Bool(false),
			Version: 1,
			Message: "Token Not Found",
		},
	},

	{
		name: "User Defined Error without Message",
		outputFileContents: executableResponse{
			Success: Bool(false),
			Version: 1,
			Code:    "404",
		},
	},

	{
		name: "User Defined Error without Fields",
		outputFileContents: executableResponse{
			Success: Bool(false),
			Version: 1,
		},
	},

	{
		name: "Expired Token",
		outputFileContents: executableResponse{
			Success:        Bool(true),
			Version:        1,
			ExpirationTime: defaultTime.Unix() - 1,
			TokenType:      "urn:ietf:params:oauth:token-type:jwt",
		},
	},
}

func TestRetrieveOutputFileSubjectTokenInvalidCache(t *testing.T) {
	for _, tt := range invalidCacheTests {
		t.Run(tt.name, func(t *testing.T) {
			outputFile, err := ioutil.TempFile("testdata", "result.*.json")
			if err != nil {
				t.Fatalf("Tempfile failed: %v", err)
			}
			defer os.Remove(outputFile.Name())

			cs := CredentialSource{
				Executable: &ExecutableConfig{
					Command:       "blarg",
					TimeoutMillis: Int(5000),
					OutputFile:    outputFile.Name(),
				},
			}

			tfc := testFileConfig
			tfc.CredentialSource = &cs

			base, err := tfc.parse(context.Background())
			if err != nil {
				t.Fatalf("parse() failed %v", err)
			}

			te := testEnvironment{
				envVars: executablesAllowed,
				jsonResponse: &executableResponse{
					Success:        Bool(true),
					Version:        1,
					ExpirationTime: defaultTime.Unix() + 3600,
					TokenType:      "urn:ietf:params:oauth:token-type:jwt",
					IdToken:        "tokentokentoken",
				},
			}

			ecs, ok := base.(executableCredentialSource)
			if !ok {
				t.Fatalf("Wrong credential type created.")
			}
			ecs.env = &te

			if err = json.NewEncoder(outputFile).Encode(tt.outputFileContents); err != nil {
				t.Errorf("Error encoding to file: %v", err)
				return
			}

			out, err := ecs.subjectToken()
			if err != nil {
				t.Errorf("retrieveSubjectToken() failed: %v", err)
				return
			}

			if deadline, deadlineSet := te.getDeadline(); !deadlineSet {
				t.Errorf("Command run without a deadline")
			} else if deadline != defaultTime.Add(5*time.Second) {
				t.Errorf("Command run with incorrect deadline")
			}

			if got, want := out, "tokentokentoken"; got != want {
				t.Errorf("Incorrect token received.\nExpected: %s\nRecieved: %s", want, got)
			}
		})
	}
}

var cacheSuccessTests = []struct {
	name               string
	outputFileContents executableResponse
}{
	{
		name: "JWT",
		outputFileContents: executableResponse{
			Success:        Bool(true),
			Version:        1,
			ExpirationTime: defaultTime.Unix() + 3600,
			TokenType:      "urn:ietf:params:oauth:token-type:jwt",
			IdToken:        "tokentokentoken",
		},
	},

	{
		name: "Id Token",
		outputFileContents: executableResponse{
			Success:        Bool(true),
			Version:        1,
			ExpirationTime: defaultTime.Unix() + 3600,
			TokenType:      "urn:ietf:params:oauth:token-type:id_token",
			IdToken:        "tokentokentoken",
		},
	},

	{
		name: "SAML",
		outputFileContents: executableResponse{
			Success:        Bool(true),
			Version:        1,
			ExpirationTime: defaultTime.Unix() + 3600,
			TokenType:      "urn:ietf:params:oauth:token-type:saml2",
			SamlResponse:   "tokentokentoken",
		},
	},
}

func TestRetrieveOutputFileSubjectTokenJwt(t *testing.T) {
	for _, tt := range cacheSuccessTests {
		t.Run(tt.name, func(t *testing.T) {

			outputFile, err := ioutil.TempFile("testdata", "result.*.json")
			if err != nil {
				t.Fatalf("Tempfile failed: %v", err)
			}
			defer os.Remove(outputFile.Name())

			cs := CredentialSource{
				Executable: &ExecutableConfig{
					Command:       "blarg",
					TimeoutMillis: Int(5000),
					OutputFile:    outputFile.Name(),
				},
			}

			tfc := testFileConfig
			tfc.CredentialSource = &cs

			base, err := tfc.parse(context.Background())
			if err != nil {
				t.Fatalf("parse() failed %v", err)
			}

			te := testEnvironment{
				envVars:      executablesAllowed,
				byteResponse: []byte{},
			}

			ecs, ok := base.(executableCredentialSource)
			if !ok {
				t.Fatalf("Wrong credential type created.")
			}
			ecs.env = &te

			if err = json.NewEncoder(outputFile).Encode(tt.outputFileContents); err != nil {
				t.Errorf("Error encoding to file: %v", err)
				return
			}

			if out, err := ecs.subjectToken(); err != nil {
				t.Errorf("retrieveSubjectToken() failed: %v", err)
			} else if got, want := out, "tokentokentoken"; got != want {
				t.Errorf("Incorrect token received.\nExpected: %s\nRecieved: %s", want, got)
			}

			if _, deadlineSet := te.getDeadline(); deadlineSet {
				t.Errorf("Executable called when it should not have been")
			}
		})
	}
}

func TestServiceAccountImpersonationRE(t *testing.T) {
	tests := []struct {
		name                           string
		serviceAccountImpersonationURL string
		want                           string
	}{
		{
			name:                           "universe domain Google Default Universe (GDU) googleapis.com",
			serviceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/test@project.iam.gserviceaccount.com:generateAccessToken",
			want:                           "test@project.iam.gserviceaccount.com",
		},
		{
			name:                           "email does not match",
			serviceAccountImpersonationURL: "test@project.iam.gserviceaccount.com",
			want:                           "",
		},
		{
			name:                           "universe domain non-GDU",
			serviceAccountImpersonationURL: "https://iamcredentials.apis-tpclp.goog/v1/projects/-/serviceAccounts/test@project.iam.gserviceaccount.com:generateAccessToken",
			want:                           "test@project.iam.gserviceaccount.com",
		},
	}
	for _, tt := range tests {
		matches := serviceAccountImpersonationRE.FindStringSubmatch(tt.serviceAccountImpersonationURL)
		if matches == nil {
			if tt.want != "" {
				t.Errorf("%q: got nil, want %q", tt.name, tt.want)
			}
		} else if matches[1] != tt.want {
			t.Errorf("%q: got %q, want %q", tt.name, matches[1], tt.want)
		}
	}
}
