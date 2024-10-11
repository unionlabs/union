//go:build go1.7 && integration
// +build go1.7,integration

package finspace

import (
	"testing"

	"github.com/aws/aws-sdk-go/awstesting/integration"
)

func TestInteg_ListEnvironments(t *testing.T) {
	sess := integration.SessionWithDefaultRegion("us-west-2")

	client := New(sess)
	_, err := client.ListEnvironments(&ListEnvironmentsInput{})
	if err != nil {
		t.Fatalf("expect API call, got %v", err)
	}
}
