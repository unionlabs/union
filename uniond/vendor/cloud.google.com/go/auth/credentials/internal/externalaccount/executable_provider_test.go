// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package externalaccount

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"sort"
	"testing"
	"time"

	"cloud.google.com/go/auth/internal"
	"cloud.google.com/go/auth/internal/credsfile"
	"github.com/google/go-cmp/cmp"
)

var executablesAllowed = map[string]string{
	allowExecutablesEnvVar: "1",
}

func TestCreateExecutableCredential(t *testing.T) {
	var tests = []struct {
		name             string
		executableConfig credsfile.ExecutableConfig
		wantErr          error
		wantTimeout      time.Duration
		skipErrorEquals  bool
	}{
		{
			name: "Basic Creation",
			executableConfig: credsfile.ExecutableConfig{
				Command:       "blarg",
				TimeoutMillis: 50000,
			},
			wantTimeout: 50000 * time.Millisecond,
		},
		{
			name: "Without Timeout",
			executableConfig: credsfile.ExecutableConfig{
				Command: "blarg",
			},
			wantTimeout: 30000 * time.Millisecond,
		},
		{
			name:             "Without Command",
			executableConfig: credsfile.ExecutableConfig{},
			skipErrorEquals:  true,
		},
		{
			name: "Timeout Too Low",
			executableConfig: credsfile.ExecutableConfig{
				Command:       "blarg",
				TimeoutMillis: 4999,
			},
			skipErrorEquals: true,
		},
		{
			name: "Timeout Lower Bound",
			executableConfig: credsfile.ExecutableConfig{
				Command:       "blarg",
				TimeoutMillis: 5000,
			},
			wantTimeout: 5000 * time.Millisecond,
		},
		{
			name: "Timeout Upper Bound",
			executableConfig: credsfile.ExecutableConfig{
				Command:       "blarg",
				TimeoutMillis: 120000,
			},
			wantTimeout: 120000 * time.Millisecond,
		},
		{
			name: "Timeout Too High",
			executableConfig: credsfile.ExecutableConfig{
				Command:       "blarg",
				TimeoutMillis: 120001,
			},
			skipErrorEquals: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ecs, err := newSubjectTokenProvider(&Options{
				Client: internal.CloneDefaultClient(),
				CredentialSource: &credsfile.CredentialSource{
					Executable: &tt.executableConfig,
				},
			})
			if tt.wantErr != nil || tt.skipErrorEquals {
				if err == nil {
					t.Fatalf("got nil, want an error")
				}
				if tt.skipErrorEquals {
					return
				}
				if got, want := err.Error(), tt.wantErr.Error(); got != want {
					t.Errorf("got %v, want %v", got, want)
				}
			} else if err != nil {
				ecJSON := "{???}"
				if ecBytes, err2 := json.Marshal(tt.executableConfig); err2 != nil {
					ecJSON = string(ecBytes)
				}

				t.Fatalf("CreateExecutableCredential with %v returned error: %v", ecJSON, err)
			} else {
				p := ecs.(*executableSubjectProvider)
				if p.Command != "blarg" {
					t.Errorf("got %v, want %v", p.Command, "blarg")
				}
				if p.Timeout != tt.wantTimeout {
					t.Errorf("got %v, want %v", p.Timeout, tt.wantTimeout)
				}
			}
		})
	}
}

