//go:build integration
// +build integration

package lexruntimev2

import (
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/awserr"
	"github.com/aws/aws-sdk-go/awstesting/integration"
)

func TestInteg_StartConversation_errorCase(t *testing.T) {
	sess := integration.SessionWithDefaultRegion("us-west-2")

	client := New(sess, &aws.Config{
		Logger:   t,
		LogLevel: aws.LogLevel(aws.LogDebugWithEventStreamBody | aws.LogDebugWithHTTPBody),
	})

	_, err := client.StartConversation(&StartConversationInput{
		BotAliasId: aws.String("mockAlias"),
		BotId:      aws.String("mockId01234567890"),
		LocaleId:   aws.String("mockLocale"),
		SessionId:  aws.String("mockSession"),
	})
	if err == nil {
		t.Fatalf("expect error, got none")
	}

	aErr, ok := err.(awserr.RequestFailure)
	if !ok {
		t.Fatalf("expect %T error, got %T, %v", aErr, err, err)
	}
	if aErr.Code() == "" {
		t.Errorf("expect error code, got none")
	}
	if aErr.Message() == "" {
		t.Errorf("expect error message, got none")
	}
}
