//go:build go1.7 && integration
// +build go1.7,integration

package lookoutmetrics

import (
	"testing"

	"github.com/aws/aws-sdk-go/awstesting/integration"
)

func TestInteg_ListAnomalyDetectors(t *testing.T) {
	sess := integration.SessionWithDefaultRegion("us-west-2")
	client := New(sess)
	_, err := client.ListAnomalyDetectors(&ListAnomalyDetectorsInput{})
	if err != nil {
		t.Fatalf("expect API call, got %v", err)
	}
}
