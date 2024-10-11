package grpc_opentracing

import (
	"fmt"
	grpc_ctxtags "github.com/grpc-ecosystem/go-grpc-middleware/tags"
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestTagsCarrier_Set_JaegerTraceFormat(t *testing.T) {
	var (
		fakeTraceSampled   = 1
		fakeInboundTraceId = "deadbeef"
		fakeInboundSpanId  = "c0decafe"
		traceHeaderName    = "uber-trace-id"
	)

	traceHeaderValue := fmt.Sprintf("%s:%s:%s:%d", fakeInboundTraceId, fakeInboundSpanId, fakeInboundSpanId, fakeTraceSampled)

	c := &tagsCarrier{
		Tags:            grpc_ctxtags.NewTags(),
		traceHeaderName: traceHeaderName,
	}

	c.Set(traceHeaderName, traceHeaderValue)

	assert.EqualValues(t, map[string]interface{}{
		TagTraceId: fakeInboundTraceId,
		TagSpanId:  fakeInboundSpanId,
		TagSampled: "true",
	}, c.Tags.Values())
}
