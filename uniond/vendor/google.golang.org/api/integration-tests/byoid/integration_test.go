// Copyright 2021 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// To run this test locally, you will need to do the following:
// • Navigate to your Google Cloud Project
// • Get a copy of a Service Account Key File for testing (should be in .json format)
// • If you are unable to obtain an existing key file, create one:
//    • > IAM and Admin > Service Accounts
//    • Under the needed Service Account > Actions > Manage Keys
//    • Add Key > Create New Key
//    • Select JSON, and the click Create
// • Look for an available VM Instance, or create one- > Compute > Compute Engine > VM Instances
// • On the VM Instance, click the SSH Button.  Then upload:
//    • Your Service Account Key File
//    • This script, along with setup.sh
//    • A copy of env.conf, containing the required environment variables (see existing skeleton)/
// • Set your environment variables (Usually this will be `source env.conf`)
// • Ensure that your VM is properly set up to run the integration test e.g.
//    • wget -c https://golang.org/dl/go1.15.2.linux-amd64.tar.gz
//       • Check https://golang.org/dl/for the latest version of Go
//    • sudo tar -C /usr/local -xvzf go1.15.2.linux-amd64.tar.gz
//    • go mod init google.golang.org/api/google-api-go-client
//    • go mod tidy
// • Run setup.sh
// • go test -tags integration`

package byoid

import (
	"context"
	"encoding/json"
	"encoding/xml"
	"flag"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/http/httptest"
	"net/url"
	"os"
	"testing"
	"time"

	"golang.org/x/oauth2/google"
	"google.golang.org/api/dns/v1"
	"google.golang.org/api/idtoken"
	"google.golang.org/api/option"
)

const (
	envCredentials  = "GOOGLE_APPLICATION_CREDENTIALS"
	envAudienceOIDC = "GCLOUD_TESTS_GOLANG_AUDIENCE_OIDC"
	envAudienceAWS  = "GCLOUD_TESTS_GOLANG_AUDIENCE_AWS"
	envProject      = "GOOGLE_CLOUD_PROJECT"
)

var (
	oidcAudience string
	awsAudience  string
	oidcToken    string
	clientID     string
	projectID    string
)

// TestMain contains all of the setup code that needs to be run once before any of the tests are run
func TestMain(m *testing.M) {
	flag.Parse()
	if testing.Short() {
		// This line runs all of our individual tests
		os.Exit(m.Run())
	}
	keyFileName := os.Getenv(envCredentials)
	if keyFileName == "" {
		log.Fatalf("Please set %s to your keyfile", envCredentials)
	}

	projectID = os.Getenv(envProject)
	if projectID == "" {
		log.Fatalf("Please set %s to the ID of the project", envProject)
	}

	oidcAudience = os.Getenv(envAudienceOIDC)
	if oidcAudience == "" {
		log.Fatalf("Please set %s to the OIDC Audience", envAudienceOIDC)
	}

	awsAudience = os.Getenv(envAudienceAWS)
	if awsAudience == "" {
		log.Fatalf("Please set %s to the AWS Audience", envAudienceAWS)
	}

	var err error

	clientID, err = getClientID(keyFileName)
	if err != nil {
		log.Fatalf("Error getting Client ID: %v", err)
	}

	oidcToken, err = generateGoogleToken(keyFileName)
	if err != nil {
		log.Fatalf("Error generating Google token: %v", err)
	}

	// This line runs all of our individual tests
	os.Exit(m.Run())
}

// keyFile is a struct to extract the relevant json fields for our ServiceAccount KeyFile
type keyFile struct {
	ClientEmail string `json:"client_email"`
	ClientID    string `json:"client_id"`
}

func getClientID(keyFileName string) (string, error) {
	kf, err := os.Open(keyFileName)
	if err != nil {
		return "", err
	}
	defer kf.Close()

	decoder := json.NewDecoder(kf)
	var keyFileSettings keyFile
	if err = decoder.Decode(&keyFileSettings); err != nil {
		return "", err
	}

	return fmt.Sprintf("projects/-/serviceAccounts/%s", keyFileSettings.ClientEmail), nil
}

