package keeper

import (
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	"github.com/prometheus/client_golang/prometheus"
)

const (
	labelPinned = "pinned"
	labelMemory = "memory"
	labelFs     = "fs"
)

// metricSource source of wasmvm metrics
type metricSource interface {
	GetMetrics() (*wasmvmtypes.Metrics, error)
}

var _ prometheus.Collector = (*WasmVMMetricsCollector)(nil)

// WasmVMMetricsCollector custom metrics collector to be used with Prometheus
type WasmVMMetricsCollector struct {
	source             metricSource
	CacheHitsDescr     *prometheus.Desc
	CacheMissesDescr   *prometheus.Desc
	CacheElementsDescr *prometheus.Desc
	CacheSizeDescr     *prometheus.Desc
}

// NewWasmVMMetricsCollector constructor
func NewWasmVMMetricsCollector(s metricSource) *WasmVMMetricsCollector {
	return &WasmVMMetricsCollector{
		source:             s,
		CacheHitsDescr:     prometheus.NewDesc("wasmvm_cache_hits_total", "Total number of cache hits", []string{"type"}, nil),
		CacheMissesDescr:   prometheus.NewDesc("wasmvm_cache_misses_total", "Total number of cache misses", nil, nil),
		CacheElementsDescr: prometheus.NewDesc("wasmvm_cache_elements_total", "Total number of elements in the cache", []string{"type"}, nil),
		CacheSizeDescr:     prometheus.NewDesc("wasmvm_cache_size_bytes", "Total number of elements in the cache", []string{"type"}, nil),
	}
}

// Register registers all metrics
func (p *WasmVMMetricsCollector) Register(r prometheus.Registerer) {
	r.MustRegister(p)
}

// Describe sends the super-set of all possible descriptors of metrics
func (p *WasmVMMetricsCollector) Describe(descs chan<- *prometheus.Desc) {
	descs <- p.CacheHitsDescr
	descs <- p.CacheMissesDescr
	descs <- p.CacheElementsDescr
	descs <- p.CacheSizeDescr
}

// Collect is called by the Prometheus registry when collecting metrics.
func (p *WasmVMMetricsCollector) Collect(c chan<- prometheus.Metric) {
	m, err := p.source.GetMetrics()
	if err != nil {
		return
	}
	c <- prometheus.MustNewConstMetric(p.CacheHitsDescr, prometheus.CounterValue, float64(m.HitsPinnedMemoryCache), labelPinned)
	c <- prometheus.MustNewConstMetric(p.CacheHitsDescr, prometheus.CounterValue, float64(m.HitsMemoryCache), labelMemory)
	c <- prometheus.MustNewConstMetric(p.CacheHitsDescr, prometheus.CounterValue, float64(m.HitsFsCache), labelFs)
	c <- prometheus.MustNewConstMetric(p.CacheMissesDescr, prometheus.CounterValue, float64(m.Misses))
	c <- prometheus.MustNewConstMetric(p.CacheElementsDescr, prometheus.GaugeValue, float64(m.ElementsPinnedMemoryCache), labelPinned)
	c <- prometheus.MustNewConstMetric(p.CacheElementsDescr, prometheus.GaugeValue, float64(m.ElementsMemoryCache), labelMemory)
	c <- prometheus.MustNewConstMetric(p.CacheSizeDescr, prometheus.GaugeValue, float64(m.SizeMemoryCache), labelMemory)
	c <- prometheus.MustNewConstMetric(p.CacheSizeDescr, prometheus.GaugeValue, float64(m.SizePinnedMemoryCache), labelPinned)
	// Node about fs metrics:
	// The number of elements and the size of elements in the file system cache cannot easily be obtained.
	// We had to either scan the whole directory of potentially thousands of files or track the values when files are added or removed.
	// Such a tracking would need to be on disk such that the values are not cleared when the node is restarted.
}
