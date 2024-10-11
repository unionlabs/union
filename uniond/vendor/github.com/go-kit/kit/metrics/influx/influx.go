// Package influx provides an InfluxDB implementation for metrics. The model is
// similar to other push-based instrumentation systems. Observations are
// aggregated locally and emitted to the Influx server on regular intervals.
package influx

import (
	"context"
	"time"

	influxdb "github.com/influxdata/influxdb1-client/v2"

	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/generic"
	"github.com/go-kit/kit/metrics/internal/lv"
	"github.com/go-kit/log"
)

// Influx is a store for metrics that will be emitted to an Influx database.
//
// Influx is a general purpose time-series database, and has no native concepts
// of counters, gauges, or histograms. Counters are modeled as a timeseries with
// one data point per flush, with a "count" field that reflects all adds since
// the last flush. Gauges are modeled as a timeseries with one data point per
// flush, with a "value" field that reflects the current state of the gauge.
// Histograms are modeled as a timeseries with one data point per combination of tags,
// with a set of quantile fields that reflects the p50, p90, p95 & p99.
//
// Influx tags are attached to the Influx object, can be given to each
// metric at construction and can be updated anytime via With function. Influx fields
// are mapped to Go kit label values directly by this collector. Actual metric
// values are provided as fields with specific names depending on the metric.
//
// All observations are collected in memory locally, and flushed on demand.
type Influx struct {
	counters   *lv.Space
	gauges     *lv.Space
	histograms *lv.Space
	tags       map[string]string
	conf       influxdb.BatchPointsConfig
	logger     log.Logger
}

// New returns an Influx, ready to create metrics and collect observations. Tags
// are applied to all metrics created from this object. The BatchPointsConfig is
// used during flushing.
func New(tags map[string]string, conf influxdb.BatchPointsConfig, logger log.Logger) *Influx {
	return &Influx{
		counters:   lv.NewSpace(),
		gauges:     lv.NewSpace(),
		histograms: lv.NewSpace(),
		tags:       tags,
		conf:       conf,
		logger:     logger,
	}
}

// NewCounter returns an Influx counter.
func (in *Influx) NewCounter(name string) *Counter {
	return &Counter{
		name: name,
		obs:  in.counters.Observe,
	}
}

// NewGauge returns an Influx gauge.
func (in *Influx) NewGauge(name string) *Gauge {
	return &Gauge{
		name: name,
		obs:  in.gauges.Observe,
		add:  in.gauges.Add,
	}
}

// NewHistogram returns an Influx histogram.
func (in *Influx) NewHistogram(name string) *Histogram {
	return &Histogram{
		name: name,
		obs:  in.histograms.Observe,
	}
}

// BatchPointsWriter captures a subset of the influxdb.Client methods necessary
// for emitting metrics observations.
type BatchPointsWriter interface {
	Write(influxdb.BatchPoints) error
}

// WriteLoop is a helper method that invokes WriteTo to the passed writer every
// time the passed channel fires. This method blocks until the channel is
// closed, so clients probably want to run it in its own goroutine. For typical
// usage, create a time.Ticker and pass its C channel to this method.
func (in *Influx) WriteLoop(ctx context.Context, c <-chan time.Time, w BatchPointsWriter) {
	for {
		select {
		case <-c:
			if err := in.WriteTo(w); err != nil {
				in.logger.Log("during", "WriteTo", "err", err)
			}
		case <-ctx.Done():
			return
		}
	}
}

