// Package influxstatsd provides support for InfluxData's StatsD Telegraf plugin. It's very
// similar to StatsD, but supports arbitrary tags per-metric, which map to Go
// kit's label values. So, while label values are no-ops in StatsD, they are
// supported here. For more details, see the article at
// https://www.influxdata.com/blog/getting-started-with-sending-statsd-metrics-to-telegraf-influxdb/
//
// This package batches observations and emits them on some schedule to the
// remote server. This is useful even if you connect to your service
// over UDP. Emitting one network packet per observation can quickly overwhelm
// even the fastest internal network.
package influxstatsd

import (
	"context"
	"fmt"
	"io"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/generic"
	"github.com/go-kit/kit/metrics/internal/lv"
	"github.com/go-kit/kit/metrics/internal/ratemap"
	"github.com/go-kit/kit/util/conn"
	"github.com/go-kit/log"
)

// Influxstatsd receives metrics observations and forwards them to a server.
// Create a Influxstatsd object, use it to create metrics, and pass those
// metrics as dependencies to the components that will use them.
//
// All metrics are buffered until WriteTo is called. Counters and gauges are
// aggregated into a single observation per timeseries per write. Timings and
// histograms are buffered but not aggregated.
//
// To regularly report metrics to an io.Writer, use the WriteLoop helper method.
// To send to a InfluxStatsD server, use the SendLoop helper method.
type Influxstatsd struct {
	mtx        sync.RWMutex
	prefix     string
	rates      *ratemap.RateMap
	counters   *lv.Space
	gauges     map[string]*gaugeNode
	timings    *lv.Space
	histograms *lv.Space
	logger     log.Logger
	lvs        lv.LabelValues
}

// New returns a Influxstatsd object that may be used to create metrics. Prefix is
// applied to all created metrics. Callers must ensure that regular calls to
// WriteTo are performed, either manually or with one of the helper methods.
func New(prefix string, logger log.Logger, lvs ...string) *Influxstatsd {
	if len(lvs)%2 != 0 {
		panic("odd number of LabelValues; programmer error!")
	}
	return &Influxstatsd{
		prefix:     prefix,
		rates:      ratemap.New(),
		counters:   lv.NewSpace(),
		gauges:     map[string]*gaugeNode{}, // https://github.com/go-kit/kit/pull/588
		timings:    lv.NewSpace(),
		histograms: lv.NewSpace(),
		logger:     logger,
		lvs:        lvs,
	}
}

// NewCounter returns a counter, sending observations to this Influxstatsd object.
func (d *Influxstatsd) NewCounter(name string, sampleRate float64) *Counter {
	d.rates.Set(name, sampleRate)
	return &Counter{
		name: name,
		obs:  d.counters.Observe,
	}
}

// NewGauge returns a gauge, sending observations to this Influxstatsd object.
func (d *Influxstatsd) NewGauge(name string) *Gauge {
	d.mtx.Lock()
	n, ok := d.gauges[name]
	if !ok {
		n = &gaugeNode{gauge: &Gauge{g: generic.NewGauge(name), influx: d}}
		d.gauges[name] = n
	}
	d.mtx.Unlock()
	return n.gauge
}

// NewTiming returns a histogram whose observations are interpreted as
// millisecond durations, and are forwarded to this Influxstatsd object.
func (d *Influxstatsd) NewTiming(name string, sampleRate float64) *Timing {
	d.rates.Set(name, sampleRate)
	return &Timing{
		name: name,
		obs:  d.timings.Observe,
	}
}

// NewHistogram returns a histogram whose observations are of an unspecified
// unit, and are forwarded to this Influxstatsd object.
func (d *Influxstatsd) NewHistogram(name string, sampleRate float64) *Histogram {
	d.rates.Set(name, sampleRate)
	return &Histogram{
		name: name,
		obs:  d.histograms.Observe,
	}
}

// WriteLoop is a helper method that invokes WriteTo to the passed writer every
// time the passed channel fires. This method blocks until ctx is canceled,
// so clients probably want to run it in its own goroutine. For typical
// usage, create a time.Ticker and pass its C channel to this method.
func (d *Influxstatsd) WriteLoop(ctx context.Context, c <-chan time.Time, w io.Writer) {
	for {
		select {
		case <-c:
			if _, err := d.WriteTo(w); err != nil {
				d.logger.Log("during", "WriteTo", "err", err)
			}
		case <-ctx.Done():
			return
		}
	}
}

// SendLoop is a helper method that wraps WriteLoop, passing a managed
// connection to the network and address. Like WriteLoop, this method blocks
// until ctx is canceled, so clients probably want to start it in its own
// goroutine. For typical usage, create a time.Ticker and pass its C channel to
// this method.
func (d *Influxstatsd) SendLoop(ctx context.Context, c <-chan time.Time, network, address string) {
	d.WriteLoop(ctx, c, conn.NewDefaultManager(network, address, d.logger))
}

