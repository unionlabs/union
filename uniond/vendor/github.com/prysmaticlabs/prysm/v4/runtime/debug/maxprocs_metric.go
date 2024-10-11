package debug

import (
	"runtime"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	_ = promauto.NewGaugeFunc(prometheus.GaugeOpts{
		Name: "go_maxprocs",
		Help: "The result of runtime.GOMAXPROCS(0)",
	}, func() float64 {
		return float64(runtime.GOMAXPROCS(0))
	})
)
