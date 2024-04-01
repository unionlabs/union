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
	"fmt"
	"os"
	"strings"

	"go.opencensus.io/trace"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	ottrace "go.opentelemetry.io/otel/trace"
	"google.golang.org/api/googleapi"
	"google.golang.org/genproto/googleapis/rpc/code"
	"google.golang.org/grpc/status"
)

const (
	// TelemetryPlatformTracingOpenCensus is the value to which the environment
	// variable GOOGLE_API_GO_EXPERIMENTAL_TELEMETRY_PLATFORM_TRACING should be
	// set to enable OpenCensus tracing.
	TelemetryPlatformTracingOpenCensus = "opencensus"
	// TelemetryPlatformTracingOpenCensus is the value to which the environment
	// variable GOOGLE_API_GO_EXPERIMENTAL_TELEMETRY_PLATFORM_TRACING should be
	// set to enable OpenTelemetry tracing.
	TelemetryPlatformTracingOpenTelemetry = "opentelemetry"
	// TelemetryPlatformTracingOpenCensus is the name of the environment
	// variable that can be set to change the default tracing from OpenCensus
	// to OpenTelemetry.
	TelemetryPlatformTracingVar = "GOOGLE_API_GO_EXPERIMENTAL_TELEMETRY_PLATFORM_TRACING"
	// OpenTelemetryTracerName is the name given to the OpenTelemetry Tracer
	// when it is obtained from the OpenTelemetry TracerProvider.
	OpenTelemetryTracerName = "cloud.google.com/go"
)

var (
	// OpenTelemetryTracingEnabled is true if the environment variable
	// GOOGLE_API_GO_EXPERIMENTAL_TELEMETRY_PLATFORM_TRACING is set to the
	// case-insensitive value "opentelemetry".
	//
	// Do not access directly. Use instead IsOpenTelemetryTracingEnabled or
	// IsOpenCensusTracingEnabled. Intended for use only in unit tests. Restore
	// original value after each test.
	OpenTelemetryTracingEnabled bool = strings.EqualFold(strings.TrimSpace(
		os.Getenv(TelemetryPlatformTracingVar)), TelemetryPlatformTracingOpenTelemetry)
)

// IsOpenCensusTracingEnabled returns true if the environment variable
// GOOGLE_API_GO_EXPERIMENTAL_TELEMETRY_PLATFORM_TRACING is NOT set to the
// case-insensitive value "opentelemetry".
func IsOpenCensusTracingEnabled() bool {
	return !IsOpenTelemetryTracingEnabled()
}

// IsOpenTelemetryTracingEnabled returns true if the environment variable
// GOOGLE_API_GO_EXPERIMENTAL_TELEMETRY_PLATFORM_TRACING is set to the
// case-insensitive value "opentelemetry".
func IsOpenTelemetryTracingEnabled() bool {
	return OpenTelemetryTracingEnabled
}

// StartSpan adds a span to the trace with the given name. If IsOpenCensusTracingEnabled
// returns true, the span will be an OpenCensus span. If IsOpenTelemetryTracingEnabled
// returns true, the span will be an OpenTelemetry span. Set the environment variable
// GOOGLE_API_GO_EXPERIMENTAL_TELEMETRY_PLATFORM_TRACING to the case-insensitive
// value "opentelemetry" before loading the package to use OpenTelemetry tracing.
// The default will remain OpenCensus until May 29, 2024, at which time the default will
// switch to "opentelemetry" and explicitly setting the environment variable to
// "opencensus" will be required to continue using OpenCensus tracing.
func StartSpan(ctx context.Context, name string) context.Context {
	if IsOpenTelemetryTracingEnabled() {
		ctx, _ = otel.GetTracerProvider().Tracer(OpenTelemetryTracerName).Start(ctx, name)
	} else {
		ctx, _ = trace.StartSpan(ctx, name)
	}
	return ctx
}

// EndSpan ends a span with the given error. If IsOpenCensusTracingEnabled
// returns true, the span will be an OpenCensus span. If IsOpenTelemetryTracingEnabled
// returns true, the span will be an OpenTelemetry span. Set the environment variable
// GOOGLE_API_GO_EXPERIMENTAL_TELEMETRY_PLATFORM_TRACING to the case-insensitive
// value "opentelemetry" before loading the package to use OpenTelemetry tracing.
// The default will remain OpenCensus until May 29, 2024, at which time the default will
// switch to "opentelemetry" and explicitly setting the environment variable to
// "opencensus" will be required to continue using OpenCensus tracing.
func EndSpan(ctx context.Context, err error) {
	if IsOpenTelemetryTracingEnabled() {
		span := ottrace.SpanFromContext(ctx)
		if err != nil {
			span.SetStatus(codes.Error, toOpenTelemetryStatusDescription(err))
			span.RecordError(err)
		}
		span.End()
	} else {
		span := trace.FromContext(ctx)
		if err != nil {
			span.SetStatus(toStatus(err))
		}
		span.End()
	}
}