func generateGoogleToken(keyFileName string) (string, error) {
	ts, err := idtoken.NewTokenSource(context.Background(), oidcAudience, option.WithCredentialsFile(keyFileName))
	if err != nil {
		return "", nil
	}

	token, err := ts.Token()
	if err != nil {
		return "", nil
	}

	return token.AccessToken, nil
}

// writeConfig writes a temporary config file to memory, and cleans it up after
// testing code is run.
func writeConfig(t *testing.T, c config, f func(name string)) {
	t.Helper()

	// Set up config file.
	configFile, err := os.CreateTemp("", "config.json")
	if err != nil {
		t.Fatalf("Error creating config file: %v", err)
	}
	defer os.Remove(configFile.Name())

	err = json.NewEncoder(configFile).Encode(c)
	if err != nil {
		t.Errorf("Error writing to config file: %v", err)
	}
	configFile.Close()

	f(configFile.Name())
}

// testBYOID makes sure that the default credentials works for
// whatever preconditions have been set beforehand
// by using those credentials to run our client libraries.
//
// In each test we will set up whatever preconditions we need,
// and then use this function.
func testBYOID(t *testing.T, c config) {
	t.Helper()

	writeConfig(t, c, func(name string) {
		// Once the default credentials are obtained,
		// we should be able to access Google Cloud resources.
		dnsService, err := dns.NewService(context.Background(), option.WithCredentialsFile(name))
		if err != nil {
			t.Fatalf("Could not establish DNS Service: %v", err)
		}

		_, err = dnsService.Projects.Get(projectID).Do()
		if err != nil {
			t.Fatalf("DNS Service failed: %v", err)
		}
	})
}

// These structs makes writing our config as json to a file much easier.
type config struct {
	Type                           string                          `json:"type"`
	Audience                       string                          `json:"audience"`
	SubjectTokenType               string                          `json:"subject_token_type"`
	TokenURL                       string                          `json:"token_url"`
	ServiceAccountImpersonationURL string                          `json:"service_account_impersonation_url"`
	ServiceAccountImpersonation    serviceAccountImpersonationInfo `json:"service_account_impersonation,omitempty"`
	CredentialSource               credentialSource                `json:"credential_source"`
}

type serviceAccountImpersonationInfo struct {
	TokenLifetimeSeconds int `json:"token_lifetime_seconds,omitempty"`
}

type credentialSource struct {
	File                        string           `json:"file,omitempty"`
	URL                         string           `json:"url,omitempty"`
	Executable                  executableConfig `json:"executable,omitempty"`
	EnvironmentID               string           `json:"environment_id,omitempty"`
	RegionURL                   string           `json:"region_url,omitempty"`
	RegionalCredVerificationURL string           `json:"regional_cred_verification_url,omitempty"`
}

type executableConfig struct {
	Command       string `json:"command,omitempty"`
	TimeoutMillis int    `json:"timeout_millis,omitempty"`
	OutputFile    string `json:"output_file,omitempty"`
}

// Tests to make sure File based external credentials continues to work.
func TestFileBasedCredentials(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test")
	}
	// Set up Token as a file
	tokenFile, err := os.CreateTemp("", "token.txt")
	if err != nil {
		t.Fatalf("Error creating token file:")
	}
	defer os.Remove(tokenFile.Name())

	tokenFile.WriteString(oidcToken)
	tokenFile.Close()

	// Run our test!
	testBYOID(t, config{
		Type:                           "external_account",
		Audience:                       oidcAudience,
		SubjectTokenType:               "urn:ietf:params:oauth:token-type:jwt",
		TokenURL:                       "https://sts.googleapis.com/v1beta/token",
		ServiceAccountImpersonationURL: fmt.Sprintf("https://iamcredentials.googleapis.com/v1/%s:generateAccessToken", clientID),
		CredentialSource: credentialSource{
			File: tokenFile.Name(),
		},
	})
}

