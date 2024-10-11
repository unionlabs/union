//go:build linux

// Package journald was copied directly from https://github.com/wercker/journalhook,
// where this library was previously hosted.
package journald

import (
	"fmt"
	"strings"

	"github.com/coreos/go-systemd/journal"
	"github.com/sirupsen/logrus"
)

type JournalHook struct{}

var (
	severityMap = map[logrus.Level]journal.Priority{
		logrus.DebugLevel: journal.PriDebug,
		logrus.InfoLevel:  journal.PriInfo,
		logrus.WarnLevel:  journal.PriWarning,
		logrus.ErrorLevel: journal.PriErr,
		logrus.FatalLevel: journal.PriCrit,
		logrus.PanicLevel: journal.PriEmerg,
	}
)

func stringifyOp(r rune) rune {
	// Journal wants uppercase strings. See `validVarName`
	// https://github.com/coreos/go-systemd/blob/ff118ad0f8d9cf99903d3391ca3a295671022cee/journal/journal.go#L137-L147
	switch {
	case r >= 'A' && r <= 'Z':
		return r
	case r >= '0' && r <= '9':
		return r
	case r == '_':
		return r
	case r >= 'a' && r <= 'z':
		return r - 32
	default:
		return rune('_')
	}
}

func stringifyKey(key string) string {
	key = strings.Map(stringifyOp, key)
	key = strings.TrimPrefix(key, "_")
	return key
}

// Journal wants strings but logrus takes anything.
func stringifyEntries(data map[string]interface{}) map[string]string {
	entries := make(map[string]string)
	for k, v := range data {
		key := stringifyKey(k)
		entries[key] = fmt.Sprint(v)
	}
	return entries
}

// Fire fires an entry into the journal.
func (hook *JournalHook) Fire(entry *logrus.Entry) error {
	return journal.Send(entry.Message, severityMap[entry.Level], stringifyEntries(entry.Data))
}

// Levels returns a slice of `Levels` the hook is fired for.
func (hook *JournalHook) Levels() []logrus.Level {
	return []logrus.Level{
		logrus.PanicLevel,
		logrus.FatalLevel,
		logrus.ErrorLevel,
		logrus.WarnLevel,
		logrus.InfoLevel,
		logrus.DebugLevel,
	}
}
