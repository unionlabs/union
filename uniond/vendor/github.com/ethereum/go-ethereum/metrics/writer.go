package metrics

import (
	"fmt"
	"io"
	"strings"
	"time"

	"golang.org/x/exp/slices"
)

// Write sorts writes each metric in the given registry periodically to the
// given io.Writer.
func Write(r Registry, d time.Duration, w io.Writer) {
	for range time.Tick(d) {
		WriteOnce(r, w)
	}
}

// WriteOnce sorts and writes metrics in the given registry to the given
// io.Writer.
func WriteOnce(r Registry, w io.Writer) {
	var namedMetrics []namedMetric
	r.Each(func(name string, i interface{}) {
		namedMetrics = append(namedMetrics, namedMetric{name, i})
	})
	slices.SortFunc(namedMetrics, namedMetric.cmp)
	for _, namedMetric := range namedMetrics {
		switch metric := namedMetric.m.(type) {
		case Counter:
			fmt.Fprintf(w, "counter %s\n", namedMetric.name)
			fmt.Fprintf(w, "  count:       %9d\n", metric.Snapshot().Count())
		case CounterFloat64:
			fmt.Fprintf(w, "counter %s\n", namedMetric.name)
			fmt.Fprintf(w, "  count:       %f\n", metric.Snapshot().Count())
		case Gauge:
			fmt.Fprintf(w, "gauge %s\n", namedMetric.name)
			fmt.Fprintf(w, "  value:       %9d\n", metric.Snapshot().Value())
		case GaugeFloat64:
			fmt.Fprintf(w, "gauge %s\n", namedMetric.name)
			fmt.Fprintf(w, "  value:       %f\n", metric.Snapshot().Value())
		case GaugeInfo:
			fmt.Fprintf(w, "gauge %s\n", namedMetric.name)
			fmt.Fprintf(w, "  value:       %s\n", metric.Snapshot().Value().String())
		case Healthcheck:
			metric.Check()
			fmt.Fprintf(w, "healthcheck %s\n", namedMetric.name)
			fmt.Fprintf(w, "  error:       %v\n", metric.Error())
		case Histogram:
			h := metric.Snapshot()
			ps := h.Percentiles([]float64{0.5, 0.75, 0.95, 0.99, 0.999})
			fmt.Fprintf(w, "histogram %s\n", namedMetric.name)
			fmt.Fprintf(w, "  count:       %9d\n", h.Count())
			fmt.Fprintf(w, "  min:         %9d\n", h.Min())
			fmt.Fprintf(w, "  max:         %9d\n", h.Max())
			fmt.Fprintf(w, "  mean:        %12.2f\n", h.Mean())
			fmt.Fprintf(w, "  stddev:      %12.2f\n", h.StdDev())
			fmt.Fprintf(w, "  median:      %12.2f\n", ps[0])
			fmt.Fprintf(w, "  75%%:         %12.2f\n", ps[1])
			fmt.Fprintf(w, "  95%%:         %12.2f\n", ps[2])
			fmt.Fprintf(w, "  99%%:         %12.2f\n", ps[3])
			fmt.Fprintf(w, "  99.9%%:       %12.2f\n", ps[4])
		case Meter:
			m := metric.Snapshot()
			fmt.Fprintf(w, "meter %s\n", namedMetric.name)
			fmt.Fprintf(w, "  count:       %9d\n", m.Count())
			fmt.Fprintf(w, "  1-min rate:  %12.2f\n", m.Rate1())
			fmt.Fprintf(w, "  5-min rate:  %12.2f\n", m.Rate5())
			fmt.Fprintf(w, "  15-min rate: %12.2f\n", m.Rate15())
			fmt.Fprintf(w, "  mean rate:   %12.2f\n", m.RateMean())
		case Timer:
			t := metric.Snapshot()
			ps := t.Percentiles([]float64{0.5, 0.75, 0.95, 0.99, 0.999})
			fmt.Fprintf(w, "timer %s\n", namedMetric.name)
			fmt.Fprintf(w, "  count:       %9d\n", t.Count())
			fmt.Fprintf(w, "  min:         %9d\n", t.Min())
			fmt.Fprintf(w, "  max:         %9d\n", t.Max())
			fmt.Fprintf(w, "  mean:        %12.2f\n", t.Mean())
			fmt.Fprintf(w, "  stddev:      %12.2f\n", t.StdDev())
			fmt.Fprintf(w, "  median:      %12.2f\n", ps[0])
			fmt.Fprintf(w, "  75%%:         %12.2f\n", ps[1])
			fmt.Fprintf(w, "  95%%:         %12.2f\n", ps[2])
			fmt.Fprintf(w, "  99%%:         %12.2f\n", ps[3])
			fmt.Fprintf(w, "  99.9%%:       %12.2f\n", ps[4])
			fmt.Fprintf(w, "  1-min rate:  %12.2f\n", t.Rate1())
			fmt.Fprintf(w, "  5-min rate:  %12.2f\n", t.Rate5())
			fmt.Fprintf(w, "  15-min rate: %12.2f\n", t.Rate15())
			fmt.Fprintf(w, "  mean rate:   %12.2f\n", t.RateMean())
		}
	}
}

type namedMetric struct {
	name string
	m    interface{}
}

func (m namedMetric) cmp(other namedMetric) int {
	return strings.Compare(m.name, other.name)
}
