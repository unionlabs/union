//go:build go1.7
// +build go1.7

package ec2_test

import (
	"bytes"
	"context"
	"fmt"
	"io/ioutil"
	"net/http"
	"net/url"
	"regexp"
	"strconv"
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	sdkclient "github.com/aws/aws-sdk-go/aws/client"
	"github.com/aws/aws-sdk-go/aws/endpoints"
	"github.com/aws/aws-sdk-go/aws/request"
	"github.com/aws/aws-sdk-go/awstesting/unit"
	"github.com/aws/aws-sdk-go/service/ec2"
)

func TestCopySnapshotPresignedURL(t *testing.T) {
	svc := ec2.New(unit.Session, &aws.Config{Region: aws.String("us-west-2")})

	func() {
		defer func() {
			if r := recover(); r != nil {
				t.Fatalf("expect CopySnapshotRequest with nill")
			}
		}()
		// Doesn't panic on nil input
		req, _ := svc.CopySnapshotRequest(nil)
		req.Sign()
	}()

	req, _ := svc.CopySnapshotRequest(&ec2.CopySnapshotInput{
		SourceRegion:     aws.String("us-west-1"),
		SourceSnapshotId: aws.String("snap-id"),
	})
	req.Sign()

	b, _ := ioutil.ReadAll(req.HTTPRequest.Body)
	q, _ := url.ParseQuery(string(b))
	u, _ := url.QueryUnescape(q.Get("PresignedUrl"))
	if e, a := "us-west-2", q.Get("DestinationRegion"); e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "us-west-1", q.Get("SourceRegion"); e != a {
		t.Errorf("expect %v, got %v", e, a)
	}

	r := regexp.MustCompile(`^https://ec2\.us-west-1\.amazonaws\.com/.+&DestinationRegion=us-west-2`)
	if !r.MatchString(u) {
		t.Errorf("expect %v to match, got %v", r.String(), u)
	}
}

