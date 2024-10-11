package sentry

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"testing"
)

func Test_spanRecorder_record(t *testing.T) {
	testRootSpan := StartSpan(context.Background(), "test", WithTransactionName("test transaction"))

	for _, tt := range []struct {
		name           string
		maxSpans       int
		toRecordSpans  int
		expectOverflow bool
	}{
		{
			name:           "record span without problems",
			maxSpans:       defaultMaxSpans,
			toRecordSpans:  1,
			expectOverflow: false,
		},
		{
			name:           "record span with overflow",
			maxSpans:       2,
			toRecordSpans:  4,
			expectOverflow: true,
		},
	} {
		t.Run(tt.name, func(t *testing.T) {
			logBuffer := bytes.Buffer{}
			Logger.SetOutput(&logBuffer)
			defer Logger.SetOutput(io.Discard)
			spanRecorder := spanRecorder{}

			currentHub.BindClient(&Client{
				options: ClientOptions{
					MaxSpans: tt.maxSpans,
				},
			})
			// Unbind the client afterwards, to not affect other tests
			defer currentHub.stackTop().SetClient(nil)

			for i := 0; i < tt.toRecordSpans; i++ {
				child := testRootSpan.StartChild(fmt.Sprintf("test %d", i))
				spanRecorder.record(child)
			}

			if tt.expectOverflow {
				assertNotEqual(t, len(spanRecorder.spans), tt.toRecordSpans, "expected overflow")
			} else {
				assertEqual(t, len(spanRecorder.spans), tt.toRecordSpans, "expected no overflow")
			}
			// check if Logger was called for overflow messages
			if bytes.Contains(logBuffer.Bytes(), []byte("Too many spans")) && !tt.expectOverflow {
				t.Error("unexpected overflow log")
			}
		})
	}
}
