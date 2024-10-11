// Copyright 2017 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package log

import (
	"errors"
	"fmt"
	"regexp"
	"runtime"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
)

// errVmoduleSyntax is returned when a user vmodule pattern is invalid.
var errVmoduleSyntax = errors.New("expect comma-separated list of filename=N")

// errTraceSyntax is returned when a user backtrace pattern is invalid.
var errTraceSyntax = errors.New("expect file.go:234")

// GlogHandler is a log handler that mimics the filtering features of Google's
// glog logger: setting global log levels; overriding with callsite pattern
// matches; and requesting backtraces at certain positions.
type GlogHandler struct {
	origin Handler // The origin handler this wraps

	level     atomic.Uint32 // Current log level, atomically accessible
	override  atomic.Bool   // Flag whether overrides are used, atomically accessible
	backtrace atomic.Bool   // Flag whether backtrace location is set

	patterns  []pattern       // Current list of patterns to override with
	siteCache map[uintptr]Lvl // Cache of callsite pattern evaluations
	location  string          // file:line location where to do a stackdump at
	lock      sync.RWMutex    // Lock protecting the override pattern list
}

// NewGlogHandler creates a new log handler with filtering functionality similar
// to Google's glog logger. The returned handler implements Handler.
func NewGlogHandler(h Handler) *GlogHandler {
	return &GlogHandler{
		origin: h,
	}
}

// SetHandler updates the handler to write records to the specified sub-handler.
func (h *GlogHandler) SetHandler(nh Handler) {
	h.origin = nh
}

// pattern contains a filter for the Vmodule option, holding a verbosity level
// and a file pattern to match.
type pattern struct {
	pattern *regexp.Regexp
	level   Lvl
}

// Verbosity sets the glog verbosity ceiling. The verbosity of individual packages
// and source files can be raised using Vmodule.
func (h *GlogHandler) Verbosity(level Lvl) {
	h.level.Store(uint32(level))
}

// Vmodule sets the glog verbosity pattern.
//
// The syntax of the argument is a comma-separated list of pattern=N, where the
// pattern is a literal file name or "glob" pattern matching and N is a V level.
//
// For instance:
//
//	pattern="gopher.go=3"
//	 sets the V level to 3 in all Go files named "gopher.go"
//
//	pattern="foo=3"
//	 sets V to 3 in all files of any packages whose import path ends in "foo"
//
//	pattern="foo/*=3"
//	 sets V to 3 in all files of any packages whose import path contains "foo"
func (h *GlogHandler) Vmodule(ruleset string) error {
	var filter []pattern
	for _, rule := range strings.Split(ruleset, ",") {
		// Empty strings such as from a trailing comma can be ignored
		if len(rule) == 0 {
			continue
		}
		// Ensure we have a pattern = level filter rule
		parts := strings.Split(rule, "=")
		if len(parts) != 2 {
			return errVmoduleSyntax
		}
		parts[0] = strings.TrimSpace(parts[0])
		parts[1] = strings.TrimSpace(parts[1])
		if len(parts[0]) == 0 || len(parts[1]) == 0 {
			return errVmoduleSyntax
		}
		// Parse the level and if correct, assemble the filter rule
		level, err := strconv.Atoi(parts[1])
		if err != nil {
			return errVmoduleSyntax
		}
		if level <= 0 {
			continue // Ignore. It's harmless but no point in paying the overhead.
		}
		// Compile the rule pattern into a regular expression
		matcher := ".*"
		for _, comp := range strings.Split(parts[0], "/") {
			if comp == "*" {
				matcher += "(/.*)?"
			} else if comp != "" {
				matcher += "/" + regexp.QuoteMeta(comp)
			}
		}
		if !strings.HasSuffix(parts[0], ".go") {
			matcher += "/[^/]+\\.go"
		}
		matcher = matcher + "$"

		re, _ := regexp.Compile(matcher)
		filter = append(filter, pattern{re, Lvl(level)})
	}
	// Swap out the vmodule pattern for the new filter system
	h.lock.Lock()
	defer h.lock.Unlock()

	h.patterns = filter
	h.siteCache = make(map[uintptr]Lvl)
	h.override.Store(len(filter) != 0)
	// Enable location storage (globally)
	if len(h.patterns) > 0 {
		stackEnabled.Store(true)
	}
	return nil
}

// BacktraceAt sets the glog backtrace location. When set to a file and line
// number holding a logging statement, a stack trace will be written to the Info
// log whenever execution hits that statement.
//
// Unlike with Vmodule, the ".go" must be present.
func (h *GlogHandler) BacktraceAt(location string) error {
	// Ensure the backtrace location contains two non-empty elements
	parts := strings.Split(location, ":")
	if len(parts) != 2 {
		return errTraceSyntax
	}
	parts[0] = strings.TrimSpace(parts[0])
	parts[1] = strings.TrimSpace(parts[1])
	if len(parts[0]) == 0 || len(parts[1]) == 0 {
		return errTraceSyntax
	}
	// Ensure the .go prefix is present and the line is valid
	if !strings.HasSuffix(parts[0], ".go") {
		return errTraceSyntax
	}
	if _, err := strconv.Atoi(parts[1]); err != nil {
		return errTraceSyntax
	}
	// All seems valid
	h.lock.Lock()
	defer h.lock.Unlock()

	h.location = location
	h.backtrace.Store(len(location) > 0)
	// Enable location storage (globally)
	stackEnabled.Store(true)
	return nil
}

// Log implements Handler.Log, filtering a log record through the global, local
// and backtrace filters, finally emitting it if either allow it through.
func (h *GlogHandler) Log(r *Record) error {
	// If backtracing is requested, check whether this is the callsite
	if h.backtrace.Load() {
		// Everything below here is slow. Although we could cache the call sites the
		// same way as for vmodule, backtracing is so rare it's not worth the extra
		// complexity.
		h.lock.RLock()
		match := h.location == r.Call.String()
		h.lock.RUnlock()

		if match {
			// Callsite matched, raise the log level to info and gather the stacks
			r.Lvl = LvlInfo

			buf := make([]byte, 1024*1024)
			buf = buf[:runtime.Stack(buf, true)]
			r.Msg += "\n\n" + string(buf)
		}
	}
	// If the global log level allows, fast track logging
	if h.level.Load() >= uint32(r.Lvl) {
		return h.origin.Log(r)
	}
	// If no local overrides are present, fast track skipping
	if !h.override.Load() {
		return nil
	}
	// Check callsite cache for previously calculated log levels
	h.lock.RLock()
	lvl, ok := h.siteCache[r.Call.Frame().PC]
	h.lock.RUnlock()

	// If we didn't cache the callsite yet, calculate it
	if !ok {
		h.lock.Lock()
		for _, rule := range h.patterns {
			if rule.pattern.MatchString(fmt.Sprintf("%+s", r.Call)) {
				h.siteCache[r.Call.Frame().PC], lvl, ok = rule.level, rule.level, true
				break
			}
		}
		// If no rule matched, remember to drop log the next time
		if !ok {
			h.siteCache[r.Call.Frame().PC] = 0
		}
		h.lock.Unlock()
	}
	if lvl >= r.Lvl {
		return h.origin.Log(r)
	}
	return nil
}
