/*
 *
 * Copyright 2020 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package sts

import (
	"bytes"
	"context"
	"crypto/x509"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/http/httputil"
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"

	"google.golang.org/grpc/credentials"
	icredentials "google.golang.org/grpc/internal/credentials"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/testutils"
)

const (
	requestedTokenType      = "urn:ietf:params:oauth:token-type:access-token"
	actorTokenPath          = "/var/run/secrets/token.jwt"
	actorTokenType          = "urn:ietf:params:oauth:token-type:refresh_token"
	actorTokenContents      = "actorToken.jwt.contents"
	accessTokenContents     = "access_token"
	subjectTokenPath        = "/var/run/secrets/token.jwt"
	subjectTokenType        = "urn:ietf:params:oauth:token-type:id_token"
	subjectTokenContents    = "subjectToken.jwt.contents"
	serviceURI              = "http://localhost"
	exampleResource         = "https://backend.example.com/api"
	exampleAudience         = "example-backend-service"
	testScope               = "https://www.googleapis.com/auth/monitoring"
	defaultTestTimeout      = 1 * time.Second
	defaultTestShortTimeout = 10 * time.Millisecond
)

var (
	goodOptions = Options{
		TokenExchangeServiceURI: serviceURI,
		Audience:                exampleAudience,
		RequestedTokenType:      requestedTokenType,
		SubjectTokenPath:        subjectTokenPath,
		SubjectTokenType:        subjectTokenType,
	}
	goodRequestParams = &requestParameters{
		GrantType:          tokenExchangeGrantType,
		Audience:           exampleAudience,
		Scope:              defaultCloudPlatformScope,
		RequestedTokenType: requestedTokenType,
		SubjectToken:       subjectTokenContents,
		SubjectTokenType:   subjectTokenType,
	}
	goodMetadata = map[string]string{
		"Authorization": fmt.Sprintf("Bearer %s", accessTokenContents),
	}
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

// A struct that implements AuthInfo interface and added to the context passed
// to GetRequestMetadata from tests.
type testAuthInfo struct {
	credentials.CommonAuthInfo
}

func (ta testAuthInfo) AuthType() string {
	return "testAuthInfo"
}

func createTestContext(ctx context.Context, s credentials.SecurityLevel) context.Context {
	auth := &testAuthInfo{CommonAuthInfo: credentials.CommonAuthInfo{SecurityLevel: s}}
	ri := credentials.RequestInfo{
		Method:   "testInfo",
		AuthInfo: auth,
	}
	return icredentials.NewRequestInfoContext(ctx, ri)
}

// errReader implements the io.Reader interface and returns an error from the
// Read method.
type errReader struct{}

func (r errReader) Read([]byte) (n int, err error) {
	return 0, errors.New("read error")
}

// We need a function to construct the response instead of simply declaring it
// as a variable since the response body will be consumed by the
// credentials, and therefore we will need a new one everytime.
func makeGoodResponse() *http.Response {
	respJSON, _ := json.Marshal(responseParameters{
		AccessToken:     accessTokenContents,
		IssuedTokenType: "urn:ietf:params:oauth:token-type:access_token",
		TokenType:       "Bearer",
		ExpiresIn:       3600,
	})
	respBody := io.NopCloser(bytes.NewReader(respJSON))
	return &http.Response{
		Status:     "200 OK",
		StatusCode: http.StatusOK,
		Body:       respBody,
	}
}

// Overrides the http.Client with a fakeClient which sends a good response.
func overrideHTTPClientGood() (*testutils.FakeHTTPClient, func()) {
	fc := &testutils.FakeHTTPClient{
		ReqChan:  testutils.NewChannel(),
		RespChan: testutils.NewChannel(),
	}
	fc.RespChan.Send(makeGoodResponse())

	origMakeHTTPDoer := makeHTTPDoer
	makeHTTPDoer = func(_ *x509.CertPool) httpDoer { return fc }
	return fc, func() { makeHTTPDoer = origMakeHTTPDoer }
}

// Overrides the http.Client with the provided fakeClient.
func overrideHTTPClient(fc *testutils.FakeHTTPClient) func() {
	origMakeHTTPDoer := makeHTTPDoer
	makeHTTPDoer = func(_ *x509.CertPool) httpDoer { return fc }
	return func() { makeHTTPDoer = origMakeHTTPDoer }
}

// Overrides the subject token read to return a const which we can compare in
// our tests.
func overrideSubjectTokenGood() func() {
	origReadSubjectTokenFrom := readSubjectTokenFrom
	readSubjectTokenFrom = func(string) ([]byte, error) {
		return []byte(subjectTokenContents), nil
	}
	return func() { readSubjectTokenFrom = origReadSubjectTokenFrom }
}

// Overrides the subject token read to always return an error.
func overrideSubjectTokenError() func() {
	origReadSubjectTokenFrom := readSubjectTokenFrom
	readSubjectTokenFrom = func(string) ([]byte, error) {
		return nil, errors.New("error reading subject token")
	}
	return func() { readSubjectTokenFrom = origReadSubjectTokenFrom }
}

// Overrides the actor token read to return a const which we can compare in
// our tests.
func overrideActorTokenGood() func() {
	origReadActorTokenFrom := readActorTokenFrom
	readActorTokenFrom = func(string) ([]byte, error) {
		return []byte(actorTokenContents), nil
	}
	return func() { readActorTokenFrom = origReadActorTokenFrom }
}

// Overrides the actor token read to always return an error.
func overrideActorTokenError() func() {
	origReadActorTokenFrom := readActorTokenFrom
	readActorTokenFrom = func(string) ([]byte, error) {
		return nil, errors.New("error reading actor token")
	}
	return func() { readActorTokenFrom = origReadActorTokenFrom }
}

// compareRequest compares the http.Request received in the test with the
// expected requestParameters specified in wantReqParams.
func compareRequest(gotRequest *http.Request, wantReqParams *requestParameters) error {
	jsonBody, err := json.Marshal(wantReqParams)
	if err != nil {
		return err
	}
	wantReq, err := http.NewRequest("POST", serviceURI, bytes.NewBuffer(jsonBody))
	if err != nil {
		return fmt.Errorf("failed to create http request: %v", err)
	}
	wantReq.Header.Set("Content-Type", "application/json")

	wantR, err := httputil.DumpRequestOut(wantReq, true)
	if err != nil {
		return err
	}
	gotR, err := httputil.DumpRequestOut(gotRequest, true)
	if err != nil {
		return err
	}
	if diff := cmp.Diff(string(wantR), string(gotR)); diff != "" {
		return fmt.Errorf("sts request diff (-want +got):\n%s", diff)
	}
	return nil
}

// receiveAndCompareRequest waits for a request to be sent out by the
// credentials implementation using the fakeHTTPClient and compares it to an
// expected goodRequest. This is expected to be called in a separate goroutine
// by the tests. So, any errors encountered are pushed to an error channel
// which is monitored by the test.
func receiveAndCompareRequest(ReqChan *testutils.Channel, errCh chan error) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	val, err := ReqChan.Receive(ctx)
	if err != nil {
		errCh <- err
		return
	}
	req := val.(*http.Request)
	if err := compareRequest(req, goodRequestParams); err != nil {
		errCh <- err
		return
	}
	errCh <- nil
}

// TestGetRequestMetadataSuccess verifies the successful case of sending an
// token exchange request and processing the response.
func (s) TestGetRequestMetadataSuccess(t *testing.T) {
	defer overrideSubjectTokenGood()()
	fc, cancel := overrideHTTPClientGood()
	defer cancel()

	creds, err := NewCredentials(goodOptions)
	if err != nil {
		t.Fatalf("NewCredentials(%v) = %v", goodOptions, err)
	}

	errCh := make(chan error, 1)
	go receiveAndCompareRequest(fc.ReqChan, errCh)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	gotMetadata, err := creds.GetRequestMetadata(createTestContext(ctx, credentials.PrivacyAndIntegrity), "")
	if err != nil {
		t.Fatalf("creds.GetRequestMetadata() = %v", err)
	}
	if !cmp.Equal(gotMetadata, goodMetadata) {
		t.Fatalf("creds.GetRequestMetadata() = %v, want %v", gotMetadata, goodMetadata)
	}
	if err := <-errCh; err != nil {
		t.Fatal(err)
	}

	// Make another call to get request metadata and this should return contents
	// from the cache. This will fail if the credentials tries to send a fresh
	// request here since we have not configured our fakeClient to return any
	// response on retries.
	gotMetadata, err = creds.GetRequestMetadata(createTestContext(ctx, credentials.PrivacyAndIntegrity), "")
	if err != nil {
		t.Fatalf("creds.GetRequestMetadata() = %v", err)
	}
	if !cmp.Equal(gotMetadata, goodMetadata) {
		t.Fatalf("creds.GetRequestMetadata() = %v, want %v", gotMetadata, goodMetadata)
	}
}

// TestGetRequestMetadataBadSecurityLevel verifies the case where the
// securityLevel specified in the context passed to GetRequestMetadata is not
// sufficient.
func (s) TestGetRequestMetadataBadSecurityLevel(t *testing.T) {
	defer overrideSubjectTokenGood()()

	creds, err := NewCredentials(goodOptions)
	if err != nil {
		t.Fatalf("NewCredentials(%v) = %v", goodOptions, err)
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	gotMetadata, err := creds.GetRequestMetadata(createTestContext(ctx, credentials.IntegrityOnly), "")
	if err == nil {
		t.Fatalf("creds.GetRequestMetadata() succeeded with metadata %v, expected to fail", gotMetadata)
	}
}

// TestGetRequestMetadataCacheExpiry verifies the case where the cached access
// token has expired, and the credentials implementation will have to send a
// fresh token exchange request.
func (s) TestGetRequestMetadataCacheExpiry(t *testing.T) {
	const expiresInSecs = 1
	defer overrideSubjectTokenGood()()
	fc := &testutils.FakeHTTPClient{
		ReqChan:  testutils.NewChannel(),
		RespChan: testutils.NewChannel(),
	}
	defer overrideHTTPClient(fc)()

	creds, err := NewCredentials(goodOptions)
	if err != nil {
		t.Fatalf("NewCredentials(%v) = %v", goodOptions, err)
	}

	// The fakeClient is configured to return an access_token with a one second
	// expiry. So, in the second iteration, the credentials will find the cache
	// entry, but that would have expired, and therefore we expect it to send
	// out a fresh request.
	for i := 0; i < 2; i++ {
		errCh := make(chan error, 1)
		go receiveAndCompareRequest(fc.ReqChan, errCh)

		respJSON, _ := json.Marshal(responseParameters{
			AccessToken:     accessTokenContents,
			IssuedTokenType: "urn:ietf:params:oauth:token-type:access_token",
			TokenType:       "Bearer",
			ExpiresIn:       expiresInSecs,
		})
		respBody := io.NopCloser(bytes.NewReader(respJSON))
		resp := &http.Response{
			Status:     "200 OK",
			StatusCode: http.StatusOK,
			Body:       respBody,
		}
		fc.RespChan.Send(resp)

		ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
		defer cancel()
		gotMetadata, err := creds.GetRequestMetadata(createTestContext(ctx, credentials.PrivacyAndIntegrity), "")
		if err != nil {
			t.Fatalf("creds.GetRequestMetadata() = %v", err)
		}
		if !cmp.Equal(gotMetadata, goodMetadata) {
			t.Fatalf("creds.GetRequestMetadata() = %v, want %v", gotMetadata, goodMetadata)
		}
		if err := <-errCh; err != nil {
			t.Fatal(err)
		}
		time.Sleep(expiresInSecs * time.Second)
	}
}

// TestGetRequestMetadataBadResponses verifies the scenario where the token
// exchange server returns bad responses.
func (s) TestGetRequestMetadataBadResponses(t *testing.T) {
	tests := []struct {
		name     string
		response *http.Response
	}{
		{
			name: "bad JSON",
			response: &http.Response{
				Status:     "200 OK",
				StatusCode: http.StatusOK,
				Body:       io.NopCloser(strings.NewReader("not JSON")),
			},
		},
		{
			name: "no access token",
			response: &http.Response{
				Status:     "200 OK",
				StatusCode: http.StatusOK,
				Body:       io.NopCloser(strings.NewReader("{}")),
			},
		},
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			defer overrideSubjectTokenGood()()

			fc := &testutils.FakeHTTPClient{
				ReqChan:  testutils.NewChannel(),
				RespChan: testutils.NewChannel(),
			}
			defer overrideHTTPClient(fc)()

			creds, err := NewCredentials(goodOptions)
			if err != nil {
				t.Fatalf("NewCredentials(%v) = %v", goodOptions, err)
			}

			errCh := make(chan error, 1)
			go receiveAndCompareRequest(fc.ReqChan, errCh)

			fc.RespChan.Send(test.response)
			if _, err := creds.GetRequestMetadata(createTestContext(ctx, credentials.PrivacyAndIntegrity), ""); err == nil {
				t.Fatal("creds.GetRequestMetadata() succeeded when expected to fail")
			}
			if err := <-errCh; err != nil {
				t.Fatal(err)
			}
		})
	}
}

// TestGetRequestMetadataBadSubjectTokenRead verifies the scenario where the
// attempt to read the subjectToken fails.
func (s) TestGetRequestMetadataBadSubjectTokenRead(t *testing.T) {
	defer overrideSubjectTokenError()()
	fc, cancel := overrideHTTPClientGood()
	defer cancel()

	creds, err := NewCredentials(goodOptions)
	if err != nil {
		t.Fatalf("NewCredentials(%v) = %v", goodOptions, err)
	}

	errCh := make(chan error, 1)
	go func() {
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
		defer cancel()
		if _, err := fc.ReqChan.Receive(ctx); err != context.DeadlineExceeded {
			errCh <- err
			return
		}
		errCh <- nil
	}()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := creds.GetRequestMetadata(createTestContext(ctx, credentials.PrivacyAndIntegrity), ""); err == nil {
		t.Fatal("creds.GetRequestMetadata() succeeded when expected to fail")
	}
	if err := <-errCh; err != nil {
		t.Fatal(err)
	}
}

func (s) TestNewCredentials(t *testing.T) {
	tests := []struct {
		name           string
		opts           Options
		errSystemRoots bool
		wantErr        bool
	}{
		{
			name: "invalid options - empty subjectTokenPath",
			opts: Options{
				TokenExchangeServiceURI: serviceURI,
			},
			wantErr: true,
		},
		{
			name:           "invalid system root certs",
			opts:           goodOptions,
			errSystemRoots: true,
			wantErr:        true,
		},
		{
			name: "good case",
			opts: goodOptions,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			if test.errSystemRoots {
				oldSystemRoots := loadSystemCertPool
				loadSystemCertPool = func() (*x509.CertPool, error) {
					return nil, errors.New("failed to load system cert pool")
				}
				defer func() {
					loadSystemCertPool = oldSystemRoots
				}()
			}

			creds, err := NewCredentials(test.opts)
			if (err != nil) != test.wantErr {
				t.Fatalf("NewCredentials(%v) = %v, want %v", test.opts, err, test.wantErr)
			}
			if err == nil {
				if !creds.RequireTransportSecurity() {
					t.Errorf("creds.RequireTransportSecurity() returned false")
				}
			}
		})
	}
}

func (s) TestValidateOptions(t *testing.T) {
	tests := []struct {
		name          string
		opts          Options
		wantErrPrefix string
	}{
		{
			name:          "empty token exchange service URI",
			opts:          Options{},
			wantErrPrefix: "empty token_exchange_service_uri in options",
		},
		{
			name: "invalid URI",
			opts: Options{
				TokenExchangeServiceURI: "\tI'm a bad URI\n",
			},
			wantErrPrefix: "invalid control character in URL",
		},
		{
			name: "unsupported scheme",
			opts: Options{
				TokenExchangeServiceURI: "unix:///path/to/socket",
			},
			wantErrPrefix: "scheme is not supported",
		},
		{
			name: "empty subjectTokenPath",
			opts: Options{
				TokenExchangeServiceURI: serviceURI,
			},
			wantErrPrefix: "required field SubjectTokenPath is not specified",
		},
		{
			name: "empty subjectTokenType",
			opts: Options{
				TokenExchangeServiceURI: serviceURI,
				SubjectTokenPath:        subjectTokenPath,
			},
			wantErrPrefix: "required field SubjectTokenType is not specified",
		},
		{
			name: "good options",
			opts: goodOptions,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			err := validateOptions(test.opts)
			if (err != nil) != (test.wantErrPrefix != "") {
				t.Errorf("validateOptions(%v) = %v, want %v", test.opts, err, test.wantErrPrefix)
			}
			if err != nil && !strings.Contains(err.Error(), test.wantErrPrefix) {
				t.Errorf("validateOptions(%v) = %v, want %v", test.opts, err, test.wantErrPrefix)
			}
		})
	}
}

func (s) TestConstructRequest(t *testing.T) {
	tests := []struct {
		name                string
		opts                Options
		subjectTokenReadErr bool
		actorTokenReadErr   bool
		wantReqParams       *requestParameters
		wantErr             bool
	}{
		{
			name:                "subject token read failure",
			subjectTokenReadErr: true,
			opts:                goodOptions,
			wantErr:             true,
		},
		{
			name:              "actor token read failure",
			actorTokenReadErr: true,
			opts: Options{
				TokenExchangeServiceURI: serviceURI,
				Audience:                exampleAudience,
				RequestedTokenType:      requestedTokenType,
				SubjectTokenPath:        subjectTokenPath,
				SubjectTokenType:        subjectTokenType,
				ActorTokenPath:          actorTokenPath,
				ActorTokenType:          actorTokenType,
			},
			wantErr: true,
		},
		{
			name:          "default cloud platform scope",
			opts:          goodOptions,
			wantReqParams: goodRequestParams,
		},
		{
			name: "all good",
			opts: Options{
				TokenExchangeServiceURI: serviceURI,
				Resource:                exampleResource,
				Audience:                exampleAudience,
				Scope:                   testScope,
				RequestedTokenType:      requestedTokenType,
				SubjectTokenPath:        subjectTokenPath,
				SubjectTokenType:        subjectTokenType,
				ActorTokenPath:          actorTokenPath,
				ActorTokenType:          actorTokenType,
			},
			wantReqParams: &requestParameters{
				GrantType:          tokenExchangeGrantType,
				Resource:           exampleResource,
				Audience:           exampleAudience,
				Scope:              testScope,
				RequestedTokenType: requestedTokenType,
				SubjectToken:       subjectTokenContents,
				SubjectTokenType:   subjectTokenType,
				ActorToken:         actorTokenContents,
				ActorTokenType:     actorTokenType,
			},
		},
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			if test.subjectTokenReadErr {
				defer overrideSubjectTokenError()()
			} else {
				defer overrideSubjectTokenGood()()
			}

			if test.actorTokenReadErr {
				defer overrideActorTokenError()()
			} else {
				defer overrideActorTokenGood()()
			}

			gotRequest, err := constructRequest(ctx, test.opts)
			if (err != nil) != test.wantErr {
				t.Fatalf("constructRequest(%v) = %v, wantErr: %v", test.opts, err, test.wantErr)
			}
			if test.wantErr {
				return
			}
			if err := compareRequest(gotRequest, test.wantReqParams); err != nil {
				t.Fatal(err)
			}
		})
	}
}

func (s) TestSendRequest(t *testing.T) {
	defer overrideSubjectTokenGood()()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	req, err := constructRequest(ctx, goodOptions)
	if err != nil {
		t.Fatal(err)
	}

	tests := []struct {
		name    string
		resp    *http.Response
		respErr error
		wantErr bool
	}{
		{
			name:    "client error",
			respErr: errors.New("http.Client.Do failed"),
			wantErr: true,
		},
		{
			name: "bad response body",
			resp: &http.Response{
				Status:     "200 OK",
				StatusCode: http.StatusOK,
				Body:       io.NopCloser(errReader{}),
			},
			wantErr: true,
		},
		{
			name: "nonOK status code",
			resp: &http.Response{
				Status:     "400 BadRequest",
				StatusCode: http.StatusBadRequest,
				Body:       io.NopCloser(strings.NewReader("")),
			},
			wantErr: true,
		},
		{
			name: "good case",
			resp: makeGoodResponse(),
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			client := &testutils.FakeHTTPClient{
				ReqChan:  testutils.NewChannel(),
				RespChan: testutils.NewChannel(),
				Err:      test.respErr,
			}
			client.RespChan.Send(test.resp)
			_, err := sendRequest(client, req)
			if (err != nil) != test.wantErr {
				t.Errorf("sendRequest(%v) = %v, wantErr: %v", req, err, test.wantErr)
			}
		})
	}
}

func (s) TestTokenInfoFromResponse(t *testing.T) {
	noAccessToken, _ := json.Marshal(responseParameters{
		IssuedTokenType: "urn:ietf:params:oauth:token-type:access_token",
		TokenType:       "Bearer",
		ExpiresIn:       3600,
	})
	goodResponse, _ := json.Marshal(responseParameters{
		IssuedTokenType: requestedTokenType,
		AccessToken:     accessTokenContents,
		TokenType:       "Bearer",
		ExpiresIn:       3600,
	})

	tests := []struct {
		name          string
		respBody      []byte
		wantTokenInfo *tokenInfo
		wantErr       bool
	}{
		{
			name:     "bad JSON",
			respBody: []byte("not JSON"),
			wantErr:  true,
		},
		{
			name:     "empty response",
			respBody: []byte(""),
			wantErr:  true,
		},
		{
			name:     "non-empty response with no access token",
			respBody: noAccessToken,
			wantErr:  true,
		},
		{
			name:     "good response",
			respBody: goodResponse,
			wantTokenInfo: &tokenInfo{
				tokenType: "Bearer",
				token:     accessTokenContents,
			},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			gotTokenInfo, err := tokenInfoFromResponse(test.respBody)
			if (err != nil) != test.wantErr {
				t.Fatalf("tokenInfoFromResponse(%+v) = %v, wantErr: %v", test.respBody, err, test.wantErr)
			}
			if test.wantErr {
				return
			}
			// Can't do a cmp.Equal on the whole struct since the expiryField
			// is populated based on time.Now().
			if gotTokenInfo.tokenType != test.wantTokenInfo.tokenType || gotTokenInfo.token != test.wantTokenInfo.token {
				t.Errorf("tokenInfoFromResponse(%+v) = %+v, want: %+v", test.respBody, gotTokenInfo, test.wantTokenInfo)
			}
		})
	}
}