func TestCopySnapshotPresignedURLConfig(t *testing.T) {
	const (
		inputKmsKeyId     = "KMS_KEY_ID"
		inputSnapshotId   = "SNAPSHOT_ID"
		clientRegion      = endpoints.UsEast1RegionID
		inputSourceRegion = endpoints.UsWest2RegionID
	)
	cases := map[string]struct {
		Encrypted         bool
		DestinationRegion string
		KmsKeyId          string
	}{
		// Not Encrypted
		"Not Encrypted": {},
		// Not Encrypted with KmsKeyId
		"Not Encrypted with KmsKeyId": {
			KmsKeyId: inputKmsKeyId,
		},
		// Not Encrypted with DestinationRegion
		"Not Encrypted with DestinationRegion": {
			DestinationRegion: endpoints.UsEast2RegionID,
		},
		// Not Encrypted with KmsKeyId and DestinationRegion
		"Not Encrypted with KmsKeyId and DestinationRegion": {
			KmsKeyId:          inputKmsKeyId,
			DestinationRegion: endpoints.UsEast2RegionID,
		},
		// Encrypted
		"Encrypted": {
			Encrypted: true,
		},
		// Encrypted with KmsKeyId
		"Encrypted with KmsKeyId": {
			Encrypted: true,
			KmsKeyId:  inputKmsKeyId,
		},
		// Encrypted with DestinationRegion
		"Encrypted with DestinationRegion": {
			Encrypted:         true,
			DestinationRegion: endpoints.UsEast2RegionID,
		},
		// Encrypted with KmsKeyId and DestinationRegion
		"Encrypted with KmsKeyId and DestinationRegion": {
			Encrypted:         true,
			KmsKeyId:          inputKmsKeyId,
			DestinationRegion: endpoints.UsEast2RegionID,
		},
	}

	for name, config := range cases {
		t.Run(name, func(t *testing.T) {
			t.Log(name)

			// Set up new client
			svc := ec2.New(unit.Session, &aws.Config{
				Region: aws.String(clientRegion),
			})

			// Base input
			input := ec2.CopySnapshotInput{
				SourceRegion:     aws.String(inputSourceRegion),
				SourceSnapshotId: aws.String(inputSnapshotId),
			}

			// Add input from test case config
			if config.Encrypted != false {
				input.Encrypted = &config.Encrypted
			}
			if config.DestinationRegion != "" {
				input.DestinationRegion = &config.DestinationRegion
			}
			if config.KmsKeyId != "" {
				input.KmsKeyId = &config.KmsKeyId
			}

			// Execute request
			req, _ := svc.CopySnapshotRequest(&input)
			req.Sign()

			// Parse request
			body, _ := ioutil.ReadAll(req.HTTPRequest.Body)
			query, _ := url.ParseQuery(string(body))

			// Test Body SourceRegion
			sourceRegion := query.Get("SourceRegion")
			if sourceRegion == "" {
				t.Errorf("SourceRegion should always be sent in the request")
			}
			if sourceRegion != inputSourceRegion {
				t.Errorf("SourceRegion should be `%v`, but found `%v`", inputSourceRegion, sourceRegion)
			}
			// Test Body SourceSnapshotId
			sourceSnapshotId := query.Get("SourceSnapshotId")
			if sourceSnapshotId == "" {
				t.Errorf("SourceSnapshotId should always be sent in the request")
			}
			if sourceSnapshotId != inputSnapshotId {
				t.Errorf("SourceSnapshotId should be `%v`, but found `%v`", inputSnapshotId, sourceSnapshotId)
			}
			// Test Body Encrypted
			encrypted := query.Get("Encrypted")
			if config.Encrypted && strconv.FormatBool(config.Encrypted) != encrypted {
				t.Errorf("Encrypted should be `%v`, but found `%v`", config.Encrypted, encrypted)
			}
			if !config.Encrypted && encrypted != "" {
				t.Errorf("Encrypted should be empty, but found `%v`", encrypted)
			}
			// Test Body DestinationRegion
			destinationRegion := query.Get("DestinationRegion")
			if destinationRegion != clientRegion {
				t.Errorf("DestinationRegion should always be equal to the client region `%v`, but found `%v`", clientRegion, destinationRegion)
			}
			if destinationRegion == "" {
				t.Errorf("DestinationRegion should never empty")
			}
			// Test Body KmsKeyId
			kmsKeyId := query.Get("KmsKeyId")
			if config.KmsKeyId != "" && config.KmsKeyId != kmsKeyId {
				t.Errorf("KmsKeyId should be `%v`, but found `%v`", config.KmsKeyId, kmsKeyId)
			}
			if config.KmsKeyId == "" && kmsKeyId != "" {
				t.Errorf("KmsKeyId should be empty, but found `%v`", kmsKeyId)
			}

			// Assert PresignedUrl
			presignedUrl, _ := url.QueryUnescape(query.Get("PresignedUrl"))
			if presignedUrl == "" {
				t.Errorf("PresignedUrl should always be sent in the request")
			}
			// Test PresignedUrl EC2 URL
			baseEc2UrlRegex := regexp.MustCompile(fmt.Sprintf(`^https://ec2\.%s\.amazonaws\.com/`, inputSourceRegion))
			if !baseEc2UrlRegex.MatchString(presignedUrl) {
				t.Errorf("Expected PresignedUrl to match `%v`, but found `%v`", baseEc2UrlRegex.String(), presignedUrl)
			}

			presignedUrlQuery, _ := url.ParseQuery(presignedUrl)
			// Test PresignedUrl SourceRegion
			presignedUrlSourceRegion := presignedUrlQuery.Get("SourceRegion")
			if presignedUrlSourceRegion == "" {
				t.Errorf("PresignedUrl SourceRegion should always be sent in the request")
			}
			if presignedUrlSourceRegion != inputSourceRegion {
				t.Errorf("PresignedUrl SourceRegion should be `%v`, but found `%v`", inputSourceRegion, presignedUrlSourceRegion)
			}
			// Test PresignedUrl SourceSnapshotId
			presignedUrlSourceSnapshotId := presignedUrlQuery.Get("SourceSnapshotId")
			if presignedUrlSourceSnapshotId == "" {
				t.Errorf("PresignedUrl SourceSnapshotId should always be sent in the request")
			}
			if presignedUrlSourceSnapshotId != inputSnapshotId {
				t.Errorf("PresignedUrl SourceSnapshotId should be `%v`, but found `%v`", inputSnapshotId, presignedUrlSourceSnapshotId)
			}
			// Test PresignedUrl Encrypted
			presignedUrlEncrypted := query.Get("Encrypted")
			if config.Encrypted && strconv.FormatBool(config.Encrypted) != presignedUrlEncrypted {
				t.Errorf("PresignedUrl Encrypted should be `%v`, but found `%v`", config.Encrypted, presignedUrlEncrypted)
			}
			if !config.Encrypted && presignedUrlEncrypted != "" {
				t.Errorf("PresignedUrl Encrypted should be empty, but found `%v`", presignedUrlEncrypted)
			}
			// Test PresignedUrl DestinationRegion
			presignedUrlDestinationRegion := presignedUrlQuery.Get("DestinationRegion")
			if presignedUrlDestinationRegion != clientRegion {
				t.Errorf("PresignedUrl DestinationRegion should always be equal to the client region `%v`, but found `%v`", clientRegion, presignedUrlDestinationRegion)
			}
			// Test PresignedUrl KmsKeyId
			presignedUrlKmsKeyId := query.Get("KmsKeyId")
			if config.KmsKeyId != "" && config.KmsKeyId != presignedUrlKmsKeyId {
				t.Errorf("PresignedUrl KmsKeyId should be `%v`, but found `%v`", config.KmsKeyId, presignedUrlKmsKeyId)
			}
			if config.KmsKeyId == "" && presignedUrlKmsKeyId != "" {
				t.Errorf("PresignedUrl KmsKeyId should be empty, but found `%v`", presignedUrlKmsKeyId)
			}
			// Test PresignedUrl X-Amz-Credential
			presignedUrlAmzCredential := presignedUrlQuery.Get("X-Amz-Credential")
			amzCredentialRegex := regexp.MustCompile(fmt.Sprintf(`^\w{4}/\d{8}/%s/ec2/aws4_request$`, inputSourceRegion))
			if !amzCredentialRegex.MatchString(presignedUrlAmzCredential) {
				t.Errorf("Expected PresignedUrl X-Amz-Credential to match `%v`, but found `%v`", amzCredentialRegex.String(), presignedUrlAmzCredential)
			}
		})
	}
}

