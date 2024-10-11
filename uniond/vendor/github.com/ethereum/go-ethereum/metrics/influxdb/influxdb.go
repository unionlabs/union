package influxdb

import (
	"fmt"

	"github.com/ethereum/go-ethereum/metrics"
)

func readMeter(namespace, name string, i interface{}) (string, map[string]interface{}) {
	switch metric := i.(type) {
	case metrics.Counter:
		measurement := fmt.Sprintf("%s%s.count", namespace, name)
		fields := map[string]interface{}{
			"value": metric.Snapshot().Count(),
		}
		return measurement, fields
	case metrics.CounterFloat64:
		measurement := fmt.Sprintf("%s%s.count", namespace, name)
		fields := map[string]interface{}{
			"value": metric.Snapshot().Count(),
		}
		return measurement, fields
	case metrics.Gauge:
		measurement := fmt.Sprintf("%s%s.gauge", namespace, name)
		fields := map[string]interface{}{
			"value": metric.Snapshot().Value(),
		}
		return measurement, fields
	case metrics.GaugeFloat64:
		measurement := fmt.Sprintf("%s%s.gauge", namespace, name)
		fields := map[string]interface{}{
			"value": metric.Snapshot().Value(),
		}
		return measurement, fields
	case metrics.GaugeInfo:
		ms := metric.Snapshot()
		measurement := fmt.Sprintf("%s%s.gauge", namespace, name)
		fields := map[string]interface{}{
			"value": ms.Value().String(),
		}
		return measurement, fields
	case metrics.Histogram:
		ms := metric.Snapshot()
		if ms.Count() <= 0 {
			break
		}
		ps := ms.Percentiles([]float64{0.25, 0.5, 0.75, 0.95, 0.99, 0.999, 0.9999})
		measurement := fmt.Sprintf("%s%s.histogram", namespace, name)
		fields := map[string]interface{}{
			"count":    ms.Count(),
			"max":      ms.Max(),
			"mean":     ms.Mean(),
			"min":      ms.Min(),
			"stddev":   ms.StdDev(),
			"variance": ms.Variance(),
			"p25":      ps[0],
			"p50":      ps[1],
			"p75":      ps[2],
			"p95":      ps[3],
			"p99":      ps[4],
			"p999":     ps[5],
			"p9999":    ps[6],
		}
		return measurement, fields
	case metrics.Meter:
		ms := metric.Snapshot()
		measurement := fmt.Sprintf("%s%s.meter", namespace, name)
		fields := map[string]interface{}{
			"count": ms.Count(),
			"m1":    ms.Rate1(),
			"m5":    ms.Rate5(),
			"m15":   ms.Rate15(),
			"mean":  ms.RateMean(),
		}
		return measurement, fields
	case metrics.Timer:
		ms := metric.Snapshot()
		ps := ms.Percentiles([]float64{0.5, 0.75, 0.95, 0.99, 0.999, 0.9999})

		measurement := fmt.Sprintf("%s%s.timer", namespace, name)
		fields := map[string]interface{}{
			"count":    ms.Count(),
			"max":      ms.Max(),
			"mean":     ms.Mean(),
			"min":      ms.Min(),
			"stddev":   ms.StdDev(),
			"variance": ms.Variance(),
			"p50":      ps[0],
			"p75":      ps[1],
			"p95":      ps[2],
			"p99":      ps[3],
			"p999":     ps[4],
			"p9999":    ps[5],
			"m1":       ms.Rate1(),
			"m5":       ms.Rate5(),
			"m15":      ms.Rate15(),
			"meanrate": ms.RateMean(),
		}
		return measurement, fields
	case metrics.ResettingTimer:
		t := metric.Snapshot()
		if t.Count() == 0 {
			break
		}
		ps := t.Percentiles([]float64{0.50, 0.95, 0.99})
		measurement := fmt.Sprintf("%s%s.span", namespace, name)
		fields := map[string]interface{}{
			"count": t.Count(),
			"max":   t.Max(),
			"mean":  t.Mean(),
			"min":   t.Min(),
			"p50":   int(ps[0]),
			"p95":   int(ps[1]),
			"p99":   int(ps[2]),
		}
		return measurement, fields
	}
	return "", nil
}
