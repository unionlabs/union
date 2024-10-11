// Copyright 2024 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//go:build go1.21

package promslog

import (
	"bytes"
	"context"
	"fmt"
	"log/slog"
	"regexp"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v2"
)

var (
	slogStyleLogRegexp  = regexp.MustCompile(`(?P<TimeKey>time)=.*level=(?P<LevelValue>WARN|INFO|ERROR|DEBUG).*(?P<SourceKey>source)=.*`)
	goKitStyleLogRegexp = regexp.MustCompile(`(?P<TimeKey>ts)=.*level=(?P<LevelValue>warn|info|error|debug).*(?P<SourceKey>caller)=.*`)
)

// Make sure creating and using a logger with an empty configuration doesn't
// result in a panic.
func TestDefaultConfig(t *testing.T) {
	require.NotPanics(t, func() {
		logger := New(&Config{})
		logger.Info("empty config `Info()` test", "hello", "world")
		logger.Log(context.Background(), slog.LevelInfo, "empty config `Log()` test", "hello", "world")
		logger.LogAttrs(context.Background(), slog.LevelInfo, "empty config `LogAttrs()` test", slog.String("hello", "world"))
	})
}

func TestUnmarshallLevel(t *testing.T) {
	l := &AllowedLevel{}
	err := yaml.Unmarshal([]byte(`debug`), l)
	if err != nil {
		t.Error(err)
	}
	if l.s != "debug" {
		t.Errorf("expected %s, got %s", "debug", l.s)
	}
}

func TestUnmarshallEmptyLevel(t *testing.T) {
	l := &AllowedLevel{}
	err := yaml.Unmarshal([]byte(``), l)
	if err != nil {
		t.Error(err)
	}
	if l.s != "" {
		t.Errorf("expected empty level, got %s", l.s)
	}
}

func TestUnmarshallBadLevel(t *testing.T) {
	l := &AllowedLevel{}
	err := yaml.Unmarshal([]byte(`debugg`), l)
	if err == nil {
		t.Error("expected error")
	}
	expErr := `unrecognized log level debugg`
	if err.Error() != expErr {
		t.Errorf("expected error %s, got %s", expErr, err.Error())
	}
	if l.s != "" {
		t.Errorf("expected empty level, got %s", l.s)
	}
}

func getLogEntryLevelCounts(s string, re *regexp.Regexp) map[string]int {
	counters := make(map[string]int)
	lines := strings.Split(s, "\n")
	for _, line := range lines {
		matches := re.FindStringSubmatch(line)
		if len(matches) > 1 {
			levelIndex := re.SubexpIndex("LevelValue")

			counters[strings.ToLower(matches[levelIndex])]++
		}
	}

	return counters
}

func TestDynamicLevels(t *testing.T) {
	var buf bytes.Buffer
	wantedLevelCounts := map[string]int{"info": 1, "debug": 1}

	tests := map[string]struct {
		logStyle         LogStyle
		logStyleRegexp   *regexp.Regexp
		wantedLevelCount map[string]int
	}{
		"slog_log_style":   {logStyle: SlogStyle, logStyleRegexp: slogStyleLogRegexp, wantedLevelCount: wantedLevelCounts},
		"go-kit_log_style": {logStyle: GoKitStyle, logStyleRegexp: goKitStyleLogRegexp, wantedLevelCount: wantedLevelCounts},
	}

	for name, tc := range tests {
		t.Run(name, func(t *testing.T) {
			buf.Reset() // Ensure buf is reset prior to tests
			config := &Config{ioWriter: &buf, Style: tc.logStyle}
			logger := New(config)

			// Test that log level can be adjusted on-the-fly to debug and that a
			// log entry can be written to the file.
			if err := config.Level.Set("debug"); err != nil {
				t.Fatal(err)
			}
			logger.Info("info", "hello", "world")
			logger.Debug("debug", "hello", "world")

			counts := getLogEntryLevelCounts(buf.String(), tc.logStyleRegexp)
			require.Equal(t, tc.wantedLevelCount["info"], counts["info"], "info log successfully detected")
			require.Equal(t, tc.wantedLevelCount["debug"], counts["debug"], "debug log successfully detected")
			// Print logs for humans to see, if needed.
			fmt.Println(buf.String())
			buf.Reset()

			// Test that log level can be adjusted on-the-fly to info and that a
			// subsequent call to write a debug level log is _not_ written to the
			// file.
			if err := config.Level.Set("info"); err != nil {
				t.Fatal(err)
			}
			logger.Info("info", "hello", "world")
			logger.Debug("debug", "hello", "world")

			counts = getLogEntryLevelCounts(buf.String(), tc.logStyleRegexp)
			require.Equal(t, tc.wantedLevelCount["info"], counts["info"], "info log successfully detected")
			require.NotEqual(t, tc.wantedLevelCount["debug"], counts["debug"], "extra debug log detected")
			// Print logs for humans to see, if needed.
			fmt.Println(buf.String())
			buf.Reset()
		})
	}
}
