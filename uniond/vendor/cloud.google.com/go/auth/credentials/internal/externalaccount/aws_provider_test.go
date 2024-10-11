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
	"errors"
	"fmt"
	"net/http"
	"net/http/httptest"
	neturl "net/url"
	"strings"
	"testing"
	"time"

	"cloud.google.com/go/auth/internal/credsfile"
	"github.com/google/go-cmp/cmp"
)

type validateHeaders func(r *http.Request)

const (
	accessKeyID     = "accessKeyID"
	secretAccessKey = "secret"
	sessionToken    = "sessionTok"
)

var (
	defaultTime            = time.Date(2011, 9, 9, 23, 36, 0, 0, time.UTC)
	secondDefaultTime      = time.Date(2020, 8, 11, 6, 55, 22, 0, time.UTC)
	requestSignerWithToken = &awsRequestSigner{
		RegionName: "us-east-2",
		AwsSecurityCredentials: &AwsSecurityCredentials{
			AccessKeyID:     accessKeyID,
			SecretAccessKey: secretAccessKey,
			SessionToken:    sessionToken,
		},
	}
)

func TestAWSv4Signature_GetRequest(t *testing.T) {
	input, _ := http.NewRequest("GET", "https://host.foo.com", nil)
	setDefaultTime(input)

	output, _ := http.NewRequest("GET", "https://host.foo.com", nil)
	output.Header = http.Header{
		"Host":          []string{"host.foo.com"},
		"Date":          []string{"Mon, 09 Sep 2011 23:36:00 GMT"},
		"Authorization": []string{"AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20110909/us-east-1/host/aws4_request, SignedHeaders=date;host, Signature=b27ccfbfa7df52a200ff74193ca6e32d4b48b8856fab7ebf1c595d0670a7e470"},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(defaultTime)

	testRequestSigner(t, defaultRequestSigner, input, output)
}

func TestAWSv4Signature_GetRequestWithRelativePath(t *testing.T) {
	input, _ := http.NewRequest("GET", "https://host.foo.com/foo/bar/../..", nil)
	setDefaultTime(input)

	output, _ := http.NewRequest("GET", "https://host.foo.com/foo/bar/../..", nil)
	output.Header = http.Header{
		"Host":          []string{"host.foo.com"},
		"Date":          []string{"Mon, 09 Sep 2011 23:36:00 GMT"},
		"Authorization": []string{"AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20110909/us-east-1/host/aws4_request, SignedHeaders=date;host, Signature=b27ccfbfa7df52a200ff74193ca6e32d4b48b8856fab7ebf1c595d0670a7e470"},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(defaultTime)

	testRequestSigner(t, defaultRequestSigner, input, output)
}

func TestAWSv4Signature_GetRequestWithDotPath(t *testing.T) {
	input, _ := http.NewRequest("GET", "https://host.foo.com/./", nil)
	setDefaultTime(input)

	output, _ := http.NewRequest("GET", "https://host.foo.com/./", nil)
	output.Header = http.Header{
		"Host":          []string{"host.foo.com"},
		"Date":          []string{"Mon, 09 Sep 2011 23:36:00 GMT"},
		"Authorization": []string{"AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20110909/us-east-1/host/aws4_request, SignedHeaders=date;host, Signature=b27ccfbfa7df52a200ff74193ca6e32d4b48b8856fab7ebf1c595d0670a7e470"},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(defaultTime)

	testRequestSigner(t, defaultRequestSigner, input, output)
}

func TestAWSv4Signature_GetRequestWithPointlessDotPath(t *testing.T) {
	input, _ := http.NewRequest("GET", "https://host.foo.com/./foo", nil)
	setDefaultTime(input)

	output, _ := http.NewRequest("GET", "https://host.foo.com/./foo", nil)
	output.Header = http.Header{
		"Host":          []string{"host.foo.com"},
		"Date":          []string{"Mon, 09 Sep 2011 23:36:00 GMT"},
		"Authorization": []string{"AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20110909/us-east-1/host/aws4_request, SignedHeaders=date;host, Signature=910e4d6c9abafaf87898e1eb4c929135782ea25bb0279703146455745391e63a"},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(defaultTime)

	testRequestSigner(t, defaultRequestSigner, input, output)
}

func TestAWSv4Signature_GetRequestWithUtf8Path(t *testing.T) {
	input, _ := http.NewRequest("GET", "https://host.foo.com/%E1%88%B4", nil)
	setDefaultTime(input)

	output, _ := http.NewRequest("GET", "https://host.foo.com/%E1%88%B4", nil)
	output.Header = http.Header{
		"Host":          []string{"host.foo.com"},
		"Date":          []string{"Mon, 09 Sep 2011 23:36:00 GMT"},
		"Authorization": []string{"AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20110909/us-east-1/host/aws4_request, SignedHeaders=date;host, Signature=8d6634c189aa8c75c2e51e106b6b5121bed103fdb351f7d7d4381c738823af74"},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(defaultTime)

	testRequestSigner(t, defaultRequestSigner, input, output)
}

func TestAWSv4Signature_GetRequestWithDuplicateQuery(t *testing.T) {
	input, _ := http.NewRequest("GET", "https://host.foo.com/?foo=Zoo&foo=aha", nil)
	setDefaultTime(input)

	output, _ := http.NewRequest("GET", "https://host.foo.com/?foo=Zoo&foo=aha", nil)
	output.Header = http.Header{
		"Host":          []string{"host.foo.com"},
		"Date":          []string{"Mon, 09 Sep 2011 23:36:00 GMT"},
		"Authorization": []string{"AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20110909/us-east-1/host/aws4_request, SignedHeaders=date;host, Signature=be7148d34ebccdc6423b19085378aa0bee970bdc61d144bd1a8c48c33079ab09"},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(defaultTime)

	testRequestSigner(t, defaultRequestSigner, input, output)
}

func TestAWSv4Signature_GetRequestWithMisorderedQuery(t *testing.T) {
	input, _ := http.NewRequest("GET", "https://host.foo.com/?foo=b&foo=a", nil)
	setDefaultTime(input)

	output, _ := http.NewRequest("GET", "https://host.foo.com/?foo=b&foo=a", nil)
	output.Header = http.Header{
		"Host":          []string{"host.foo.com"},
		"Date":          []string{"Mon, 09 Sep 2011 23:36:00 GMT"},
		"Authorization": []string{"AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20110909/us-east-1/host/aws4_request, SignedHeaders=date;host, Signature=feb926e49e382bec75c9d7dcb2a1b6dc8aa50ca43c25d2bc51143768c0875acc"},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(defaultTime)

	testRequestSigner(t, defaultRequestSigner, input, output)
}

func TestAWSv4Signature_GetRequestWithUtf8Query(t *testing.T) {
	input, _ := http.NewRequest("GET", "https://host.foo.com/?ሴ=bar", nil)
	setDefaultTime(input)

	output, _ := http.NewRequest("GET", "https://host.foo.com/?ሴ=bar", nil)
	output.Header = http.Header{
		"Host":          []string{"host.foo.com"},
		"Date":          []string{"Mon, 09 Sep 2011 23:36:00 GMT"},
		"Authorization": []string{"AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20110909/us-east-1/host/aws4_request, SignedHeaders=date;host, Signature=6fb359e9a05394cc7074e0feb42573a2601abc0c869a953e8c5c12e4e01f1a8c"},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(defaultTime)

	testRequestSigner(t, defaultRequestSigner, input, output)
}

func TestAWSv4Signature_PostRequest(t *testing.T) {
	input, _ := http.NewRequest("POST", "https://host.foo.com/", nil)
	setDefaultTime(input)
	input.Header.Set("ZOO", "zoobar")

	output, _ := http.NewRequest("POST", "https://host.foo.com/", nil)
	output.Header = http.Header{
		"Host":          []string{"host.foo.com"},
		"Date":          []string{"Mon, 09 Sep 2011 23:36:00 GMT"},
		"Authorization": []string{"AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20110909/us-east-1/host/aws4_request, SignedHeaders=date;host;zoo, Signature=b7a95a52518abbca0964a999a880429ab734f35ebbf1235bd79a5de87756dc4a"},
		"Zoo":           []string{"zoobar"},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(defaultTime)

	testRequestSigner(t, defaultRequestSigner, input, output)
}

func TestAWSv4Signature_PostRequestWithCapitalizedHeaderValue(t *testing.T) {
	input, _ := http.NewRequest("POST", "https://host.foo.com/", nil)
	setDefaultTime(input)
	input.Header.Set("zoo", "ZOOBAR")

	output, _ := http.NewRequest("POST", "https://host.foo.com/", nil)
	output.Header = http.Header{
		"Host":          []string{"host.foo.com"},
		"Date":          []string{"Mon, 09 Sep 2011 23:36:00 GMT"},
		"Authorization": []string{"AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20110909/us-east-1/host/aws4_request, SignedHeaders=date;host;zoo, Signature=273313af9d0c265c531e11db70bbd653f3ba074c1009239e8559d3987039cad7"},
		"Zoo":           []string{"ZOOBAR"},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(defaultTime)

	testRequestSigner(t, defaultRequestSigner, input, output)
}

func TestAWSv4Signature_PostRequestPhfft(t *testing.T) {
	input, _ := http.NewRequest("POST", "https://host.foo.com/", nil)
	setDefaultTime(input)
	input.Header.Set("p", "phfft")

	output, _ := http.NewRequest("POST", "https://host.foo.com/", nil)
	output.Header = http.Header{
		"Host":          []string{"host.foo.com"},
		"Date":          []string{"Mon, 09 Sep 2011 23:36:00 GMT"},
		"Authorization": []string{"AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20110909/us-east-1/host/aws4_request, SignedHeaders=date;host;p, Signature=debf546796015d6f6ded8626f5ce98597c33b47b9164cf6b17b4642036fcb592"},
		"P":             []string{"phfft"},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(defaultTime)

	testRequestSigner(t, defaultRequestSigner, input, output)
}

func TestAWSv4Signature_PostRequestWithBody(t *testing.T) {
	input, _ := http.NewRequest("POST", "https://host.foo.com/", strings.NewReader("foo=bar"))
	setDefaultTime(input)
	input.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	output, _ := http.NewRequest("POST", "https://host.foo.com/", nil)
	output.Header = http.Header{
		"Host":          []string{"host.foo.com"},
		"Date":          []string{"Mon, 09 Sep 2011 23:36:00 GMT"},
		"Content-Type":  []string{"application/x-www-form-urlencoded"},
		"Authorization": []string{"AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20110909/us-east-1/host/aws4_request, SignedHeaders=content-type;date;host, Signature=5a15b22cf462f047318703b92e6f4f38884e4a7ab7b1d6426ca46a8bd1c26cbc"},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(defaultTime)

	testRequestSigner(t, defaultRequestSigner, input, output)
}

func TestAWSv4Signature_PostRequestWithQueryString(t *testing.T) {
	input, _ := http.NewRequest("POST", "https://host.foo.com/?foo=bar", nil)
	setDefaultTime(input)

	output, _ := http.NewRequest("POST", "https://host.foo.com/?foo=bar", nil)
	output.Header = http.Header{
		"Host":          []string{"host.foo.com"},
		"Date":          []string{"Mon, 09 Sep 2011 23:36:00 GMT"},
		"Authorization": []string{"AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20110909/us-east-1/host/aws4_request, SignedHeaders=date;host, Signature=b6e3b79003ce0743a491606ba1035a804593b0efb1e20a11cba83f8c25a57a92"},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(defaultTime)

	testRequestSigner(t, defaultRequestSigner, input, output)
}

func TestAWSv4Signature_GetRequestWithSecurityToken(t *testing.T) {
	input, _ := http.NewRequest("GET", "https://ec2.us-east-2.amazonaws.com?Action=DescribeRegions&Version=2013-10-15", nil)

	output, _ := http.NewRequest("GET", "https://ec2.us-east-2.amazonaws.com?Action=DescribeRegions&Version=2013-10-15", nil)
	output.Header = http.Header{
		"Host":                 []string{"ec2.us-east-2.amazonaws.com"},
		"Authorization":        []string{"AWS4-HMAC-SHA256 Credential=" + accessKeyID + "/20200811/us-east-2/ec2/aws4_request, SignedHeaders=host;x-amz-date;x-amz-security-token, Signature=361dc730fd43b4330fa20603a7fbc305ef10b5be125d219ebef40f18569ef5b2"},
		"X-Amz-Date":           []string{"20200811T065522Z"},
		"X-Amz-Security-Token": []string{sessionToken},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(secondDefaultTime)

	testRequestSigner(t, requestSignerWithToken, input, output)
}

func TestAWSv4Signature_PostRequestWithSecurityToken(t *testing.T) {
	input, _ := http.NewRequest("POST", "https://sts.us-east-2.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15", nil)

	output, _ := http.NewRequest("POST", "https://sts.us-east-2.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15", nil)
	output.Header = http.Header{
		"Authorization":        []string{"AWS4-HMAC-SHA256 Credential=" + accessKeyID + "/20200811/us-east-2/sts/aws4_request, SignedHeaders=host;x-amz-date;x-amz-security-token, Signature=48541de09ff691ab53f9e017f5123ce338fecbadc1b278480bab3af221ca4f38"},
		"Host":                 []string{"sts.us-east-2.amazonaws.com"},
		"X-Amz-Date":           []string{"20200811T065522Z"},
		"X-Amz-Security-Token": []string{sessionToken},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(secondDefaultTime)

	testRequestSigner(t, requestSignerWithToken, input, output)
}

func TestAWSv4Signature_PostRequestWithSecurityTokenAndAdditionalHeaders(t *testing.T) {
	requestParams := "{\"KeySchema\":[{\"KeyType\":\"HASH\",\"AttributeName\":\"Id\"}],\"TableName\":\"TestTable\",\"AttributeDefinitions\":[{\"AttributeName\":\"Id\",\"AttributeType\":\"S\"}],\"ProvisionedThroughput\":{\"WriteCapacityUnits\":5,\"ReadCapacityUnits\":5}}"
	input, _ := http.NewRequest("POST", "https://dynamodb.us-east-2.amazonaws.com/", strings.NewReader(requestParams))
	input.Header.Set("Content-Type", "application/x-amz-json-1.0")
	input.Header.Set("x-amz-target", "DynamoDB_20120810.CreateTable")

	output, _ := http.NewRequest("POST", "https://dynamodb.us-east-2.amazonaws.com/", strings.NewReader(requestParams))
	output.Header = http.Header{
		"Authorization":        []string{"AWS4-HMAC-SHA256 Credential=" + accessKeyID + "/20200811/us-east-2/dynamodb/aws4_request, SignedHeaders=content-type;host;x-amz-date;x-amz-security-token;x-amz-target, Signature=ae7b75b3c0a9ef64626a4e9e6c3d503745dd0a109cb35d56d87c617eae804c00"},
		"Host":                 []string{"dynamodb.us-east-2.amazonaws.com"},
		"X-Amz-Date":           []string{"20200811T065522Z"},
		"Content-Type":         []string{"application/x-amz-json-1.0"},
		"X-Amz-Target":         []string{"DynamoDB_20120810.CreateTable"},
		"X-Amz-Security-Token": []string{sessionToken},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(secondDefaultTime)

	testRequestSigner(t, requestSignerWithToken, input, output)
}

func TestAWSv4Signature_PostRequestWithAmzDateButNoSecurityToken(t *testing.T) {
	var requestSigner = &awsRequestSigner{
		RegionName: "us-east-2",
		AwsSecurityCredentials: &AwsSecurityCredentials{
			AccessKeyID:     accessKeyID,
			SecretAccessKey: secretAccessKey,
		},
	}

	input, _ := http.NewRequest("POST", "https://sts.us-east-2.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15", nil)

	output, _ := http.NewRequest("POST", "https://sts.us-east-2.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15", nil)
	output.Header = http.Header{
		"Authorization": []string{"AWS4-HMAC-SHA256 Credential=" + accessKeyID + "/20200811/us-east-2/sts/aws4_request, SignedHeaders=host;x-amz-date, Signature=bff58112d4696faecff9c104c8b7b963141e8b3cc4ba46a0664938fe6d112e41"},
		"Host":          []string{"sts.us-east-2.amazonaws.com"},
		"X-Amz-Date":    []string{"20200811T065522Z"},
	}

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = setTime(secondDefaultTime)

	testRequestSigner(t, requestSigner, input, output)
}

type testAwsServer struct {
	url                         string
	securityCredentialURL       string
	regionURL                   string
	regionalCredVerificationURL string
	imdsv2SessionTokenURL       string

	Credentials map[string]string

	WriteRolename            func(http.ResponseWriter, *http.Request)
	WriteSecurityCredentials func(http.ResponseWriter, *http.Request)
	WriteRegion              func(http.ResponseWriter, *http.Request)
	WriteIMDSv2SessionToken  func(http.ResponseWriter, *http.Request)
}

func createAwsTestServer(url, regionURL, regionalCredVerificationURL, imdsv2SessionTokenURL string, rolename, region string, credentials map[string]string, imdsv2SessionToken string, validateHeaders validateHeaders) *testAwsServer {
	server := &testAwsServer{
		url:                         url,
		securityCredentialURL:       fmt.Sprintf("%s/%s", url, rolename),
		regionURL:                   regionURL,
		regionalCredVerificationURL: regionalCredVerificationURL,
		imdsv2SessionTokenURL:       imdsv2SessionTokenURL,
		Credentials:                 credentials,
		WriteRolename: func(w http.ResponseWriter, r *http.Request) {
			validateHeaders(r)
			w.Write([]byte(rolename))
		},
		WriteRegion: func(w http.ResponseWriter, r *http.Request) {
			validateHeaders(r)
			w.Write([]byte(region))
		},
		WriteIMDSv2SessionToken: func(w http.ResponseWriter, r *http.Request) {
			validateHeaders(r)
			w.Write([]byte(imdsv2SessionToken))
		},
	}

	server.WriteSecurityCredentials = func(w http.ResponseWriter, r *http.Request) {
		validateHeaders(r)
		jsonCredentials, _ := json.Marshal(server.Credentials)
		w.Write(jsonCredentials)
	}

	return server
}

func createDefaultAwsTestServer() *testAwsServer {
	return createAwsTestServer(
		"/latest/meta-data/iam/security-credentials",
		"/latest/meta-data/placement/availability-zone",
		"https://sts.{region}.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		"",
		"gcp-aws-role",
		"us-east-2b",
		map[string]string{
			"SecretAccessKey": secretAccessKey,
			"AccessKeyId":     accessKeyID,
			"Token":           sessionToken,
		},
		"",
		noHeaderValidation,
	)
}

func createDefaultAwsTestServerWithImdsv2(t *testing.T) *testAwsServer {
	validateSessionTokenHeaders := func(r *http.Request) {
		if r.URL.Path == "/latest/api/token" {
			headerValue := r.Header.Get(awsIMDSv2SessionTTLHeader)
			if headerValue != awsIMDSv2SessionTTL {
				t.Errorf("%q = \n%q\n want \n%q", awsIMDSv2SessionTTLHeader, headerValue, awsIMDSv2SessionTTL)
			}
		} else {
			headerValue := r.Header.Get(awsIMDSv2SessionTokenHeader)
			if headerValue != "sessiontoken" {
				t.Errorf("%q = \n%q\n want \n%q", awsIMDSv2SessionTokenHeader, headerValue, "sessiontoken")
			}
		}
	}

	return createAwsTestServer(
		"/latest/meta-data/iam/security-credentials",
		"/latest/meta-data/placement/availability-zone",
		"https://sts.{region}.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		"/latest/api/token",
		"gcp-aws-role",
		"us-east-2b",
		map[string]string{
			"SecretAccessKey": secretAccessKey,
			"AccessKeyId":     accessKeyID,
			"Token":           sessionToken,
		},
		"sessiontoken",
		validateSessionTokenHeaders,
	)
}

func (server *testAwsServer) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	switch p := r.URL.Path; p {
	case server.url:
		server.WriteRolename(w, r)
	case server.securityCredentialURL:
		server.WriteSecurityCredentials(w, r)
	case server.regionURL:
		server.WriteRegion(w, r)
	case server.imdsv2SessionTokenURL:
		server.WriteIMDSv2SessionToken(w, r)
	}
}

func notFound(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(404)
	w.Write([]byte("Not Found"))
}

func noHeaderValidation(r *http.Request) {}

func (server *testAwsServer) getCredentialSource(url string) *credsfile.CredentialSource {
	return &credsfile.CredentialSource{
		EnvironmentID:               "aws1",
		URL:                         url + server.url,
		RegionURL:                   url + server.regionURL,
		RegionalCredVerificationURL: server.regionalCredVerificationURL,
		IMDSv2SessionTokenURL:       url + server.imdsv2SessionTokenURL,
	}
}

func getExpectedSubjectToken(url, region, accessKeyID, secretAccessKey, sessionToken string) string {
	req, _ := http.NewRequest("POST", url, nil)
	req.Header.Set("x-goog-cloud-target-resource", cloneTestOpts().Audience)
	signer := &awsRequestSigner{
		RegionName: region,
		AwsSecurityCredentials: &AwsSecurityCredentials{
			AccessKeyID:     accessKeyID,
			SecretAccessKey: secretAccessKey,
			SessionToken:    sessionToken,
		},
	}
	signer.signRequest(req)

	result := awsRequest{
		URL:    url,
		Method: "POST",
		Headers: []awsRequestHeader{
			{
				Key:   "Authorization",
				Value: req.Header.Get("Authorization"),
			}, {
				Key:   "Host",
				Value: req.Header.Get("Host"),
			}, {
				Key:   "X-Amz-Date",
				Value: req.Header.Get("X-Amz-Date"),
			},
		},
	}

	if sessionToken != "" {
		result.Headers = append(result.Headers, awsRequestHeader{
			Key:   "X-Amz-Security-Token",
			Value: sessionToken,
		})
	}

	result.Headers = append(result.Headers, awsRequestHeader{
		Key:   "X-Goog-Cloud-Target-Resource",
		Value: cloneTestOpts().Audience,
	})

	str, _ := json.Marshal(result)
	return neturl.QueryEscape(string(str))
}

func TestAWSCredential_BasicRequest(t *testing.T) {
	server := createDefaultAwsTestServer()
	ts := httptest.NewServer(server)

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)

	oldGetenv := getenv
	oldNow := Now
	defer func() {
		getenv = oldGetenv
		Now = oldNow
	}()
	getenv = setEnvironment(map[string]string{})
	Now = setTime(defaultTime)

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("retrieveSubjectToken() failed: %v", err)
	}

	if got, want := base.providerType(), awsProviderType; got != want {
		t.Fatalf("got %q, want %q", got, want)
	}

	want := getExpectedSubjectToken(
		"https://sts.us-east-2.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		"us-east-2",
		accessKeyID,
		secretAccessKey,
		sessionToken,
	)

	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestAWSCredential_IMDSv2(t *testing.T) {
	server := createDefaultAwsTestServerWithImdsv2(t)
	ts := httptest.NewServer(server)

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)

	oldGetenv := getenv
	oldNow := Now
	defer func() {
		getenv = oldGetenv
		Now = oldNow
	}()
	getenv = setEnvironment(map[string]string{})
	Now = setTime(defaultTime)

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("retrieveSubjectToken() failed: %v", err)
	}

	want := getExpectedSubjectToken(
		"https://sts.us-east-2.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		"us-east-2",
		accessKeyID,
		secretAccessKey,
		sessionToken,
	)

	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestAWSCredential_BasicRequestWithoutSecurityToken(t *testing.T) {
	server := createDefaultAwsTestServer()
	ts := httptest.NewServer(server)
	delete(server.Credentials, "Token")

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)

	oldGetenv := getenv
	oldNow := Now
	defer func() {
		getenv = oldGetenv
		Now = oldNow
	}()
	getenv = setEnvironment(map[string]string{})
	Now = setTime(defaultTime)

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("retrieveSubjectToken() failed: %v", err)
	}

	want := getExpectedSubjectToken(
		"https://sts.us-east-2.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		"us-east-2",
		accessKeyID,
		secretAccessKey,
		"",
	)

	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestAWSCredential_BasicRequestWithEnv(t *testing.T) {
	server := createDefaultAwsTestServer()
	ts := httptest.NewServer(server)

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)

	oldGetenv := getenv
	oldNow := Now
	defer func() {
		getenv = oldGetenv
		Now = oldNow
	}()
	getenv = setEnvironment(map[string]string{
		"AWS_ACCESS_KEY_ID":     "AKIDEXAMPLE",
		"AWS_SECRET_ACCESS_KEY": "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
		"AWS_REGION":            "us-west-1",
	})
	Now = setTime(defaultTime)

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("retrieveSubjectToken() failed: %v", err)
	}

	want := getExpectedSubjectToken(
		"https://sts.us-west-1.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		"us-west-1",
		"AKIDEXAMPLE",
		"wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
		"",
	)

	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestAWSCredential_BasicRequestWithDefaultEnv(t *testing.T) {
	server := createDefaultAwsTestServer()
	ts := httptest.NewServer(server)

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)

	oldGetenv := getenv
	oldNow := Now
	defer func() {
		getenv = oldGetenv
		Now = oldNow
	}()
	getenv = setEnvironment(map[string]string{
		"AWS_ACCESS_KEY_ID":     "AKIDEXAMPLE",
		"AWS_SECRET_ACCESS_KEY": "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
		"AWS_REGION":            "us-west-1",
	})
	Now = setTime(defaultTime)

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("retrieveSubjectToken() failed: %v", err)
	}
	want := getExpectedSubjectToken(
		"https://sts.us-west-1.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		"us-west-1",
		"AKIDEXAMPLE",
		"wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
		"",
	)

	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestAWSCredential_BasicRequestWithTwoRegions(t *testing.T) {
	server := createDefaultAwsTestServer()
	ts := httptest.NewServer(server)
	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)

	oldGetenv := getenv
	oldNow := Now
	defer func() {
		getenv = oldGetenv
		Now = oldNow
	}()
	getenv = setEnvironment(map[string]string{
		"AWS_ACCESS_KEY_ID":     "AKIDEXAMPLE",
		"AWS_SECRET_ACCESS_KEY": "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
		"AWS_REGION":            "us-west-1",
		"AWS_DEFAULT_REGION":    "us-east-1",
	})
	Now = setTime(defaultTime)

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("retrieveSubjectToken() failed: %v", err)
	}
	want := getExpectedSubjectToken(
		"https://sts.us-west-1.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		"us-west-1",
		"AKIDEXAMPLE",
		"wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
		"",
	)

	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestAWSCredential_RequestWithBadVersion(t *testing.T) {
	server := createDefaultAwsTestServer()
	ts := httptest.NewServer(server)

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)
	opts.CredentialSource.EnvironmentID = "aws3"

	oldGetenv := getenv
	defer func() {
		getenv = oldGetenv
	}()
	getenv = setEnvironment(map[string]string{})

	_, err := newSubjectTokenProvider(opts)
	if got, want := err.Error(), "credentials: aws version '3' is not supported in the current build"; got != want {
		t.Errorf("subjectToken = %q, want %q", got, want)
	}
}

func TestAWSCredential_RequestWithNoRegionURL(t *testing.T) {
	server := createDefaultAwsTestServer()
	ts := httptest.NewServer(server)

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)
	opts.CredentialSource.RegionURL = ""

	oldGetenv := getenv
	defer func() {
		getenv = oldGetenv
	}()
	getenv = setEnvironment(map[string]string{})

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	_, err = base.subjectToken(context.Background())
	if err == nil {
		t.Fatalf("retrieveSubjectToken() should have failed")
	}

	if got, want := err.Error(), "credentials: unable to determine AWS region"; got != want {
		t.Errorf("subjectToken = %q, want %q", got, want)
	}
}

func TestAWSCredential_RequestWithBadRegionURL(t *testing.T) {
	server := createDefaultAwsTestServer()
	ts := httptest.NewServer(server)
	server.WriteRegion = notFound

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)

	oldGetenv := getenv
	defer func() {
		getenv = oldGetenv
	}()
	getenv = setEnvironment(map[string]string{})

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	_, err = base.subjectToken(context.Background())
	if err == nil {
		t.Fatalf("retrieveSubjectToken() should have failed")
	}

	if got, want := err.Error(), "credentials: unable to retrieve AWS region - Not Found"; got != want {
		t.Errorf("subjectToken = %q, want %q", got, want)
	}
}

func TestAWSCredential_RequestWithMissingCredential(t *testing.T) {
	server := createDefaultAwsTestServer()
	ts := httptest.NewServer(server)
	server.WriteSecurityCredentials = func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte("{}"))
	}

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)

	oldGetenv := getenv
	defer func() {
		getenv = oldGetenv
	}()
	getenv = setEnvironment(map[string]string{})

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	_, err = base.subjectToken(context.Background())
	if err == nil {
		t.Fatalf("retrieveSubjectToken() should have failed")
	}

	if got, want := err.Error(), "credentials: missing AccessKeyId credential"; got != want {
		t.Errorf("subjectToken = %q, want %q", got, want)
	}
}

func TestAWSCredential_RequestWithIncompleteCredential(t *testing.T) {
	server := createDefaultAwsTestServer()
	ts := httptest.NewServer(server)
	server.WriteSecurityCredentials = func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte(`{"AccessKeyId":"FOOBARBAS"}`))
	}

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)

	oldGetenv := getenv
	defer func() {
		getenv = oldGetenv
	}()
	getenv = setEnvironment(map[string]string{})

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	_, err = base.subjectToken(context.Background())
	if err == nil {
		t.Fatalf("retrieveSubjectToken() should have failed")
	}

	if got, want := err.Error(), "credentials: missing SecretAccessKey credential"; got != want {
		t.Errorf("subjectToken = %q, want %q", got, want)
	}
}

