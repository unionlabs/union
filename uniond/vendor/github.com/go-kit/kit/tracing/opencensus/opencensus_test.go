package opencensus_test

import (
	"sync"

	"go.opencensus.io/trace"
)

type recordingExporter struct {
	mu   sync.Mutex
	data []*trace.SpanData
}

func (e *recordingExporter) ExportSpan(d *trace.SpanData) {
	e.mu.Lock()
	defer e.mu.Unlock()

	e.data = append(e.data, d)
}

func (e *recordingExporter) Flush() (data []*trace.SpanData) {
	e.mu.Lock()
	defer e.mu.Unlock()

	data = e.data
	e.data = nil
	return
}
