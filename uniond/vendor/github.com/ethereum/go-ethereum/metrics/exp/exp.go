// Hook go-metrics into expvar
// on any /debug/metrics request, load all vars from the registry into expvar, and execute regular expvar handler
package exp

import (
	"expvar"
	"fmt"
	"net/http"
	"sync"

	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/metrics"
	"github.com/ethereum/go-ethereum/metrics/prometheus"
)

type exp struct {
	expvarLock sync.Mutex // expvar panics if you try to register the same var twice, so we must probe it safely
	registry   metrics.Registry
}

func (exp *exp) expHandler(w http.ResponseWriter, r *http.Request) {
	// load our variables into expvar
	exp.syncToExpvar()

	// now just run the official expvar handler code (which is not publicly callable, so pasted inline)
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	fmt.Fprintf(w, "{\n")
	first := true
	expvar.Do(func(kv expvar.KeyValue) {
		if !first {
			fmt.Fprintf(w, ",\n")
		}
		first = false
		fmt.Fprintf(w, "%q: %s", kv.Key, kv.Value)
	})
	fmt.Fprintf(w, "\n}\n")
}

// Exp will register an expvar powered metrics handler with http.DefaultServeMux on "/debug/vars"
func Exp(r metrics.Registry) {
	h := ExpHandler(r)
	// this would cause a panic:
	// panic: http: multiple registrations for /debug/vars
	// http.HandleFunc("/debug/vars", e.expHandler)
	// haven't found an elegant way, so just use a different endpoint
	http.Handle("/debug/metrics", h)
	http.Handle("/debug/metrics/prometheus", prometheus.Handler(r))
}

// ExpHandler will return an expvar powered metrics handler.
func ExpHandler(r metrics.Registry) http.Handler {
	e := exp{sync.Mutex{}, r}
	return http.HandlerFunc(e.expHandler)
}

// Setup starts a dedicated metrics server at the given address.
// This function enables metrics reporting separate from pprof.
func Setup(address string) {
	m := http.NewServeMux()
	m.Handle("/debug/metrics", ExpHandler(metrics.DefaultRegistry))
	m.Handle("/debug/metrics/prometheus", prometheus.Handler(metrics.DefaultRegistry))
	log.Info("Starting metrics server", "addr", fmt.Sprintf("http://%s/debug/metrics", address))
	go func() {
		if err := http.ListenAndServe(address, m); err != nil {
			log.Error("Failure in running metrics server", "err", err)
		}
	}()
}

func (exp *exp) getInt(name string) *expvar.Int {
	var v *expvar.Int
	exp.expvarLock.Lock()
	p := expvar.Get(name)
	if p != nil {
		v = p.(*expvar.Int)
	} else {
		v = new(expvar.Int)
		expvar.Publish(name, v)
	}
	exp.expvarLock.Unlock()
	return v
}

func (exp *exp) getFloat(name string) *expvar.Float {
	var v *expvar.Float
	exp.expvarLock.Lock()
	p := expvar.Get(name)
	if p != nil {
		v = p.(*expvar.Float)
	} else {
		v = new(expvar.Float)
		expvar.Publish(name, v)
	}
	exp.expvarLock.Unlock()
	return v
}

func (exp *exp) getInfo(name string) *expvar.String {
	var v *expvar.String
	exp.expvarLock.Lock()
	p := expvar.Get(name)
	if p != nil {
		v = p.(*expvar.String)
	} else {
		v = new(expvar.String)
		expvar.Publish(name, v)
	}
	exp.expvarLock.Unlock()
	return v
}

func (exp *exp) publishCounter(name string, metric metrics.CounterSnapshot) {
	v := exp.getInt(name)
	v.Set(metric.Count())
}

func (exp *exp) publishCounterFloat64(name string, metric metrics.CounterFloat64Snapshot) {
	v := exp.getFloat(name)
	v.Set(metric.Count())
}

func (exp *exp) publishGauge(name string, metric metrics.GaugeSnapshot) {
	v := exp.getInt(name)
	v.Set(metric.Value())
}
func (exp *exp) publishGaugeFloat64(name string, metric metrics.GaugeFloat64Snapshot) {
	exp.getFloat(name).Set(metric.Value())
}

func (exp *exp) publishGaugeInfo(name string, metric metrics.GaugeInfoSnapshot) {
	exp.getInfo(name).Set(metric.Value().String())
}

