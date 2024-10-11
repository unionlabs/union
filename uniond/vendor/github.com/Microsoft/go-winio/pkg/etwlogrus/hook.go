//go:build windows
// +build windows

package etwlogrus

import (
	"errors"
	"sort"

	"github.com/sirupsen/logrus"

	"github.com/Microsoft/go-winio/pkg/etw"
)

const defaultEventName = "LogrusEntry"

// ErrNoProvider is returned when a hook is created without a provider being configured.
var ErrNoProvider = errors.New("no ETW registered provider")

// HookOpt is an option to change the behavior of the Logrus ETW hook.
type HookOpt func(*Hook) error

// Hook is a Logrus hook which logs received events to ETW.
type Hook struct {
	provider      *etw.Provider
	closeProvider bool
	// allows setting the entry name
	getName func(*logrus.Entry) string
	// returns additional options to add to the event
	getEventsOpts func(*logrus.Entry) []etw.EventOpt
}

// NewHook registers a new ETW provider and returns a hook to log from it.
// The provider will be closed when the hook is closed.
func NewHook(providerName string, opts ...HookOpt) (*Hook, error) {
	opts = append(opts, WithNewETWProvider(providerName))

	return NewHookFromOpts(opts...)
}

// NewHookFromProvider creates a new hook based on an existing ETW provider.
// The provider will not be closed when the hook is closed.
func NewHookFromProvider(provider *etw.Provider, opts ...HookOpt) (*Hook, error) {
	opts = append(opts, WithExistingETWProvider(provider))

	return NewHookFromOpts(opts...)
}

// NewHookFromOpts creates a new hook with the provided options.
// An error is returned if the hook does not have a valid provider.
func NewHookFromOpts(opts ...HookOpt) (*Hook, error) {
	h := defaultHook()

	for _, o := range opts {
		if err := o(h); err != nil {
			return nil, err
		}
	}
	return h, h.validate()
}

func defaultHook() *Hook {
	h := &Hook{}
	return h
}

func (h *Hook) validate() error {
	if h.provider == nil {
		return ErrNoProvider
	}
	return nil
}

// Levels returns the set of levels that this hook wants to receive log entries
// for.
func (*Hook) Levels() []logrus.Level {
	return logrus.AllLevels
}

var logrusToETWLevelMap = map[logrus.Level]etw.Level{
	logrus.PanicLevel: etw.LevelAlways,
	logrus.FatalLevel: etw.LevelCritical,
	logrus.ErrorLevel: etw.LevelError,
	logrus.WarnLevel:  etw.LevelWarning,
	logrus.InfoLevel:  etw.LevelInfo,
	logrus.DebugLevel: etw.LevelVerbose,
	logrus.TraceLevel: etw.LevelVerbose,
}

// Fire receives each Logrus entry as it is logged, and logs it to ETW.
func (h *Hook) Fire(e *logrus.Entry) error {
	// Logrus defines more levels than ETW typically uses, but analysis is
	// easiest when using a consistent set of levels across ETW providers, so we
	// map the Logrus levels to ETW levels.
	level := logrusToETWLevelMap[e.Level]
	if !h.provider.IsEnabledForLevel(level) {
		return nil
	}

	name := defaultEventName
	if h.getName != nil {
		if n := h.getName(e); n != "" {
			name = n
		}
	}

	// extra room for two more options in addition to log level to avoid repeated reallocations
	// if the user also provides options
	opts := make([]etw.EventOpt, 0, 3)
	opts = append(opts, etw.WithLevel(level))
	if h.getEventsOpts != nil {
		opts = append(opts, h.getEventsOpts(e)...)
	}

	// Sort the fields by name so they are consistent in each instance
	// of an event. Otherwise, the fields don't line up in WPA.
	names := make([]string, 0, len(e.Data))
	hasError := false
	for k := range e.Data {
		if k == logrus.ErrorKey {
			// Always put the error last because it is optional in some events.
			hasError = true
		} else {
			names = append(names, k)
		}
	}
	sort.Strings(names)

	// Reserve extra space for the message and time fields.
	fields := make([]etw.FieldOpt, 0, len(e.Data)+2)
	fields = append(fields, etw.StringField("Message", e.Message))
	fields = append(fields, etw.Time("Time", e.Time))
	for _, k := range names {
		fields = append(fields, etw.SmartField(k, e.Data[k]))
	}
	if hasError {
		fields = append(fields, etw.SmartField(logrus.ErrorKey, e.Data[logrus.ErrorKey]))
	}

	// Firing an ETW event is essentially best effort, as the event write can
	// fail for reasons completely out of the control of the event writer (such
	// as a session listening for the event having no available space in its
	// buffers). Therefore, we don't return the error from WriteEvent, as it is
	// just noise in many cases.
	_ = h.provider.WriteEvent(name, opts, fields)

	return nil
}

// Close cleans up the hook and closes the ETW provider. If the provder was
// registered by etwlogrus, it will be closed as part of `Close`. If the
// provider was passed in, it will not be closed.
func (h *Hook) Close() error {
	if h.closeProvider {
		return h.provider.Close()
	}
	return nil
}
