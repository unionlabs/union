package writer

import (
	"io"

	log "github.com/sirupsen/logrus"
)

// Hook is a hook that writes logs of specified LogLevels to specified Writer
type Hook struct {
	Writer    io.Writer
	LogLevels []log.Level
}

// Fire will be called when some logging function is called with current hook
// It will format log entry to string and write it to appropriate writer
func (hook *Hook) Fire(entry *log.Entry) error {
	line, err := entry.Bytes()
	if err != nil {
		return err
	}
	_, err = hook.Writer.Write(line)
	return err
}

// Levels define on which log levels this hook would trigger
func (hook *Hook) Levels() []log.Level {
	return hook.LogLevels
}
