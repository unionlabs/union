package log

import (
	"encoding/json"
	"fmt"
	"io"
)

// NewFilterWriter returns a writer that filters out all key/value pairs that do not match the filter.
// If the filter is nil, the writer will pass all events through.
// The filter function is called with the module and level of the event.
func NewFilterWriter(parent io.Writer, filter FilterFunc) io.Writer {
	return &filterWriter{parent, filter}
}

type filterWriter struct {
	parent io.Writer
	filter FilterFunc
}

func (fw *filterWriter) Write(p []byte) (n int, err error) {
	if fw.filter == nil {
		return fw.parent.Write(p)
	}

	var event struct {
		Level  string `json:"level"`
		Module string `json:"module"`
	}

	if err := json.Unmarshal(p, &event); err != nil {
		return 0, fmt.Errorf("failed to unmarshal event: %w", err)
	}

	// only filter module keys
	if fw.filter(event.Module, event.Level) {
		return len(p), nil
	}

	return fw.parent.Write(p)
}
