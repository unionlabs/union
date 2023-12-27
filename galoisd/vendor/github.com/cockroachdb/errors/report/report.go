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

package report

import (
	"fmt"
	"strings"

	"github.com/cockroachdb/errors/domains"
	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/withstack"
	"github.com/cockroachdb/redact"
	sentry "github.com/getsentry/sentry-go"
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
// //////////////
// Title: (1) some prefix in bold (2) one line for a stack trace
// (3) a single-line subtitle
//
// (4) the tags, as a tag soup (concatenated in a single paragraph,
// unsorted)
//
// (5) a "message"
//
// (6) zero or more "exceptions", each composed of:
//
//	(7) a bold title
//	(8) some freeform text
//	(9) a stack trace
//
// (10) metadata fields: environment, arch, etc
//
// (11) "Additional data" fields
//
// (12) SDK version
// ///////////////
//
// These visual items map to the Sentry Event object as follows:
//
// (1) the Type field of the 1st Exception object, if any
//
//	otherwise the Message field
//
// (2) the topmost entry from the Stacktrace field of the 1st Exception object, if any
// (3) the Value field of the 1st Exception object, if any, unwrapped as a single line
// (4) the Tags field
// (5) the Message field
// (7) the Type field (same as (1) for 1st exception)
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
// (3)/(8): first line of verbose error printout
// (4): not populated in this function, caller is to manage this
// (5): detailed structure of the entire error object, with references to
//
//	additional "exception" objects
//
// (9): generated from innermost stack trace
// (6): every exception object after the 1st reports additional stack trace contexts
// (11): the detailed error types and their error mark.
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
func BuildSentryReport(err error) (event *sentry.Event, extraDetails map[string]interface{}) {
	if err == nil {
		// No error: do nothing.
		return
	}

	// First step is to collect the details.
	var stacks []*withstack.ReportableStackTrace
	var details []errbase.SafeDetailPayload
	visitAllMulti(err, func(c error) {
		st := withstack.GetReportableStackTrace(c)
		stacks = append(stacks, st)

		sd := errbase.GetSafeDetails(c)
		details = append(details, sd)
	})
	module := string(domains.GetDomain(err))

	// firstDetailLine is the first detail string encountered.
	// This is added as decoration to the first Exception
	// payload (either from the error object or synthetic)
	// so as to populate the Sentry report title.
	var firstDetailLine string

	// longMsgBuf will become the Message field, which contains the full
	// structure of the error.
	var longMsgBuf strings.Builder
	if f, l, _, ok := withstack.GetOneLineSource(err); ok {
		fmt.Fprintf(&longMsgBuf, "%s:%d: ", f, l)
	}
	// Include the verbose error printout, with sensitive bits redacted out.
	verboseErr := redact.Sprintf("%+v", err).Redact().StripMarkers()
	if verboseErr != redactedMarker {
		idx := strings.IndexByte(verboseErr, '\n')
		if idx == -1 {
			firstDetailLine = verboseErr
		} else {
			firstDetailLine = verboseErr[:idx]
		}
	}
	fmt.Fprint(&longMsgBuf, verboseErr)

	// sep is used to separate the entries in the longMsgBuf / Message
	// payload.
	sep := ""

	// extras will become the per-layer "Additional data" fields.
	extras := make(map[string]interface{})

	// extraNum counts the number of "Additional data" payloads and is
	// used to generate the cross-reference counters in the Message
	// payload.
	extraNum := 1

	// typesBuf will become the payload for the "error types" Additional
	// data field. It explains the Go types of the layers in the error
	// object.
	var typesBuf strings.Builder

	// exceptions accumulates the Exception payloads.
	var exceptions []sentry.Exception

	// leafErrorType is the type name of the leaf error.
	// This is used as fallback when no Exception payload is generated.
	var leafErrorType string

	// Iterate from the last (innermost) to first (outermost) error
	// layer. We iterate in this order because we want to describe the
	// error from innermost to outermost layer in longMsgBuf and
	// typesBuf.
	longMsgBuf.WriteString("\n-- report composition:\n")
	for i := len(details) - 1; i >= 0; i-- {
		// Collect the type name for this layer of error wrapping, towards
		// the "error types" additional data field.
		fullTypeName := details[i].OriginalTypeName
		mark := details[i].ErrorTypeMark
		fm := "*"
		if fullTypeName != mark.FamilyName {
			// fullTypeName can be different from the family when an error type has
			// been renamed or moved.
			fm = mark.FamilyName
		}
		fmt.Fprintf(&typesBuf, "%s (%s::%s)\n", fullTypeName, fm, mark.Extension)
		shortTypename := lastPathComponent(fullTypeName)
		if i == len(details)-1 {
			leafErrorType = shortTypename
		}

		// Compose the Message line for this layer.
		//
		// The message line consists of:
		// - optionally, a file/line reference, if a stack trace was available.
		// - the error/wrapper type name, with file prefix removed.
		// - optionally, the first line of the first detail string, if one is available.
		// - optionally, references to stack trace / details.

		// If not at the first layer, separate from the previous layer
		// with a newline character.
		longMsgBuf.WriteString(sep)
		sep = "\n"
		// Add a file:lineno prefix, if there's a stack trace entry with
		// that info.
		var file, fn string
		var lineno int
		if stacks[i] != nil && len(stacks[i].Frames) > 0 {
			f := stacks[i].Frames[len(stacks[i].Frames)-1]
			file = lastPathComponent(f.Filename)
			fn = f.Function
			lineno = f.Lineno
			fmt.Fprintf(&longMsgBuf, "%s:%d: ", file, f.Lineno)
		}
		longMsgBuf.WriteString(shortTypename)

		// Now decide what kind of payload we want to add to the Event
		// object.

		// Is there a stack trace?
		if st := stacks[i]; st != nil {
			var excType strings.Builder
			if file != "" {
				fmt.Fprintf(&excType, "%s:%d ", file, lineno)
			}
			if fn != "" {
				fmt.Fprintf(&excType, "(%s)", fn)
			}
			if excType.Len() == 0 {
				excType.WriteString("<unknown error>")
			}
			exc := sentry.Exception{
				Module:     module,
				Stacktrace: st,
				Type:       excType.String(),
				Value:      shortTypename,
			}

			// Refer to the exception payload in the Message field.
			//
			// We only add a numeric counter for every exception *after* the
			// first one. This is because the 1st exception payload is
			// special, it is used as report title for Sentry and we don't
			// want to pollute that title with a counter.
			if len(exceptions) == 0 {
				longMsgBuf.WriteString(" (top exception)")
			} else {
				counterStr := fmt.Sprintf("(%d)", extraNum)
				extraNum++
				exc.Type = counterStr + " " + exc.Type
				fmt.Fprintf(&longMsgBuf, " %s", counterStr)
			}

			exceptions = append(exceptions, exc)
		} else {
			// No stack trace.
			// Are there safe details? If so, print the first safe detail
			// string (we're assuming that all the important bits will
			// also be included in the verbose printout, so there's no
			// need to dig out more safe strings here.)
			//
			// TODO(knz): the SafeDetails API is not really meant for Sentry
			// reporting. Once we have more experience to prove that the
			// verbose printout is sufficient, we can remove the SafeDetails
			// from sentry reports.
			//
			// Note: we only print the details if no stack trace was found
			// at that level. This is because stack trace annotations also
			// produce the stack trace as safe detail string.
			if len(details[i].SafeDetails) > 0 {
				d := details[i].SafeDetails[0]
				if j := strings.IndexByte(d, '\n'); j >= 0 {
					d = d[:j]
				}
				if d != "" {
					longMsgBuf.WriteString(": ")
					longMsgBuf.WriteString(d)
					if firstDetailLine == "" {
						// Keep the string for later.
						firstDetailLine = d
					}
				}
			}
		}
	}

	if extraNum > 1 {
		// Make the message part more informational.
		longMsgBuf.WriteString("\n(check the extra data payloads)")
	}

	// Produce the full error type description.
	extras["error types"] = typesBuf.String()

	// Sentry is mightily annoying.
	reverseExceptionOrder(exceptions)

	// Start assembling the event.
	event = sentry.NewEvent()
	event.Message = longMsgBuf.String()
	event.Exception = exceptions

	// If there is no exception payload, synthesize one.
	if len(event.Exception) == 0 {
		// We know we don't have a stack trace to extract line/function
		// info from (if we had, we'd have an Exception payload at that
		// point). Instead, we make a best effort using bits and pieces
		// assembled so far.
		event.Exception = append(event.Exception, sentry.Exception{
			Module: module,
			Type:   leafErrorType,
			Value:  firstDetailLine,
		})
	} else {
		// We have at least one exception payload already. In that case,
		// decorate the first exception with the first detail line if
		// there is one. This enhances the title of the Sentry report.
		//
		// This goes from:
		//    <file> (func)
		//    <type>
		//
		// to:
		//    <file> (func)
		//    wrapped <leaftype>[: <detail>]
		//    via <type>
		// if wrapped; or if leaf:
		//    <file> (func)
		//    <leaftype>[: <detail>]

		var newValueBuf strings.Builder
		// Note that "first exception" is the last item in the slice,
		// because... Sentry is annoying.
		firstExc := &event.Exception[len(event.Exception)-1]
		// Add the leaf error type if different from the type at this
		// level (this is going to be the common case, unless using
		// pkg/errors.WithStack).
		wrapped := false
		if firstExc.Value == leafErrorType {
			newValueBuf.WriteString(firstExc.Value)
		} else {
			newValueBuf.WriteString(leafErrorType)
			wrapped = true
		}
		// Add the detail info line, if any.
		if firstDetailLine != "" {
			fmt.Fprintf(&newValueBuf, ": %s", firstDetailLine)
		}
		if wrapped {
			fmt.Fprintf(&newValueBuf, "\nvia %s", firstExc.Value)
		}
		firstExc.Value = newValueBuf.String()
	}

	return event, extras
}

var redactedMarker = redact.RedactableString(redact.RedactedMarker()).StripMarkers()

// ReportError reports the given error to Sentry. The caller is responsible for
// checking whether telemetry is enabled, and calling the sentry.Flush()
// function to wait for the report to be uploaded. (By default,
// Sentry submits reports asynchronously.)
//
// Note: an empty 'eventID' can be returned which signifies that the error was
// not reported. This can occur when Sentry client hasn't been properly
// configured or Sentry client decided to not report the error (due to
// configured sampling rate, callbacks, Sentry's event processors, etc).
func ReportError(err error) (eventID string) {
	event, extraDetails := BuildSentryReport(err)

	for extraKey, extraValue := range extraDetails {
		event.Extra[extraKey] = extraValue
	}

	// Avoid leaking the machine's hostname by injecting the literal "<redacted>".
	// Otherwise, sentry.Client.Capture will see an empty ServerName field and
	// automatically fill in the machine's hostname.
	event.ServerName = "<redacted>"

	tags := map[string]string{
		"report_type": "error",
	}
	for key, value := range tags {
		event.Tags[key] = value
	}

	res := sentry.CaptureEvent(event)
	if res != nil {
		eventID = string(*res)
	}
	return
}

func lastPathComponent(tn string) string {
	// Strip the path prefix.
	if i := strings.LastIndexByte(tn, '/'); i >= 0 {
		tn = tn[i+1:]
	}
	return tn
}

func reverseExceptionOrder(ex []sentry.Exception) {
	for i := 0; i < len(ex)/2; i++ {
		ex[i], ex[len(ex)-i-1] = ex[len(ex)-i-1], ex[i]
	}
}

func visitAllMulti(err error, f func(error)) {
	f(err)
	if e := errbase.UnwrapOnce(err); e != nil {
		visitAllMulti(e, f)
	}
	for _, e := range errbase.UnwrapMulti(err) {
		visitAllMulti(e, f)
	}
}
