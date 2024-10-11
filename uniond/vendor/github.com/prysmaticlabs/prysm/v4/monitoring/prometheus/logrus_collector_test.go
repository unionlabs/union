package prometheus_test

import (
	"fmt"
	"io"
	"net/http"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/monitoring/prometheus"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	log "github.com/sirupsen/logrus"
)

const addr = "127.0.0.1:8989"

type logger interface {
	Info(args ...interface{})
	Warn(args ...interface{})
	Error(args ...interface{})
}

func TestLogrusCollector(t *testing.T) {
	service := prometheus.NewService(addr, nil)
	hook := prometheus.NewLogrusCollector()
	log.AddHook(hook)
	go service.Start()
	defer func() {
		err := service.Stop()
		require.NoError(t, err)
	}()

	tests := []struct {
		name   string
		want   int
		count  int
		prefix string
		level  log.Level
	}{
		{"info message with empty prefix", 3, 3, "", log.InfoLevel},
		{"warn message with empty prefix", 2, 2, "", log.WarnLevel},
		{"error message with empty prefix", 1, 1, "", log.ErrorLevel},
		{"error message with prefix", 1, 1, "foo", log.ErrorLevel},
		{"info message with prefix", 3, 3, "foo", log.InfoLevel},
		{"warn message with prefix", 2, 2, "foo", log.WarnLevel},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			prefix := "global"
			for i := 0; i < tt.count; i++ {
				if tt.prefix != "" {
					prefix = tt.prefix
					subLog := log.WithField("prefix", tt.prefix)
					logExampleMessage(subLog, tt.level)
					continue
				}
				logExampleMessage(log.StandardLogger(), tt.level)
			}
			time.Sleep(time.Millisecond)
			metrics := metrics(t)
			count := valueFor(t, metrics, prefix, tt.level)
			if count != tt.want {
				t.Errorf("Expecting %d and receive %d", tt.want, count)
			}
		})
	}
}

func metrics(t *testing.T) []string {
	resp, err := http.Get(fmt.Sprintf("http://%s/metrics", addr))
	require.NoError(t, err)
	body, err := io.ReadAll(resp.Body)
	require.NoError(t, err)
	return strings.Split(string(body), "\n")
}

func valueFor(t *testing.T, metrics []string, prefix string, level log.Level) int {
	// Expect line with this pattern:
	//   # HELP log_entries_total Total number of log messages.
	//   # TYPE log_entries_total counter
	//   log_entries_total{level="error",prefix="empty"} 1
	pattern := fmt.Sprintf("log_entries_total{level=\"%s\",prefix=\"%s\"}", level, prefix)
	for _, line := range metrics {
		if strings.HasPrefix(line, pattern) {
			parts := strings.Split(line, " ")
			count, err := strconv.ParseFloat(parts[1], 64)
			assert.NoError(t, err)
			return int(count)
		}
	}
	t.Errorf("Pattern \"%s\" not found", pattern)
	return 0
}

func logExampleMessage(logger logger, level log.Level) {
	switch level {
	case log.InfoLevel:
		logger.Info("Info message")
	case log.WarnLevel:
		logger.Warn("Warning message!")
	case log.ErrorLevel:
		logger.Error("Error message!!")
	}
}