// Tests to make sure URL based external credentials work properly.
func TestURLBasedCredentials(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test")
	}
	//Set up a server to return a token
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			t.Errorf("Unexpected request method, %v is found", r.Method)
		}
		w.Write([]byte(oidcToken))
	}))

	testBYOID(t, config{
		Type:                           "external_account",
		Audience:                       oidcAudience,
		SubjectTokenType:               "urn:ietf:params:oauth:token-type:jwt",
		TokenURL:                       "https://sts.googleapis.com/v1/token",
		ServiceAccountImpersonationURL: fmt.Sprintf("https://iamcredentials.googleapis.com/v1/%s:generateAccessToken", clientID),
		CredentialSource: credentialSource{
			URL: ts.URL,
		},
	})
}

// Tests to make sure AWS based external credentials work properly.
func TestAWSBasedCredentials(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test")
	}
	data := url.Values{}
	data.Set("audience", clientID)
	data.Set("includeEmail", "true")

	client, err := google.DefaultClient(context.Background(), "https://www.googleapis.com/auth/cloud-platform")
	if err != nil {
		t.Fatalf("Failed to create default client: %v", err)
	}
	resp, err := client.PostForm(fmt.Sprintf("https://iamcredentials.googleapis.com/v1/%s:generateIdToken", clientID), data)
	if err != nil {
		t.Fatalf("Failed to generate an ID token: %v", err)
	}
	if resp.StatusCode != 200 {
		t.Fatalf("Failed to get Google ID token for AWS test: %v", err)
	}

	var res map[string]interface{}

	if err = json.NewDecoder(resp.Body).Decode(&res); err != nil {
		t.Fatalf("Could not successfully parse response from generateIDToken: %v", err)
	}
	token, ok := res["token"]
	if !ok {
		t.Fatalf("Didn't receieve an ID token back from generateIDToken")
	}

	data = url.Values{}
	data.Set("Action", "AssumeRoleWithWebIdentity")
	data.Set("Version", "2011-06-15")
	data.Set("DurationSeconds", "3600")
	data.Set("RoleSessionName", os.Getenv("GCLOUD_TESTS_GOLANG_AWS_ROLE_NAME"))
	data.Set("RoleArn", os.Getenv("GCLOUD_TESTS_GOLANG_AWS_ROLE_ID"))
	data.Set("WebIdentityToken", token.(string))

	resp, err = http.PostForm("https://sts.amazonaws.com/", data)
	if err != nil {
		t.Fatalf("Failed to post data to AWS: %v", err)
	}
	bodyBytes, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("Failed to parse response body from AWS: %v", err)
	}

	var respVars struct {
		SessionToken    string `xml:"AssumeRoleWithWebIdentityResult>Credentials>SessionToken"`
		SecretAccessKey string `xml:"AssumeRoleWithWebIdentityResult>Credentials>SecretAccessKey"`
		AccessKeyID     string `xml:"AssumeRoleWithWebIdentityResult>Credentials>AccessKeyId"`
	}

	if err = xml.Unmarshal(bodyBytes, &respVars); err != nil {
		t.Fatalf("Failed to unmarshal XML response from AWS.")
	}

	if respVars.SessionToken == "" || respVars.SecretAccessKey == "" || respVars.AccessKeyID == "" {
		t.Fatalf("Couldn't find the required variables in the response from the AWS server.")
	}

	currSessTokEnv := os.Getenv("AWS_SESSION_TOKEN")
	defer os.Setenv("AWS_SESSION_TOKEN", currSessTokEnv)
	os.Setenv("AWS_SESSION_TOKEN", respVars.SessionToken)

	currSecAccKey := os.Getenv("AWS_SECRET_ACCESS_KEY")
	defer os.Setenv("AWS_SECRET_ACCESS_KEY", currSecAccKey)
	os.Setenv("AWS_SECRET_ACCESS_KEY", respVars.SecretAccessKey)

	currAccKeyID := os.Getenv("AWS_ACCESS_KEY_ID")
	defer os.Setenv("AWS_ACCESS_KEY_ID", currAccKeyID)
	os.Setenv("AWS_ACCESS_KEY_ID", respVars.AccessKeyID)

	currRegion := os.Getenv("AWS_REGION")
	defer os.Setenv("AWS_REGION", currRegion)
	os.Setenv("AWS_REGION", "us-east-1")

	testBYOID(t, config{
		Type:                           "external_account",
		Audience:                       awsAudience,
		SubjectTokenType:               "urn:ietf:params:aws:token-type:aws4_request",
		TokenURL:                       "https://sts.googleapis.com/v1/token",
		ServiceAccountImpersonationURL: fmt.Sprintf("https://iamcredentials.googleapis.com/v1/%s:generateAccessToken", clientID),
		CredentialSource: credentialSource{
			EnvironmentID:               "aws1",
			RegionalCredVerificationURL: "https://sts.us-east-1.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		},
	})
}

