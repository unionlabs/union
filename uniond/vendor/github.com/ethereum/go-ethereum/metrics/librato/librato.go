package librato

import (
	"fmt"
	"log"
	"math"
	"regexp"
	"time"

	"github.com/ethereum/go-ethereum/metrics"
)

// a regexp for extracting the unit from time.Duration.String
var unitRegexp = regexp.MustCompile(`[^\\d]+$`)

// a helper that turns a time.Duration into librato display attributes for timer metrics
func translateTimerAttributes(d time.Duration) (attrs map[string]interface{}) {
	attrs = make(map[string]interface{})
	attrs[DisplayTransform] = fmt.Sprintf("x/%d", int64(d))
	attrs[DisplayUnitsShort] = string(unitRegexp.Find([]byte(d.String())))
	return
}

type Reporter struct {
	Email, Token    string
	Namespace       string
	Source          string
	Interval        time.Duration
	Registry        metrics.Registry
	Percentiles     []float64              // percentiles to report on histogram metrics
	TimerAttributes map[string]interface{} // units in which timers will be displayed
	intervalSec     int64
}

func NewReporter(r metrics.Registry, d time.Duration, e string, t string, s string, p []float64, u time.Duration) *Reporter {
	return &Reporter{e, t, "", s, d, r, p, translateTimerAttributes(u), int64(d / time.Second)}
}

func Librato(r metrics.Registry, d time.Duration, e string, t string, s string, p []float64, u time.Duration) {
	NewReporter(r, d, e, t, s, p, u).Run()
}

func (rep *Reporter) Run() {
	log.Printf("WARNING: This client has been DEPRECATED! It has been moved to https://github.com/mihasya/go-metrics-librato and will be removed from rcrowley/go-metrics on August 5th 2015")
	ticker := time.NewTicker(rep.Interval)
	defer ticker.Stop()
	metricsApi := &LibratoClient{rep.Email, rep.Token}
	for now := range ticker.C {
		var metrics Batch
		var err error
		if metrics, err = rep.BuildRequest(now, rep.Registry); err != nil {
			log.Printf("ERROR constructing librato request body %s", err)
			continue
		}
		if err := metricsApi.PostMetrics(metrics); err != nil {
			log.Printf("ERROR sending metrics to librato %s", err)
			continue
		}
	}
}

// calculate sum of squares from data provided by metrics.Histogram
// see http://en.wikipedia.org/wiki/Standard_deviation#Rapid_calculation_methods
func sumSquares(icount int64, mean, stDev float64) float64 {
	count := float64(icount)
	sumSquared := math.Pow(count*mean, 2)
	sumSquares := math.Pow(count*stDev, 2) + sumSquared/count
	if math.IsNaN(sumSquares) {
		return 0.0
	}
	return sumSquares
}
func sumSquaresTimer(t metrics.TimerSnapshot) float64 {
	count := float64(t.Count())
	sumSquared := math.Pow(count*t.Mean(), 2)
	sumSquares := math.Pow(count*t.StdDev(), 2) + sumSquared/count
	if math.IsNaN(sumSquares) {
		return 0.0
	}
	return sumSquares
}