func TestNoCustomRetryerWithMaxRetries(t *testing.T) {
	cases := map[string]struct {
		Config           aws.Config
		ExpectMaxRetries int
	}{
		"With custom retrier": {
			Config: aws.Config{
				Retryer: sdkclient.DefaultRetryer{
					NumMaxRetries: 10,
				},
			},
			ExpectMaxRetries: 10,
		},
		"with max retries": {
			Config: aws.Config{
				MaxRetries: aws.Int(10),
			},
			ExpectMaxRetries: 10,
		},
		"no options set": {
			ExpectMaxRetries: sdkclient.DefaultRetryerMaxNumRetries,
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			client := ec2.New(unit.Session, &aws.Config{
				DisableParamValidation: aws.Bool(true),
			}, c.Config.Copy())
			client.ModifyNetworkInterfaceAttributeWithContext(context.Background(), nil, checkRetryerMaxRetries(t, c.ExpectMaxRetries))
			client.AssignPrivateIpAddressesWithContext(context.Background(), nil, checkRetryerMaxRetries(t, c.ExpectMaxRetries))
		})
	}

}

func checkRetryerMaxRetries(t *testing.T, maxRetries int) func(*request.Request) {
	return func(r *request.Request) {
		r.Handlers.Send.Clear()
		r.Handlers.Send.PushBack(func(rr *request.Request) {
			if e, a := maxRetries, rr.Retryer.MaxRetries(); e != a {
				t.Errorf("%s, expect %v max retries, got %v", rr.Operation.Name, e, a)
			}
			rr.HTTPResponse = &http.Response{
				StatusCode: 200,
				Header:     http.Header{},
				Body:       ioutil.NopCloser(&bytes.Buffer{}),
			}
		})
	}
}