// WriteTo flushes the buffered content of the metrics to the writer, in
// InfluxStatsD format. WriteTo abides best-effort semantics, so observations are
// lost if there is a problem with the write. Clients should be sure to call
// WriteTo regularly, ideally through the WriteLoop or SendLoop helper methods.
func (d *Influxstatsd) WriteTo(w io.Writer) (count int64, err error) {
	var n int

	d.counters.Reset().Walk(func(name string, lvs lv.LabelValues, values []float64) bool {
		n, err = fmt.Fprintf(w, "%s%s%s:%f|c%s\n", d.prefix, name, d.tagValues(lvs), sum(values), sampling(d.rates.Get(name)))
		if err != nil {
			return false
		}
		count += int64(n)
		return true
	})
	if err != nil {
		return count, err
	}

	d.mtx.RLock()
	for _, root := range d.gauges {
		root.walk(func(name string, lvs lv.LabelValues, value float64) bool {
			n, err = fmt.Fprintf(w, "%s%s%s:%f|g\n", d.prefix, name, d.tagValues(lvs), value)
			if err != nil {
				return false
			}
			count += int64(n)
			return true
		})
	}
	d.mtx.RUnlock()

	d.timings.Reset().Walk(func(name string, lvs lv.LabelValues, values []float64) bool {
		sampleRate := d.rates.Get(name)
		for _, value := range values {
			n, err = fmt.Fprintf(w, "%s%s%s:%f|ms%s\n", d.prefix, name, d.tagValues(lvs), value, sampling(sampleRate))
			if err != nil {
				return false
			}
			count += int64(n)
		}
		return true
	})
	if err != nil {
		return count, err
	}

	d.histograms.Reset().Walk(func(name string, lvs lv.LabelValues, values []float64) bool {
		sampleRate := d.rates.Get(name)
		for _, value := range values {
			n, err = fmt.Fprintf(w, "%s%s%s:%f|h%s\n", d.prefix, name, d.tagValues(lvs), value, sampling(sampleRate))
			if err != nil {
				return false
			}
			count += int64(n)
		}
		return true
	})
	if err != nil {
		return count, err
	}

	return count, err
}

func sum(a []float64) float64 {
	var v float64
	for _, f := range a {
		v += f
	}
	return v
}

func sampling(r float64) string {
	var sv string
	if r < 1.0 {
		sv = fmt.Sprintf("|@%f", r)
	}
	return sv
}

func (d *Influxstatsd) tagValues(labelValues []string) string {
	if len(labelValues) == 0 && len(d.lvs) == 0 {
		return ""
	}
	if len(labelValues)%2 != 0 {
		panic("tagValues received a labelValues with an odd number of strings")
	}
	pairs := make([]string, 0, (len(d.lvs)+len(labelValues))/2)
	for i := 0; i < len(d.lvs); i += 2 {
		pairs = append(pairs, d.lvs[i]+"="+d.lvs[i+1])
	}
	for i := 0; i < len(labelValues); i += 2 {
		pairs = append(pairs, labelValues[i]+"="+labelValues[i+1])
	}
	return "," + strings.Join(pairs, ",")
}

type observeFunc func(name string, lvs lv.LabelValues, value float64)

// Counter is a InfluxStatsD counter. Observations are forwarded to a Influxstatsd
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

// Gauge is a InfluxStatsD gauge. Observations are forwarded to a Influxstatsd
// object, and aggregated (the last observation selected) per timeseries.
type Gauge struct {
	g      *generic.Gauge
	influx *Influxstatsd
	set    int32
}

// With implements metrics.Gauge.
func (g *Gauge) With(labelValues ...string) metrics.Gauge {
	g.influx.mtx.RLock()
	node := g.influx.gauges[g.g.Name]
	g.influx.mtx.RUnlock()

	ga := &Gauge{g: g.g.With(labelValues...).(*generic.Gauge), influx: g.influx}
	return node.addGauge(ga, ga.g.LabelValues())
}

// Set implements metrics.Gauge.
func (g *Gauge) Set(value float64) {
	g.g.Set(value)
	g.touch()
}

// Add implements metrics.Gauge.
func (g *Gauge) Add(delta float64) {
	g.g.Add(delta)
	g.touch()
}

// Timing is a InfluxStatsD timing, or metrics.Histogram. Observations are
// forwarded to a Influxstatsd object, and collected (but not aggregated) per
// timeseries.
type Timing struct {
	name string
	lvs  lv.LabelValues
	obs  observeFunc
}

// With implements metrics.Timing.
func (t *Timing) With(labelValues ...string) metrics.Histogram {
	return &Timing{
		name: t.name,
		lvs:  t.lvs.With(labelValues...),
		obs:  t.obs,
	}
}

// Observe implements metrics.Histogram. Value is interpreted as milliseconds.
func (t *Timing) Observe(value float64) {
	t.obs(t.name, t.lvs, value)
}

// Histogram is a InfluxStatsD histrogram. Observations are forwarded to a
// Influxstatsd object, and collected (but not aggregated) per timeseries.
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

type pair struct{ label, value string }

type gaugeNode struct {
	mtx      sync.RWMutex
	gauge    *Gauge
	children map[pair]*gaugeNode
}

func (n *gaugeNode) addGauge(g *Gauge, lvs lv.LabelValues) *Gauge {
	n.mtx.Lock()
	defer n.mtx.Unlock()
	if len(lvs) == 0 {
		if n.gauge == nil {
			n.gauge = g
		}
		return n.gauge
	}
	if len(lvs) < 2 {
		panic("too few LabelValues; programmer error!")
	}
	head, tail := pair{lvs[0], lvs[1]}, lvs[2:]
	if n.children == nil {
		n.children = map[pair]*gaugeNode{}
	}
	child, ok := n.children[head]
	if !ok {
		child = &gaugeNode{}
		n.children[head] = child
	}
	return child.addGauge(g, tail)
}

func (n *gaugeNode) walk(fn func(string, lv.LabelValues, float64) bool) bool {
	n.mtx.RLock()
	defer n.mtx.RUnlock()
	if n.gauge != nil {
		value, ok := n.gauge.read()
		if ok && !fn(n.gauge.g.Name, n.gauge.g.LabelValues(), value) {
			return false
		}
	}
	for _, child := range n.children {
		if !child.walk(fn) {
			return false
		}
	}
	return true
}

func (g *Gauge) touch() {
	atomic.StoreInt32(&(g.set), 1)
}

func (g *Gauge) read() (float64, bool) {
	set := atomic.SwapInt32(&(g.set), 0)
	return g.g.Value(), set != 0
}
