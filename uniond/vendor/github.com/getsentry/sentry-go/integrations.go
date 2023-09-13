package sentry

import (
	"fmt"
	"regexp"
	"runtime"
	"runtime/debug"
	"strings"
	"sync"
)

// ================================
// Modules Integration
// ================================

type modulesIntegration struct {
	once    sync.Once
	modules map[string]string
}

func (mi *modulesIntegration) Name() string {
	return "Modules"
}

func (mi *modulesIntegration) SetupOnce(client *Client) {
	client.AddEventProcessor(mi.processor)
}

func (mi *modulesIntegration) processor(event *Event, hint *EventHint) *Event {
	if len(event.Modules) == 0 {
		mi.once.Do(func() {
			info, ok := debug.ReadBuildInfo()
			if !ok {
				Logger.Print("The Modules integration is not available in binaries built without module support.")
				return
			}
			mi.modules = extractModules(info)
		})
	}
	event.Modules = mi.modules
	return event
}

func extractModules(info *debug.BuildInfo) map[string]string {
	modules := map[string]string{
		info.Main.Path: info.Main.Version,
	}
	for _, dep := range info.Deps {
		ver := dep.Version
		if dep.Replace != nil {
			ver += fmt.Sprintf(" => %s %s", dep.Replace.Path, dep.Replace.Version)
		}
		modules[dep.Path] = strings.TrimSuffix(ver, " ")
	}
	return modules
}

// ================================
// Environment Integration
// ================================

type environmentIntegration struct{}

func (ei *environmentIntegration) Name() string {
	return "Environment"
}

func (ei *environmentIntegration) SetupOnce(client *Client) {
	client.AddEventProcessor(ei.processor)
}

func (ei *environmentIntegration) processor(event *Event, hint *EventHint) *Event {
	// Initialize maps as necessary.
	contextNames := []string{"device", "os", "runtime"}
	if event.Contexts == nil {
		event.Contexts = make(map[string]Context, len(contextNames))
	}
	for _, name := range contextNames {
		if event.Contexts[name] == nil {
			event.Contexts[name] = make(Context)
		}
	}

	// Set contextual information preserving existing data. For each context, if
	// the existing value is not of type map[string]interface{}, then no
	// additional information is added.
	if deviceContext, ok := event.Contexts["device"]; ok {
		if _, ok := deviceContext["arch"]; !ok {
			deviceContext["arch"] = runtime.GOARCH
		}
		if _, ok := deviceContext["num_cpu"]; !ok {
			deviceContext["num_cpu"] = runtime.NumCPU()
		}
	}
	if osContext, ok := event.Contexts["os"]; ok {
		if _, ok := osContext["name"]; !ok {
			osContext["name"] = runtime.GOOS
		}
	}
	if runtimeContext, ok := event.Contexts["runtime"]; ok {
		if _, ok := runtimeContext["name"]; !ok {
			runtimeContext["name"] = "go"
		}
		if _, ok := runtimeContext["version"]; !ok {
			runtimeContext["version"] = runtime.Version()
		}
		if _, ok := runtimeContext["go_numroutines"]; !ok {
			runtimeContext["go_numroutines"] = runtime.NumGoroutine()
		}
		if _, ok := runtimeContext["go_maxprocs"]; !ok {
			runtimeContext["go_maxprocs"] = runtime.GOMAXPROCS(0)
		}
		if _, ok := runtimeContext["go_numcgocalls"]; !ok {
			runtimeContext["go_numcgocalls"] = runtime.NumCgoCall()
		}
	}
	return event
}

// ================================
// Ignore Errors Integration
// ================================

type ignoreErrorsIntegration struct {
	ignoreErrors []*regexp.Regexp
}

func (iei *ignoreErrorsIntegration) Name() string {
	return "IgnoreErrors"
}

func (iei *ignoreErrorsIntegration) SetupOnce(client *Client) {
	iei.ignoreErrors = transformStringsIntoRegexps(client.options.IgnoreErrors)
	client.AddEventProcessor(iei.processor)
}