func TestAWSCredential_RequestWithNoCredentialURL(t *testing.T) {
	server := createDefaultAwsTestServer()
	ts := httptest.NewServer(server)

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)
	opts.CredentialSource.URL = ""

	oldGetenv := getenv
	defer func() {
		getenv = oldGetenv
	}()
	getenv = setEnvironment(map[string]string{})

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	_, err = base.subjectToken(context.Background())
	if err == nil {
		t.Fatalf("retrieveSubjectToken() should have failed")
	}

	if got, want := err.Error(), "credentials: unable to determine the AWS metadata server security credentials endpoint"; got != want {
		t.Errorf("subjectToken = %q, want %q", got, want)
	}
}

func TestAWSCredential_RequestWithBadCredentialURL(t *testing.T) {
	server := createDefaultAwsTestServer()
	ts := httptest.NewServer(server)
	server.WriteRolename = notFound

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)

	oldGetenv := getenv
	defer func() {
		getenv = oldGetenv
	}()
	getenv = setEnvironment(map[string]string{})

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	_, err = base.subjectToken(context.Background())
	if err == nil {
		t.Fatalf("retrieveSubjectToken() should have failed")
	}

	if got, want := err.Error(), "credentials: unable to retrieve AWS role name - Not Found"; got != want {
		t.Errorf("subjectToken = %q, want %q", got, want)
	}
}

