// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package testutil

import (
	"context"

	"go.opentelemetry.io/otel"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	"go.opentelemetry.io/otel/sdk/trace/tracetest"
)

// OpenTelemetryTestExporter is a test utility exporter. It should be created
// with NewOpenTelemetryTestExporter.
type OpenTelemetryTestExporter struct {
	exporter *tracetest.InMemoryExporter
	tp       *sdktrace.TracerProvider
}

// NewOpenTelemetryTestExporter creates a OpenTelemetryTestExporter with
// underlying InMemoryExporter and TracerProvider from OpenTelemetry.
func NewOpenTelemetryTestExporter() *OpenTelemetryTestExporter {
	exporter := tracetest.NewInMemoryExporter()
	tp := sdktrace.NewTracerProvider(
		sdktrace.WithSyncer(exporter),
		sdktrace.WithSampler(sdktrace.AlwaysSample()),
	)
	otel.SetTracerProvider(tp)
	return &OpenTelemetryTestExporter{
		exporter: exporter,
		tp:       tp,
	}
}

// Spans returns the current in-memory stored spans.
func (te *OpenTelemetryTestExporter) Spans() tracetest.SpanStubs {
	return te.exporter.GetSpans()
}

// Unregister shuts down the underlying OpenTelemetry TracerProvider.
func (te *OpenTelemetryTestExporter) Unregister(ctx context.Context) {
	te.tp.Shutdown(ctx)
}
