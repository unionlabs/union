package level_test

import (
	"io/ioutil"
	"testing"

	"github.com/go-kit/log"
	"github.com/go-kit/log/level"
)

func Benchmark(b *testing.B) {
	contexts := []struct {
		name    string
		context func(log.Logger) log.Logger
	}{
		{"NoContext", func(l log.Logger) log.Logger {
			return l
		}},
		{"TimeContext", func(l log.Logger) log.Logger {
			return log.With(l, "time", log.DefaultTimestampUTC)
		}},
		{"CallerContext", func(l log.Logger) log.Logger {
			return log.With(l, "caller", log.DefaultCaller)
		}},
		{"TimeCallerReqIDContext", func(l log.Logger) log.Logger {
			return log.With(l, "time", log.DefaultTimestampUTC, "caller", log.DefaultCaller, "reqID", 29)
		}},
	}

	loggers := []struct {
		name   string
		logger log.Logger
	}{
		{"Nop", log.NewNopLogger()},
		{"Logfmt", log.NewLogfmtLogger(ioutil.Discard)},
		{"JSON", log.NewJSONLogger(ioutil.Discard)},
	}

	filters := []struct {
		name   string
		filter func(log.Logger) log.Logger
	}{
		{"Baseline", func(l log.Logger) log.Logger {
			return l
		}},
		{"DisallowedLevel", func(l log.Logger) log.Logger {
			return level.NewFilter(l, level.AllowInfo())
		}},
		{"AllowedLevel", func(l log.Logger) log.Logger {
			return level.NewFilter(l, level.AllowAll())
		}},
	}

	for _, c := range contexts {
		b.Run(c.name, func(b *testing.B) {
			for _, f := range filters {
				b.Run(f.name, func(b *testing.B) {
					for _, l := range loggers {
						b.Run(l.name, func(b *testing.B) {
							logger := c.context(f.filter(l.logger))
							b.ResetTimer()
							b.ReportAllocs()
							for i := 0; i < b.N; i++ {
								level.Debug(logger).Log("foo", "bar")
							}
						})
					}
				})
			}
		})
	}
}