func TestAWSCredential_RequestWithBadFinalCredentialURL(t *testing.T) {
	server := createDefaultAwsTestServer()
	ts := httptest.NewServer(server)
	server.WriteSecurityCredentials = notFound

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)

	oldGetenv := getenv
	defer func() {
		getenv = oldGetenv
	}()
	getenv = setEnvironment(map[string]string{})

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	_, err = base.subjectToken(context.Background())
	if err == nil {
		t.Fatalf("retrieveSubjectToken() should have failed")
	}

	if got, want := err.Error(), "credentials: unable to retrieve AWS security credentials - Not Found"; got != want {
		t.Errorf("subjectToken = %q, want %q", got, want)
	}
}

func TestAWSCredential_ShouldNotCallMetadataEndpointWhenCredsAreInEnv(t *testing.T) {
	server := createDefaultAwsTestServer()
	ts := httptest.NewServer(server)

	metadataTs := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Error("Metadata server should not have been called.")
	}))

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)
	opts.CredentialSource.IMDSv2SessionTokenURL = metadataTs.URL

	oldGetenv := getenv
	oldNow := Now
	defer func() {
		getenv = oldGetenv
		Now = oldNow
	}()
	getenv = setEnvironment(map[string]string{
		"AWS_ACCESS_KEY_ID":     "AKIDEXAMPLE",
		"AWS_SECRET_ACCESS_KEY": "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
		"AWS_REGION":            "us-west-1",
	})
	Now = setTime(defaultTime)

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("retrieveSubjectToken() failed: %v", err)
	}

	want := getExpectedSubjectToken(
		"https://sts.us-west-1.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		"us-west-1",
		"AKIDEXAMPLE",
		"wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
		"",
	)

	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestAWSCredential_ShouldCallMetadataEndpointWhenNoRegion(t *testing.T) {
	server := createDefaultAwsTestServerWithImdsv2(t)
	ts := httptest.NewServer(server)

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)

	oldGetenv := getenv
	oldNow := Now
	defer func() {
		getenv = oldGetenv
		Now = oldNow
	}()
	getenv = setEnvironment(map[string]string{
		"AWS_ACCESS_KEY_ID":     accessKeyID,
		"AWS_SECRET_ACCESS_KEY": secretAccessKey,
	})
	Now = setTime(defaultTime)

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("retrieveSubjectToken() failed: %v", err)
	}

	want := getExpectedSubjectToken(
		"https://sts.us-east-2.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		"us-east-2",
		accessKeyID,
		secretAccessKey,
		"",
	)

	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestAWSCredential_ShouldCallMetadataEndpointWhenNoAccessKey(t *testing.T) {
	server := createDefaultAwsTestServerWithImdsv2(t)
	ts := httptest.NewServer(server)

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)

	oldGetenv := getenv
	oldNow := Now
	defer func() {
		getenv = oldGetenv
		Now = oldNow
	}()
	getenv = setEnvironment(map[string]string{
		"AWS_SECRET_ACCESS_KEY": "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
		"AWS_REGION":            "us-west-1",
	})
	Now = setTime(defaultTime)

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("retrieveSubjectToken() failed: %v", err)
	}

	want := getExpectedSubjectToken(
		"https://sts.us-west-1.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		"us-west-1",
		accessKeyID,
		secretAccessKey,
		sessionToken,
	)

	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestAWSCredential_ShouldCallMetadataEndpointWhenNoSecretAccessKey(t *testing.T) {
	server := createDefaultAwsTestServerWithImdsv2(t)
	ts := httptest.NewServer(server)

	opts := cloneTestOpts()
	opts.CredentialSource = server.getCredentialSource(ts.URL)

	oldGetenv := getenv
	oldNow := Now
	defer func() {
		getenv = oldGetenv
		Now = oldNow
	}()
	getenv = setEnvironment(map[string]string{
		"AWS_ACCESS_KEY_ID": "AKIDEXAMPLE",
		"AWS_REGION":        "us-west-1",
	})
	Now = setTime(defaultTime)

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("retrieveSubjectToken() failed: %v", err)
	}

	want := getExpectedSubjectToken(
		"https://sts.us-west-1.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		"us-west-1",
		accessKeyID,
		secretAccessKey,
		sessionToken,
	)

	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestAWSCredential_Validations(t *testing.T) {
	var metadataServerValidityTests = []struct {
		name       string
		credSource *credsfile.CredentialSource
		errText    string
	}{
		{
			name: "No Metadata Server URLs",
			credSource: &credsfile.CredentialSource{
				EnvironmentID:         "aws1",
				RegionURL:             "",
				URL:                   "",
				IMDSv2SessionTokenURL: "",
			},
		}, {
			name: "IPv4 Metadata Server URLs",
			credSource: &credsfile.CredentialSource{
				EnvironmentID:         "aws1",
				RegionURL:             "http://169.254.169.254/latest/meta-data/placement/availability-zone",
				URL:                   "http://169.254.169.254/latest/meta-data/iam/security-credentials",
				IMDSv2SessionTokenURL: "http://169.254.169.254/latest/api/token",
			},
		}, {
			name: "IPv6 Metadata Server URLs",
			credSource: &credsfile.CredentialSource{
				EnvironmentID:         "aws1",
				RegionURL:             "http://[fd00:ec2::254]/latest/meta-data/placement/availability-zone",
				URL:                   "http://[fd00:ec2::254]/latest/meta-data/iam/security-credentials",
				IMDSv2SessionTokenURL: "http://[fd00:ec2::254]/latest/api/token",
			},
		},
	}

	for _, tt := range metadataServerValidityTests {
		t.Run(tt.name, func(t *testing.T) {
			opts := cloneTestOpts()
			opts.CredentialSource = tt.credSource

			oldGetenv := getenv
			defer func() { getenv = oldGetenv }()
			getenv = setEnvironment(map[string]string{})

			_, err := newSubjectTokenProvider(opts)
			if err != nil {
				if tt.errText == "" {
					t.Errorf("Didn't expect an error, but got %v", err)
				} else if tt.errText != err.Error() {
					t.Errorf("got %v, want %v", err, tt.errText)
				}
			} else {
				if tt.errText != "" {
					t.Errorf("got nil, want %v", tt.errText)
				}
			}
		})
	}
}

