// Copyright 2018 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package trace

import (
	"context"
	"errors"
	"net/http"
	"sort"
	"testing"

	"cloud.google.com/go/internal/testutil"
	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/googleapis/gax-go/v2/apierror"
	octrace "go.opencensus.io/trace"
	"go.opentelemetry.io/otel/attribute"
	otcodes "go.opentelemetry.io/otel/codes"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	"google.golang.org/api/googleapi"
	"google.golang.org/genproto/googleapis/rpc/code"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

var (
	ignoreEventFields = cmpopts.IgnoreFields(sdktrace.Event{}, "Time")
	ignoreValueFields = cmpopts.IgnoreFields(attribute.Value{}, "vtype", "numeric", "stringly", "slice")
)

func TestStartSpan_OpenCensus(t *testing.T) {
	old := IsOpenTelemetryTracingEnabled()
	SetOpenTelemetryTracingEnabledField(false)
	te := testutil.NewTestExporter()
	t.Cleanup(func() {
		SetOpenTelemetryTracingEnabledField(old)
		te.Unregister()
	})

	ctx := context.Background()
	ctx = StartSpan(ctx, "test-span")

	TracePrintf(ctx, annotationData(), "Add my annotations")

	err := &googleapi.Error{Code: http.StatusBadRequest, Message: "INVALID ARGUMENT"}
	EndSpan(ctx, err)

	if !IsOpenCensusTracingEnabled() {
		t.Errorf("got false, want true")
	}
	if IsOpenTelemetryTracingEnabled() {
		t.Errorf("got true, want false")
	}
	spans := te.Spans()
	if len(spans) != 1 {
		t.Fatalf("got %d, want 1", len(spans))
	}
	if got, want := spans[0].Name, "test-span"; got != want {
		t.Fatalf("got %s, want %s", got, want)
	}
	if want := int32(3); spans[0].Status.Code != want {
		t.Errorf("got %v, want %v", spans[0].Status.Code, want)
	}
	if want := "INVALID ARGUMENT"; spans[0].Status.Message != want {
		t.Errorf("got %v, want %v", spans[0].Status.Message, want)
	}
	if len(spans[0].Annotations) != 1 {
		t.Fatalf("got %d, want 1", len(spans[0].Annotations))
	}
	got := spans[0].Annotations[0].Attributes
	want := make(map[string]interface{})
	want["my_bool"] = true
	want["my_float"] = "0.9"
	want["my_int"] = int64(123)
	want["my_int64"] = int64(456)
	want["my_string"] = "my string"
	opt := cmpopts.SortMaps(func(a, b int) bool {
		return a < b
	})
	if !cmp.Equal(got, want, opt) {
		t.Errorf("got(-), want(+),: \n%s", cmp.Diff(got, want, opt))
	}
}

func TestStartSpan_OpenTelemetry(t *testing.T) {
	old := IsOpenTelemetryTracingEnabled()
	SetOpenTelemetryTracingEnabledField(true)
	ctx := context.Background()
	te := testutil.NewOpenTelemetryTestExporter()
	t.Cleanup(func() {
		SetOpenTelemetryTracingEnabledField(old)
		te.Unregister(ctx)
	})

	ctx = StartSpan(ctx, "test-span")

	TracePrintf(ctx, annotationData(), "Add my annotations")

	err := &googleapi.Error{Code: http.StatusBadRequest, Message: "INVALID ARGUMENT"}
	EndSpan(ctx, err)

	if IsOpenCensusTracingEnabled() {
		t.Errorf("got true, want false")
	}
	if !IsOpenTelemetryTracingEnabled() {
		t.Errorf("got false, want true")
	}
	spans := te.Spans()
	if len(spans) != 1 {
		t.Fatalf("got %d, want 1", len(spans))
	}
	if got, want := spans[0].Name, "test-span"; got != want {
		t.Fatalf("got %s, want %s", got, want)
	}
	if want := otcodes.Error; spans[0].Status.Code != want {
		t.Errorf("got %v, want %v", spans[0].Status.Code, want)
	}
	if want := "INVALID ARGUMENT"; spans[0].Status.Description != want {
		t.Errorf("got %v, want %v", spans[0].Status.Description, want)
	}

	want := []attribute.KeyValue{
		attribute.Key("my_bool").Bool(true),
		attribute.Key("my_float").String("0.9"),
		attribute.Key("my_int").Int(123),
		attribute.Key("my_int64").Int64(int64(456)),
		attribute.Key("my_string").String("my string"),
	}
	got := spans[0].Events[0].Attributes
	// Sorting is required since the TracePrintf parameter is a map.
	sort.Slice(got, func(i, j int) bool {
		return got[i].Key < got[j].Key
	})
	if !cmp.Equal(got, want, ignoreEventFields, ignoreValueFields) {
		t.Errorf("got %v, want %v", got, want)
	}
	wantEvent := sdktrace.Event{
		Name: "exception",
		Attributes: []attribute.KeyValue{
			// KeyValues are NOT sorted by key, but the sort is deterministic,
			// since this Event was created by Span.RecordError.
			attribute.Key("exception.type").String("*googleapi.Error"),
			attribute.Key("exception.message").String("googleapi: Error 400: INVALID ARGUMENT"),
		},
	}
	if !cmp.Equal(spans[0].Events[1], wantEvent, ignoreEventFields, ignoreValueFields) {
		t.Errorf("got %v, want %v", spans[0].Events[1], want)
	}
}

func TestToStatus(t *testing.T) {
	for _, testcase := range []struct {
		input error
		want  octrace.Status
	}{
		{
			errors.New("some random error"),
			octrace.Status{Code: int32(code.Code_UNKNOWN), Message: "some random error"},
		},
		{
			&googleapi.Error{Code: http.StatusConflict, Message: "some specific googleapi http error"},
			octrace.Status{Code: int32(code.Code_ALREADY_EXISTS), Message: "some specific googleapi http error"},
		},
		{
			status.Error(codes.DataLoss, "some specific grpc error"),
			octrace.Status{Code: int32(code.Code_DATA_LOSS), Message: "some specific grpc error"},
		},
	} {
		got := toStatus(testcase.input)
		if r := testutil.Diff(got, testcase.want); r != "" {
			t.Errorf("got -, want +:\n%s", r)
		}
	}
}

func TestToOpenTelemetryStatusDescription(t *testing.T) {
	for _, testcase := range []struct {
		input error
		want  string
	}{
		{
			errors.New("some random error"),
			"some random error",
		},
		{
			&googleapi.Error{Code: http.StatusConflict, Message: "some specific googleapi http error"},
			"some specific googleapi http error",
		},
		{
			status.Error(codes.DataLoss, "some specific grpc error"),
			"some specific grpc error",
		},
	} {
		// Wrap supported types in apierror.APIError as GAPIC clients
		// do, but fall back to the unwrapped error if not supported.
		// https://github.com/googleapis/gax-go/blob/v2.12.0/v2/invoke.go#L95
		var err error
		err, ok := apierror.FromError(testcase.input)
		if !ok {
			err = testcase.input
		}

		got := toOpenTelemetryStatusDescription(err)
		if got != testcase.want {
			t.Errorf("got %s, want %s", got, testcase.want)
		}
	}
}

func TestToStatus_APIError(t *testing.T) {
	for _, testcase := range []struct {
		input error
		want  octrace.Status
	}{
		{
			// Apparently nonsensical error, but this is supported by the implementation.
			&googleapi.Error{Code: 200, Message: "OK"},
			octrace.Status{Code: int32(code.Code_OK), Message: "OK"},
		},
		{
			&googleapi.Error{Code: 499, Message: "error 499"},
			octrace.Status{Code: int32(code.Code_CANCELLED), Message: "error 499"},
		},
		{
			&googleapi.Error{Code: http.StatusInternalServerError, Message: "error 500"},
			octrace.Status{Code: int32(code.Code_UNKNOWN), Message: "error 500"},
		},
		{
			&googleapi.Error{Code: http.StatusBadRequest, Message: "error 400"},
			octrace.Status{Code: int32(code.Code_INVALID_ARGUMENT), Message: "error 400"},
		},
		{
			&googleapi.Error{Code: http.StatusGatewayTimeout, Message: "error 504"},
			octrace.Status{Code: int32(code.Code_DEADLINE_EXCEEDED), Message: "error 504"},
		},
		{
			&googleapi.Error{Code: http.StatusNotFound, Message: "error 404"},
			octrace.Status{Code: int32(code.Code_NOT_FOUND), Message: "error 404"},
		},
		{
			&googleapi.Error{Code: http.StatusConflict, Message: "error 409"},
			octrace.Status{Code: int32(code.Code_ALREADY_EXISTS), Message: "error 409"},
		},
		{
			&googleapi.Error{Code: http.StatusForbidden, Message: "error 403"},
			octrace.Status{Code: int32(code.Code_PERMISSION_DENIED), Message: "error 403"},
		},
		{
			&googleapi.Error{Code: http.StatusUnauthorized, Message: "error 401"},
			octrace.Status{Code: int32(code.Code_UNAUTHENTICATED), Message: "error 401"},
		},
		{
			&googleapi.Error{Code: http.StatusTooManyRequests, Message: "error 429"},
			octrace.Status{Code: int32(code.Code_RESOURCE_EXHAUSTED), Message: "error 429"},
		},
		{
			&googleapi.Error{Code: http.StatusNotImplemented, Message: "error 501"},
			octrace.Status{Code: int32(code.Code_UNIMPLEMENTED), Message: "error 501"},
		},
		{
			&googleapi.Error{Code: http.StatusServiceUnavailable, Message: "error 503"},
			octrace.Status{Code: int32(code.Code_UNAVAILABLE), Message: "error 503"},
		},
		{
			&googleapi.Error{Code: http.StatusMovedPermanently, Message: "error 301"},
			octrace.Status{Code: int32(code.Code_UNKNOWN), Message: "error 301"},
		},
	} {
		// Wrap googleapi.Error in apierror.APIError as GAPIC clients do.
		// https://github.com/googleapis/gax-go/blob/v2.12.0/v2/invoke.go#L95
		err, ok := apierror.FromError(testcase.input)
		if !ok {
			t.Fatalf("apierror.FromError failed to parse %v", testcase.input)
		}
		got := toStatus(err)
		if r := testutil.Diff(got, testcase.want); r != "" {
			t.Errorf("got -, want +:\n%s", r)
		}
	}
}

func annotationData() map[string]interface{} {
	attrMap := make(map[string]interface{})
	attrMap["my_string"] = "my string"
	attrMap["my_bool"] = true
	attrMap["my_int"] = 123
	attrMap["my_int64"] = int64(456)
	attrMap["my_float"] = 0.9
	return attrMap
}