func (iei *ignoreErrorsIntegration) processor(event *Event, hint *EventHint) *Event {
	suspects := getIgnoreErrorsSuspects(event)

	for _, suspect := range suspects {
		for _, pattern := range iei.ignoreErrors {
			if pattern.Match([]byte(suspect)) {
				Logger.Printf("Event dropped due to being matched by `IgnoreErrors` option."+
					"| Value matched: %s | Filter used: %s", suspect, pattern)
				return nil
			}
		}
	}

	return event
}

func transformStringsIntoRegexps(strings []string) []*regexp.Regexp {
	var exprs []*regexp.Regexp

	for _, s := range strings {
		r, err := regexp.Compile(s)
		if err == nil {
			exprs = append(exprs, r)
		}
	}

	return exprs
}

func getIgnoreErrorsSuspects(event *Event) []string {
	suspects := []string{}

	if event.Message != "" {
		suspects = append(suspects, event.Message)
	}

	for _, ex := range event.Exception {
		suspects = append(suspects, ex.Type, ex.Value)
	}

	return suspects
}

// ================================
// Contextify Frames Integration
// ================================

type contextifyFramesIntegration struct {
	sr              sourceReader
	contextLines    int
	cachedLocations sync.Map
}

func (cfi *contextifyFramesIntegration) Name() string {
	return "ContextifyFrames"
}

func (cfi *contextifyFramesIntegration) SetupOnce(client *Client) {
	cfi.sr = newSourceReader()
	cfi.contextLines = 5

	client.AddEventProcessor(cfi.processor)
}

func (cfi *contextifyFramesIntegration) processor(event *Event, hint *EventHint) *Event {
	// Range over all exceptions
	for _, ex := range event.Exception {
		// If it has no stacktrace, just bail out
		if ex.Stacktrace == nil {
			continue
		}

		// If it does, it should have frames, so try to contextify them
		ex.Stacktrace.Frames = cfi.contextify(ex.Stacktrace.Frames)
	}

	// Range over all threads
	for _, th := range event.Threads {
		// If it has no stacktrace, just bail out
		if th.Stacktrace == nil {
			continue
		}

		// If it does, it should have frames, so try to contextify them
		th.Stacktrace.Frames = cfi.contextify(th.Stacktrace.Frames)
	}

	return event
}

func (cfi *contextifyFramesIntegration) contextify(frames []Frame) []Frame {
	contextifiedFrames := make([]Frame, 0, len(frames))

	for _, frame := range frames {
		if !frame.InApp {
			contextifiedFrames = append(contextifiedFrames, frame)
			continue
		}

		var path string

		if cachedPath, ok := cfi.cachedLocations.Load(frame.AbsPath); ok {
			if p, ok := cachedPath.(string); ok {
				path = p
			}
		} else {
			// Optimize for happy path here
			if fileExists(frame.AbsPath) {
				path = frame.AbsPath
			} else {
				path = cfi.findNearbySourceCodeLocation(frame.AbsPath)
			}
		}

		if path == "" {
			contextifiedFrames = append(contextifiedFrames, frame)
			continue
		}

		lines, contextLine := cfi.sr.readContextLines(path, frame.Lineno, cfi.contextLines)
		contextifiedFrames = append(contextifiedFrames, cfi.addContextLinesToFrame(frame, lines, contextLine))
	}

	return contextifiedFrames
}

func (cfi *contextifyFramesIntegration) findNearbySourceCodeLocation(originalPath string) string {
	trimmedPath := strings.TrimPrefix(originalPath, "/")
	components := strings.Split(trimmedPath, "/")

	for len(components) > 0 {
		components = components[1:]
		possibleLocation := strings.Join(components, "/")

		if fileExists(possibleLocation) {
			cfi.cachedLocations.Store(originalPath, possibleLocation)
			return possibleLocation
		}
	}

	cfi.cachedLocations.Store(originalPath, "")
	return ""
}

func (cfi *contextifyFramesIntegration) addContextLinesToFrame(frame Frame, lines [][]byte, contextLine int) Frame {
	for i, line := range lines {
		switch {
		case i < contextLine:
			frame.PreContext = append(frame.PreContext, string(line))
		case i == contextLine:
			frame.ContextLine = string(line)
		default:
			frame.PostContext = append(frame.PostContext, string(line))
		}
	}
	return frame
}