// Tests to make sure executable based external credentials continues to work.
// We're using the same setup as file based external account credentials, and using `cat` as the command
func TestExecutableBasedCredentials(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test")
	}

	// Set up Script as a executable file
	scriptFile, err := os.CreateTemp("", "script.sh")
	if err != nil {
		t.Fatalf("Error creating token file:")
	}
	defer os.Remove(scriptFile.Name())

	fmt.Fprintf(scriptFile, `#!/bin/bash
echo "{\"success\":true,\"version\":1,\"expiration_time\":%v,\"token_type\":\"urn:ietf:params:oauth:token-type:jwt\",\"id_token\":\"%v\"}"`,
		time.Now().Add(time.Hour).Unix(), oidcToken)
	scriptFile.Close()
	os.Chmod(scriptFile.Name(), 0700)

	// Run our test!
	testBYOID(t, config{
		Type:                           "external_account",
		Audience:                       oidcAudience,
		SubjectTokenType:               "urn:ietf:params:oauth:token-type:jwt",
		TokenURL:                       "https://sts.googleapis.com/v1/token",
		ServiceAccountImpersonationURL: fmt.Sprintf("https://iamcredentials.googleapis.com/v1/%s:generateAccessToken", clientID),
		CredentialSource: credentialSource{
			Executable: executableConfig{
				Command: scriptFile.Name(),
			},
		},
	})
}

func TestConfigurableTokenLifetime(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test")
	}

	// Set up Token as a file
	tokenFile, err := os.CreateTemp("", "token.txt")
	if err != nil {
		t.Fatalf("Error creating token file:")
	}
	defer os.Remove(tokenFile.Name())

	tokenFile.WriteString(oidcToken)
	tokenFile.Close()

	const tokenLifetimeSeconds = 2800
	const safetyBuffer = 5

	writeConfig(t, config{
		Type:                           "external_account",
		Audience:                       oidcAudience,
		SubjectTokenType:               "urn:ietf:params:oauth:token-type:jwt",
		TokenURL:                       "https://sts.googleapis.com/v1/token",
		ServiceAccountImpersonationURL: fmt.Sprintf("https://iamcredentials.googleapis.com/v1/%s:generateAccessToken", clientID),
		ServiceAccountImpersonation: serviceAccountImpersonationInfo{
			TokenLifetimeSeconds: tokenLifetimeSeconds,
		},
		CredentialSource: credentialSource{
			File: tokenFile.Name(),
		},
	}, func(filename string) {
		b, err := os.ReadFile(filename)
		if err != nil {
			t.Fatalf("Coudn't read temp config file")
		}

		creds, err := google.CredentialsFromJSON(context.Background(), b, "https://www.googleapis.com/auth/cloud-platform")
		if err != nil {
			t.Fatalf("Error retrieving credentials")
		}

		token, err := creds.TokenSource.Token()
		if err != nil {
			t.Fatalf("Error getting token")
		}

		now := time.Now()
		expiryMax := now.Add((safetyBuffer + tokenLifetimeSeconds) * time.Second)
		expiryMin := now.Add((tokenLifetimeSeconds - safetyBuffer) * time.Second)
		if token.Expiry.Before(expiryMin) || token.Expiry.After(expiryMax) {
			t.Fatalf("Expiry time not set correctly.  Got %v, want %v", token.Expiry, expiryMax)
		}
	})
}
