package telemetry

import (
	"encoding/json"
	"strings"
	"testing"
	"time"

	"github.com/hashicorp/go-metrics"
	"github.com/stretchr/testify/require"
)

func TestMetrics_Disabled(t *testing.T) {
	m, err := New(Config{Enabled: false})
	require.Nil(t, m)
	require.Nil(t, err)
}

func TestMetrics_InMem(t *testing.T) {
	m, err := New(Config{
		MetricsSink:    MetricSinkInMem,
		Enabled:        true,
		EnableHostname: false,
		ServiceName:    "test",
	})
	require.NoError(t, err)
	require.NotNil(t, m)

	emitMetrics()

	gr, err := m.Gather(FormatText)
	require.NoError(t, err)
	require.Equal(t, gr.ContentType, "application/json")

	jsonMetrics := make(map[string]interface{})
	require.NoError(t, json.Unmarshal(gr.Metrics, &jsonMetrics))

	counters := jsonMetrics["Counters"].([]interface{})
	require.Equal(t, counters[0].(map[string]interface{})["Count"].(float64), 10.0)
	require.Equal(t, counters[0].(map[string]interface{})["Name"].(string), "test.dummy_counter")
}

func TestMetrics_Prom(t *testing.T) {
	m, err := New(Config{
		MetricsSink:             MetricSinkInMem,
		Enabled:                 true,
		EnableHostname:          false,
		ServiceName:             "test",
		PrometheusRetentionTime: 60,
		EnableHostnameLabel:     false,
	})
	require.NoError(t, err)
	require.NotNil(t, m)
	require.True(t, m.prometheusEnabled)

	emitMetrics()

	gr, err := m.Gather(FormatPrometheus)
	require.NoError(t, err)
	require.Equal(t, gr.ContentType, string(ContentTypeText))

	require.True(t, strings.Contains(string(gr.Metrics), "test_dummy_counter 30"))
}

func emitMetrics() {
	ticker := time.NewTicker(time.Second)
	timeout := time.After(30 * time.Second)

	for {
		select {
		case <-ticker.C:
			metrics.IncrCounter([]string{"dummy_counter"}, 1.0)
		case <-timeout:
			return
		}
	}
}
