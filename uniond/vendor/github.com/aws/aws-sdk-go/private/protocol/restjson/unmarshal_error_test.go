//go:build go1.8
// +build go1.8

package restjson

import (
	"bytes"
	"io/ioutil"
	"net/http"
	"reflect"
	"strings"
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/awserr"
	"github.com/aws/aws-sdk-go/aws/request"
	"github.com/aws/aws-sdk-go/private/protocol"
)

type SimpleModeledError struct {
	_ struct{} `type:"structure"`
	error

	Message2 *string `type:"string" locationName:"message"`
	Foo      *int64  `type:"integer" locationName:"foo"`
}

type ComplexModeledError struct {
	_ struct{} `type:"structure"`
	error

	Message2  *string      `type:"string" locationName:"message"`
	Foo       *ErrorNested `type:"structure" locationName:"foo"`
	HeaderVal *string      `type:"string" location:"header" locationName:"some-header"`
	Status    *int64       `type:"integer" location:"statusCode"`
}

type ErrorNested struct {
	_ struct{} `type:"structure"`

	Bar *string `type:"string" locationName:"bar"`
	Baz *int64  `type:"integer" locationName:"baz"`
}

func TestUnmarshalTypedError(t *testing.T) {
	respMeta := protocol.ResponseMetadata{
		StatusCode: 400,
		RequestID:  "abc123",
	}

	exceptions := map[string]func(protocol.ResponseMetadata) error{
		"SimpleError": func(meta protocol.ResponseMetadata) error {
			return &SimpleModeledError{}
		},
		"ComplexError": func(meta protocol.ResponseMetadata) error {
			return &ComplexModeledError{}
		},
	}

	cases := map[string]struct {
		Response *http.Response
		Expect   error
		Err      string
	}{
		"SimpleError": {
			Response: &http.Response{
				Header: http.Header{
					errorTypeHeader: []string{"SimpleError"},
				},
				Body: ioutil.NopCloser(strings.NewReader(`{"code":"SimpleError","message":"simple error message","foo":1}`)),
			},
			Expect: &SimpleModeledError{
				Message2: aws.String("simple error message"),
				Foo:      aws.Int64(1),
			},
		},
		"SimpleError, w/namespace": {
			Response: &http.Response{
				Header: http.Header{
					errorTypeHeader: []string{"namespace#SimpleError"},
				},
				Body: ioutil.NopCloser(strings.NewReader(`{"code":"SimpleError","message":"simple error message","foo":1}`)),
			},
			Expect: &SimpleModeledError{
				Message2: aws.String("simple error message"),
				Foo:      aws.Int64(1),
			},
		},
		"SimpleError, w/details": {
			Response: &http.Response{
				Header: http.Header{
					errorTypeHeader: []string{"SimpleError:details:1"},
				},
				Body: ioutil.NopCloser(strings.NewReader(`{"code":"SimpleError","message":"simple error message","foo":1}`)),
			},
			Expect: &SimpleModeledError{
				Message2: aws.String("simple error message"),
				Foo:      aws.Int64(1),
			},
		},
		"SimpleError, w/namespace+details": {
			Response: &http.Response{
				Header: http.Header{
					errorTypeHeader: []string{"namespace#SimpleError:details:1"},
				},
				Body: ioutil.NopCloser(strings.NewReader(`{"code":"SimpleError","message":"simple error message","foo":1}`)),
			},
			Expect: &SimpleModeledError{
				Message2: aws.String("simple error message"),
				Foo:      aws.Int64(1),
			},
		},
		"SimpleError, no header": {
			Response: &http.Response{
				Header: http.Header{},
				Body:   ioutil.NopCloser(strings.NewReader(`{"code":"SimpleError","message":"simple error message","foo":1}`)),
			},
			Expect: &SimpleModeledError{
				Message2: aws.String("simple error message"),
				Foo:      aws.Int64(1),
			},
		},
		"SimpleError, no header, body __type": {
			Response: &http.Response{
				Header: http.Header{},
				Body:   ioutil.NopCloser(strings.NewReader(`{"__type":"SimpleError","message":"simple error message","foo":1}`)),
			},
			Expect: &SimpleModeledError{
				Message2: aws.String("simple error message"),
				Foo:      aws.Int64(1),
			},
		},
		"SimpleError, no header, w/namespace": {
			Response: &http.Response{
				Header: http.Header{},
				Body:   ioutil.NopCloser(strings.NewReader(`{"code":"namespace#SimpleError","message":"simple error message","foo":1}`)),
			},
			Expect: &SimpleModeledError{
				Message2: aws.String("simple error message"),
				Foo:      aws.Int64(1),
			},
		},
		"SimpleError, no header, w/details": {
			Response: &http.Response{
				Header: http.Header{},
				Body:   ioutil.NopCloser(strings.NewReader(`{"code":"SimpleError:details:1","message":"simple error message","foo":1}`)),
			},
			Expect: &SimpleModeledError{
				Message2: aws.String("simple error message"),
				Foo:      aws.Int64(1),
			},
		},
		"SimpleError, no header, w/namespace+details": {
			Response: &http.Response{
				Header: http.Header{},
				Body:   ioutil.NopCloser(strings.NewReader(`{"code":"namespace#SimpleError:details:1","message":"simple error message","foo":1}`)),
			},
			Expect: &SimpleModeledError{
				Message2: aws.String("simple error message"),
				Foo:      aws.Int64(1),
			},
		},
		"SimpleError, override message header": {
			Response: &http.Response{
				Header: http.Header{
					errorMessageHeader: []string{"overriden error message"},
				},
				Body: ioutil.NopCloser(strings.NewReader(`{"code":"SimpleError","message":"simple error message","foo":1}`)),
			},
			Expect: &SimpleModeledError{
				Message2: aws.String("simple error message"),
				Foo:      aws.Int64(1),
			},
		},
		"SimpleError, no body": {
			Response: &http.Response{
				Header: http.Header{
					errorTypeHeader:    []string{"SimpleError"},
					errorMessageHeader: []string{"simple error message"},
				},
				Body: http.NoBody,
			},
			Expect: &SimpleModeledError{},
		},
		"ComplexError": {
			Response: &http.Response{
				StatusCode: 400,
				Header: http.Header{
					errorTypeHeader: []string{"ComplexError"},
					"Some-Header":   []string{"headval"},
				},
				Body: ioutil.NopCloser(strings.NewReader(`{"code":"ComplexError","message":"complex error message","foo":{"bar":"abc123","baz":123}}`)),
			},
			Expect: &ComplexModeledError{
				Message2:  aws.String("complex error message"),
				HeaderVal: aws.String("headval"),
				Status:    aws.Int64(400),
				Foo: &ErrorNested{
					Bar: aws.String("abc123"),
					Baz: aws.Int64(123),
				},
			},
		},
		"ComplexError, no type header": {
			Response: &http.Response{
				StatusCode: 400,
				Header: http.Header{
					"Some-Header": []string{"headval"},
				},
				Body: ioutil.NopCloser(strings.NewReader(`{"code":"ComplexError","message":"complex error message","foo":{"bar":"abc123","baz":123}}`)),
			},
			Expect: &ComplexModeledError{
				Message2:  aws.String("complex error message"),
				HeaderVal: aws.String("headval"),
				Status:    aws.Int64(400),
				Foo: &ErrorNested{
					Bar: aws.String("abc123"),
					Baz: aws.Int64(123),
				},
			},
		},
		"ComplexError, override message header": {
			Response: &http.Response{
				StatusCode: 400,
				Header: http.Header{
					errorMessageHeader: []string{"overriden error message"},
					"Some-Header":      []string{"headval"},
				},
				Body: ioutil.NopCloser(strings.NewReader(`{"code":"ComplexError","message":"complex error message","foo":{"bar":"abc123","baz":123}}`)),
			},
			Expect: &ComplexModeledError{
				Message2:  aws.String("complex error message"),
				HeaderVal: aws.String("headval"),
				Status:    aws.Int64(400),
				Foo: &ErrorNested{
					Bar: aws.String("abc123"),
					Baz: aws.Int64(123),
				},
			},
		},
		"ComplexError, no body": {
			Response: &http.Response{
				StatusCode: 400,
				Header: http.Header{
					errorTypeHeader: []string{"ComplexError"},
					"Some-Header":   []string{"headval"},
				},
				Body: http.NoBody,
			},
			Expect: &ComplexModeledError{
				Status:    aws.Int64(400),
				HeaderVal: aws.String("headval"),
			},
		},
		"UnknownError": {
			Response: &http.Response{
				Header: http.Header{
					errorTypeHeader:    []string{"UnknownError"},
					errorMessageHeader: []string{"error message"},
				},
				Body: http.NoBody,
			},
			Expect: awserr.NewRequestFailure(
				awserr.New("UnknownError", "error message", nil),
				respMeta.StatusCode,
				respMeta.RequestID,
			),
		},
		"UnknownError, no message": {
			Response: &http.Response{
				Header: http.Header{
					errorTypeHeader: []string{"UnknownError"},
				},
				Body: http.NoBody,
			},
			Expect: awserr.NewRequestFailure(
				awserr.New("UnknownError", "", nil),
				respMeta.StatusCode,
				respMeta.RequestID,
			),
		},
		"UnknownError, no header type, body __type": {
			Response: &http.Response{
				Header: http.Header{},
				Body:   ioutil.NopCloser(strings.NewReader(`{"__type":"UnknownError"}`)),
			},
			Expect: awserr.NewRequestFailure(
				awserr.New("UnknownError", "", nil),
				respMeta.StatusCode,
				respMeta.RequestID,
			),
		},
		"UnknownError, no header type, body code": {
			Response: &http.Response{
				Header: http.Header{},
				Body:   ioutil.NopCloser(strings.NewReader(`{"code":"UnknownError"}`)),
			},
			Expect: awserr.NewRequestFailure(
				awserr.New("UnknownError", "", nil),
				respMeta.StatusCode,
				respMeta.RequestID,
			),
		},
		"UnknownError, body only": {
			Response: &http.Response{
				Header: http.Header{},
				Body:   ioutil.NopCloser(strings.NewReader(`{"code":"UnknownError","message":"unknown error message"}`)),
			},
			Expect: awserr.NewRequestFailure(
				awserr.New("UnknownError", "unknown error message", nil),
				respMeta.StatusCode,
				respMeta.RequestID,
			),
		},
		"no code": {
			Response: &http.Response{
				Header: http.Header{
					errorMessageHeader: []string{"no code message"},
				},
				Body: http.NoBody,
			},
			Expect: awserr.NewRequestFailure(
				awserr.New("", "no code message", nil),
				respMeta.StatusCode,
				respMeta.RequestID,
			),
		},
		"no information": {
			Response: &http.Response{
				StatusCode: 400,
				Header:     http.Header{},
				Body:       http.NoBody,
			},
			Expect: awserr.NewRequestFailure(
				awserr.New("", "", nil),
				respMeta.StatusCode,
				respMeta.RequestID,
			),
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			u := NewUnmarshalTypedError(exceptions)
			v, err := u.UnmarshalError(c.Response, respMeta)

			if len(c.Err) != 0 {
				if err == nil {
					t.Fatalf("expect error, got none")
				}
				if e, a := c.Err, err.Error(); !strings.Contains(a, e) {
					t.Fatalf("expect %v in error, got %v", e, a)
				}
			} else if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}

			if e, a := c.Expect, v; !reflect.DeepEqual(e, a) {
				t.Errorf("expect %+#v, got %#+v", e, a)
			}
		})
	}
}

func TestUnmarshalError_SerializationError(t *testing.T) {
	cases := map[string]struct {
		Request     *request.Request
		ExpectMsg   string
		ExpectBytes []byte
	}{
		"HTML body": {
			Request: &request.Request{
				Data: &struct{}{},
				HTTPResponse: &http.Response{
					StatusCode: 400,
					Header: http.Header{
						"X-Amzn-Requestid": []string{"abc123"},
					},
					Body: ioutil.NopCloser(
						bytes.NewReader([]byte(`<html></html>`)),
					),
				},
			},
			ExpectBytes: []byte(`<html></html>`),
			ExpectMsg:   "failed to decode response body",
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			req := c.Request

			UnmarshalError(req)
			if req.Error == nil {
				t.Fatal("expect error, got none")
			}

			aerr := req.Error.(awserr.RequestFailure)
			if e, a := request.ErrCodeSerialization, aerr.Code(); e != a {
				t.Errorf("expect %v, got %v", e, a)
			}

			uerr := aerr.OrigErr().(awserr.UnmarshalError)
			if e, a := c.ExpectMsg, uerr.Message(); !strings.Contains(a, e) {
				t.Errorf("Expect %q, in %q", e, a)
			}
		})
	}
}