func TestAWSCredential_ProgrammaticAuth(t *testing.T) {
	opts := cloneTestOpts()
	opts.AwsSecurityCredentialsProvider = &fakeAwsCredsProvider{
		awsRegion: "us-east-2",
		creds: &AwsSecurityCredentials{
			AccessKeyID:     accessKeyID,
			SecretAccessKey: secretAccessKey,
			SessionToken:    sessionToken,
		},
	}

	oldNow := Now
	defer func() {
		Now = oldNow
	}()
	Now = setTime(defaultTime)

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("newSubjectTokenProvider() = %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("subjectToken() = %v", err)
	}

	want := getExpectedSubjectToken(
		"https://sts.us-east-2.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		"us-east-2",
		accessKeyID,
		secretAccessKey,
		sessionToken,
	)

	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestAWSCredential_ProgrammaticAuthNoSessionToken(t *testing.T) {
	opts := cloneTestOpts()
	opts.AwsSecurityCredentialsProvider = fakeAwsCredsProvider{
		awsRegion: "us-east-2",
		creds: &AwsSecurityCredentials{
			AccessKeyID:     accessKeyID,
			SecretAccessKey: secretAccessKey,
		},
	}

	oldNow := Now
	defer func() {
		Now = oldNow
	}()
	Now = setTime(defaultTime)

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("newSubjectTokenProvider() = %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("subjectToken() = %v", err)
	}

	want := getExpectedSubjectToken(
		"https://sts.us-east-2.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
		"us-east-2",
		accessKeyID,
		secretAccessKey,
		"",
	)

	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestAWSCredential_ProgrammaticAuthError(t *testing.T) {
	opts := cloneTestOpts()
	testErr := errors.New("test error")
	opts.AwsSecurityCredentialsProvider = fakeAwsCredsProvider{
		awsRegion: "us-east-2",
		credsErr:  testErr,
	}

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("newSubjectTokenProvider() = %v", err)
	}

	_, gotErr := base.subjectToken(context.Background())
	if gotErr == nil {
		t.Fatalf("subjectToken() = nil, want error")
	}
	if gotErr != testErr {
		t.Errorf("got = %v, want %v", err, testErr)
	}
}

func TestAWSCredential_ProgrammaticAuthRegionError(t *testing.T) {
	opts := cloneTestOpts()
	testErr := errors.New("test error")
	opts.AwsSecurityCredentialsProvider = fakeAwsCredsProvider{
		regionErr: testErr,
		creds: &AwsSecurityCredentials{
			AccessKeyID:     accessKeyID,
			SecretAccessKey: secretAccessKey,
		},
	}

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("newSubjectTokenProvider() = %v", err)
	}

	_, gotErr := base.subjectToken(context.Background())
	if gotErr == nil {
		t.Fatalf("subjectToken() = nil, want error")
	}
	if gotErr != testErr {
		t.Errorf("got = %v, want %v", err, testErr)
	}
}

