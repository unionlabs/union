package sentry

import (
	"sync"
)

// A spanRecorder stores a span tree that makes up a transaction. Safe for
// concurrent use. It is okay to add child spans from multiple goroutines.
type spanRecorder struct {
	mu           sync.Mutex
	spans        []*Span
	overflowOnce sync.Once
}

// record stores a span. The first stored span is assumed to be the root of a
// span tree.
func (r *spanRecorder) record(s *Span) {
	maxSpans := defaultMaxSpans
	if client := CurrentHub().Client(); client != nil {
		maxSpans = client.options.MaxSpans
	}
	r.mu.Lock()
	defer r.mu.Unlock()
	if len(r.spans) >= maxSpans {
		r.overflowOnce.Do(func() {
			root := r.spans[0]
			Logger.Printf("Too many spans: dropping spans from transaction with TraceID=%s SpanID=%s limit=%d",
				root.TraceID, root.SpanID, maxSpans)
		})
		// TODO(tracing): mark the transaction event in some way to
		// communicate that spans were dropped.
		return
	}
	r.spans = append(r.spans, s)
}

// root returns the first recorded span. Returns nil if none have been recorded.
func (r *spanRecorder) root() *Span {
	r.mu.Lock()
	defer r.mu.Unlock()
	if len(r.spans) == 0 {
		return nil
	}
	return r.spans[0]
}

// children returns a list of all recorded spans, except the root. Returns nil
// if there are no children.
func (r *spanRecorder) children() []*Span {
	r.mu.Lock()
	defer r.mu.Unlock()
	if len(r.spans) < 2 {
		return nil
	}
	return r.spans[1:]
}
