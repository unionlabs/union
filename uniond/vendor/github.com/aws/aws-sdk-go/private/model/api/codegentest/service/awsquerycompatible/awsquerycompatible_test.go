//go:build go1.13
// +build go1.13

package awsquerycompatible

import (
	"errors"
	"io"
	"io/ioutil"
	"net/http"
	"strings"
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/awserr"
	"github.com/aws/aws-sdk-go/aws/corehandlers"
	"github.com/aws/aws-sdk-go/aws/request"
	"github.com/aws/aws-sdk-go/awstesting/unit"
	"github.com/aws/aws-sdk-go/service/sqs"
)

func TestAWSQuery(t *testing.T) {
	cases := map[string]struct {
		statusCode   int
		responseBody interface {
			io.Reader
			Len() int
		}
		headers         map[string]string
		expectErrorCode string
	}{
		"when header is present": {
			statusCode:      400,
			responseBody:    strings.NewReader(`{"__type":"com.amazonaws.awsquerycompatible#QueueDeletedRecently", "message":"Some user-visible message"}`),
			expectErrorCode: sqs.ErrCodeQueueDeletedRecently,
			headers:         map[string]string{"x-amzn-query-error": "AWS.SimpleQueueService.QueueDeletedRecently;Sender"},
		},
		"for unmodeled error code": {
			statusCode:      400,
			responseBody:    strings.NewReader(`{"__type":"com.amazonaws.awsquerycompatible#AccessDeniedException", "message":"Some user-visible message"}`),
			expectErrorCode: "AccessDenied",
			headers:         map[string]string{"x-amzn-query-error": "AccessDenied;Sender"},
		},
		"when header is not present": {
			statusCode:      400,
			responseBody:    strings.NewReader(`{"__type":"com.amazonaws.awsquerycompatible#AccessDeniedException", "message":"Some user-visible message"}`),
			expectErrorCode: "AccessDeniedException",
			headers:         map[string]string{},
		},
		"when header is nil": {
			statusCode:      400,
			responseBody:    strings.NewReader(`{"__type":"com.amazonaws.awsquerycompatible#AccessDeniedException", "message":"Some user-visible message"}`),
			expectErrorCode: "AccessDeniedException",
			headers:         nil,
		},
		"when header is malformed": {
			statusCode:      400,
			responseBody:    strings.NewReader(`{"__type":"com.amazonaws.awsquerycompatible#QueueDeletedRecently", "message":"Some user-visible message"}`),
			expectErrorCode: "QueueDeletedRecently",
			headers:         map[string]string{"x-amzn-query-error": "AWS.SimpleQueueService.QueueDeletedRecently-Sender"},
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			client := New(unit.Session.Copy(), &aws.Config{
				MaxRetries: aws.Int(0),
			})

			client.Handlers.Send.Swap(corehandlers.SendHandler.Name, request.NamedHandler{
				Name: corehandlers.SendHandler.Name,
				Fn: func(r *request.Request) {
					r.HTTPResponse = &http.Response{
						StatusCode: c.statusCode,
						ContentLength: func() int64 {
							if c.responseBody == nil {
								return 0
							}
							return int64(c.responseBody.Len())
						}(),
						Header: func() http.Header {
							h := http.Header{}
							if c.headers != nil {
								for key, value := range c.headers {
									h.Set(key, value)
								}
							}
							return h
						}(),
						Body: ioutil.NopCloser(c.responseBody),
					}
				},
			})

			_, err := client.CreateQueue(&CreateQueueInput{
				QueueName: aws.String("queueName"),
			})
			if err == nil {
				t.Fatalf("expect error, got none")
			}

			var awsErr awserr.RequestFailure
			if !errors.As(err, &awsErr) {
				t.Fatalf("expect RequestFailure error, got %#v", err)
			}

			if e, a := c.expectErrorCode, awsErr.Code(); e != a {
				t.Errorf("expect %v code, got %v", e, a)
			}
		})
	}
}
