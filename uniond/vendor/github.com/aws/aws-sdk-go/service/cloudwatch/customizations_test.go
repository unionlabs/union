//go:build go1.7
// +build go1.7

package cloudwatch_test

import (
	"context"

	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/awstesting/unit"
	"github.com/aws/aws-sdk-go/service/cloudwatch"
)

func ExampleCloudWatch_PutMetricDataWithContext_withGzipRequest() {
	client := cloudwatch.New(sess)

	// The WithContext form of the operation methods accept request options.
	// The WithGzipRequest option will gzip the request payload before it is
	// sent.
	result, err := client.PutMetricDataWithContext(context.TODO(), params, cloudwatch.WithGzipRequest)

	_, _ = result, err
}

var params *cloudwatch.PutMetricDataInput
var sess *session.Session = unit.Session
