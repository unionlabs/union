// +build !windows
// +build !binary_log

package zerolog

import (
	"io"
)

// See http://cee.mitre.org/language/1.0-beta1/clt.html#syslog
// or https://www.rsyslog.com/json-elasticsearch/
const ceePrefix = "@cee:"

// SyslogWriter is an interface matching a syslog.Writer struct.
type SyslogWriter interface {
	io.Writer
	Debug(m string) error
	Info(m string) error
	Warning(m string) error
	Err(m string) error
	Emerg(m string) error
	Crit(m string) error
}

type syslogWriter struct {
	w      SyslogWriter
	prefix string
}

// SyslogLevelWriter wraps a SyslogWriter and call the right syslog level
// method matching the zerolog level.
func SyslogLevelWriter(w SyslogWriter) LevelWriter {
	return syslogWriter{w, ""}
}

// SyslogCEEWriter wraps a SyslogWriter with a SyslogLevelWriter that adds a
// MITRE CEE prefix for JSON syslog entries, compatible with rsyslog 
// and syslog-ng JSON logging support. 
// See https://www.rsyslog.com/json-elasticsearch/
func SyslogCEEWriter(w SyslogWriter) LevelWriter {
	return syslogWriter{w, ceePrefix}
}

func (sw syslogWriter) Write(p []byte) (n int, err error) {
	var pn int
	if sw.prefix != "" {
		pn, err = sw.w.Write([]byte(sw.prefix))
		if err != nil {
			return pn, err
		}
	}
	n, err = sw.w.Write(p)
	return pn + n, err
}

// WriteLevel implements LevelWriter interface.
func (sw syslogWriter) WriteLevel(level Level, p []byte) (n int, err error) {
	switch level {
	case TraceLevel:
	case DebugLevel:
		err = sw.w.Debug(sw.prefix + string(p))
	case InfoLevel:
		err = sw.w.Info(sw.prefix + string(p))
	case WarnLevel:
		err = sw.w.Warning(sw.prefix + string(p))
	case ErrorLevel:
		err = sw.w.Err(sw.prefix + string(p))
	case FatalLevel:
		err = sw.w.Emerg(sw.prefix + string(p))
	case PanicLevel:
		err = sw.w.Crit(sw.prefix + string(p))
	case NoLevel:
		err = sw.w.Info(sw.prefix + string(p))
	default:
		panic("invalid level")
	}
	// Any CEE prefix is not part of the message, so we don't include its length
	n = len(p)
	return
}
