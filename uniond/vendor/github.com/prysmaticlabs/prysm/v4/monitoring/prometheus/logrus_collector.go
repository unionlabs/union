package prometheus

import (
	"errors"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	"github.com/sirupsen/logrus"
)

// LogrusCollector is a logrus hook to collect log counters.
type LogrusCollector struct {
	counterVec *prometheus.CounterVec
}

var (
	supportedLevels = []logrus.Level{logrus.InfoLevel, logrus.WarnLevel, logrus.ErrorLevel}
	counterVec      = promauto.NewCounterVec(prometheus.CounterOpts{
		Name: "log_entries_total",
		Help: "Total number of log messages.",
	}, []string{"level", "prefix"})
)

const prefixKey = "prefix"
const defaultprefix = "global"

// NewLogrusCollector register internal metrics and return an logrus hook to collect log counters
// This function can be called only once, if more than one call is made an error will be returned.
func NewLogrusCollector() *LogrusCollector {
	return &LogrusCollector{
		counterVec: counterVec,
	}
}

// Fire is called on every log call.
func (hook *LogrusCollector) Fire(entry *logrus.Entry) error {
	prefix := defaultprefix
	if prefixValue, ok := entry.Data[prefixKey]; ok {
		prefix, ok = prefixValue.(string)
		if !ok {
			return errors.New("prefix is not a string")
		}
	}
	hook.counterVec.WithLabelValues(entry.Level.String(), prefix).Inc()
	return nil
}

// Levels return a slice of levels supported by this hook;
func (_ *LogrusCollector) Levels() []logrus.Level {
	return supportedLevels
}