func TestAWSCredential_ProgrammaticAuthOptions(t *testing.T) {
	opts := cloneTestOpts()
	wantOpts := &RequestOptions{Audience: opts.Audience, SubjectTokenType: opts.SubjectTokenType}

	opts.AwsSecurityCredentialsProvider = fakeAwsCredsProvider{
		awsRegion: "us-east-2",
		creds: &AwsSecurityCredentials{
			AccessKeyID:     accessKeyID,
			SecretAccessKey: secretAccessKey,
		},
		reqOpts: wantOpts,
	}

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("newSubjectTokenProvider() = %v", err)
	}

	_, err = base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("subjectToken() = %v", err)
	}
}

func setTime(testTime time.Time) func() time.Time {
	return func() time.Time {
		return testTime
	}
}

func setEnvironment(env map[string]string) func(string) string {
	return func(key string) string {
		return env[key]
	}
}

var defaultRequestSigner = &awsRequestSigner{
	RegionName: "us-east-1",
	AwsSecurityCredentials: &AwsSecurityCredentials{
		AccessKeyID:     "AKIDEXAMPLE",
		SecretAccessKey: "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
	},
}

func setDefaultTime(req *http.Request) {
	// Don't use time.Format for this
	// Our output signature expects this to be a Monday, even though Sept 9, 2011 is a Friday
	req.Header.Set("date", "Mon, 09 Sep 2011 23:36:00 GMT")
}

