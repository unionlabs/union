package statsd

import (
	"fmt"
	"io"
	"net"
	"sort"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type testTelemetryData struct {
	gauge         int
	count         int
	histogram     int
	distribution  int
	set           int
	timing        int
	event         int
	service_check int

	aggregated_context      int
	aggregated_gauge        int
	aggregated_set          int
	aggregated_count        int
	aggregated_histogram    int
	aggregated_distribution int
	aggregated_timing       int

	metric_dropped_on_receive int
	packets_sent              int
	packets_dropped           int
	packets_dropped_queue     int
	packets_dropped_writer    int
	bytes_sent                int
	bytes_dropped             int
	bytes_dropped_queue       int
	bytes_dropped_writer      int
}

// testServer acts as a fake server and keep track of what was sent to a client. This allows end-to-end testing of the
// dogstatsd client
type testServer struct {
	sync.Mutex

	conn      io.ReadCloser
	data      []string
	errors    []string
	proto     string
	addr      string
	stopped   chan struct{}
	tags      string
	namespace string

	devMode             bool
	aggregation         bool
	extendedAggregation bool
	telemetry           testTelemetryData
	telemetryEnabled    bool
}

func newClientAndTestServer(t *testing.T, proto string, addr string, tags []string, options ...Option) (*testServer, *Client) {

	opt, err := resolveOptions(options)
	require.NoError(t, err)

	ts := &testServer{
		proto:               proto,
		data:                []string{},
		addr:                addr,
		stopped:             make(chan struct{}),
		devMode:             opt.DevMode,
		aggregation:         opt.Aggregation,
		extendedAggregation: opt.ExtendedAggregation,
		telemetryEnabled:    opt.Telemetry,
		telemetry:           testTelemetryData{},
		namespace:           opt.Namespace,
	}

	if tags != nil {
		ts.tags = strings.Join(tags, ",")
	}

	switch proto {
	case "udp":
		udpAddr, err := net.ResolveUDPAddr("udp", addr)
		require.NoError(t, err)

		conn, err := net.ListenUDP("udp", udpAddr)
		require.NoError(t, err)
		ts.conn = conn
	case "uds":
		conn, err := net.Dial("unix", addr[:7]) // we remove the 'unix://' prefix
		require.NoError(t, err)
		ts.conn = conn
	default:
		require.FailNow(t, "unknown proto '%s'", proto)
	}

	client, err := New(addr, options...)
	require.NoError(t, err)

	go ts.start()
	return ts, client
}

func (ts *testServer) start() {
	buffer := make([]byte, 2048)
	for {
		n, err := ts.conn.Read(buffer)
		if err != nil {
			// connection has been closed
			if strings.HasSuffix(err.Error(), " use of closed network connection") {
				return
			}
			ts.errors = append(ts.errors, err.Error())
			continue
		}
		payload := strings.Split(string(buffer[:n]), "\n")

		ts.Lock()
		for _, s := range payload {
			if s != "" {
				ts.data = append(ts.data, s)
			}
		}
		ts.Unlock()
	}
}

func (ts *testServer) assertMetric(t *testing.T, expected []string) {
	if ts.telemetryEnabled {
		expected = append(expected, ts.getTelemetry()...)
	}
	sort.Strings(expected)
	received := ts.getData()

	assert.Equal(t, len(expected), len(received), fmt.Sprintf("expected %d metrics but got actual %d", len(expected), len(received)))

	max := len(received)
	if len(expected) > max {
		max = len(expected)
	}
	for idx := 0; idx < max; idx++ {
		if strings.HasPrefix(expected[idx], "datadog.dogstatsd.client.bytes_sent") {
			continue
		}
		if strings.HasPrefix(expected[idx], "datadog.dogstatsd.client.packets_sent") {
			continue
		}
		assert.Equal(t, expected[idx], received[idx])
	}
}

func (ts *testServer) wait(t *testing.T, expected []string, timeout int) {
	start := time.Now()

	nbExpectedMetric := len(expected)
	// compute how many read we're expecting
	if ts.telemetryEnabled {
		nbExpectedMetric += 12 // 12 metrics by default
		if ts.devMode {
			nbExpectedMetric += 6 // dev mode add 6
		}

		if ts.aggregation {
			nbExpectedMetric += 1
			if ts.devMode {
				nbExpectedMetric += 6
			}
		}
	}

	for {
		ts.Lock()
		if nbExpectedMetric <= len(ts.data) || time.Now().Sub(start) > time.Duration(timeout)*time.Second {
			ts.Unlock()
			ts.conn.Close()
			close(ts.stopped)
			return
		}
		ts.Unlock()
		time.Sleep(100 * time.Millisecond)
	}
}

// meta helper: most test send all typy and thenn assert
func (ts *testServer) sendAllAndAssert(t *testing.T, client *Client) {
	expectedMetrics := ts.sendAllType(client)
	client.Flush()
	client.telemetry.sendTelemetry()
	ts.wait(t, expectedMetrics, 5)
	ts.assertMetric(t, expectedMetrics)
}

func (ts *testServer) getData() []string {
	ts.Lock()
	defer ts.Unlock()

	data := make([]string, len(ts.data))
	copy(data, ts.data)
	sort.Strings(data)
	return data
}

func (ts *testServer) getTelemetry() []string {
	ts.Lock()
	defer ts.Unlock()

	tags := ts.getFinalTelemetryTags()

	totalMetrics := ts.telemetry.gauge +
		ts.telemetry.count +
		ts.telemetry.histogram +
		ts.telemetry.distribution +
		ts.telemetry.set +
		ts.telemetry.timing

	metrics := []string{
		fmt.Sprintf("datadog.dogstatsd.client.metrics:%d|c%s", totalMetrics, tags),
		fmt.Sprintf("datadog.dogstatsd.client.events:%d|c%s", ts.telemetry.event, tags),
		fmt.Sprintf("datadog.dogstatsd.client.service_checks:%d|c%s", ts.telemetry.service_check, tags),
		fmt.Sprintf("datadog.dogstatsd.client.metric_dropped_on_receive:%d|c%s", ts.telemetry.metric_dropped_on_receive, tags),
		fmt.Sprintf("datadog.dogstatsd.client.packets_sent:%d|c%s", ts.telemetry.packets_sent, tags),
		fmt.Sprintf("datadog.dogstatsd.client.packets_dropped:%d|c%s", ts.telemetry.packets_dropped, tags),
		fmt.Sprintf("datadog.dogstatsd.client.packets_dropped_queue:%d|c%s", ts.telemetry.packets_dropped_queue, tags),
		fmt.Sprintf("datadog.dogstatsd.client.packets_dropped_writer:%d|c%s", ts.telemetry.packets_dropped_writer, tags),
		fmt.Sprintf("datadog.dogstatsd.client.bytes_sent:%d|c%s", ts.telemetry.bytes_sent, tags),
		fmt.Sprintf("datadog.dogstatsd.client.bytes_dropped:%d|c%s", ts.telemetry.bytes_dropped, tags),
		fmt.Sprintf("datadog.dogstatsd.client.bytes_dropped_queue:%d|c%s", ts.telemetry.bytes_dropped_queue, tags),
		fmt.Sprintf("datadog.dogstatsd.client.bytes_dropped_writer:%d|c%s", ts.telemetry.bytes_dropped_writer, tags),
	}
	if ts.devMode {
		metrics = append(metrics, []string{
			fmt.Sprintf("datadog.dogstatsd.client.metrics_by_type:%d|c%s,metrics_type:gauge", ts.telemetry.gauge, tags),
			fmt.Sprintf("datadog.dogstatsd.client.metrics_by_type:%d|c%s,metrics_type:count", ts.telemetry.count, tags),
			fmt.Sprintf("datadog.dogstatsd.client.metrics_by_type:%d|c%s,metrics_type:histogram", ts.telemetry.histogram, tags),
			fmt.Sprintf("datadog.dogstatsd.client.metrics_by_type:%d|c%s,metrics_type:distribution", ts.telemetry.distribution, tags),
			fmt.Sprintf("datadog.dogstatsd.client.metrics_by_type:%d|c%s,metrics_type:set", ts.telemetry.set, tags),
			fmt.Sprintf("datadog.dogstatsd.client.metrics_by_type:%d|c%s,metrics_type:timing", ts.telemetry.timing, tags),
		}...)
	}

	if ts.aggregation {
		metrics = append(metrics, fmt.Sprintf("datadog.dogstatsd.client.aggregated_context:%d|c%s", ts.telemetry.aggregated_context, tags))

		if ts.devMode {
			metrics = append(metrics, []string{
				fmt.Sprintf("datadog.dogstatsd.client.aggregated_context_by_type:%d|c%s,metrics_type:gauge", ts.telemetry.aggregated_gauge, tags),
				fmt.Sprintf("datadog.dogstatsd.client.aggregated_context_by_type:%d|c%s,metrics_type:count", ts.telemetry.aggregated_count, tags),
				fmt.Sprintf("datadog.dogstatsd.client.aggregated_context_by_type:%d|c%s,metrics_type:set", ts.telemetry.aggregated_set, tags),
				fmt.Sprintf("datadog.dogstatsd.client.aggregated_context_by_type:%d|c%s,metrics_type:distribution", ts.telemetry.aggregated_distribution, tags),
				fmt.Sprintf("datadog.dogstatsd.client.aggregated_context_by_type:%d|c%s,metrics_type:histogram", ts.telemetry.aggregated_histogram, tags),
				fmt.Sprintf("datadog.dogstatsd.client.aggregated_context_by_type:%d|c%s,metrics_type:timing", ts.telemetry.aggregated_timing, tags),
			}...)
		}
	}
	return metrics
}

// Default testing scenarios

func (ts *testServer) getFinalTags(t ...string) string {
	if t == nil && ts.tags == "" {
		return ""
	}

	res := "|#"
	if ts.tags != "" {
		res += ts.tags
	}

	if t != nil {
		if ts.tags != "" {
			res += ","
		}
		res += strings.Join(t, ",")
	}
	return res
}

func (ts *testServer) getFinalTelemetryTags() string {
	base := "|#"
	if ts.tags != "" {
		base += ts.tags + ","
	}
	return base + strings.Join(
		[]string{clientTelemetryTag, clientVersionTelemetryTag, "client_transport:" + ts.proto},
		",")
}

func (ts *testServer) sendAllMetrics(c *Client) []string {
	tags := []string{"custom:1", "custom:2"}
	c.Gauge("Gauge", 1, tags, 1)
	c.Count("Count", 2, tags, 1)
	c.Histogram("Histogram", 3, tags, 1)
	c.Distribution("Distribution", 4, tags, 1)
	c.Decr("Decr", tags, 1)
	c.Incr("Incr", tags, 1)
	c.Set("Set", "value", tags, 1)
	c.Timing("Timing", 5*time.Second, tags, 1)
	c.TimeInMilliseconds("TimeInMilliseconds", 6, tags, 1)

	ts.telemetry.gauge += 1
	ts.telemetry.histogram += 1
	ts.telemetry.distribution += 1
	ts.telemetry.count += 3
	ts.telemetry.set += 1
	ts.telemetry.timing += 2

	if ts.aggregation {
		ts.telemetry.aggregated_context += 5
		ts.telemetry.aggregated_gauge += 1
		ts.telemetry.aggregated_count += 3
		ts.telemetry.aggregated_set += 1
	}
	if ts.extendedAggregation {
		ts.telemetry.aggregated_context += 4
		ts.telemetry.aggregated_histogram += 1
		ts.telemetry.aggregated_distribution += 1
		ts.telemetry.aggregated_timing += 2
	}

	finalTags := ts.getFinalTags(tags...)

	return []string{
		ts.namespace + "Gauge:1|g" + finalTags,
		ts.namespace + "Count:2|c" + finalTags,
		ts.namespace + "Histogram:3|h" + finalTags,
		ts.namespace + "Distribution:4|d" + finalTags,
		ts.namespace + "Decr:-1|c" + finalTags,
		ts.namespace + "Incr:1|c" + finalTags,
		ts.namespace + "Set:value|s" + finalTags,
		ts.namespace + "Timing:5000.000000|ms" + finalTags,
		ts.namespace + "TimeInMilliseconds:6.000000|ms" + finalTags,
	}
}

func (ts *testServer) sendAllType(c *Client) []string {
	res := ts.sendAllMetrics(c)
	c.SimpleEvent("hello", "world")
	c.SimpleServiceCheck("hello", Warn)

	ts.telemetry.event += 1
	ts.telemetry.service_check += 1

	finalTags := ts.getFinalTags()

	return append(
		res,
		"_e{5,5}:hello|world"+finalTags,
		"_sc|hello|1"+finalTags,
	)
}

func (ts *testServer) sendBasicAggregationMetrics(client *Client) []string {
	tags := []string{"custom:1", "custom:2"}
	client.Gauge("gauge", 1, tags, 1)
	client.Gauge("gauge", 21, tags, 1)
	client.Count("count", 1, tags, 1)
	client.Count("count", 3, tags, 1)
	client.Set("set", "my_id", tags, 1)
	client.Set("set", "my_id", tags, 1)

	finalTags := ts.getFinalTags(tags...)
	return []string{
		ts.namespace + "set:my_id|s" + finalTags,
		ts.namespace + "gauge:21|g" + finalTags,
		ts.namespace + "count:4|c" + finalTags,
	}
}

func (ts *testServer) sendExtendedBasicAggregationMetrics(client *Client) []string {
	tags := []string{"custom:1", "custom:2"}
	client.Gauge("gauge", 1, tags, 1)
	client.Count("count", 2, tags, 1)
	client.Set("set", "3_id", tags, 1)
	client.Histogram("histo", 4, tags, 1)
	client.Distribution("distro", 5, tags, 1)
	client.Timing("timing", 6*time.Second, tags, 1)

	finalTags := ts.getFinalTags(tags...)
	return []string{
		ts.namespace + "gauge:1|g" + finalTags,
		ts.namespace + "count:2|c" + finalTags,
		ts.namespace + "set:3_id|s" + finalTags,
		ts.namespace + "histo:4|h" + finalTags,
		ts.namespace + "distro:5|d" + finalTags,
		ts.namespace + "timing:6000.000000|ms" + finalTags,
	}
}
