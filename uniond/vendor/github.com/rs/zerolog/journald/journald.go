//go:build !windows
// +build !windows

// Package journald provides a io.Writer to send the logs
// to journalD component of systemd.

package journald

// This file provides a zerolog writer so that logs printed
// using zerolog library can be sent to a journalD.

// Zerolog's Top level key/Value Pairs are translated to
// journald's args - all Values are sent to journald as strings.
// And all key strings are converted to uppercase before sending
// to journald (as required by journald).

// In addition, entire log message (all Key Value Pairs), is also
// sent to journald under the key "JSON".

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"strings"

	"github.com/coreos/go-systemd/v22/journal"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/internal/cbor"
)

const defaultJournalDPrio = journal.PriNotice

// NewJournalDWriter returns a zerolog log destination
// to be used as parameter to New() calls. Writing logs
// to this writer will send the log messages to journalD
// running in this system.
func NewJournalDWriter() io.Writer {
	return journalWriter{}
}

type journalWriter struct {
}

// levelToJPrio converts zerolog Level string into
// journalD's priority values. JournalD has more
// priorities than zerolog.
func levelToJPrio(zLevel string) journal.Priority {
	lvl, _ := zerolog.ParseLevel(zLevel)

	switch lvl {
	case zerolog.TraceLevel:
		return journal.PriDebug
	case zerolog.DebugLevel:
		return journal.PriDebug
	case zerolog.InfoLevel:
		return journal.PriInfo
	case zerolog.WarnLevel:
		return journal.PriWarning
	case zerolog.ErrorLevel:
		return journal.PriErr
	case zerolog.FatalLevel:
		return journal.PriCrit
	case zerolog.PanicLevel:
		return journal.PriEmerg
	case zerolog.NoLevel:
		return journal.PriNotice
	}
	return defaultJournalDPrio
}

func (w journalWriter) Write(p []byte) (n int, err error) {
	var event map[string]interface{}
	origPLen := len(p)
	p = cbor.DecodeIfBinaryToBytes(p)
	d := json.NewDecoder(bytes.NewReader(p))
	d.UseNumber()
	err = d.Decode(&event)
	jPrio := defaultJournalDPrio
	args := make(map[string]string)
	if err != nil {
		return
	}
	if l, ok := event[zerolog.LevelFieldName].(string); ok {
		jPrio = levelToJPrio(l)
	}

	msg := ""
	for key, value := range event {
		jKey := strings.ToUpper(key)
		switch key {
		case zerolog.LevelFieldName, zerolog.TimestampFieldName:
			continue
		case zerolog.MessageFieldName:
			msg, _ = value.(string)
			continue
		}

		switch v := value.(type) {
		case string:
			args[jKey] = v
		case json.Number:
			args[jKey] = fmt.Sprint(value)
		default:
			b, err := zerolog.InterfaceMarshalFunc(value)
			if err != nil {
				args[jKey] = fmt.Sprintf("[error: %v]", err)
			} else {
				args[jKey] = string(b)
			}
		}
	}
	args["JSON"] = string(p)
	err = journal.Send(msg, jPrio, args)

	if err == nil {
		n = origPLen
	}

	return
}