// WriteTo flushes the buffered content of the metrics to the writer, in an
// Influx BatchPoints format. WriteTo abides best-effort semantics, so
// observations are lost if there is a problem with the write. Clients should be
// sure to call WriteTo regularly, ideally through the WriteLoop helper method.
func (in *Influx) WriteTo(w BatchPointsWriter) (err error) {
	bp, err := influxdb.NewBatchPoints(in.conf)
	if err != nil {
		return err
	}

	now := time.Now()

	in.counters.Reset().Walk(func(name string, lvs lv.LabelValues, values []float64) bool {
		tags := mergeTags(in.tags, lvs)
		var p *influxdb.Point
		fields := map[string]interface{}{"count": sum(values)}
		p, err = influxdb.NewPoint(name, tags, fields, now)
		if err != nil {
			return false
		}
		bp.AddPoint(p)
		return true
	})
	if err != nil {
		return err
	}

	in.gauges.Reset().Walk(func(name string, lvs lv.LabelValues, values []float64) bool {
		tags := mergeTags(in.tags, lvs)
		var p *influxdb.Point
		fields := map[string]interface{}{"value": last(values)}
		p, err = influxdb.NewPoint(name, tags, fields, now)
		if err != nil {
			return false
		}
		bp.AddPoint(p)
		return true
	})
	if err != nil {
		return err
	}

	in.histograms.Reset().Walk(func(name string, lvs lv.LabelValues, values []float64) bool {
		histogram := generic.NewHistogram(name, 50)
		tags := mergeTags(in.tags, lvs)
		var p *influxdb.Point
		for _, v := range values {
			histogram.Observe(v)
		}
		fields := map[string]interface{}{
			"p50": histogram.Quantile(0.50),
			"p90": histogram.Quantile(0.90),
			"p95": histogram.Quantile(0.95),
			"p99": histogram.Quantile(0.99),
		}
		p, err = influxdb.NewPoint(name, tags, fields, now)
		if err != nil {
			return false
		}
		bp.AddPoint(p)
		return true
	})
	if err != nil {
		return err
	}

	return w.Write(bp)
}

func mergeTags(tags map[string]string, labelValues []string) map[string]string {
	if len(labelValues)%2 != 0 {
		panic("mergeTags received a labelValues with an odd number of strings")
	}
	ret := make(map[string]string, len(tags)+len(labelValues)/2)
	for k, v := range tags {
		ret[k] = v
	}
	for i := 0; i < len(labelValues); i += 2 {
		ret[labelValues[i]] = labelValues[i+1]
	}
	return ret
}

func sum(a []float64) float64 {
	var v float64
	for _, f := range a {
		v += f
	}
	return v
}

func last(a []float64) float64 {
	return a[len(a)-1]
}

type observeFunc func(name string, lvs lv.LabelValues, value float64)

// Counter is an Influx counter. Observations are forwarded to an Influx
// object, and aggregated (summed) per timeseries.
type Counter struct {
	name string
	lvs  lv.LabelValues
	obs  observeFunc
}

// With implements metrics.Counter.
func (c *Counter) With(labelValues ...string) metrics.Counter {
	return &Counter{
		name: c.name,
		lvs:  c.lvs.With(labelValues...),
		obs:  c.obs,
	}
}

// Add implements metrics.Counter.
func (c *Counter) Add(delta float64) {
	c.obs(c.name, c.lvs, delta)
}

// Gauge is an Influx gauge. Observations are forwarded to a Dogstatsd
// object, and aggregated (the last observation selected) per timeseries.
type Gauge struct {
	name string
	lvs  lv.LabelValues
	obs  observeFunc
	add  observeFunc
}

// With implements metrics.Gauge.
func (g *Gauge) With(labelValues ...string) metrics.Gauge {
	return &Gauge{
		name: g.name,
		lvs:  g.lvs.With(labelValues...),
		obs:  g.obs,
		add:  g.add,
	}
}

// Set implements metrics.Gauge.
func (g *Gauge) Set(value float64) {
	g.obs(g.name, g.lvs, value)
}

// Add implements metrics.Gauge.
func (g *Gauge) Add(delta float64) {
	g.add(g.name, g.lvs, delta)
}

// Histogram is an Influx histrogram. Observations are aggregated into a
// generic.Histogram and emitted as per-quantile gauges to the Influx server.
type Histogram struct {
	name string
	lvs  lv.LabelValues
	obs  observeFunc
}

// With implements metrics.Histogram.
func (h *Histogram) With(labelValues ...string) metrics.Histogram {
	return &Histogram{
		name: h.name,
		lvs:  h.lvs.With(labelValues...),
		obs:  h.obs,
	}
}

// Observe implements metrics.Histogram.
func (h *Histogram) Observe(value float64) {
	h.obs(h.name, h.lvs, value)
}