func TestExecutableCredentialGetEnvironment(t *testing.T) {
	var tests = []struct {
		name            string
		opts            *Options
		environment     testEnvironment
		wantEnvironment []string
	}{
		{
			name: "Minimal Executable Config",
			opts: &Options{
				Audience:         "//iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/pool/providers/oidc",
				SubjectTokenType: jwtTokenType,
				CredentialSource: &credsfile.CredentialSource{
					Executable: &credsfile.ExecutableConfig{
						Command: "blarg",
					},
				},
			},
			environment: testEnvironment{
				envVars: map[string]string{
					"A": "B",
				},
			},
			wantEnvironment: []string{
				"A=B",
				"GOOGLE_EXTERNAL_ACCOUNT_AUDIENCE=//iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/pool/providers/oidc",
				"GOOGLE_EXTERNAL_ACCOUNT_TOKEN_TYPE=urn:ietf:params:oauth:token-type:jwt",
				"GOOGLE_EXTERNAL_ACCOUNT_INTERACTIVE=0",
			},
		},
		{
			name: "Full Impersonation URL",
			opts: &Options{
				Audience:                       "//iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/pool/providers/oidc",
				ServiceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/test@project.iam.gserviceaccount.com:generateAccessToken",
				SubjectTokenType:               jwtTokenType,
				CredentialSource: &credsfile.CredentialSource{
					Executable: &credsfile.ExecutableConfig{
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
			wantEnvironment: []string{
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
			opts: &Options{
				Audience:                       "//iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/pool/providers/oidc",
				ServiceAccountImpersonationURL: "test@project.iam.gserviceaccount.com",
				SubjectTokenType:               jwtTokenType,
				CredentialSource: &credsfile.CredentialSource{
					Executable: &credsfile.ExecutableConfig{
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
			wantEnvironment: []string{
				"A=B",
				"GOOGLE_EXTERNAL_ACCOUNT_AUDIENCE=//iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/pool/providers/oidc",
				"GOOGLE_EXTERNAL_ACCOUNT_TOKEN_TYPE=urn:ietf:params:oauth:token-type:jwt",
				"GOOGLE_EXTERNAL_ACCOUNT_INTERACTIVE=0",
				"GOOGLE_EXTERNAL_ACCOUNT_OUTPUT_FILE=/path/to/generated/cached/credentials",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			opts := tt.opts

			newSubjectTokenProvider(opts)
			ecs, err := newSubjectTokenProvider(opts)
			if err != nil {
				t.Fatalf("creation failed %v", err)
			}
			ecs.(*executableSubjectProvider).env = &tt.environment

			// This Transformer sorts a []string.
			sorter := cmp.Transformer("Sort", func(in []string) []string {
				out := append([]string(nil), in...) // Copy input to avoid mutating it
				sort.Strings(out)
				return out
			})

			if got, want := ecs.(*executableSubjectProvider).executableEnvironment(), tt.wantEnvironment; !cmp.Equal(got, want, sorter) {
				t.Errorf("ecs.executableEnvironment() = %v, want %v", got, want)
			}
		})
	}
}

func TestRetrieveExecutableSubjectTokenExecutableErrors(t *testing.T) {
	cs := &credsfile.CredentialSource{
		Executable: &credsfile.ExecutableConfig{
			Command:       "blarg",
			TimeoutMillis: 5000,
		},
	}

	opts := cloneTestOpts()
	opts.CredentialSource = cs

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	ecs, ok := base.(*executableSubjectProvider)
	if !ok {
		t.Fatalf("Wrong credential type created.")
	}

	var tests = []struct {
		name            string
		testEnvironment testEnvironment
		noExecution     bool
		wantErr         error
		skipErrorEquals bool
	}{
		{
			name: "Environment Variable Not Set",
			testEnvironment: testEnvironment{
				byteResponse: []byte{},
			},
			noExecution:     true,
			skipErrorEquals: true,
		},
		{
			name: "Invalid Token",
			testEnvironment: testEnvironment{
				envVars:      executablesAllowed,
				byteResponse: []byte("tokentokentoken"),
			},
			wantErr: jsonParsingError(executableSource, "tokentokentoken"),
		},
		{
			name: "Version Field Missing",
			testEnvironment: testEnvironment{
				envVars: executablesAllowed,
				jsonResponse: &executableResponse{
					Success: Bool(true),
				},
			},
			wantErr: missingFieldError(executableSource, "version"),
		},
		{
			name: "Success Field Missing",
			testEnvironment: testEnvironment{
				envVars: executablesAllowed,
				jsonResponse: &executableResponse{
					Version: 1,
				},
			},
			wantErr: missingFieldError(executableSource, "success"),
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
			wantErr: userDefinedError("404", "Token Not Found"),
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
			wantErr: malformedFailureError(),
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
			wantErr: malformedFailureError(),
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
			wantErr: malformedFailureError(),
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
			wantErr: unsupportedVersionError(executableSource, 2),
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
			wantErr: missingFieldError(executableSource, "token_type"),
		},
		{
			name: "Token Expired",
			testEnvironment: testEnvironment{
				envVars: executablesAllowed,
				jsonResponse: &executableResponse{
					Success:        Bool(true),
					Version:        1,
					ExpirationTime: defaultTime.Unix() - 1,
					TokenType:      jwtTokenType,
				},
			},
			wantErr: tokenExpiredError(),
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
			wantErr: tokenTypeError(executableSource),
		},
		{
			name: "Missing JWT",
			testEnvironment: testEnvironment{
				envVars: executablesAllowed,
				jsonResponse: &executableResponse{
					Success:        Bool(true),
					Version:        1,
					ExpirationTime: defaultTime.Unix(),
					TokenType:      jwtTokenType,
				},
			},
			wantErr: missingFieldError(executableSource, "id_token"),
		},
		{
			name: "Missing ID Token",
			testEnvironment: testEnvironment{
				envVars: executablesAllowed,
				jsonResponse: &executableResponse{
					Success:        Bool(true),
					Version:        1,
					ExpirationTime: defaultTime.Unix(),
					TokenType:      idTokenType,
				},
			},
			wantErr: missingFieldError(executableSource, "id_token"),
		},
		{
			name: "Missing SAML Token",
			testEnvironment: testEnvironment{
				envVars: executablesAllowed,
				jsonResponse: &executableResponse{
					Success:        Bool(true),
					Version:        1,
					ExpirationTime: defaultTime.Unix(),
					TokenType:      saml2TokenType,
				},
			},
			wantErr: missingFieldError(executableSource, "saml_response"),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ecs.env = &tt.testEnvironment

			if got, want := ecs.providerType(), executableProviderType; got != want {
				t.Fatalf("got %q, want %q", got, want)
			}
			if _, err = ecs.subjectToken(context.Background()); err == nil {
				t.Fatalf("got nil, want an error")
			} else if tt.skipErrorEquals {
				// Do no more validation
			} else if got, want := err.Error(), tt.wantErr.Error(); got != want {
				t.Errorf("got %v, want %v", got, want)
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

func TestRetrieveExecutableSubjectTokenSuccesses(t *testing.T) {
	cs := &credsfile.CredentialSource{
		Executable: &credsfile.ExecutableConfig{
			Command:       "blarg",
			TimeoutMillis: 5000,
		},
	}

	opts := cloneTestOpts()
	opts.CredentialSource = cs

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	ecs, ok := base.(*executableSubjectProvider)
	if !ok {
		t.Fatalf("Wrong credential type created.")
	}

	var tests = []struct {
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
					TokenType:      jwtTokenType,
					IDToken:        "tokentokentoken",
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
					TokenType:      idTokenType,
					IDToken:        "tokentokentoken",
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
					TokenType:      saml2TokenType,
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
					TokenType: jwtTokenType,
					IDToken:   "tokentokentoken",
				},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ecs.env = &tt.testEnvironment

			out, err := ecs.subjectToken(context.Background())
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
				t.Errorf("got %v, want %v", got, want)
			}
		})
	}
}

func TestRetrieveOutputFileSubjectTokenNotJSON(t *testing.T) {
	outputFile, err := os.CreateTemp("testdata", "result.*.json")
	if err != nil {
		t.Fatalf("Tempfile failed: %v", err)
	}
	defer os.Remove(outputFile.Name())

	cs := &credsfile.CredentialSource{
		Executable: &credsfile.ExecutableConfig{
			Command:       "blarg",
			TimeoutMillis: 5000,
			OutputFile:    outputFile.Name(),
		},
	}

	opts := cloneTestOpts()
	opts.CredentialSource = cs

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	ecs, ok := base.(*executableSubjectProvider)
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

	if _, err = base.subjectToken(context.Background()); err == nil {
		t.Fatalf("got nil, want an error")
	} else if got, want := err.Error(), jsonParsingError(outputFileSource, "tokentokentoken").Error(); got != want {
		t.Errorf("got %v, want %v", got, want)
	}

	_, deadlineSet := te.getDeadline()
	if deadlineSet {
		t.Errorf("Executable called when it should not have been")
	}
}

func TestRetrieveOutputFileSubjectTokenFailureTests(t *testing.T) {
	// These are errors in the output file that should be reported to the user.
	// Most of these will help the developers debug their code.
	var tests = []struct {
		name               string
		outputFileContents executableResponse
		wantErr            error
	}{
		{
			name: "Missing Version",
			outputFileContents: executableResponse{
				Success: Bool(true),
			},
			wantErr: missingFieldError(outputFileSource, "version"),
		},

		{
			name: "Missing Success",
			outputFileContents: executableResponse{
				Version: 1,
			},
			wantErr: missingFieldError(outputFileSource, "success"),
		},

		{
			name: "Newer Version",
			outputFileContents: executableResponse{
				Success: Bool(true),
				Version: 2,
			},
			wantErr: unsupportedVersionError(outputFileSource, 2),
		},

		{
			name: "Missing Token Type",
			outputFileContents: executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix(),
			},
			wantErr: missingFieldError(outputFileSource, "token_type"),
		},

		{
			name: "Missing Expiration",
			outputFileContents: executableResponse{
				Success:   Bool(true),
				Version:   1,
				TokenType: jwtTokenType,
			},
			wantErr: missingFieldError(outputFileSource, "expiration_time"),
		},

		{
			name: "Invalid Token Type",
			outputFileContents: executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix(),
				TokenType:      "urn:ietf:params:oauth:token-type:invalid",
			},
			wantErr: tokenTypeError(outputFileSource),
		},

		{
			name: "Missing JWT",
			outputFileContents: executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix() + 3600,
				TokenType:      jwtTokenType,
			},
			wantErr: missingFieldError(outputFileSource, "id_token"),
		},

		{
			name: "Missing ID Token",
			outputFileContents: executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix() + 3600,
				TokenType:      idTokenType,
			},
			wantErr: missingFieldError(outputFileSource, "id_token"),
		},

		{
			name: "Missing SAML",
			outputFileContents: executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix() + 3600,
				TokenType:      jwtTokenType,
			},
			wantErr: missingFieldError(outputFileSource, "id_token"),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			outputFile, err := os.CreateTemp("testdata", "result.*.json")
			if err != nil {
				t.Fatalf("Tempfile failed: %v", err)
			}
			defer os.Remove(outputFile.Name())

			cs := &credsfile.CredentialSource{
				Executable: &credsfile.ExecutableConfig{
					Command:       "blarg",
					TimeoutMillis: 5000,
					OutputFile:    outputFile.Name(),
				},
			}

			opts := cloneTestOpts()
			opts.CredentialSource = cs

			base, err := newSubjectTokenProvider(opts)
			if err != nil {
				t.Fatalf("parse() failed %v", err)
			}

			ecs, ok := base.(*executableSubjectProvider)
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
			if _, err = ecs.subjectToken(context.Background()); err == nil {
				t.Errorf("got nil, want an error")
			} else if got, want := err.Error(), tt.wantErr.Error(); got != want {
				t.Errorf("got %v, want %v", got, want)
			}

			if _, deadlineSet := te.getDeadline(); deadlineSet {
				t.Errorf("Executable called when it should not have been")
			}
		})
	}
}

func TestRetrieveOutputFileSubjectTokenInvalidCache(t *testing.T) {
	// These tests should ignore the error in the output file, and check the executable.
	var tests = []struct {
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
				TokenType:      jwtTokenType,
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			outputFile, err := os.CreateTemp("testdata", "result.*.json")
			if err != nil {
				t.Fatalf("Tempfile failed: %v", err)
			}
			defer os.Remove(outputFile.Name())

			cs := &credsfile.CredentialSource{
				Executable: &credsfile.ExecutableConfig{
					Command:       "blarg",
					TimeoutMillis: 5000,
					OutputFile:    outputFile.Name(),
				},
			}

			opts := cloneTestOpts()
			opts.CredentialSource = cs

			base, err := newSubjectTokenProvider(opts)
			if err != nil {
				t.Fatalf("parse() failed %v", err)
			}

			te := testEnvironment{
				envVars: executablesAllowed,
				jsonResponse: &executableResponse{
					Success:        Bool(true),
					Version:        1,
					ExpirationTime: defaultTime.Unix() + 3600,
					TokenType:      jwtTokenType,
					IDToken:        "tokentokentoken",
				},
			}

			ecs, ok := base.(*executableSubjectProvider)
			if !ok {
				t.Fatalf("Wrong credential type created.")
			}
			ecs.env = &te

			if err = json.NewEncoder(outputFile).Encode(tt.outputFileContents); err != nil {
				t.Errorf("Error encoding to file: %v", err)
				return
			}

			out, err := ecs.subjectToken(context.Background())
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
				t.Errorf("got %v, want %v", got, want)
			}
		})
	}
}

