package statsd

import (
	"io"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestTelemetry(t *testing.T) {
	ts, client := newClientAndTestServer(t,
		"udp",
		"localhost:8765",
		nil,
	)

	ts.sendAllAndAssert(t, client)
}

func TestTelemetryWithNamespace(t *testing.T) {
	ts, client := newClientAndTestServer(t,
		"udp",
		"localhost:8765",
		nil,
		WithNamespace("test_namespace"),
	)

	ts.sendAllAndAssert(t, client)
}

func TestTelemetryDevMode(t *testing.T) {
	ts, client := newClientAndTestServer(t,
		"udp",
		"localhost:8765",
		nil,
		WithDevMode(),
	)

	ts.sendAllAndAssert(t, client)
}

func TestTelemetryChannelMode(t *testing.T) {
	ts, client := newClientAndTestServer(t,
		"udp",
		"localhost:8765",
		nil,
		WithChannelMode(),
	)

	ts.sendAllAndAssert(t, client)
}

func TestTelemetryWithGlobalTags(t *testing.T) {
	orig := os.Getenv("DD_ENV")
	os.Setenv("DD_ENV", "test")
	defer os.Setenv("DD_ENV", orig)

	ts, client := newClientAndTestServer(t,
		"udp",
		"localhost:8765",
		[]string{"tag1", "tag2", "env:test"},
		WithTags([]string{"tag1", "tag2"}),
	)

	ts.sendAllAndAssert(t, client)
}

func TestTelemetryWithAggregationBasic(t *testing.T) {
	ts, client := newClientAndTestServer(t,
		"udp",
		"localhost:8765",
		nil,
		WithClientSideAggregation(),
	)

	ts.sendAllAndAssert(t, client)
}

func TestTelemetryWithExtendedAggregation(t *testing.T) {
	ts, client := newClientAndTestServer(t,
		"udp",
		"localhost:8765",
		nil,
		WithExtendedClientSideAggregation(),
	)

	ts.sendAllAndAssert(t, client)
}

func TestTelemetryWithAggregationDevMode(t *testing.T) {
	ts, client := newClientAndTestServer(t,
		"udp",
		"localhost:8765",
		nil,
		WithExtendedClientSideAggregation(),
		WithDevMode(),
	)

	ts.sendAllAndAssert(t, client)
}

func TestTelemetryAllOptions(t *testing.T) {
	orig := os.Getenv("DD_ENV")
	os.Setenv("DD_ENV", "test")
	defer os.Setenv("DD_ENV", orig)

	ts, client := newClientAndTestServer(t,
		"udp",
		"localhost:8765",
		[]string{"tag1", "tag2", "env:test"},
		WithClientSideAggregation(),
		WithExtendedClientSideAggregation(),
		WithDevMode(),
		WithTags([]string{"tag1", "tag2"}),
		WithNamespace("test_namespace"),
	)

	ts.sendAllAndAssert(t, client)
}

func TestTelemetryCustomAddr(t *testing.T) {
	telAddr := "localhost:8764"
	ts, client := newClientAndTestServer(t,
		"udp",
		"localhost:8765",
		nil,
		WithTelemetryAddr(telAddr),
		WithNamespace("test_namespace"),
	)

	server := getTestServer(t, telAddr)
	defer server.Close()

	readDone := make(chan struct{})
	buffer := make([]byte, 4096)
	n := 0
	go func() {
		n, _ = io.ReadAtLeast(server, buffer, 1)
		close(readDone)
	}()

	ts.sendAllType(client)
	client.Flush()
	client.telemetry.sendTelemetry()

	select {
	case <-readDone:
	case <-time.After(2 * time.Second):
		require.Fail(t, "No data was flush on Close")
	}

	result := string(buffer[:n])

	expectedPayload := "datadog.dogstatsd.client.metrics:9|c|#client:go," + clientVersionTelemetryTag + ",client_transport:udp\n" +
		"datadog.dogstatsd.client.events:1|c|#client:go," + clientVersionTelemetryTag + ",client_transport:udp\n" +
		"datadog.dogstatsd.client.service_checks:1|c|#client:go," + clientVersionTelemetryTag + ",client_transport:udp\n" +
		"datadog.dogstatsd.client.metric_dropped_on_receive:0|c|#client:go," + clientVersionTelemetryTag + ",client_transport:udp\n" +
		"datadog.dogstatsd.client.packets_sent:10|c|#client:go," + clientVersionTelemetryTag + ",client_transport:udp\n" +
		"datadog.dogstatsd.client.bytes_sent:473|c|#client:go," + clientVersionTelemetryTag + ",client_transport:udp\n" +
		"datadog.dogstatsd.client.packets_dropped:0|c|#client:go," + clientVersionTelemetryTag + ",client_transport:udp\n" +
		"datadog.dogstatsd.client.bytes_dropped:0|c|#client:go," + clientVersionTelemetryTag + ",client_transport:udp\n" +
		"datadog.dogstatsd.client.packets_dropped_queue:0|c|#client:go," + clientVersionTelemetryTag + ",client_transport:udp\n" +
		"datadog.dogstatsd.client.bytes_dropped_queue:0|c|#client:go," + clientVersionTelemetryTag + ",client_transport:udp\n" +
		"datadog.dogstatsd.client.packets_dropped_writer:0|c|#client:go," + clientVersionTelemetryTag + ",client_transport:udp\n" +
		"datadog.dogstatsd.client.bytes_dropped_writer:0|c|#client:go," + clientVersionTelemetryTag + ",client_transport:udp\n"
	assert.Equal(t, expectedPayload, result)
}