func (rep *Reporter) BuildRequest(now time.Time, r metrics.Registry) (snapshot Batch, err error) {
	snapshot = Batch{
		// coerce timestamps to a stepping fn so that they line up in Librato graphs
		MeasureTime: (now.Unix() / rep.intervalSec) * rep.intervalSec,
		Source:      rep.Source,
	}
	snapshot.Gauges = make([]Measurement, 0)
	snapshot.Counters = make([]Measurement, 0)
	histogramGaugeCount := 1 + len(rep.Percentiles)
	r.Each(func(name string, metric interface{}) {
		if rep.Namespace != "" {
			name = fmt.Sprintf("%s.%s", rep.Namespace, name)
		}
		measurement := Measurement{}
		measurement[Period] = rep.Interval.Seconds()
		switch m := metric.(type) {
		case metrics.Counter:
			ms := m.Snapshot()
			if ms.Count() > 0 {
				measurement[Name] = fmt.Sprintf("%s.%s", name, "count")
				measurement[Value] = float64(ms.Count())
				measurement[Attributes] = map[string]interface{}{
					DisplayUnitsLong:  Operations,
					DisplayUnitsShort: OperationsShort,
					DisplayMin:        "0",
				}
				snapshot.Counters = append(snapshot.Counters, measurement)
			}
		case metrics.CounterFloat64:
			if count := m.Snapshot().Count(); count > 0 {
				measurement[Name] = fmt.Sprintf("%s.%s", name, "count")
				measurement[Value] = count
				measurement[Attributes] = map[string]interface{}{
					DisplayUnitsLong:  Operations,
					DisplayUnitsShort: OperationsShort,
					DisplayMin:        "0",
				}
				snapshot.Counters = append(snapshot.Counters, measurement)
			}
		case metrics.Gauge:
			measurement[Name] = name
			measurement[Value] = float64(m.Snapshot().Value())
			snapshot.Gauges = append(snapshot.Gauges, measurement)
		case metrics.GaugeFloat64:
			measurement[Name] = name
			measurement[Value] = m.Snapshot().Value()
			snapshot.Gauges = append(snapshot.Gauges, measurement)
		case metrics.GaugeInfo:
			measurement[Name] = name
			measurement[Value] = m.Snapshot().Value()
			snapshot.Gauges = append(snapshot.Gauges, measurement)
		case metrics.Histogram:
			ms := m.Snapshot()
			if ms.Count() > 0 {
				gauges := make([]Measurement, histogramGaugeCount)
				measurement[Name] = fmt.Sprintf("%s.%s", name, "hist")
				measurement[Count] = uint64(ms.Count())
				measurement[Max] = float64(ms.Max())
				measurement[Min] = float64(ms.Min())
				measurement[Sum] = float64(ms.Sum())
				measurement[SumSquares] = sumSquares(ms.Count(), ms.Mean(), ms.StdDev())
				gauges[0] = measurement
				for i, p := range rep.Percentiles {
					gauges[i+1] = Measurement{
						Name:   fmt.Sprintf("%s.%.2f", measurement[Name], p),
						Value:  ms.Percentile(p),
						Period: measurement[Period],
					}
				}
				snapshot.Gauges = append(snapshot.Gauges, gauges...)
			}
		case metrics.Meter:
			ms := m.Snapshot()
			measurement[Name] = name
			measurement[Value] = float64(ms.Count())
			snapshot.Counters = append(snapshot.Counters, measurement)
			snapshot.Gauges = append(snapshot.Gauges,
				Measurement{
					Name:   fmt.Sprintf("%s.%s", name, "1min"),
					Value:  ms.Rate1(),
					Period: int64(rep.Interval.Seconds()),
					Attributes: map[string]interface{}{
						DisplayUnitsLong:  Operations,
						DisplayUnitsShort: OperationsShort,
						DisplayMin:        "0",
					},
				},
				Measurement{
					Name:   fmt.Sprintf("%s.%s", name, "5min"),
					Value:  ms.Rate5(),
					Period: int64(rep.Interval.Seconds()),
					Attributes: map[string]interface{}{
						DisplayUnitsLong:  Operations,
						DisplayUnitsShort: OperationsShort,
						DisplayMin:        "0",
					},
				},
				Measurement{
					Name:   fmt.Sprintf("%s.%s", name, "15min"),
					Value:  ms.Rate15(),
					Period: int64(rep.Interval.Seconds()),
					Attributes: map[string]interface{}{
						DisplayUnitsLong:  Operations,
						DisplayUnitsShort: OperationsShort,
						DisplayMin:        "0",
					},
				},
			)
		case metrics.Timer:
			ms := m.Snapshot()
			measurement[Name] = name
			measurement[Value] = float64(ms.Count())
			snapshot.Counters = append(snapshot.Counters, measurement)
			if ms.Count() > 0 {
				libratoName := fmt.Sprintf("%s.%s", name, "timer.mean")
				gauges := make([]Measurement, histogramGaugeCount)
				gauges[0] = Measurement{
					Name:       libratoName,
					Count:      uint64(ms.Count()),
					Sum:        ms.Mean() * float64(ms.Count()),
					Max:        float64(ms.Max()),
					Min:        float64(ms.Min()),
					SumSquares: sumSquaresTimer(ms),
					Period:     int64(rep.Interval.Seconds()),
					Attributes: rep.TimerAttributes,
				}
				for i, p := range rep.Percentiles {
					gauges[i+1] = Measurement{
						Name:       fmt.Sprintf("%s.timer.%2.0f", name, p*100),
						Value:      ms.Percentile(p),
						Period:     int64(rep.Interval.Seconds()),
						Attributes: rep.TimerAttributes,
					}
				}
				snapshot.Gauges = append(snapshot.Gauges, gauges...)
				snapshot.Gauges = append(snapshot.Gauges,
					Measurement{
						Name:   fmt.Sprintf("%s.%s", name, "rate.1min"),
						Value:  ms.Rate1(),
						Period: int64(rep.Interval.Seconds()),
						Attributes: map[string]interface{}{
							DisplayUnitsLong:  Operations,
							DisplayUnitsShort: OperationsShort,
							DisplayMin:        "0",
						},
					},
					Measurement{
						Name:   fmt.Sprintf("%s.%s", name, "rate.5min"),
						Value:  ms.Rate5(),
						Period: int64(rep.Interval.Seconds()),
						Attributes: map[string]interface{}{
							DisplayUnitsLong:  Operations,
							DisplayUnitsShort: OperationsShort,
							DisplayMin:        "0",
						},
					},
					Measurement{
						Name:   fmt.Sprintf("%s.%s", name, "rate.15min"),
						Value:  ms.Rate15(),
						Period: int64(rep.Interval.Seconds()),
						Attributes: map[string]interface{}{
							DisplayUnitsLong:  Operations,
							DisplayUnitsShort: OperationsShort,
							DisplayMin:        "0",
						},
					},
				)
			}
		}
	})
	return
}
