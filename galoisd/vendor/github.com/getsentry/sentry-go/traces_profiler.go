package sentry

import (
	"sync"
	"time"
)

// Checks whether the transaction should be profiled (according to ProfilesSampleRate)
// and starts a profiler if so.
func (span *Span) sampleTransactionProfile() {
	var sampleRate = span.clientOptions().ProfilesSampleRate
	switch {
	case sampleRate < 0.0 || sampleRate > 1.0:
		Logger.Printf("Skipping transaction profiling: ProfilesSampleRate out of range [0.0, 1.0]: %f\n", sampleRate)
	case sampleRate == 0.0 || rng.Float64() >= sampleRate:
		Logger.Printf("Skipping transaction profiling: ProfilesSampleRate is: %f\n", sampleRate)
	default:
		startProfilerOnce.Do(startGlobalProfiler)
		if globalProfiler == nil {
			Logger.Println("Skipping transaction profiling: the profiler couldn't be started")
		} else {
			span.collectProfile = collectTransactionProfile
		}
	}
}

// transactionProfiler collects a profile for a given span.
type transactionProfiler func(span *Span) *profileInfo

var startProfilerOnce sync.Once
var globalProfiler profiler

func startGlobalProfiler() {
	globalProfiler = startProfiling(time.Now())
}

func collectTransactionProfile(span *Span) *profileInfo {
	result := globalProfiler.GetSlice(span.StartTime, span.EndTime)
	if result == nil || result.trace == nil {
		return nil
	}

	info := &profileInfo{
		Version: "1",
		EventID: uuid(),
		// See https://github.com/getsentry/sentry-go/pull/626#discussion_r1204870340 for explanation why we use the Transaction time.
		Timestamp: span.StartTime,
		Trace:     result.trace,
		Transaction: profileTransaction{
			DurationNS: uint64(span.EndTime.Sub(span.StartTime).Nanoseconds()),
			Name:       span.Name,
			TraceID:    span.TraceID.String(),
		},
	}
	if len(info.Transaction.Name) == 0 {
		// Name is required by Relay so use the operation name if the span name is empty.
		info.Transaction.Name = span.Op
	}
	if result.callerGoID > 0 {
		info.Transaction.ActiveThreadID = result.callerGoID
	}
	return info
}

func (info *profileInfo) UpdateFromEvent(event *Event) {
	info.Environment = event.Environment
	info.Platform = event.Platform
	info.Release = event.Release
	info.Dist = event.Dist
	info.Transaction.ID = event.EventID

	if runtimeContext, ok := event.Contexts["runtime"]; ok {
		if value, ok := runtimeContext["name"]; !ok {
			info.Runtime.Name = value.(string)
		}
		if value, ok := runtimeContext["version"]; !ok {
			info.Runtime.Version = value.(string)
		}
	}
	if osContext, ok := event.Contexts["os"]; ok {
		if value, ok := osContext["name"]; !ok {
			info.OS.Name = value.(string)
		}
	}
	if deviceContext, ok := event.Contexts["device"]; ok {
		if value, ok := deviceContext["arch"]; !ok {
			info.Device.Architecture = value.(string)
		}
	}
}
