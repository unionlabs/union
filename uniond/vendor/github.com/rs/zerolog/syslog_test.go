// +build !binary_log
// +build !windows

package zerolog

import (
	"bytes"
	"reflect"
	"strings"
	"testing"
)

type syslogEvent struct {
	level string
	msg   string
}
type syslogTestWriter struct {
	events []syslogEvent
}

func (w *syslogTestWriter) Write(p []byte) (int, error) {
	return 0, nil
}
func (w *syslogTestWriter) Trace(m string) error {
	w.events = append(w.events, syslogEvent{"Trace", m})
	return nil
}
func (w *syslogTestWriter) Debug(m string) error {
	w.events = append(w.events, syslogEvent{"Debug", m})
	return nil
}
func (w *syslogTestWriter) Info(m string) error {
	w.events = append(w.events, syslogEvent{"Info", m})
	return nil
}
func (w *syslogTestWriter) Warning(m string) error {
	w.events = append(w.events, syslogEvent{"Warning", m})
	return nil
}
func (w *syslogTestWriter) Err(m string) error {
	w.events = append(w.events, syslogEvent{"Err", m})
	return nil
}
func (w *syslogTestWriter) Emerg(m string) error {
	w.events = append(w.events, syslogEvent{"Emerg", m})
	return nil
}
func (w *syslogTestWriter) Crit(m string) error {
	w.events = append(w.events, syslogEvent{"Crit", m})
	return nil
}

func TestSyslogWriter(t *testing.T) {
	sw := &syslogTestWriter{}
	log := New(SyslogLevelWriter(sw))
	log.Trace().Msg("trace")
	log.Debug().Msg("debug")
	log.Info().Msg("info")
	log.Warn().Msg("warn")
	log.Error().Msg("error")
	log.Log().Msg("nolevel")
	want := []syslogEvent{
		{"Debug", `{"level":"debug","message":"debug"}` + "\n"},
		{"Info", `{"level":"info","message":"info"}` + "\n"},
		{"Warning", `{"level":"warn","message":"warn"}` + "\n"},
		{"Err", `{"level":"error","message":"error"}` + "\n"},
		{"Info", `{"message":"nolevel"}` + "\n"},
	}
	if got := sw.events; !reflect.DeepEqual(got, want) {
		t.Errorf("Invalid syslog message routing: want %v, got %v", want, got)
	}
}

type testCEEwriter struct {
	buf *bytes.Buffer
}

// Only implement one method as we're just testing the prefixing
func (c testCEEwriter) Debug(m string) error { return nil }

func (c testCEEwriter) Info(m string) error {
	_, err := c.buf.Write([]byte(m))
	return err
}

func (c testCEEwriter) Warning(m string) error { return nil }

func (c testCEEwriter) Err(m string) error { return nil }

func (c testCEEwriter) Emerg(m string) error { return nil }

func (c testCEEwriter) Crit(m string) error { return nil }

func (c testCEEwriter) Write(b []byte) (int, error) {
	return c.buf.Write(b)
}

func TestSyslogWriter_WithCEE(t *testing.T) {
	var buf bytes.Buffer
	sw := testCEEwriter{&buf}
	log := New(SyslogCEEWriter(sw))
	log.Info().Str("key", "value").Msg("message string")
	got := buf.String()
	want := "@cee:{"
	if !strings.HasPrefix(got, want) {
		t.Errorf("Bad CEE message start: want %v, got %v", want, got)
	}
}
