//go:build go1.7
// +build go1.7

package s3

import (
	"bytes"
	"fmt"
	"io/ioutil"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/endpoints"
	"github.com/aws/aws-sdk-go/aws/request"
	"github.com/aws/aws-sdk-go/awstesting/unit"
)

type testCase struct {
	bucket                string
	config                *aws.Config
	req                   func(svc *S3) *request.Request
	expectedEndpoint      string
	expectedSigningName   string
	expectedSigningRegion string
	expectedErr           string
}

func TestEndpoint(t *testing.T) {
	cases := map[string]testCase{
		"standard custom endpoint url": {
			bucket: "bucketname",
			config: &aws.Config{
				Region:   aws.String("us-west-2"),
				Endpoint: aws.String("beta.example.com"),
			},
			expectedEndpoint:      "https://bucketname.beta.example.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"Object Lambda with no UseARNRegion flag set": {
			bucket: "arn:aws:s3-object-lambda:us-west-2:123456789012:accesspoint/myap",
			config: &aws.Config{
				Region: aws.String("us-west-2"),
			},
			expectedEndpoint:      "https://myap-123456789012.s3-object-lambda.us-west-2.amazonaws.com",
			expectedSigningName:   "s3-object-lambda",
			expectedSigningRegion: "us-west-2",
		},
		"Object Lambda with UseARNRegion flag set": {
			bucket: "arn:aws:s3-object-lambda:us-east-1:123456789012:accesspoint/myap",
			config: &aws.Config{
				Region:         aws.String("us-west-2"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedEndpoint:      "https://myap-123456789012.s3-object-lambda.us-east-1.amazonaws.com",
			expectedSigningName:   "s3-object-lambda",
			expectedSigningRegion: "us-east-1",
		},
		"Object Lambda with Cross-Region error": {
			bucket: "arn:aws:s3-object-lambda:us-east-1:123456789012:accesspoint/myap",
			config: &aws.Config{
				Region: aws.String("us-west-2"),
			},
			expectedErr: "client region does not match provided ARN region",
		},
		"Object Lambda Pseudo-Region with UseARNRegion flag set": {
			bucket: "arn:aws:s3-object-lambda:us-east-1:123456789012:accesspoint/myap",
			config: &aws.Config{
				Region:         aws.String("aws-global"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedEndpoint:      "https://myap-123456789012.s3-object-lambda.us-east-1.amazonaws.com",
			expectedSigningRegion: "us-east-1",
			expectedSigningName:   "s3-object-lambda",
		},
		"Object Lambda Cross-Region DualStack (deprecated) error": {
			bucket: "arn:aws:s3-object-lambda:us-east-1:123456789012:accesspoint/myap",
			config: &aws.Config{
				Region:         aws.String("us-west-2"),
				UseDualStack:   aws.Bool(true),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedErr: "client configured for S3 Dual-stack but is not supported with resource ARN",
		},
		"Object Lambda Cross-Region DualStack error": {
			bucket: "arn:aws:s3-object-lambda:us-east-1:123456789012:accesspoint/myap",
			config: &aws.Config{
				Region:               aws.String("us-west-2"),
				UseDualStackEndpoint: endpoints.DualStackEndpointStateEnabled,
				S3UseARNRegion:       aws.Bool(true),
			},
			expectedErr: "client configured for S3 Dual-stack but is not supported with resource ARN",
		},
		"Object Lambda Cross-Partition error": {
			bucket: "arn:aws-cn:s3-object-lambda:cn-north-1:123456789012:accesspoint/myap",
			config: &aws.Config{
				Region:         aws.String("us-west-2"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedErr: "client partition does not match provided ARN partition",
		},
		"Object Lambda FIPS Pseudo-Region (deprecated)": {
			bucket: "arn:aws-us-gov:s3-object-lambda:us-gov-west-1:123456789012:accesspoint/myap",
			config: &aws.Config{
				Region: aws.String("fips-us-gov-west-1"),
			},
			expectedEndpoint:      "https://myap-123456789012.s3-object-lambda-fips.us-gov-west-1.amazonaws.com",
			expectedSigningRegion: "us-gov-west-1",
			expectedSigningName:   "s3-object-lambda",
		},
		"Object Lambda FIPS Region": {
			bucket: "arn:aws-us-gov:s3-object-lambda:us-gov-west-1:123456789012:accesspoint/myap",
			config: &aws.Config{
				Region:          aws.String("us-gov-west-1"),
				UseFIPSEndpoint: endpoints.FIPSEndpointStateEnabled,
			},
			expectedEndpoint:      "https://myap-123456789012.s3-object-lambda-fips.us-gov-west-1.amazonaws.com",
			expectedSigningRegion: "us-gov-west-1",
			expectedSigningName:   "s3-object-lambda",
		},
		"Object Lambda FIPS Pseudo-Region (deprecated) with UseARNRegion flag set": {
			bucket: "arn:aws-us-gov:s3-object-lambda:us-gov-west-1:123456789012:accesspoint/myap",
			config: &aws.Config{
				Region:         aws.String("fips-us-gov-west-1"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedEndpoint:      "https://myap-123456789012.s3-object-lambda-fips.us-gov-west-1.amazonaws.com",
			expectedSigningRegion: "us-gov-west-1",
			expectedSigningName:   "s3-object-lambda",
		},
		"Object Lambda FIPS Region with UseARNRegion flag set": {
			bucket: "arn:aws-us-gov:s3-object-lambda:us-gov-west-1:123456789012:accesspoint/myap",
			config: &aws.Config{
				Region:          aws.String("us-gov-west-1"),
				UseFIPSEndpoint: endpoints.FIPSEndpointStateEnabled,
				S3UseARNRegion:  aws.Bool(true),
			},
			expectedEndpoint:      "https://myap-123456789012.s3-object-lambda-fips.us-gov-west-1.amazonaws.com",
			expectedSigningRegion: "us-gov-west-1",
			expectedSigningName:   "s3-object-lambda",
		},
		"Object Lambda with Accelerate": {
			bucket: "arn:aws:s3-object-lambda:us-west-2:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:          aws.String("us-west-2"),
				S3UseAccelerate: aws.Bool(true),
			},
			expectedErr: "client configured for S3 Accelerate but is not supported with resource ARN",
		},
		"Object Lambda with Custom Endpoint": {
			bucket: "arn:aws:s3-object-lambda:us-west-2:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:   aws.String("us-west-2"),
				Endpoint: aws.String("my-domain.com"),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.my-domain.com",
			expectedSigningName:   "s3-object-lambda",
			expectedSigningRegion: "us-west-2",
		},
		"AccessPoint with custom endpoint url": {
			bucket: "arn:aws:s3:us-west-2:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:   aws.String("us-west-2"),
				Endpoint: aws.String("beta.example.com"),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.beta.example.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"Outpost AccessPoint with custom endpoint url": {
			bucket: "arn:aws:s3-outposts:us-west-2:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:   aws.String("us-west-2"),
				Endpoint: aws.String("beta.example.com"),
			},
			expectedEndpoint:      "https://myaccesspoint-123456789012.op-01234567890123456.beta.example.com",
			expectedSigningName:   "s3-outposts",
			expectedSigningRegion: "us-west-2",
		},
		"ListBucket with custom endpoint url": {
			config: &aws.Config{
				Region:   aws.String("us-west-2"),
				Endpoint: aws.String("bucket.vpce-123-abc.s3.us-west-2.vpce.amazonaws.com"),
			},
			req: func(svc *S3) *request.Request {
				req, _ := svc.ListBucketsRequest(&ListBucketsInput{})
				return req
			},
			expectedEndpoint:      "https://bucket.vpce-123-abc.s3.us-west-2.vpce.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"Path-style addressing with custom endpoint url": {
			bucket: "bucketname",
			config: &aws.Config{
				Region:           aws.String("us-west-2"),
				Endpoint:         aws.String("bucket.vpce-123-abc.s3.us-west-2.vpce.amazonaws.com"),
				S3ForcePathStyle: aws.Bool(true),
			},
			expectedEndpoint:      "https://bucket.vpce-123-abc.s3.us-west-2.vpce.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"Virtual host addressing with custom endpoint url": {
			bucket: "bucketname",
			config: &aws.Config{
				Region:   aws.String("us-west-2"),
				Endpoint: aws.String("bucket.vpce-123-abc.s3.us-west-2.vpce.amazonaws.com"),
			},
			expectedEndpoint:      "https://bucketname.bucket.vpce-123-abc.s3.us-west-2.vpce.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"Access-point with custom endpoint url and use_arn_region set": {
			bucket: "arn:aws:s3:us-west-2:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:         aws.String("eu-west-1"),
				Endpoint:       aws.String("accesspoint.vpce-123-abc.s3.us-west-2.vpce.amazonaws.com"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.accesspoint.vpce-123-abc.s3.us-west-2.vpce.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"Custom endpoint url with Dualstack (deprecated)": {
			bucket: "bucketname",
			config: &aws.Config{
				Region:       aws.String("us-west-2"),
				Endpoint:     aws.String("beta.example.com"),
				UseDualStack: aws.Bool(true),
			},
			expectedEndpoint:      "https://bucketname.beta.example.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"Custom endpoint url with Dualstack": {
			bucket: "bucketname",
			config: &aws.Config{
				Region:               aws.String("us-west-2"),
				Endpoint:             aws.String("beta.example.com"),
				UseDualStackEndpoint: endpoints.DualStackEndpointStateEnabled,
			},
			expectedEndpoint:      "https://bucketname.beta.example.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"Outpost with custom endpoint url and Dualstack (deprecated)": {
			bucket: "arn:aws:s3-outposts:us-west-2:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:       aws.String("us-west-2"),
				Endpoint:     aws.String("beta.example.com"),
				UseDualStack: aws.Bool(true),
			},
			expectedErr: "client configured for S3 Dual-stack but is not supported with resource ARN",
		},
		"Outpost with custom endpoint url and Dualstack": {
			bucket: "arn:aws:s3-outposts:us-west-2:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:               aws.String("us-west-2"),
				Endpoint:             aws.String("beta.example.com"),
				UseDualStackEndpoint: endpoints.DualStackEndpointStateEnabled,
			},
			expectedErr: "client configured for S3 Dual-stack but is not supported with resource ARN",
		},
		"Outpost AccessPoint with no S3UseARNRegion flag set": {
			bucket: "arn:aws:s3-outposts:us-west-2:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region: aws.String("us-west-2"),
			},
			expectedEndpoint:      "https://myaccesspoint-123456789012.op-01234567890123456.s3-outposts.us-west-2.amazonaws.com",
			expectedSigningName:   "s3-outposts",
			expectedSigningRegion: "us-west-2",
		},
		"Outpost AccessPoint Cross-Region Enabled": {
			bucket: "arn:aws:s3-outposts:us-east-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:         aws.String("us-west-2"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedEndpoint:      "https://myaccesspoint-123456789012.op-01234567890123456.s3-outposts.us-east-1.amazonaws.com",
			expectedSigningName:   "s3-outposts",
			expectedSigningRegion: "us-east-1",
		},
		"Outpost AccessPoint Cross-Region Disabled": {
			bucket: "arn:aws:s3-outposts:us-east-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region: aws.String("us-west-2"),
			},
			expectedErr: "client region does not match provided ARN region",
		},
		"Outpost AccessPoint Cross-Region Disabled FIPS (deprecated)": {
			bucket: "arn:aws-us-gov:s3-outposts:us-gov-west-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region: aws.String("fips-us-gov-east-1"),
			},
			expectedErr: "client region does not match provided ARN region",
		},
		"Outpost AccessPoint Cross-Region Disabled FIPS": {
			bucket: "arn:aws-us-gov:s3-outposts:us-gov-west-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:          aws.String("us-gov-east-1"),
				UseFIPSEndpoint: endpoints.FIPSEndpointStateEnabled,
			},
			expectedErr: "client region does not match provided ARN region",
		},
		"Outpost AccessPoint other partition": {
			bucket: "arn:aws-cn:s3-outposts:cn-north-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:         aws.String("us-west-2"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedErr: "ConfigurationError: client partition does not match provided ARN partition",
		},
		"Outpost AccessPoint cn partition": {
			bucket: "arn:aws-cn:s3-outposts:cn-north-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region: aws.String("cn-north-1"),
			},
			expectedEndpoint:      "https://myaccesspoint-123456789012.op-01234567890123456.s3-outposts.cn-north-1.amazonaws.com.cn",
			expectedSigningName:   "s3-outposts",
			expectedSigningRegion: "cn-north-1",
		},
		"Outpost AccessPoint us-gov region": {
			bucket: "arn:aws-us-gov:s3-outposts:us-gov-east-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:         aws.String("us-gov-east-1"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedEndpoint:      "https://myaccesspoint-123456789012.op-01234567890123456.s3-outposts.us-gov-east-1.amazonaws.com",
			expectedSigningName:   "s3-outposts",
			expectedSigningRegion: "us-gov-east-1",
		},
		"Outpost AccessPoint FIPS (deprecated) client region, resolved signing region does not match ARN region": {
			bucket: "arn:aws-us-gov:s3-outposts:us-gov-unknown-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				EndpointResolver: endpoints.AwsUsGovPartition(),
				Region:           aws.String("fips-us-gov-unknown-1"),
			},
			expectedErr: "use of ARN is not supported when client or request is configured for FIPS",
		},
		"Outpost AccessPoint FIPS client region, resolved signing region does not match ARN region": {
			bucket: "arn:aws-us-gov:s3-outposts:us-gov-unknown-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				EndpointResolver: endpoints.AwsUsGovPartition(),
				Region:           aws.String("us-gov-unknown-1"),
				UseFIPSEndpoint:  endpoints.FIPSEndpointStateEnabled,
			},
			expectedErr: "use of ARN is not supported when client or request is configured for FIPS",
		},
		"Outpost AccessPoint FIPS (deprecated) client region, resolved signing region does match ARN region": {
			bucket: "arn:aws-us-gov:s3-outposts:us-gov-west-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region: aws.String("fips-us-gov-west-1"),
			},
			expectedErr: "use of ARN is not supported when client or request is configured for FIPS",
		},
		"Outpost AccessPoint FIPS client region, resolved signing region does match ARN region": {
			bucket: "arn:aws-us-gov:s3-outposts:us-gov-west-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:          aws.String("us-gov-west-1"),
				UseFIPSEndpoint: endpoints.FIPSEndpointStateEnabled,
			},
			expectedErr: "use of ARN is not supported when client or request is configured for FIPS",
		},
		"Outpost AccessPoint FIPS (deprecated) client region with matching ARN region": {
			bucket: "arn:aws-us-gov:s3-outposts:us-gov-east-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:         aws.String("fips-us-gov-east-1"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedErr: "use of ARN is not supported when client or request is configured for FIPS",
		},
		"Outpost AccessPoint FIPS client region with matching ARN region": {
			bucket: "arn:aws-us-gov:s3-outposts:us-gov-east-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:          aws.String("fips-us-gov-east-1"),
				UseFIPSEndpoint: endpoints.FIPSEndpointStateEnabled,
				S3UseARNRegion:  aws.Bool(true),
			},
			expectedErr: "use of ARN is not supported when client or request is configured for FIPS",
		},
		"Outpost AccessPoint FIPS (deprecated) client region with cross-region ARN": {
			bucket: "arn:aws-us-gov:s3-outposts:us-gov-west-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:         aws.String("fips-us-gov-east-1"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedErr: "use of ARN is not supported when client or request is configured for FIPS",
		},
		"Outpost AccessPoint FIPS client region with cross-region ARN": {
			bucket: "arn:aws-us-gov:s3-outposts:us-gov-west-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:          aws.String("us-gov-east-1"),
				UseFIPSEndpoint: endpoints.FIPSEndpointStateEnabled,
				S3UseARNRegion:  aws.Bool(true),
			},
			expectedErr: "use of ARN is not supported when client or request is configured for FIPS",
		},
		"Outpost AccessPoint with DualStack (deprecated)": {
			bucket: "arn:aws:s3-outposts:us-west-2:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:       aws.String("us-west-2"),
				UseDualStack: aws.Bool(true),
			},
			expectedErr: "ConfigurationError: client configured for S3 Dual-stack but is not supported with resource ARN",
		},
		"Outpost AccessPoint with DualStack": {
			bucket: "arn:aws:s3-outposts:us-west-2:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:               aws.String("us-west-2"),
				UseDualStackEndpoint: endpoints.DualStackEndpointStateEnabled,
			},
			expectedErr: "ConfigurationError: client configured for S3 Dual-stack but is not supported with resource ARN",
		},
		"Outpost AccessPoint with Accelerate": {
			bucket: "arn:aws:s3-outposts:us-west-2:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:          aws.String("us-west-2"),
				S3UseAccelerate: aws.Bool(true),
			},
			expectedErr: "ConfigurationError: client configured for S3 Accelerate but is not supported with resource ARN",
		},
		"AccessPoint": {
			bucket: "arn:aws:s3:us-west-2:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region: aws.String("us-west-2"),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.s3-accesspoint.us-west-2.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"AccessPoint slash delimiter": {
			bucket: "arn:aws:s3:us-west-2:123456789012:accesspoint/myendpoint",
			config: &aws.Config{
				Region: aws.String("us-west-2"),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.s3-accesspoint.us-west-2.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"AccessPoint other partition": {
			bucket: "arn:aws-cn:s3:cn-north-1:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region: aws.String("cn-north-1"),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.s3-accesspoint.cn-north-1.amazonaws.com.cn",
			expectedSigningName:   "s3",
			expectedSigningRegion: "cn-north-1",
		},
		"AccessPoint Cross-Region Disabled": {
			bucket: "arn:aws:s3:ap-south-1:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region: aws.String("us-west-2"),
			},
			expectedErr: "client region does not match provided ARN region",
		},
		"AccessPoint Cross-Region Enabled": {
			bucket: "arn:aws:s3:ap-south-1:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:         aws.String("us-west-2"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.s3-accesspoint.ap-south-1.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "ap-south-1",
		},
		"AccessPoint us-east-1": {
			bucket: "arn:aws:s3:us-east-1:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:         aws.String("us-east-1"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.s3-accesspoint.us-east-1.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-east-1",
		},
		"AccessPoint us-east-1 cross region": {
			bucket: "arn:aws:s3:us-east-1:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:         aws.String("us-west-2"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.s3-accesspoint.us-east-1.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-east-1",
		},
		"AccessPoint Cross-Partition not supported": {
			bucket: "arn:aws-cn:s3:cn-north-1:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:         aws.String("us-west-2"),
				UseDualStack:   aws.Bool(true),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedErr: "client partition does not match provided ARN partition",
		},
		"AccessPoint DualStack (deprecated)": {
			bucket: "arn:aws:s3:us-west-2:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:       aws.String("us-west-2"),
				UseDualStack: aws.Bool(true),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.s3-accesspoint.dualstack.us-west-2.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"AccessPoint DualStack": {
			bucket: "arn:aws:s3:us-west-2:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:               aws.String("us-west-2"),
				UseDualStackEndpoint: endpoints.DualStackEndpointStateEnabled,
			},
			expectedEndpoint:      "https://myendpoint-123456789012.s3-accesspoint.dualstack.us-west-2.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"AccessPoint FIPS same region with cross region disabled": {
			bucket: "arn:aws-us-gov:s3:us-gov-west-1:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region: aws.String("fips-us-gov-west-1"),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.s3-accesspoint-fips.us-gov-west-1.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-gov-west-1",
		},
		"AccessPoint FIPS same region with cross region enabled": {
			bucket: "arn:aws-us-gov:s3:us-gov-west-1:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:         aws.String("fips-us-gov-west-1"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.s3-accesspoint-fips.us-gov-west-1.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-gov-west-1",
		},
		"AccessPoint FIPS cross region not supported": {
			bucket: "arn:aws-us-gov:s3:us-gov-east-1:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:         aws.String("fips-us-gov-west-1"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.s3-accesspoint-fips.us-gov-east-1.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-gov-east-1",
		},
		"AccessPoint Accelerate not supported": {
			bucket: "arn:aws:s3:us-west-2:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:          aws.String("us-west-2"),
				S3UseAccelerate: aws.Bool(true),
			},
			expectedErr: "client configured for S3 Accelerate",
		},
		"Custom Resolver Without PartitionID in ClientInfo": {
			bucket: "arn:aws:s3:us-west-2:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region: aws.String("us-west-2"),
				EndpointResolver: endpoints.ResolverFunc(
					func(service, region string, opts ...func(*endpoints.Options)) (endpoints.ResolvedEndpoint, error) {
						switch region {
						case "us-west-2":
							return endpoints.ResolvedEndpoint{
								URL:           "s3.us-west-2.amazonaws.com",
								SigningRegion: "us-west-2",
								SigningName:   service,
								SigningMethod: "s3v4",
							}, nil
						}
						return endpoints.ResolvedEndpoint{}, nil
					}),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.s3-accesspoint.us-west-2.amazonaws.com",
			expectedSigningRegion: "us-west-2",
			expectedSigningName:   "s3",
		},
		"Custom Resolver Without PartitionID in Cross-Region Target": {
			bucket: "arn:aws:s3:us-west-2:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:         aws.String("us-east-1"),
				S3UseARNRegion: aws.Bool(true),
				EndpointResolver: endpoints.ResolverFunc(
					func(service, region string, opts ...func(*endpoints.Options)) (endpoints.ResolvedEndpoint, error) {
						switch region {
						case "us-west-2":
							return endpoints.ResolvedEndpoint{
								URL:           "s3.us-west-2.amazonaws.com",
								PartitionID:   "aws",
								SigningRegion: "us-west-2",
								SigningName:   service,
								SigningMethod: "s3v4",
							}, nil
						case "us-east-1":
							return endpoints.ResolvedEndpoint{
								URL:           "s3.us-east-1.amazonaws.com",
								SigningRegion: "us-east-1",
								SigningName:   service,
								SigningMethod: "s3v4",
							}, nil
						}

						return endpoints.ResolvedEndpoint{}, nil
					}),
			},
			expectedEndpoint:      "https://myendpoint-123456789012.s3-accesspoint.us-west-2.amazonaws.com",
			expectedSigningRegion: "us-west-2",
			expectedSigningName:   "s3",
		},
		"bucket host-style": {
			bucket:                "mock-bucket",
			config:                &aws.Config{Region: aws.String("us-west-2")},
			expectedEndpoint:      "https://mock-bucket.s3.us-west-2.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"bucket path-style": {
			bucket: "mock-bucket",
			config: &aws.Config{
				Region:           aws.String("us-west-2"),
				S3ForcePathStyle: aws.Bool(true),
			},
			expectedEndpoint:      "https://s3.us-west-2.amazonaws.com",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"bucket host-style endpoint with default port": {
			bucket: "mock-bucket",
			config: &aws.Config{
				Region:   aws.String("us-west-2"),
				Endpoint: aws.String("https://s3.us-west-2.amazonaws.com:443"),
			},
			expectedEndpoint:      "https://mock-bucket.s3.us-west-2.amazonaws.com:443",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"bucket host-style endpoint with non-default port": {
			bucket: "mock-bucket",
			config: &aws.Config{
				Region:   aws.String("us-west-2"),
				Endpoint: aws.String("https://s3.us-west-2.amazonaws.com:8443"),
			},
			expectedEndpoint:      "https://mock-bucket.s3.us-west-2.amazonaws.com:8443",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"bucket path-style endpoint with default port": {
			bucket: "mock-bucket",
			config: &aws.Config{
				Region:           aws.String("us-west-2"),
				Endpoint:         aws.String("https://s3.us-west-2.amazonaws.com:443"),
				S3ForcePathStyle: aws.Bool(true),
			},
			expectedEndpoint:      "https://s3.us-west-2.amazonaws.com:443",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"bucket path-style endpoint with non-default port": {
			bucket: "mock-bucket",
			config: &aws.Config{
				Region:           aws.String("us-west-2"),
				Endpoint:         aws.String("https://s3.us-west-2.amazonaws.com:8443"),
				S3ForcePathStyle: aws.Bool(true),
			},
			expectedEndpoint:      "https://s3.us-west-2.amazonaws.com:8443",
			expectedSigningName:   "s3",
			expectedSigningRegion: "us-west-2",
		},
		"Invalid AccessPoint ARN with FIPS pseudo-region (prefix)": {
			bucket: "arn:aws:s3:fips-us-east-1:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:         aws.String("us-west-2"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedErr: "FIPS region not allowed in ARN",
		},
		"Invalid AccessPoint ARN with FIPS pseudo-region (suffix)": {
			bucket: "arn:aws:s3:us-east-1-fips:123456789012:accesspoint:myendpoint",
			config: &aws.Config{
				Region:         aws.String("us-west-2"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedErr: "FIPS region not allowed in ARN",
		},
		"Invalid Outpost AccessPoint ARN with FIPS pseudo-region (prefix)": {
			bucket: "arn:aws:s3-outposts:fips-us-east-1:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:         aws.String("us-west-2"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedErr: "FIPS region not allowed in ARN",
		},
		"Invalid Outpost AccessPoint ARN with FIPS pseudo-region (suffix)": {
			bucket: "arn:aws:s3-outposts:us-east-1-fips:123456789012:outpost:op-01234567890123456:accesspoint:myaccesspoint",
			config: &aws.Config{
				Region:         aws.String("us-west-2"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedErr: "FIPS region not allowed in ARN",
		},
		"Invalid Object Lambda ARN with FIPS pseudo-region (prefix)": {
			bucket: "arn:aws:s3-object-lambda:fips-us-east-1:123456789012:accesspoint/myap",
			config: &aws.Config{
				Region:         aws.String("us-west-2"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedErr: "FIPS region not allowed in ARN",
		},
		"Invalid Object Lambda ARN with FIPS pseudo-region (suffix)": {
			bucket: "arn:aws:s3-object-lambda:us-east-1-fips:123456789012:accesspoint/myap",
			config: &aws.Config{
				Region:         aws.String("us-west-2"),
				S3UseARNRegion: aws.Bool(true),
			},
			expectedErr: "FIPS region not allowed in ARN",
		},
	}

	runValidations(t, cases)
}

func runValidations(t *testing.T, cases map[string]testCase) {
	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			if strings.EqualFold("az", name) {
				fmt.Print()
			}

			sess := unit.Session.Copy(c.config)

			svc := New(sess)

			var req *request.Request
			if c.req == nil {
				req, _ = svc.ListObjectsRequest(&ListObjectsInput{
					Bucket: &c.bucket,
				})
			} else {
				req = c.req(svc)
			}

			req.Handlers.Send.Clear()
			req.Handlers.Send.PushBack(func(r *request.Request) {
				defer func() {
					r.HTTPResponse = &http.Response{
						StatusCode:    200,
						ContentLength: 0,
						Body:          ioutil.NopCloser(bytes.NewReader(nil)),
					}
				}()
				if len(c.expectedErr) != 0 {
					return
				}

				endpoint := fmt.Sprintf("%s://%s", r.HTTPRequest.URL.Scheme, r.HTTPRequest.URL.Host)
				if e, a := c.expectedEndpoint, endpoint; e != a {
					t.Errorf("expected %v, got %v", e, a)
				}

				if e, a := c.expectedSigningName, r.ClientInfo.SigningName; e != a {
					t.Errorf("expected %v, got %v", e, a)
				}
				if e, a := c.expectedSigningRegion, r.ClientInfo.SigningRegion; e != a {
					t.Errorf("expected %v, got %v", e, a)
				}
			})
			err := req.Send()
			if len(c.expectedErr) == 0 && err != nil {
				t.Errorf("expected no error but got: %v", err)
			} else if len(c.expectedErr) != 0 && err == nil {
				t.Errorf("expected err %q, but got nil", c.expectedErr)
			} else if len(c.expectedErr) != 0 && err != nil && !strings.Contains(err.Error(), c.expectedErr) {
				t.Errorf("expected %v, got %v", c.expectedErr, err.Error())
			}
		})
	}
}

func TestWriteGetObjectResponse_UpdateEndpoint(t *testing.T) {
	cases := map[string]struct {
		config                *aws.Config
		expectedEndpoint      string
		expectedSigningRegion string
		expectedSigningName   string
		expectedErr           string
	}{
		"standard endpoint": {
			config: &aws.Config{
				Region: aws.String("us-west-2"),
			},
			expectedEndpoint:      "https://test-route.s3-object-lambda.us-west-2.amazonaws.com",
			expectedSigningRegion: "us-west-2",
			expectedSigningName:   "s3-object-lambda",
		},
		"fips endpoint": {
			config: &aws.Config{
				Region: aws.String("fips-us-gov-west-1"),
			},
			expectedEndpoint:      "https://test-route.s3-object-lambda-fips.us-gov-west-1.amazonaws.com",
			expectedSigningRegion: "us-gov-west-1",
			expectedSigningName:   "s3-object-lambda",
		},
		"duakstack endpoint (deprecated)": {
			config: &aws.Config{
				Region:       aws.String("us-west-2"),
				UseDualStack: aws.Bool(true),
			},
			expectedErr: "client configured for dualstack but not supported for operation",
		},
		"duakstack endpoint": {
			config: &aws.Config{
				Region:               aws.String("us-west-2"),
				UseDualStackEndpoint: endpoints.DualStackEndpointStateEnabled,
			},
			expectedErr: "client configured for dualstack but not supported for operation",
		},
		"accelerate endpoint": {
			config: &aws.Config{
				Region:          aws.String("us-west-2"),
				S3UseAccelerate: aws.Bool(true),
			},
			expectedErr: "client configured for accelerate but not supported for operation",
		},
		"custom endpoint": {
			config: &aws.Config{
				Region:   aws.String("us-west-2"),
				Endpoint: aws.String("https://my-domain.com"),
			},
			expectedEndpoint:      "https://test-route.my-domain.com",
			expectedSigningRegion: "us-west-2",
			expectedSigningName:   "s3-object-lambda",
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			sess := unit.Session.Copy(c.config)

			svc := New(sess)

			var req *request.Request
			req, _ = svc.WriteGetObjectResponseRequest(&WriteGetObjectResponseInput{
				RequestRoute: aws.String("test-route"),
				RequestToken: aws.String("test-token"),
			})

			req.Handlers.Send.Clear()
			req.Handlers.Send.PushBack(func(r *request.Request) {
				defer func() {
					r.HTTPResponse = &http.Response{
						StatusCode:    200,
						ContentLength: 0,
						Body:          ioutil.NopCloser(bytes.NewReader(nil)),
					}
				}()
				if len(c.expectedErr) != 0 {
					return
				}

				endpoint := fmt.Sprintf("%s://%s", r.HTTPRequest.URL.Scheme, r.HTTPRequest.URL.Host)
				if e, a := c.expectedEndpoint, endpoint; e != a {
					t.Errorf("expected %v, got %v", e, a)
				}

				if e, a := c.expectedSigningName, r.ClientInfo.SigningName; e != a {
					t.Errorf("expected %v, got %v", e, a)
				}
				if e, a := c.expectedSigningRegion, r.ClientInfo.SigningRegion; e != a {
					t.Errorf("expected %v, got %v", e, a)
				}
			})
			err := req.Send()
			if len(c.expectedErr) == 0 && err != nil {
				t.Errorf("expected no error but got: %v", err)
			} else if len(c.expectedErr) != 0 && err == nil {
				t.Errorf("expected err %q, but got nil", c.expectedErr)
			} else if len(c.expectedErr) != 0 && err != nil && !strings.Contains(err.Error(), c.expectedErr) {
				t.Errorf("expected %v, got %v", c.expectedErr, err.Error())
			}
		})
	}
}

type readSeeker struct {
	br *bytes.Reader
}

func (r *readSeeker) Read(p []byte) (int, error) {
	return r.br.Read(p)
}

func (r *readSeeker) Seek(offset int64, whence int) (int64, error) {
	return r.br.Seek(offset, whence)
}

type readOnlyReader struct {
	br *bytes.Reader
}

func (r *readOnlyReader) Read(p []byte) (int, error) {
	return r.br.Read(p)
}

type lenReader struct {
	br *bytes.Reader
}

func (r *lenReader) Read(p []byte) (int, error) {
	return r.br.Read(p)
}

func (r *lenReader) Len() int {
	return r.br.Len()
}

func TestWriteGetObjectResponse(t *testing.T) {
	cases := map[string]struct {
		Handler func(*testing.T) http.Handler
		Input   WriteGetObjectResponseInput
	}{
		"Content-Length seekable": {
			Handler: func(t *testing.T) http.Handler {
				return http.HandlerFunc(func(writer http.ResponseWriter, request *http.Request) {
					expectedInput := []byte("test input")

					if len(request.TransferEncoding) != 0 {
						t.Errorf("expect no transfer-encoding")
					}

					if e, a := fmt.Sprintf("%d", len(expectedInput)), request.Header.Get("Content-Length"); e != a {
						t.Errorf("expect %v, got %v", e, a)
					}

					if e, a := "UNSIGNED-PAYLOAD", request.Header.Get("X-Amz-Content-Sha256"); e != a {
						t.Errorf("expect %v, got %v", e, a)
					}

					all, err := ioutil.ReadAll(request.Body)
					if err != nil {
						t.Errorf("expect no error, got %v", err)
					}
					if !bytes.Equal(all, expectedInput) {
						t.Error("input did not match expected")
					}
					writer.WriteHeader(200)
				})
			},
			Input: WriteGetObjectResponseInput{
				RequestRoute: aws.String("route"),
				RequestToken: aws.String("token"),
				Body:         &readSeeker{br: bytes.NewReader([]byte("test input"))},
			},
		},
		"Content-Length Len Interface": {
			Handler: func(t *testing.T) http.Handler {
				return http.HandlerFunc(func(writer http.ResponseWriter, request *http.Request) {
					expectedInput := []byte("test input")

					if len(request.TransferEncoding) != 0 {
						t.Errorf("expect no transfer-encoding")
					}

					if e, a := fmt.Sprintf("%d", len(expectedInput)), request.Header.Get("Content-Length"); e != a {
						t.Errorf("expect %v, got %v", e, a)
					}

					if e, a := "UNSIGNED-PAYLOAD", request.Header.Get("X-Amz-Content-Sha256"); e != a {
						t.Errorf("expect %v, got %v", e, a)
					}

					all, err := ioutil.ReadAll(request.Body)
					if err != nil {
						t.Errorf("expect no error, got %v", err)
					}
					if !bytes.Equal(all, expectedInput) {
						t.Error("input did not match expected")
					}
					writer.WriteHeader(200)
				})
			},
			Input: WriteGetObjectResponseInput{
				RequestRoute: aws.String("route"),
				RequestToken: aws.String("token"),
				Body:         aws.ReadSeekCloser(&lenReader{bytes.NewReader([]byte("test input"))}),
			},
		},
		"Content-Length Input Parameter": {
			Handler: func(t *testing.T) http.Handler {
				return http.HandlerFunc(func(writer http.ResponseWriter, request *http.Request) {
					expectedInput := []byte("test input")

					if len(request.TransferEncoding) != 0 {
						t.Errorf("expect no transfer-encoding")
					}

					if e, a := fmt.Sprintf("%d", len(expectedInput)), request.Header.Get("Content-Length"); e != a {
						t.Errorf("expect %v, got %v", e, a)
					}

					if e, a := "UNSIGNED-PAYLOAD", request.Header.Get("X-Amz-Content-Sha256"); e != a {
						t.Errorf("expect %v, got %v", e, a)
					}

					all, err := ioutil.ReadAll(request.Body)
					if err != nil {
						t.Errorf("expect no error, got %v", err)
					}
					if !bytes.Equal(all, expectedInput) {
						t.Error("input did not match expected")
					}
					writer.WriteHeader(200)
				})
			},
			Input: WriteGetObjectResponseInput{
				RequestRoute:  aws.String("route"),
				RequestToken:  aws.String("token"),
				Body:          aws.ReadSeekCloser(&readOnlyReader{bytes.NewReader([]byte("test input"))}),
				ContentLength: aws.Int64(10),
			},
		},
		"Content-Length Not Provided": {
			Handler: func(t *testing.T) http.Handler {
				return http.HandlerFunc(func(writer http.ResponseWriter, request *http.Request) {
					expectedInput := []byte("test input")

					encoding := ""
					if len(request.TransferEncoding) == 1 {
						encoding = request.TransferEncoding[0]
					}
					if encoding != "chunked" {
						t.Errorf("expect transfer-encoding chunked, got %v", encoding)
					}

					if e, a := "", request.Header.Get("Content-Length"); e != a {
						t.Errorf("expect %v, got %v", e, a)
					}

					if e, a := "UNSIGNED-PAYLOAD", request.Header.Get("X-Amz-Content-Sha256"); e != a {
						t.Errorf("expect %v, got %v", e, a)
					}

					all, err := ioutil.ReadAll(request.Body)
					if err != nil {
						t.Errorf("expect no error, got %v", err)
					}
					if !bytes.Equal(all, expectedInput) {
						t.Error("input did not match expected")
					}
					writer.WriteHeader(200)
				})
			},
			Input: WriteGetObjectResponseInput{
				RequestRoute: aws.String("route"),
				RequestToken: aws.String("token"),
				Body:         aws.ReadSeekCloser(&readOnlyReader{bytes.NewReader([]byte("test input"))}),
			},
		},
	}

	for name, tt := range cases {
		t.Run(name, func(t *testing.T) {
			server := httptest.NewServer(tt.Handler(t))
			defer server.Close()

			sess := unit.Session.Copy(&aws.Config{
				Region:                    aws.String("us-west-2"),
				Endpoint:                  &server.URL,
				DisableEndpointHostPrefix: aws.Bool(true),
			})

			client := New(sess)

			_, err := client.WriteGetObjectResponse(&tt.Input)
			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}
		})
	}
}

func TestUseDualStackClientBehavior(t *testing.T) {
	cases := map[string]testCase{
		"UseDualStack unset, UseDualStackEndpoints unset": {
			bucket: "test-bucket",
			config: &aws.Config{
				Region: aws.String("us-west-2"),
			},
			expectedEndpoint:      "https://test-bucket.s3.us-west-2.amazonaws.com",
			expectedSigningRegion: "us-west-2",
			expectedSigningName:   "s3",
		},
		"UseDualStack false, UseDualStackEndpoints unset": {
			bucket: "test-bucket",
			config: &aws.Config{
				Region:       aws.String("us-west-2"),
				UseDualStack: aws.Bool(false),
			},
			expectedEndpoint:      "https://test-bucket.s3.us-west-2.amazonaws.com",
			expectedSigningRegion: "us-west-2",
			expectedSigningName:   "s3",
		},
		"UseDualStack true, UseDualStackEndpoints unset": {
			bucket: "test-bucket",
			config: &aws.Config{
				Region:       aws.String("us-west-2"),
				UseDualStack: aws.Bool(true),
			},
			expectedEndpoint:      "https://test-bucket.s3.dualstack.us-west-2.amazonaws.com",
			expectedSigningRegion: "us-west-2",
			expectedSigningName:   "s3",
		},
		"UseDualStack unset, UseDualStackEndpoints disabled": {
			bucket: "test-bucket",
			config: &aws.Config{
				Region:               aws.String("us-west-2"),
				UseDualStackEndpoint: endpoints.DualStackEndpointStateDisabled,
			},
			expectedEndpoint:      "https://test-bucket.s3.us-west-2.amazonaws.com",
			expectedSigningRegion: "us-west-2",
			expectedSigningName:   "s3",
		},
		"UseDualStack unset, UseDualStackEndpoint enabled": {
			bucket: "test-bucket",
			config: &aws.Config{
				Region:               aws.String("us-west-2"),
				UseDualStackEndpoint: endpoints.DualStackEndpointStateEnabled,
			},
			expectedEndpoint:      "https://test-bucket.s3.dualstack.us-west-2.amazonaws.com",
			expectedSigningRegion: "us-west-2",
			expectedSigningName:   "s3",
		},
		"UseDualStack true, UseDualStackEndpoint disabled": {
			bucket: "test-bucket",
			config: &aws.Config{
				Region:               aws.String("us-west-2"),
				UseDualStack:         aws.Bool(true),
				UseDualStackEndpoint: endpoints.DualStackEndpointStateDisabled,
			},
			expectedEndpoint:      "https://test-bucket.s3.us-west-2.amazonaws.com",
			expectedSigningRegion: "us-west-2",
			expectedSigningName:   "s3",
		},
		"UseDualStack false, UseDualStackEndpoint enabled": {
			bucket: "test-bucket",
			config: &aws.Config{
				Region:               aws.String("us-west-2"),
				UseDualStack:         aws.Bool(false),
				UseDualStackEndpoint: endpoints.DualStackEndpointStateEnabled,
			},
			expectedEndpoint:      "https://test-bucket.s3.dualstack.us-west-2.amazonaws.com",
			expectedSigningRegion: "us-west-2",
			expectedSigningName:   "s3",
		},
	}

	runValidations(t, cases)
}
