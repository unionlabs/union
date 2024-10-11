//go:build windows

package etwlogrus

import (
	"github.com/sirupsen/logrus"

	"github.com/Microsoft/go-winio/pkg/etw"
)

// etw provider

// WithNewETWProvider registers a new ETW provider and sets the hook to log using it.
// The provider will be closed when the hook is closed.
func WithNewETWProvider(n string) HookOpt {
	return func(h *Hook) error {
		provider, err := etw.NewProvider(n, nil)
		if err != nil {
			return err
		}

		h.provider = provider
		h.closeProvider = true
		return nil
	}
}

// WithExistingETWProvider configures the hook to use an existing ETW provider.
// The provider will not be closed when the hook is closed.
func WithExistingETWProvider(p *etw.Provider) HookOpt {
	return func(h *Hook) error {
		h.provider = p
		h.closeProvider = false
		return nil
	}
}

// WithGetName sets the ETW EventName of an event to the value returned by f
// If the name is empty, the default event name will be used.
func WithGetName(f func(*logrus.Entry) string) HookOpt {
	return func(h *Hook) error {
		h.getName = f
		return nil
	}
}

// WithEventOpts allows additional ETW event properties (keywords, tags, etc.) to be specified.
func WithEventOpts(f func(*logrus.Entry) []etw.EventOpt) HookOpt {
	return func(h *Hook) error {
		h.getEventsOpts = f
		return nil
	}
}