// toStatus converts an error to an equivalent OpenCensus status.
func toStatus(err error) trace.Status {
	var err2 *googleapi.Error
	if ok := errors.As(err, &err2); ok {
		return trace.Status{Code: httpStatusCodeToOCCode(err2.Code), Message: err2.Message}
	} else if s, ok := status.FromError(err); ok {
		return trace.Status{Code: int32(s.Code()), Message: s.Message()}
	} else {
		return trace.Status{Code: int32(code.Code_UNKNOWN), Message: err.Error()}
	}
}

// toOpenTelemetryStatus converts an error to an equivalent OpenTelemetry status description.
func toOpenTelemetryStatusDescription(err error) string {
	var err2 *googleapi.Error
	if ok := errors.As(err, &err2); ok {
		return err2.Message
	} else if s, ok := status.FromError(err); ok {
		return s.Message()
	} else {
		return err.Error()
	}
}

// TODO(deklerk): switch to using OpenCensus function when it becomes available.
// Reference: https://github.com/googleapis/googleapis/blob/26b634d2724ac5dd30ae0b0cbfb01f07f2e4050e/google/rpc/code.proto
func httpStatusCodeToOCCode(httpStatusCode int) int32 {
	switch httpStatusCode {
	case 200:
		return int32(code.Code_OK)
	case 499:
		return int32(code.Code_CANCELLED)
	case 500:
		return int32(code.Code_UNKNOWN) // Could also be Code_INTERNAL, Code_DATA_LOSS
	case 400:
		return int32(code.Code_INVALID_ARGUMENT) // Could also be Code_OUT_OF_RANGE
	case 504:
		return int32(code.Code_DEADLINE_EXCEEDED)
	case 404:
		return int32(code.Code_NOT_FOUND)
	case 409:
		return int32(code.Code_ALREADY_EXISTS) // Could also be Code_ABORTED
	case 403:
		return int32(code.Code_PERMISSION_DENIED)
	case 401:
		return int32(code.Code_UNAUTHENTICATED)
	case 429:
		return int32(code.Code_RESOURCE_EXHAUSTED)
	case 501:
		return int32(code.Code_UNIMPLEMENTED)
	case 503:
		return int32(code.Code_UNAVAILABLE)
	default:
		return int32(code.Code_UNKNOWN)
	}
}

// TracePrintf retrieves the current OpenCensus or OpenTelemetry span from context, then:
// * calls Span.Annotatef if OpenCensus is enabled; or
// * calls Span.AddEvent if OpenTelemetry is enabled.
//
// If IsOpenCensusTracingEnabled returns true, the expected span must be an
// OpenCensus span. If IsOpenTelemetryTracingEnabled returns true, the expected
// span must be an OpenTelemetry span. Set the environment variable
// GOOGLE_API_GO_EXPERIMENTAL_TELEMETRY_PLATFORM_TRACING to the case-insensitive
// value "opentelemetry" before loading the package to use OpenTelemetry tracing.
// The default will remain OpenCensus until May 29, 2024, at which time the default will
// switch to "opentelemetry" and explicitly setting the environment variable to
// "opencensus" will be required to continue using OpenCensus tracing.
func TracePrintf(ctx context.Context, attrMap map[string]interface{}, format string, args ...interface{}) {
	if IsOpenTelemetryTracingEnabled() {
		attrs := otAttrs(attrMap)
		ottrace.SpanFromContext(ctx).AddEvent(fmt.Sprintf(format, args...), ottrace.WithAttributes(attrs...))
	} else {
		attrs := ocAttrs(attrMap)
		// TODO: (odeke-em): perhaps just pass around spans due to the cost
		// incurred from using trace.FromContext(ctx) yet we could avoid
		// throwing away the work done by ctx, span := trace.StartSpan.
		trace.FromContext(ctx).Annotatef(attrs, format, args...)
	}
}

// ocAttrs converts a generic map to OpenCensus attributes.
func ocAttrs(attrMap map[string]interface{}) []trace.Attribute {
	var attrs []trace.Attribute
	for k, v := range attrMap {
		var a trace.Attribute
		switch v := v.(type) {
		case string:
			a = trace.StringAttribute(k, v)
		case bool:
			a = trace.BoolAttribute(k, v)
		case int:
			a = trace.Int64Attribute(k, int64(v))
		case int64:
			a = trace.Int64Attribute(k, v)
		default:
			a = trace.StringAttribute(k, fmt.Sprintf("%#v", v))
		}
		attrs = append(attrs, a)
	}
	return attrs
}

// otAttrs converts a generic map to OpenTelemetry attributes.
func otAttrs(attrMap map[string]interface{}) []attribute.KeyValue {
	var attrs []attribute.KeyValue
	for k, v := range attrMap {
		var a attribute.KeyValue
		switch v := v.(type) {
		case string:
			a = attribute.Key(k).String(v)
		case bool:
			a = attribute.Key(k).Bool(v)
		case int:
			a = attribute.Key(k).Int(v)
		case int64:
			a = attribute.Key(k).Int64(v)
		default:
			a = attribute.Key(k).String(fmt.Sprintf("%#v", v))
		}
		attrs = append(attrs, a)
	}
	return attrs
}
