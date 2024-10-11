package request

import (
	"errors"
	"fmt"
	"net/http"
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/awserr"
	"github.com/aws/aws-sdk-go/aws/client/metadata"
)

func TestRequestIsErrorThrottle(t *testing.T) {
	cases := []struct {
		Err      error
		Throttle bool
		Req      Request
	}{
		{
			Err:      awserr.New("ProvisionedThroughputExceededException", "", nil),
			Throttle: true,
		},
		{
			Err:      awserr.New("ThrottledException", "", nil),
			Throttle: true,
		},
		{
			Err:      awserr.New("Throttling", "", nil),
			Throttle: true,
		},
		{
			Err:      awserr.New("ThrottlingException", "", nil),
			Throttle: true,
		},
		{
			Err:      awserr.New("RequestLimitExceeded", "", nil),
			Throttle: true,
		},
		{
			Err:      awserr.New("RequestThrottled", "", nil),
			Throttle: true,
		},
		{
			Err:      awserr.New("TooManyRequestsException", "", nil),
			Throttle: true,
		},
		{
			Err:      awserr.New("PriorRequestNotComplete", "", nil),
			Throttle: true,
		},
		{
			Err:      awserr.New("TransactionInProgressException", "", nil),
			Throttle: true,
		},
		{
			Err:      awserr.New("EC2ThrottledException", "", nil),
			Throttle: true,
		},
		{
			Err: awserr.NewRequestFailure(
				awserr.New(ErrCodeSerialization, "some error",
					awserr.NewUnmarshalError(nil, "blah", []byte{}),
				),
				503,
				"request-id",
			),
			Req: Request{
				HTTPResponse: &http.Response{
					StatusCode: 503,
					Header:     http.Header{},
				},
			},
			Throttle: true,
		},
		{
			Err: awserr.NewRequestFailure(
				awserr.New(ErrCodeSerialization, "some error",
					awserr.NewUnmarshalError(nil, "blah", []byte{}),
				),
				400,
				"request-id",
			),
			Req: Request{
				HTTPResponse: &http.Response{
					StatusCode: 400,
					Header:     http.Header{},
				},
			},
			Throttle: false,
		},
	}

	for i, c := range cases {
		req := c.Req
		req.Error = c.Err
		if e, a := c.Throttle, req.IsErrorThrottle(); e != a {
			t.Errorf("%d, expect %v to be throttled, was %t", i, c.Err, a)
		}
	}
}

type mockTempError bool

func (e mockTempError) Error() string {
	return fmt.Sprintf("mock temporary error: %t", e.Temporary())
}
func (e mockTempError) Temporary() bool {
	return bool(e)
}

func TestRequestIsErrorRetryable(t *testing.T) {
	cases := []struct {
		Err       error
		Req       Request
		Retryable bool
	}{
		{
			Err:       awserr.New(ErrCodeSerialization, "temporary error", mockTempError(true)),
			Retryable: true,
		},
		{
			Err:       awserr.New(ErrCodeSerialization, "temporary error", mockTempError(false)),
			Retryable: false,
		},
		{
			Err:       awserr.New(ErrCodeSerialization, "some error", errors.New("blah")),
			Retryable: true,
		},
		{
			Err: awserr.NewRequestFailure(
				awserr.New(ErrCodeSerialization, "some error",
					awserr.NewUnmarshalError(nil, "blah", []byte{}),
				),
				503,
				"request-id",
			),
			Req: Request{
				HTTPResponse: &http.Response{
					StatusCode: 503,
					Header:     http.Header{},
				},
			},
			Retryable: false, // classified as throttled not retryable
		},
		{
			Err: awserr.NewRequestFailure(
				awserr.New(ErrCodeSerialization, "some error",
					awserr.NewUnmarshalError(nil, "blah", []byte{}),
				),
				400,
				"request-id",
			),
			Req: Request{
				HTTPResponse: &http.Response{
					StatusCode: 400,
					Header:     http.Header{},
				},
			},
			Retryable: false,
		},
		{
			Err:       awserr.New("SomeError", "some error", nil),
			Retryable: false,
		},
		{
			Err:       awserr.New(ErrCodeRequestError, "some error", nil),
			Retryable: true,
		},
		{
			Err:       nil,
			Retryable: false,
		},
	}

	for i, c := range cases {
		req := c.Req
		req.Error = c.Err

		if e, a := c.Retryable, req.IsErrorRetryable(); e != a {
			t.Errorf("%d, expect %v to be retryable, was %t", i, c.Err, a)
		}
	}
}

func TestRequest_NilRetyer(t *testing.T) {
	clientInfo := metadata.ClientInfo{Endpoint: "https://mock.region.amazonaws.com"}
	req := New(aws.Config{}, clientInfo, Handlers{}, nil, &Operation{}, nil, nil)

	if req.Retryer == nil {
		t.Fatalf("expect retryer to be set")
	}

	if e, a := 0, req.MaxRetries(); e != a {
		t.Errorf("expect no retries, got %v", a)
	}
}