func TestRetrieveOutputFileSubjectTokenJwt(t *testing.T) {
	var tests = []struct {
		name               string
		outputFileContents executableResponse
	}{
		{
			name: "JWT",
			outputFileContents: executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix() + 3600,
				TokenType:      jwtTokenType,
				IDToken:        "tokentokentoken",
			},
		},

		{
			name: "Id Token",
			outputFileContents: executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix() + 3600,
				TokenType:      idTokenType,
				IDToken:        "tokentokentoken",
			},
		},

		{
			name: "SAML",
			outputFileContents: executableResponse{
				Success:        Bool(true),
				Version:        1,
				ExpirationTime: defaultTime.Unix() + 3600,
				TokenType:      saml2TokenType,
				SamlResponse:   "tokentokentoken",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {

			outputFile, err := os.CreateTemp("testdata", "result.*.json")
			if err != nil {
				t.Fatalf("Tempfile failed: %v", err)
			}
			defer os.Remove(outputFile.Name())

			cs := &credsfile.CredentialSource{
				Executable: &credsfile.ExecutableConfig{
					Command:       "blarg",
					TimeoutMillis: 5000,
					OutputFile:    outputFile.Name(),
				},
			}

			opts := cloneTestOpts()
			opts.CredentialSource = cs

			base, err := newSubjectTokenProvider(opts)
			if err != nil {
				t.Fatalf("parse() failed %v", err)
			}

			te := testEnvironment{
				envVars:      executablesAllowed,
				byteResponse: []byte{},
			}

			ecs, ok := base.(*executableSubjectProvider)
			if !ok {
				t.Fatalf("Wrong credential type created.")
			}
			ecs.env = &te

			if err = json.NewEncoder(outputFile).Encode(tt.outputFileContents); err != nil {
				t.Errorf("Error encoding to file: %v", err)
				return
			}

			if out, err := ecs.subjectToken(context.Background()); err != nil {
				t.Errorf("retrieveSubjectToken() failed: %v", err)
			} else if got, want := out, "tokentokentoken"; got != want {
				t.Errorf("got %v, want %v", got, want)
			}

			if _, deadlineSet := te.getDeadline(); deadlineSet {
				t.Errorf("Executable called when it should not have been")
			}
		})
	}
}

type testEnvironment struct {
	envVars      map[string]string
	deadline     time.Time
	deadlineSet  bool
	byteResponse []byte
	jsonResponse *executableResponse
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
				t.Errorf("got nil, want %q", tt.want)
			}
		} else if matches[1] != tt.want {
			t.Errorf("got %q, want %q", matches[1], tt.want)
		}
	}
}