func testRequestSigner(t *testing.T, rs *awsRequestSigner, input, wantOutput *http.Request) {
	t.Helper()

	err := rs.signRequest(input)
	if err != nil {
		t.Fatal(err)
	}

	if got, want := input.URL.String(), wantOutput.URL.String(); got != want {
		t.Errorf("url = %q, want %q", got, want)
	}
	if got, want := input.Method, wantOutput.Method; got != want {
		t.Errorf("method = %q, want %q", got, want)
	}
	for header := range wantOutput.Header {
		if got, want := input.Header[header], wantOutput.Header[header]; !cmp.Equal(got, want) {
			t.Errorf("header[%q] = %q, want %q", header, got, want)
		}
	}
}

type fakeAwsCredsProvider struct {
	credsErr  error
	regionErr error
	awsRegion string
	creds     *AwsSecurityCredentials
	reqOpts   *RequestOptions
}

func (acp fakeAwsCredsProvider) AwsRegion(ctx context.Context, opts *RequestOptions) (string, error) {
	if acp.regionErr != nil {
		return "", acp.regionErr
	}
	if acp.reqOpts != nil {
		if acp.reqOpts.Audience != opts.Audience {
			return "", errors.New("audience does not match")
		}
		if acp.reqOpts.SubjectTokenType != opts.SubjectTokenType {
			return "", errors.New("audience does not match")
		}
	}
	return acp.awsRegion, nil
}

func (acp fakeAwsCredsProvider) AwsSecurityCredentials(ctx context.Context, opts *RequestOptions) (*AwsSecurityCredentials, error) {
	if acp.credsErr != nil {
		return nil, acp.credsErr
	}
	if acp.reqOpts != nil {
		if acp.reqOpts.Audience != opts.Audience {
			return nil, errors.New("Audience does not match")
		}
		if acp.reqOpts.SubjectTokenType != opts.SubjectTokenType {
			return nil, errors.New("Audience does not match")
		}
	}
	return acp.creds, nil
}
