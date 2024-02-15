package version

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var prysmInfo = promauto.NewGauge(prometheus.GaugeOpts{
	Name: "prysm_version",
	ConstLabels: prometheus.Labels{
		"version":   gitTag,
		"commit":    gitCommit,
		"buildDate": buildDateUnix},
})

func init() {
	prysmInfo.Set(float64(1))
}