func (exp *exp) publishHistogram(name string, metric metrics.Histogram) {
	h := metric.Snapshot()
	ps := h.Percentiles([]float64{0.5, 0.75, 0.95, 0.99, 0.999})
	exp.getInt(name + ".count").Set(h.Count())
	exp.getFloat(name + ".min").Set(float64(h.Min()))
	exp.getFloat(name + ".max").Set(float64(h.Max()))
	exp.getFloat(name + ".mean").Set(h.Mean())
	exp.getFloat(name + ".std-dev").Set(h.StdDev())
	exp.getFloat(name + ".50-percentile").Set(ps[0])
	exp.getFloat(name + ".75-percentile").Set(ps[1])
	exp.getFloat(name + ".95-percentile").Set(ps[2])
	exp.getFloat(name + ".99-percentile").Set(ps[3])
	exp.getFloat(name + ".999-percentile").Set(ps[4])
}

func (exp *exp) publishMeter(name string, metric metrics.Meter) {
	m := metric.Snapshot()
	exp.getInt(name + ".count").Set(m.Count())
	exp.getFloat(name + ".one-minute").Set(m.Rate1())
	exp.getFloat(name + ".five-minute").Set(m.Rate5())
	exp.getFloat(name + ".fifteen-minute").Set(m.Rate15())
	exp.getFloat(name + ".mean").Set(m.RateMean())
}

func (exp *exp) publishTimer(name string, metric metrics.Timer) {
	t := metric.Snapshot()
	ps := t.Percentiles([]float64{0.5, 0.75, 0.95, 0.99, 0.999})
	exp.getInt(name + ".count").Set(t.Count())
	exp.getFloat(name + ".min").Set(float64(t.Min()))
	exp.getFloat(name + ".max").Set(float64(t.Max()))
	exp.getFloat(name + ".mean").Set(t.Mean())
	exp.getFloat(name + ".std-dev").Set(t.StdDev())
	exp.getFloat(name + ".50-percentile").Set(ps[0])
	exp.getFloat(name + ".75-percentile").Set(ps[1])
	exp.getFloat(name + ".95-percentile").Set(ps[2])
	exp.getFloat(name + ".99-percentile").Set(ps[3])
	exp.getFloat(name + ".999-percentile").Set(ps[4])
	exp.getFloat(name + ".one-minute").Set(t.Rate1())
	exp.getFloat(name + ".five-minute").Set(t.Rate5())
	exp.getFloat(name + ".fifteen-minute").Set(t.Rate15())
	exp.getFloat(name + ".mean-rate").Set(t.RateMean())
}

func (exp *exp) publishResettingTimer(name string, metric metrics.ResettingTimer) {
	t := metric.Snapshot()
	ps := t.Percentiles([]float64{0.50, 0.75, 0.95, 0.99})
	exp.getInt(name + ".count").Set(int64(t.Count()))
	exp.getFloat(name + ".mean").Set(t.Mean())
	exp.getFloat(name + ".50-percentile").Set(ps[0])
	exp.getFloat(name + ".75-percentile").Set(ps[1])
	exp.getFloat(name + ".95-percentile").Set(ps[2])
	exp.getFloat(name + ".99-percentile").Set(ps[3])
}

func (exp *exp) syncToExpvar() {
	exp.registry.Each(func(name string, i interface{}) {
		switch i := i.(type) {
		case metrics.Counter:
			exp.publishCounter(name, i.Snapshot())
		case metrics.CounterFloat64:
			exp.publishCounterFloat64(name, i.Snapshot())
		case metrics.Gauge:
			exp.publishGauge(name, i.Snapshot())
		case metrics.GaugeFloat64:
			exp.publishGaugeFloat64(name, i.Snapshot())
		case metrics.GaugeInfo:
			exp.publishGaugeInfo(name, i.Snapshot())
		case metrics.Histogram:
			exp.publishHistogram(name, i)
		case metrics.Meter:
			exp.publishMeter(name, i)
		case metrics.Timer:
			exp.publishTimer(name, i)
		case metrics.ResettingTimer:
			exp.publishResettingTimer(name, i)
		default:
			panic(fmt.Sprintf("unsupported type for '%s': %T", name, i))
		}
	})
}
