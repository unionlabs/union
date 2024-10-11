//go:build !windows && !plan9 && !nacl
// +build !windows,!plan9,!nacl

package syslog

import (
	"fmt"
	"reflect"
	"testing"

	gosyslog "log/syslog"

	"github.com/go-kit/log"
	"github.com/go-kit/log/level"
)

func TestSyslogLoggerDefaultPrioritySelector(t *testing.T) {
	w := &testSyslogWriter{}
	l := NewSyslogLogger(w, log.NewLogfmtLogger)

	l.Log("level", level.WarnValue(), "msg", "one")
	l.Log("level", "undefined", "msg", "two")
	l.Log("level", level.InfoValue(), "msg", "three")
	l.Log("level", level.ErrorValue(), "msg", "four")
	l.Log("level", level.DebugValue(), "msg", "five")

	l.Log("msg", "six", "level", level.ErrorValue())
	l.Log("msg", "seven", "level", level.DebugValue())
	l.Log("msg", "eight", "level", level.InfoValue())
	l.Log("msg", "nine", "level", "undefined")
	l.Log("msg", "ten", "level", level.WarnValue())

	l.Log("level", level.ErrorValue(), "msg")
	l.Log("msg", "eleven", "level")

	want := []string{
		"warning: level=warn msg=one\n",
		"info: level=undefined msg=two\n",
		"info: level=info msg=three\n",
		"err: level=error msg=four\n",
		"debug: level=debug msg=five\n",

		"err: msg=six level=error\n",
		"debug: msg=seven level=debug\n",
		"info: msg=eight level=info\n",
		"info: msg=nine level=undefined\n",
		"warning: msg=ten level=warn\n",

		"err: level=error msg=null\n",
		"info: msg=eleven level=null\n",
	}
	have := w.writes
	if !reflect.DeepEqual(want, have) {
		t.Errorf("wrong writes: want %s, have %s", want, have)
	}
}

func TestSyslogLoggerExhaustivePrioritySelector(t *testing.T) {
	w := &testSyslogWriter{}
	selector := func(keyvals ...interface{}) gosyslog.Priority {
		for i := 0; i < len(keyvals); i += 2 {
			if keyvals[i] == level.Key() {
				if v, ok := keyvals[i+1].(string); ok {
					switch v {
					case "emergency":
						return gosyslog.LOG_EMERG
					case "alert":
						return gosyslog.LOG_ALERT
					case "critical":
						return gosyslog.LOG_CRIT
					case "error":
						return gosyslog.LOG_ERR
					case "warning":
						return gosyslog.LOG_WARNING
					case "notice":
						return gosyslog.LOG_NOTICE
					case "info":
						return gosyslog.LOG_INFO
					case "debug":
						return gosyslog.LOG_DEBUG
					}
					return gosyslog.LOG_LOCAL0
				}
			}
		}
		return gosyslog.LOG_LOCAL0
	}
	l := NewSyslogLogger(w, log.NewLogfmtLogger, PrioritySelectorOption(selector))

	l.Log("level", "warning", "msg", "one")
	l.Log("level", "error", "msg", "two")
	l.Log("level", "critical", "msg", "three")
	l.Log("level", "debug", "msg", "four")
	l.Log("level", "info", "msg", "five")
	l.Log("level", "alert", "msg", "six")
	l.Log("level", "emergency", "msg", "seven")
	l.Log("level", "notice", "msg", "eight")
	l.Log("level", "unknown", "msg", "nine")

	want := []string{
		"warning: level=warning msg=one\n",
		"err: level=error msg=two\n",
		"crit: level=critical msg=three\n",
		"debug: level=debug msg=four\n",
		"info: level=info msg=five\n",
		"alert: level=alert msg=six\n",
		"emerg: level=emergency msg=seven\n",
		"notice: level=notice msg=eight\n",
		"write: level=unknown msg=nine\n",
	}
	have := w.writes
	if !reflect.DeepEqual(want, have) {
		t.Errorf("wrong writes: want %s, have %s", want, have)
	}
}

type testSyslogWriter struct {
	writes []string
}

func (w *testSyslogWriter) Write(b []byte) (int, error) {
	msg := string(b)
	w.writes = append(w.writes, fmt.Sprintf("write: %s", msg))
	return len(msg), nil
}

func (w *testSyslogWriter) Close() error {
	return nil
}

func (w *testSyslogWriter) Emerg(msg string) error {
	w.writes = append(w.writes, fmt.Sprintf("emerg: %s", msg))
	return nil
}

func (w *testSyslogWriter) Alert(msg string) error {
	w.writes = append(w.writes, fmt.Sprintf("alert: %s", msg))
	return nil
}

func (w *testSyslogWriter) Crit(msg string) error {
	w.writes = append(w.writes, fmt.Sprintf("crit: %s", msg))
	return nil
}

func (w *testSyslogWriter) Err(msg string) error {
	w.writes = append(w.writes, fmt.Sprintf("err: %s", msg))
	return nil
}

func (w *testSyslogWriter) Warning(msg string) error {
	w.writes = append(w.writes, fmt.Sprintf("warning: %s", msg))
	return nil
}

func (w *testSyslogWriter) Notice(msg string) error {
	w.writes = append(w.writes, fmt.Sprintf("notice: %s", msg))
	return nil
}

func (w *testSyslogWriter) Info(msg string) error {
	w.writes = append(w.writes, fmt.Sprintf("info: %s", msg))
	return nil
}

func (w *testSyslogWriter) Debug(msg string) error {
	w.writes = append(w.writes, fmt.Sprintf("debug: %s", msg))
	return nil
}
