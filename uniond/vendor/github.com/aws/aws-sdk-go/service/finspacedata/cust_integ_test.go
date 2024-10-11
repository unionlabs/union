//go:build go1.7 && integration
// +build go1.7,integration

package finspacedata

import (
	"os"
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/awstesting/integration"
)

func TestInteg_GetWorkingLocation(t *testing.T) {
	if v := os.Getenv("AWS_SDK_GO_MANUAL_TESTS"); len(v) == 0 {
		t.Skip("manual test")
	}
	sess := integration.SessionWithDefaultRegion("us-west-2")

	client := New(sess)
	_, err := client.GetWorkingLocation(&GetWorkingLocationInput{
		LocationType: aws.String("INGESTION"),
	})
	if err != nil {
		t.Fatalf("expect API call, got %v, %#v", err, err)
	}
}
