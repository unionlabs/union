// Package etw provides support for TraceLogging-based ETW (Event Tracing
// for Windows). TraceLogging is a format of ETW events that are self-describing
// (the event contains information on its own schema). This allows them to be
// decoded without needing a separate manifest with event information. The
// implementation here is based on the information found in
// TraceLoggingProvider.h in the Windows SDK, which implements TraceLogging as a
// set of C macros.
package etw
