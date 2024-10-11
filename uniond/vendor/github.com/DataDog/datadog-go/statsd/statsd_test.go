package statsd

import (
	"fmt"
	"io"
	"net"
	"os"
	"sort"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

var (
	defaultAddr = "localhost:1201"
)

type statsdWriterWrapper struct{}

func (statsdWriterWrapper) SetWriteTimeout(time.Duration) error {
	return nil
}

func (statsdWriterWrapper) Close() error {
	return nil
}

func (statsdWriterWrapper) Write(p []byte) (n int, err error) {
	return 0, nil
}

func TestCustomWriterBufferConfiguration(t *testing.T) {
	client, err := NewWithWriter(statsdWriterWrapper{})
	require.Nil(t, err)
	defer client.Close()

	assert.Equal(t, OptimalUDPPayloadSize, client.sender.pool.bufferMaxSize)
	assert.Equal(t, DefaultUDPBufferPoolSize, cap(client.sender.pool.pool))
	assert.Equal(t, DefaultUDPBufferPoolSize, cap(client.sender.queue))
}

// TestConcurrentSend sends various metric types in separate goroutines to
// trigger any possible data races. It is intended to be run with the data race
// detector enabled.
func TestConcurrentSend(t *testing.T) {
	tests := []struct {
		description   string
		clientOptions []Option
	}{
		{
			description:   "Client with default options",
			clientOptions: []Option{},
		},
		{
			description:   "Client with mutex mode enabled",
			clientOptions: []Option{WithMutexMode()},
		},
		{
			description:   "Client with channel mode enabled",
			clientOptions: []Option{WithChannelMode()},
		},
	}

	for _, test := range tests {
		test := test // Capture range variable.
		t.Run(test.description, func(t *testing.T) {
			t.Parallel()

			client, err := New("localhost:9876", test.clientOptions...)
			require.Nil(t, err, fmt.Sprintf("failed to create client: %s", err))

			var wg sync.WaitGroup
			wg.Add(1)
			go func() {
				client.Gauge("name", 1, []string{"tag"}, 0.1)
				wg.Done()
			}()

			wg.Add(1)
			go func() {
				client.Count("name", 1, []string{"tag"}, 0.1)
				wg.Done()
			}()

			wg.Add(1)
			go func() {
				client.Timing("name", 1, []string{"tag"}, 0.1)
				wg.Done()
			}()

			wg.Wait()
			err = client.Close()
			require.Nil(t, err, fmt.Sprintf("failed to close client: %s", err))
		})
	}
}

func getTestServer(t *testing.T, addr string) *net.UDPConn {
	udpAddr, err := net.ResolveUDPAddr("udp", addr)
	require.Nil(t, err, fmt.Sprintf("could not resolve udp '%s': %s", addr, err))

	server, err := net.ListenUDP("udp", udpAddr)
	require.Nil(t, err, fmt.Sprintf("Could not listen to UDP addr: %s", err))
	return server
}

func TestCloneWithExtraOptions(t *testing.T) {
	client, err := New(defaultAddr, WithTags([]string{"tag1", "tag2"}))
	require.Nil(t, err, fmt.Sprintf("failed to create client: %s", err))

	assert.Equal(t, client.Tags, []string{"tag1", "tag2"})
	assert.Equal(t, client.Namespace, "")
	assert.Equal(t, client.workersMode, MutexMode)
	assert.Equal(t, client.addrOption, defaultAddr)
	assert.Len(t, client.options, 1)

	cloneClient, err := CloneWithExtraOptions(client, WithNamespace("test"), WithChannelMode())
	require.Nil(t, err, fmt.Sprintf("failed to clone client: %s", err))

	assert.Equal(t, cloneClient.Tags, []string{"tag1", "tag2"})
	assert.Equal(t, cloneClient.Namespace, "test.")
	assert.Equal(t, cloneClient.workersMode, ChannelMode)
	assert.Equal(t, cloneClient.addrOption, defaultAddr)
	assert.Len(t, cloneClient.options, 3)
}

func sendOneMetrics(client *Client) string {
	client.Count("name", 1, []string{"tag"}, 1)
	return "name:1|c|#tag\n"
}

func sendBasicMetrics(client *Client) string {
	client.Gauge("gauge", 1, []string{"tag"}, 1)
	client.Gauge("gauge", 21, []string{"tag"}, 1)
	client.Count("count", 1, []string{"tag"}, 1)
	client.Count("count", 3, []string{"tag"}, 1)
	client.Set("set", "my_id", []string{"tag"}, 1)
	client.Set("set", "my_id", []string{"tag"}, 1)

	return "set:my_id|s|#tag\ngauge:21|g|#tag\ncount:4|c|#tag\n"
}

func sendAllMetrics(client *Client) string {
	client.Gauge("gauge", 1, []string{"tag"}, 1)
	client.Count("count", 2, []string{"tag"}, 1)
	client.Set("set", "3_id", []string{"tag"}, 1)
	client.Histogram("histo", 4, []string{"tag"}, 1)
	client.Distribution("distro", 5, []string{"tag"}, 1)
	client.Timing("timing", 6*time.Second, []string{"tag"}, 1)

	return "gauge:1|g|#tag\ncount:2|c|#tag\nset:3_id|s|#tag\nhisto:4|h|#tag\ndistro:5|d|#tag\ntiming:6000.000000|ms|#tag\n"
}

func sendAllMetricsWithBasicAggregation(client *Client) string {
	client.Gauge("gauge", 1, []string{"tag"}, 1)
	client.Gauge("gauge", 21, []string{"tag"}, 1)
	client.Count("count", 1, []string{"tag"}, 1)
	client.Count("count", 3, []string{"tag"}, 1)
	client.Set("set", "my_id", []string{"tag"}, 1)
	client.Set("set", "my_id", []string{"tag"}, 1)
	client.Histogram("histo", 3, []string{"tag"}, 1)
	client.Histogram("histo", 31, []string{"tag"}, 1)
	client.Distribution("distro", 3, []string{"tag"}, 1)
	client.Distribution("distro", 22, []string{"tag"}, 1)
	client.Timing("timing", 3*time.Second, []string{"tag"}, 1)
	client.Timing("timing", 12*time.Second, []string{"tag"}, 1)

	return "histo:3|h|#tag\nhisto:31|h|#tag\ndistro:3|d|#tag\ndistro:22|d|#tag\ntiming:3000.000000|ms|#tag\ntiming:12000.000000|ms|#tag\nset:my_id|s|#tag\ngauge:21|g|#tag\ncount:4|c|#tag\n"
}

func sendExtendedMetricsWithExtentedAggregation(client *Client) string {
	client.Gauge("gauge", 1, []string{"tag"}, 1)
	client.Gauge("gauge", 21, []string{"tag"}, 1)
	client.Count("count", 1, []string{"tag"}, 1)
	client.Count("count", 3, []string{"tag"}, 1)
	client.Set("set", "my_id", []string{"tag"}, 1)
	client.Set("set", "my_id", []string{"tag"}, 1)
	client.Histogram("histo", 3, []string{"tag"}, 1)
	client.Histogram("histo", 31, []string{"tag"}, 1)
	client.Distribution("distro", 3, []string{"tag"}, 1)
	client.Distribution("distro", 22, []string{"tag"}, 1)
	client.Timing("timing", 3*time.Second, []string{"tag"}, 1)
	client.Timing("timing", 12*time.Second, []string{"tag"}, 1)

	return "set:my_id|s|#tag\ngauge:21|g|#tag\ncount:4|c|#tag\nhisto:3:31|h|#tag\ndistro:3:22|d|#tag\ntiming:3000.000000:12000.000000|ms|#tag\n"
}

func testStatsdPipeline(t *testing.T, client *Client, genMetric func(*Client) string, flush func(*Client)) {
	server := getTestServer(t, defaultAddr)
	defer server.Close()

	readDone := make(chan struct{})
	buffer := make([]byte, 4096)
	n := 0
	go func() {
		n, _ = io.ReadAtLeast(server, buffer, 1)
		close(readDone)
	}()

	expectedResults := genMetric(client)

	flush(client)

	select {
	case <-readDone:
	case <-time.After(2 * time.Second):
		require.Fail(t, "No data was flush on Close")
	}

	assert.Equal(t, expectedResults, string(buffer[:n]))
}

func TestGroupClient(t *testing.T) {
	type testCase struct {
		opt       []Option
		genMetric func(*Client) string
		flushFunc func(*Client)
	}

	testMap := map[string]testCase{
		"MutexMode": testCase{
			[]Option{WithBufferShardCount(1)},
			sendAllMetrics,
			func(*Client) {},
		},
		"ChannelMode": testCase{
			[]Option{WithChannelMode(), WithBufferShardCount(1)},
			sendAllMetrics,
			func(*Client) {},
		},
		"DevMode": testCase{
			[]Option{WithDevMode()},
			sendOneMetrics,
			func(*Client) {},
		},
		"BasicAggregation + Close": testCase{
			[]Option{WithClientSideAggregation(), WithBufferShardCount(1)},
			sendBasicMetrics,
			func(c *Client) { c.Close() },
		},
		"BasicAggregation all metric + Close": testCase{
			[]Option{WithClientSideAggregation(), WithBufferShardCount(1)},
			sendAllMetricsWithBasicAggregation,
			func(c *Client) { c.Close() },
		},
		"BasicAggregation + Flush": testCase{
			[]Option{WithClientSideAggregation(), WithBufferShardCount(1)},
			sendBasicMetrics,
			func(c *Client) { c.Flush() },
		},
		"BasicAggregationChannelMode + Close": testCase{
			[]Option{WithClientSideAggregation(), WithBufferShardCount(1), WithChannelMode()},
			sendBasicMetrics,
			func(c *Client) { c.Close() },
		},
		"BasicAggregationChannelMode + Flush": testCase{
			[]Option{WithClientSideAggregation(), WithBufferShardCount(1), WithChannelMode()},
			sendBasicMetrics,
			func(c *Client) { c.Flush() },
		},
		"ExtendedAggregation + Close": testCase{
			[]Option{WithExtendedClientSideAggregation(), WithBufferShardCount(1)},
			sendExtendedMetricsWithExtentedAggregation,
			func(c *Client) { c.Close() },
		},
		"ExtendedAggregation + Close + ChannelMode": testCase{
			[]Option{WithExtendedClientSideAggregation(), WithBufferShardCount(1), WithChannelMode()},
			sendExtendedMetricsWithExtentedAggregation,
			func(c *Client) {
				// since we're using ChannelMode we give a second to the worker to
				// empty the channel. A second should be more than enough to pull 6
				// items from a channel.
				time.Sleep(1 * time.Second)
				c.Close()
			},
		},
	}

	for testName, c := range testMap {
		t.Run(testName, func(t *testing.T) {
			client, err := New(defaultAddr, c.opt...)
			require.Nil(t, err, fmt.Sprintf("failed to create client: %s", err))
			testStatsdPipeline(t, client, c.genMetric, c.flushFunc)
		})
	}
}

func TestResolveAddressFromEnvironment(t *testing.T) {
	hostInitialValue, hostInitiallySet := os.LookupEnv(agentHostEnvVarName)
	if hostInitiallySet {
		defer os.Setenv(agentHostEnvVarName, hostInitialValue)
	} else {
		defer os.Unsetenv(agentHostEnvVarName)
	}
	portInitialValue, portInitiallySet := os.LookupEnv(agentPortEnvVarName)
	if portInitiallySet {
		defer os.Setenv(agentPortEnvVarName, portInitialValue)
	} else {
		defer os.Unsetenv(agentPortEnvVarName)
	}

	for _, tc := range []struct {
		name         string
		addrParam    string
		hostEnv      string
		portEnv      string
		expectedAddr string
	}{
		{"UPD Nominal case", "127.0.0.1:1234", "", "", "127.0.0.1:1234"},
		{"UPD Parameter overrides environment", "127.0.0.1:8125", "10.12.16.9", "1234", "127.0.0.1:8125"},
		{"UPD Host and port passed as env", "", "10.12.16.9", "1234", "10.12.16.9:1234"},
		{"UPD Host env, default port", "", "10.12.16.9", "", "10.12.16.9:8125"},
		{"UPD Host passed, ignore env port", "10.12.16.9", "", "1234", "10.12.16.9:8125"},

		{"UDS socket passed", "unix://test/path.socket", "", "", "unix://test/path.socket"},
		{"UDS socket env", "", "unix://test/path.socket", "", "unix://test/path.socket"},
		{"UDS socket env with port", "", "unix://test/path.socket", "8125", "unix://test/path.socket"},

		{"Pipe passed", "\\\\.\\pipe\\my_pipe", "", "", "\\\\.\\pipe\\my_pipe"},
		{"Pipe env", "", "\\\\.\\pipe\\my_pipe", "", "\\\\.\\pipe\\my_pipe"},
		{"Pipe env with port", "", "\\\\.\\pipe\\my_pipe", "8125", "\\\\.\\pipe\\my_pipe"},

		{"No autodetection failed", "", "", "", ""},
	} {
		t.Run(tc.name, func(t *testing.T) {
			os.Setenv(agentHostEnvVarName, tc.hostEnv)
			os.Setenv(agentPortEnvVarName, tc.portEnv)

			addr := resolveAddr(tc.addrParam)
			assert.Equal(t, tc.expectedAddr, addr)
		})
	}
}

func TestEnvTags(t *testing.T) {
	entityIDEnvName := "DD_ENTITY_ID"
	ddEnvName := "DD_ENV"
	ddServiceName := "DD_SERVICE"
	ddVersionName := "DD_VERSION"

	defer func() { os.Unsetenv(entityIDEnvName) }()
	defer func() { os.Unsetenv(ddEnvName) }()
	defer func() { os.Unsetenv(ddServiceName) }()
	defer func() { os.Unsetenv(ddVersionName) }()

	os.Setenv(entityIDEnvName, "test_id")
	os.Setenv(ddEnvName, "test_env")
	os.Setenv(ddServiceName, "test_service")
	os.Setenv(ddVersionName, "test_version")

	expectedTags := []string{"dd.internal.entity_id:test_id", "env:test_env", "service:test_service", "version:test_version"}
	ts, client := newClientAndTestServer(t,
		"udp",
		"localhost:8765",
		expectedTags,
	)

	sort.Strings(client.Tags)
	assert.Equal(t, client.Tags, expectedTags)
	ts.sendAllAndAssert(t, client)
}

func TestEnvTagsWithCustomTags(t *testing.T) {
	entityIDEnvName := "DD_ENTITY_ID"
	ddEnvName := "DD_ENV"
	ddServiceName := "DD_SERVICE"
	ddVersionName := "DD_VERSION"

	defer func() { os.Unsetenv(entityIDEnvName) }()
	defer func() { os.Unsetenv(ddEnvName) }()
	defer func() { os.Unsetenv(ddServiceName) }()
	defer func() { os.Unsetenv(ddVersionName) }()

	os.Setenv(entityIDEnvName, "test_id")
	os.Setenv(ddEnvName, "test_env")
	os.Setenv(ddServiceName, "test_service")
	os.Setenv(ddVersionName, "test_version")

	expectedTags := []string{"tag1", "tag2", "dd.internal.entity_id:test_id", "env:test_env", "service:test_service", "version:test_version"}
	ts, client := newClientAndTestServer(t,
		"udp",
		"localhost:8765",
		expectedTags,
		WithTags([]string{"tag1", "tag2"}),
	)

	ts.sendAllAndAssert(t, client)

	sort.Strings(expectedTags)
	sort.Strings(client.Tags)
	assert.Equal(t, client.Tags, expectedTags)
}

func TestEnvTagsEmptyString(t *testing.T) {
	entityIDEnvName := "DD_ENTITY_ID"
	ddEnvName := "DD_ENV"
	ddServiceName := "DD_SERVICE"
	ddVersionName := "DD_VERSION"

	defer func() { os.Unsetenv(entityIDEnvName) }()
	defer func() { os.Unsetenv(ddEnvName) }()
	defer func() { os.Unsetenv(ddServiceName) }()
	defer func() { os.Unsetenv(ddVersionName) }()

	os.Setenv(entityIDEnvName, "")
	os.Setenv(ddEnvName, "")
	os.Setenv(ddServiceName, "")
	os.Setenv(ddVersionName, "")

	ts, client := newClientAndTestServer(t,
		"udp",
		"localhost:8765",
		nil,
	)

	assert.Len(t, client.Tags, 0)
	ts.sendAllAndAssert(t, client)
}
