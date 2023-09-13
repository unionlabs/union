// Copyright 2019 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package errors

import (
	"github.com/cockroachdb/errors/report"
	"github.com/getsentry/sentry-go"
)

// BuildSentryReport builds the components of a sentry report.  This
// can be used instead of ReportError() below to use additional custom
// conditions in the reporting or add additional reporting tags.
//
// The Sentry Event is populated for maximal utility when exploited in
// the Sentry.io web interface and database.
//
// A Sentry report is displayed visually in the Sentry UI as follows:
//
////////////////
// Title: (1) some prefix in bold (2) one line for a stack trace
// (3) a single-line subtitle
//
// (4) the tags, as a tag soup (concatenated in a single paragrah,
// unsorted)
//
// (5) a "message"
//
// (6) zero or more "exceptions", each composed of:
//    (7) a bold title
//    (8) some freeform text
//    (9) a stack trace
//
// (10) metadata fields: environment, arch, etc
//
// (11) "Additional data" fields
//
// (12) SDK version
/////////////////
//
// These visual items map to the Sentry Event object as follows:
//
// (1) the Type field of the 1st Exception object, if any
//     otherwise the Message field
// (2) the topmost entry from the Stacktrace field of the 1st Exception object, if any
// (3) the Value field of the 1st Exception object, if any, unwrapped as a single line
// (4) the Tags field
// (5) the Message field
// (7) the Type field (same as (1) for 1st execption)
// (8) the Value field (same as (3) for 1st exception)
// (9) the Stacktrace field (input to (2) on 1st exception)
// (10) the other fields on the Event object
// (11) the Extra field
//
// (Note how the top-level title fields (1) (3) are unrelated to the
// Message field in the event, which is surprising!)
//
// Given this mapping, an error object is decomposed as follows:
//
// (1)/(7): <filename>:<lineno> (<functionname>)
// (3)/(8): <error type>: <first safe detail line, if any>
// (4): not populated in this function, caller is to manage this
// (5): detailed structure of the entire error object, with references to "additional data"
//      and additional "exception" objects
// (9): generated from innermost stack trace
// (6): every exception object after the 1st reports additional stack trace contexts
// (11): "additional data" populated from safe detail payloads
//
// If there is no stack trace in the error, a synthetic Exception
// object is still produced to provide visual detail in the Sentry UI.
//
// Note that if a layer in the error has both a stack trace (ie
// provides the `StackTrace()` interface) and also safe details
// (`SafeDetails()`) other than the stack trace, only the stack trace
// is included in the Sentry report. This does not affect error types
// provided by the library, but could impact error types defined by
// 3rd parties. This limitation may be lifted in a later version.
//
func BuildSentryReport(err error) (*sentry.Event, map[string]interface{}) {
	return report.BuildSentryReport(err)
}

// ReportError reports the given error to Sentry. The caller is responsible for
// checking whether telemetry is enabled, and calling the sentry.Flush()
// function to wait for the report to be uploaded. (By default,
// Sentry submits reports asynchronously.)
//
// Note: an empty 'eventID' can be returned which signifies that the error was
// not reported. This can occur when Sentry client hasn't been properly
// configured or Sentry client decided to not report the error (due to
// configured sampling rate, callbacks, Sentry's event processors, etc).
func ReportError(err error) string { return report.ReportError(err) }
