package statsd

import (
	"math"
	"sort"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewCountMetric(t *testing.T) {
	c := newCountMetric("test", 21, []string{"tag1", "tag2"})
	assert.Equal(t, c.value, int64(21))
	assert.Equal(t, c.name, "test")
	assert.Equal(t, c.tags, []string{"tag1", "tag2"})
}

func TestCountMetricSample(t *testing.T) {
	c := newCountMetric("test", 21, []string{"tag1", "tag2"})
	c.sample(12)
	assert.Equal(t, c.value, int64(33))
	assert.Equal(t, c.name, "test")
	assert.Equal(t, c.tags, []string{"tag1", "tag2"})
}

func TestFlushUnsafeCountMetricSample(t *testing.T) {
	c := newCountMetric("test", 21, []string{"tag1", "tag2"})
	m := c.flushUnsafe()
	assert.Equal(t, m.metricType, count)
	assert.Equal(t, m.ivalue, int64(21))
	assert.Equal(t, m.name, "test")
	assert.Equal(t, m.tags, []string{"tag1", "tag2"})

	c.sample(12)
	m = c.flushUnsafe()
	assert.Equal(t, m.metricType, count)
	assert.Equal(t, m.ivalue, int64(33))
	assert.Equal(t, m.name, "test")
	assert.Equal(t, m.tags, []string{"tag1", "tag2"})
}

func TestNewGaugeMetric(t *testing.T) {
	g := newGaugeMetric("test", 21, []string{"tag1", "tag2"})
	assert.Equal(t, math.Float64frombits(g.value), float64(21))
	assert.Equal(t, g.name, "test")
	assert.Equal(t, g.tags, []string{"tag1", "tag2"})
}

func TestGaugeMetricSample(t *testing.T) {
	g := newGaugeMetric("test", 21, []string{"tag1", "tag2"})
	g.sample(12)
	assert.Equal(t, math.Float64frombits(g.value), float64(12))
	assert.Equal(t, g.name, "test")
	assert.Equal(t, g.tags, []string{"tag1", "tag2"})
}

func TestFlushUnsafeGaugeMetricSample(t *testing.T) {
	g := newGaugeMetric("test", 21, []string{"tag1", "tag2"})
	m := g.flushUnsafe()
	assert.Equal(t, m.metricType, gauge)
	assert.Equal(t, m.fvalue, float64(21))
	assert.Equal(t, m.name, "test")
	assert.Equal(t, m.tags, []string{"tag1", "tag2"})

	g.sample(12)
	m = g.flushUnsafe()
	assert.Equal(t, m.metricType, gauge)
	assert.Equal(t, m.fvalue, float64(12))
	assert.Equal(t, m.name, "test")
	assert.Equal(t, m.tags, []string{"tag1", "tag2"})
}

func TestNewSetMetric(t *testing.T) {
	s := newSetMetric("test", "value1", []string{"tag1", "tag2"})
	assert.Equal(t, s.data, map[string]struct{}{"value1": struct{}{}})
	assert.Equal(t, s.name, "test")
	assert.Equal(t, s.tags, []string{"tag1", "tag2"})
}

func TestSetMetricSample(t *testing.T) {
	s := newSetMetric("test", "value1", []string{"tag1", "tag2"})
	s.sample("value2")
	assert.Equal(t, s.data, map[string]struct{}{"value1": struct{}{}, "value2": struct{}{}})
	assert.Equal(t, s.name, "test")
	assert.Equal(t, s.tags, []string{"tag1", "tag2"})
}

func TestFlushUnsafeSetMetricSample(t *testing.T) {
	s := newSetMetric("test", "value1", []string{"tag1", "tag2"})
	m := s.flushUnsafe()

	require.Len(t, m, 1)

	assert.Equal(t, m[0].metricType, set)
	assert.Equal(t, m[0].svalue, "value1")
	assert.Equal(t, m[0].name, "test")
	assert.Equal(t, m[0].tags, []string{"tag1", "tag2"})

	s.sample("value1")
	s.sample("value2")
	m = s.flushUnsafe()

	sort.Slice(m, func(i, j int) bool {
		return strings.Compare(m[i].svalue, m[j].svalue) != 1
	})

	require.Len(t, m, 2)
	assert.Equal(t, m[0].metricType, set)
	assert.Equal(t, m[0].svalue, "value1")
	assert.Equal(t, m[0].name, "test")
	assert.Equal(t, m[0].tags, []string{"tag1", "tag2"})
	assert.Equal(t, m[1].metricType, set)
	assert.Equal(t, m[1].svalue, "value2")
	assert.Equal(t, m[1].name, "test")
	assert.Equal(t, m[1].tags, []string{"tag1", "tag2"})
}

func TestNewHistogramMetric(t *testing.T) {
	s := newHistogramMetric("test", 1.0, "tag1,tag2")
	assert.Equal(t, s.data, []float64{1.0})
	assert.Equal(t, s.name, "test")
	assert.Equal(t, s.tags, "tag1,tag2")
	assert.Equal(t, s.mtype, histogramAggregated)
}

func TestHistogramMetricSample(t *testing.T) {
	s := newHistogramMetric("test", 1.0, "tag1,tag2")
	s.sample(123.45)
	assert.Equal(t, s.data, []float64{1.0, 123.45})
	assert.Equal(t, s.name, "test")
	assert.Equal(t, s.tags, "tag1,tag2")
	assert.Equal(t, s.mtype, histogramAggregated)
}

func TestFlushUnsafeHistogramMetricSample(t *testing.T) {
	s := newHistogramMetric("test", 1.0, "tag1,tag2")
	m := s.flushUnsafe()

	assert.Equal(t, m.metricType, histogramAggregated)
	assert.Equal(t, m.fvalues, []float64{1.0})
	assert.Equal(t, m.name, "test")
	assert.Equal(t, m.stags, "tag1,tag2")
	assert.Nil(t, m.tags)

	s.sample(21)
	s.sample(123.45)
	m = s.flushUnsafe()

	assert.Equal(t, m.metricType, histogramAggregated)
	assert.Equal(t, m.fvalues, []float64{1.0, 21.0, 123.45})
	assert.Equal(t, m.name, "test")
	assert.Equal(t, m.stags, "tag1,tag2")
	assert.Nil(t, m.tags)
}

func TestNewDistributionMetric(t *testing.T) {
	s := newDistributionMetric("test", 1.0, "tag1,tag2")
	assert.Equal(t, s.data, []float64{1.0})
	assert.Equal(t, s.name, "test")
	assert.Equal(t, s.tags, "tag1,tag2")
	assert.Equal(t, s.mtype, distributionAggregated)
}

func TestDistributionMetricSample(t *testing.T) {
	s := newDistributionMetric("test", 1.0, "tag1,tag2")
	s.sample(123.45)
	assert.Equal(t, s.data, []float64{1.0, 123.45})
	assert.Equal(t, s.name, "test")
	assert.Equal(t, s.tags, "tag1,tag2")
	assert.Equal(t, s.mtype, distributionAggregated)
}

func TestFlushUnsafeDistributionMetricSample(t *testing.T) {
	s := newDistributionMetric("test", 1.0, "tag1,tag2")
	m := s.flushUnsafe()

	assert.Equal(t, m.metricType, distributionAggregated)
	assert.Equal(t, m.fvalues, []float64{1.0})
	assert.Equal(t, m.name, "test")
	assert.Equal(t, m.stags, "tag1,tag2")
	assert.Nil(t, m.tags)

	s.sample(21)
	s.sample(123.45)
	m = s.flushUnsafe()

	assert.Equal(t, m.metricType, distributionAggregated)
	assert.Equal(t, m.fvalues, []float64{1.0, 21.0, 123.45})
	assert.Equal(t, m.name, "test")
	assert.Equal(t, m.stags, "tag1,tag2")
	assert.Nil(t, m.tags)
}

func TestNewTimingMetric(t *testing.T) {
	s := newTimingMetric("test", 1.0, "tag1,tag2")
	assert.Equal(t, s.data, []float64{1.0})
	assert.Equal(t, s.name, "test")
	assert.Equal(t, s.tags, "tag1,tag2")
	assert.Equal(t, s.mtype, timingAggregated)
}

func TestTimingMetricSample(t *testing.T) {
	s := newTimingMetric("test", 1.0, "tag1,tag2")
	s.sample(123.45)
	assert.Equal(t, s.data, []float64{1.0, 123.45})
	assert.Equal(t, s.name, "test")
	assert.Equal(t, s.tags, "tag1,tag2")
	assert.Equal(t, s.mtype, timingAggregated)
}

func TestFlushUnsafeTimingMetricSample(t *testing.T) {
	s := newTimingMetric("test", 1.0, "tag1,tag2")
	m := s.flushUnsafe()

	assert.Equal(t, m.metricType, timingAggregated)
	assert.Equal(t, m.fvalues, []float64{1.0})
	assert.Equal(t, m.name, "test")
	assert.Equal(t, m.stags, "tag1,tag2")
	assert.Nil(t, m.tags)

	s.sample(21)
	s.sample(123.45)
	m = s.flushUnsafe()

	assert.Equal(t, m.metricType, timingAggregated)
	assert.Equal(t, m.fvalues, []float64{1.0, 21.0, 123.45})
	assert.Equal(t, m.name, "test")
	assert.Equal(t, m.stags, "tag1,tag2")
	assert.Nil(t, m.tags)
}
