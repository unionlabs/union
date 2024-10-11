// Copyright 2018 Google LLC
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
	"log"
	"sync"
	"time"

	"go.opencensus.io/plugin/ocgrpc"
	"go.opencensus.io/stats/view"
	"go.opencensus.io/trace"
)

// TestExporter is a test utility exporter. It should be created with NewtestExporter.
type TestExporter struct {
	mu    sync.Mutex
	spans []*trace.SpanData

	Stats chan *view.Data
	Views []*view.View
}

// NewTestExporter creates a TestExporter and registers it with OpenCensus.
func NewTestExporter(views ...*view.View) *TestExporter {
	if len(views) == 0 {
		views = ocgrpc.DefaultClientViews
	}
	te := &TestExporter{Stats: make(chan *view.Data), Views: views}

	view.RegisterExporter(te)
	view.SetReportingPeriod(time.Millisecond)
	if err := view.Register(views...); err != nil {
		log.Fatal(err)
	}

	trace.RegisterExporter(te)
	trace.ApplyConfig(trace.Config{DefaultSampler: trace.AlwaysSample()})

	return te
}

// ExportSpan exports a span.
func (te *TestExporter) ExportSpan(s *trace.SpanData) {
	te.mu.Lock()
	defer te.mu.Unlock()
	te.spans = append(te.spans, s)
}

// Spans returns the exported spans.
func (te *TestExporter) Spans() []*trace.SpanData {
	te.mu.Lock()
	defer te.mu.Unlock()
	spans := make([]*trace.SpanData, len(te.spans))
	copy(spans, te.spans)
	return spans
}

// ExportView exports a view.
func (te *TestExporter) ExportView(vd *view.Data) {
	if len(vd.Rows) > 0 {
		select {
		case te.Stats <- vd:
		default:
		}
	}
}

// Unregister unregisters the exporter from OpenCensus.
func (te *TestExporter) Unregister() {
	view.Unregister(te.Views...)
	view.UnregisterExporter(te)
	trace.UnregisterExporter(te)
	view.SetReportingPeriod(0) // reset to default value
}
