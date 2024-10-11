package statsd

import (
	"fmt"
	"sync"
	"time"
)

/*
TelemetryInterval is the interval at which telemetry will be sent by the client.
*/
const TelemetryInterval = 10 * time.Second

/*
clientTelemetryTag is a tag identifying this specific client.
*/
var clientTelemetryTag = "client:go"

/*
clientVersionTelemetryTag is a tag identifying this specific client version.
*/
var clientVersionTelemetryTag = "client_version:4.8.3"

type telemetryClient struct {
	c          *Client
	tags       []string
	tagsByType map[metricType][]string
	sender     *sender
	worker     *worker
	devMode    bool
}

func newTelemetryClient(c *Client, transport string, devMode bool) *telemetryClient {
	t := &telemetryClient{
		c:          c,
		tags:       append(c.Tags, clientTelemetryTag, clientVersionTelemetryTag, "client_transport:"+transport),
		tagsByType: map[metricType][]string{},
		devMode:    devMode,
	}

	if devMode {
		t.tagsByType[gauge] = append(append([]string{}, t.tags...), "metrics_type:gauge")
		t.tagsByType[count] = append(append([]string{}, t.tags...), "metrics_type:count")
		t.tagsByType[set] = append(append([]string{}, t.tags...), "metrics_type:set")
		t.tagsByType[timing] = append(append([]string{}, t.tags...), "metrics_type:timing")
		t.tagsByType[histogram] = append(append([]string{}, t.tags...), "metrics_type:histogram")
		t.tagsByType[distribution] = append(append([]string{}, t.tags...), "metrics_type:distribution")
		t.tagsByType[timing] = append(append([]string{}, t.tags...), "metrics_type:timing")
	}
	return t
}

func newTelemetryClientWithCustomAddr(c *Client, transport string, devMode bool, telemetryAddr string, pool *bufferPool) (*telemetryClient, error) {
	telemetryWriter, _, err := createWriter(telemetryAddr)
	if err != nil {
		return nil, fmt.Errorf("Could not resolve telemetry address: %v", err)
	}

	t := newTelemetryClient(c, transport, devMode)

	// Creating a custom sender/worker with 1 worker in mutex mode for the
	// telemetry that share the same bufferPool.
	// FIXME due to performance pitfall, we're always using UDP defaults
	// even for UDS.
	t.sender = newSender(telemetryWriter, DefaultUDPBufferPoolSize, pool)
	t.worker = newWorker(pool, t.sender)
	return t, nil
}

func (t *telemetryClient) run(wg *sync.WaitGroup, stop chan struct{}) {
	wg.Add(1)
	go func() {
		defer wg.Done()
		ticker := time.NewTicker(TelemetryInterval)
		for {
			select {
			case <-ticker.C:
				t.sendTelemetry()
			case <-stop:
				ticker.Stop()
				if t.sender != nil {
					t.sender.close()
				}
				return
			}
		}
	}()
}

func (t *telemetryClient) sendTelemetry() {
	for _, m := range t.flush() {
		if t.worker != nil {
			t.worker.processMetric(m)
		} else {
			t.c.send(m)
		}
	}

	if t.worker != nil {
		t.worker.flush()
	}
}

// flushTelemetry returns Telemetry metrics to be flushed. It's its own function to ease testing.
func (t *telemetryClient) flush() []metric {
	m := []metric{}

	// same as Count but without global namespace
	telemetryCount := func(name string, value int64, tags []string) {
		m = append(m, metric{metricType: count, name: name, ivalue: value, tags: tags, rate: 1})
	}

	clientMetrics := t.c.FlushTelemetryMetrics()
	telemetryCount("datadog.dogstatsd.client.metrics", int64(clientMetrics.TotalMetrics), t.tags)
	if t.devMode {
		telemetryCount("datadog.dogstatsd.client.metrics_by_type", int64(clientMetrics.TotalMetricsGauge), t.tagsByType[gauge])
		telemetryCount("datadog.dogstatsd.client.metrics_by_type", int64(clientMetrics.TotalMetricsCount), t.tagsByType[count])
		telemetryCount("datadog.dogstatsd.client.metrics_by_type", int64(clientMetrics.TotalMetricsHistogram), t.tagsByType[histogram])
		telemetryCount("datadog.dogstatsd.client.metrics_by_type", int64(clientMetrics.TotalMetricsDistribution), t.tagsByType[distribution])
		telemetryCount("datadog.dogstatsd.client.metrics_by_type", int64(clientMetrics.TotalMetricsSet), t.tagsByType[set])
		telemetryCount("datadog.dogstatsd.client.metrics_by_type", int64(clientMetrics.TotalMetricsTiming), t.tagsByType[timing])
	}

	telemetryCount("datadog.dogstatsd.client.events", int64(clientMetrics.TotalEvents), t.tags)
	telemetryCount("datadog.dogstatsd.client.service_checks", int64(clientMetrics.TotalServiceChecks), t.tags)
	telemetryCount("datadog.dogstatsd.client.metric_dropped_on_receive", int64(clientMetrics.TotalDroppedOnReceive), t.tags)

	senderMetrics := t.c.sender.flushTelemetryMetrics()
	telemetryCount("datadog.dogstatsd.client.packets_sent", int64(senderMetrics.TotalSentPayloads), t.tags)
	telemetryCount("datadog.dogstatsd.client.bytes_sent", int64(senderMetrics.TotalSentBytes), t.tags)
	telemetryCount("datadog.dogstatsd.client.packets_dropped", int64(senderMetrics.TotalDroppedPayloads), t.tags)
	telemetryCount("datadog.dogstatsd.client.bytes_dropped", int64(senderMetrics.TotalDroppedBytes), t.tags)
	telemetryCount("datadog.dogstatsd.client.packets_dropped_queue", int64(senderMetrics.TotalDroppedPayloadsQueueFull), t.tags)
	telemetryCount("datadog.dogstatsd.client.bytes_dropped_queue", int64(senderMetrics.TotalDroppedBytesQueueFull), t.tags)
	telemetryCount("datadog.dogstatsd.client.packets_dropped_writer", int64(senderMetrics.TotalDroppedPayloadsWriter), t.tags)
	telemetryCount("datadog.dogstatsd.client.bytes_dropped_writer", int64(senderMetrics.TotalDroppedBytesWriter), t.tags)

	if aggMetrics := t.c.agg.flushTelemetryMetrics(); aggMetrics != nil {
		telemetryCount("datadog.dogstatsd.client.aggregated_context", int64(aggMetrics.nbContext), t.tags)
		if t.devMode {
			telemetryCount("datadog.dogstatsd.client.aggregated_context_by_type", int64(aggMetrics.nbContextGauge), t.tagsByType[gauge])
			telemetryCount("datadog.dogstatsd.client.aggregated_context_by_type", int64(aggMetrics.nbContextSet), t.tagsByType[set])
			telemetryCount("datadog.dogstatsd.client.aggregated_context_by_type", int64(aggMetrics.nbContextCount), t.tagsByType[count])
			telemetryCount("datadog.dogstatsd.client.aggregated_context_by_type", int64(aggMetrics.nbContextHistogram), t.tagsByType[histogram])
			telemetryCount("datadog.dogstatsd.client.aggregated_context_by_type", int64(aggMetrics.nbContextDistribution), t.tagsByType[distribution])
			telemetryCount("datadog.dogstatsd.client.aggregated_context_by_type", int64(aggMetrics.nbContextTiming), t.tagsByType[timing])
		}
	}

	return m
}
