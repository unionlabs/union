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

// This file is forked and modified from golang.org/x/xerrors,
// at commit 3ee3066db522c6628d440a3a91c4abdd7f5ef22f (2019-05-10).
// From the original code:
// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
//
// Changes specific to this fork marked as inline comments.

package errbase

import (
	"bytes"
	"fmt"
	"io"
	"reflect"
	"strings"

	"github.com/cockroachdb/redact"
	"github.com/kr/pretty"
	pkgErr "github.com/pkg/errors"
)

// FormatError formats an error according to s and verb.
// This is a helper meant for use when implementing the fmt.Formatter
// interface on custom error objects.
//
// If the error implements errors.Formatter, FormatError calls its
// FormatError method of f with an errors.Printer configured according
// to s and verb, and writes the result to s.
//
// Otherwise, if it is a wrapper, FormatError prints out its error prefix,
// then recurses on its cause.
//
// Otherwise, its Error() text is printed.
func FormatError(err error, s fmt.State, verb rune) {
	formatErrorInternal(err, s, verb, false /* redactableOutput */)
}

// FormatRedactableError formats an error as a safe object.
//
// Note that certain verb/flags combinations are currently not
// supported, and result in a rendering that considers the entire
// object as unsafe. For example, %q, %#v are not yet supported.
func FormatRedactableError(err error, s redact.SafePrinter, verb rune) {
	formatErrorInternal(err, s, verb, true /* redactable */)
}

func init() {
	// Also inform the redact package of how to print an error
	// safely. This is used when an error is passed as argument
	// to one of the redact print functions.
	redact.RegisterRedactErrorFn(FormatRedactableError)
}

// Formattable wraps an error into a fmt.Formatter which
// will provide "smart" formatting even if the outer layer
// of the error does not implement the Formatter interface.
func Formattable(err error) fmt.Formatter {
	return &errorFormatter{err}
}

// formatErrorInternal is the shared logic between FormatError
// and FormatErrorRedactable.
//
// When the redactableOutput argument is true, the fmt.State argument
// is really a redact.SafePrinter and casted down as necessary.
//
// If verb and flags are not one of the supported error formatting
// combinations (in particular, %q, %#v etc), then the redactableOutput
// argument is ignored. This limitation may be lifted in a later
// version.
func formatErrorInternal(err error, s fmt.State, verb rune, redactableOutput bool) {
	// Assuming this function is only called from the Format method, and given
	// that FormatError takes precedence over Format, it cannot be called from
	// any package that supports errors.Formatter. It is therefore safe to
	// disregard that State may be a specific printer implementation and use one
	// of our choice instead.

	p := state{State: s, redactableOutput: redactableOutput}

	switch {
	case verb == 'v' && s.Flag('+') && !s.Flag('#'):
		// Here we are going to format as per %+v, into p.buf.
		//
		// We need to start with the innermost (root cause) error first,
		// then the layers of wrapping from innermost to outermost, so as
		// to enable stack trace de-duplication. This requires a
		// post-order traversal. Since we have a linked list, the best we
		// can do is a recursion.
		p.formatRecursive(err, true /* isOutermost */, true /* withDetail */)

		// We now have all the data, we can render the result.
		p.formatEntries(err)

		// We're done formatting. Apply width/precision parameters.
		p.finishDisplay(verb)

	case !redactableOutput && verb == 'v' && s.Flag('#'):
		// We only know how to process %#v if redactable output is not
		// requested. This is because the structured output may emit
		// arbitrary unsafe strings without redaction markers,
		// or improperly balanced/escaped redaction markers.
		if stringer, ok := err.(fmt.GoStringer); ok {
			io.WriteString(&p.finalBuf, stringer.GoString())
		} else {
			// Not a GoStringer: delegate to the pretty library.
			fmt.Fprintf(&p.finalBuf, "%# v", pretty.Formatter(err))
		}
		p.finishDisplay(verb)

	case verb == 's' ||
		// We only handle %v/%+v or other combinations here; %#v is unsupported.
		(verb == 'v' && !s.Flag('#')) ||
		// If redactable output is not requested, then we also
		// know how to format %x/%X (print bytes of error message in hex)
		// and %q (quote the result).
		// If redactable output is requested, then we don't know
		// how to perform these exotic verbs, because they
		// may muck with the redaction markers. In this case,
		// we simply refuse the format as per the default clause below.
		(!redactableOutput && (verb == 'x' || verb == 'X' || verb == 'q')):
		// Only the error message.
		//
		// Use an intermediate buffer because there may be alignment
		// instructions to obey in the final rendering or
		// quotes to add (for %q).
		//
		// Conceptually, we could just do
		//       p.buf.WriteString(err.Error())
		// However we also advertise that Error() can be implemented
		// by calling FormatError(), in which case we'd get an infinite
		// recursion. So we have no choice but to peel the data
		// and then assemble the pieces ourselves.
		p.formatRecursive(err, true /* isOutermost */, false /* withDetail */)
		p.formatSingleLineOutput()
		p.finishDisplay(verb)

	default:
		// Unknown verb. Do like fmt.Printf and tell the user we're
		// confused.
		//
		// Note that the following logic is correct regardless of the
		// value of 'redactableOutput', because the display of the verb and type
		// are always safe for redaction. If/when this code is changed to
		// print more details, care is to be taken to add redaction
		// markers if s.redactableOutput is set.
		p.finalBuf.WriteString("%!")
		p.finalBuf.WriteRune(verb)
		p.finalBuf.WriteByte('(')
		switch {
		case err != nil:
			p.finalBuf.WriteString(reflect.TypeOf(err).String())
		default:
			p.finalBuf.WriteString("<nil>")
		}
		p.finalBuf.WriteByte(')')
		io.Copy(s, &p.finalBuf)
	}
}

// formatEntries reads the entries from s.entries and produces a
// detailed rendering in s.finalBuf.
//
// Note that if s.redactableOutput is true, s.finalBuf is to contain a
// RedactableBytes. However, we are not using the helper facilities
// from redact.SafePrinter to do this, so care should be taken below
// to properly escape markers, etc.
func (s *state) formatEntries(err error) {
	// The first entry at the top is special. We format it as follows:
	//
	//   <complete error message>
	//   (1) <details>
	s.formatSingleLineOutput()
	s.finalBuf.WriteString("\n(1)")

	s.printEntry(s.entries[len(s.entries)-1])

	// All the entries that follow are printed as follows:
	//
	// Wraps: (N) <details>
	//
	for i, j := len(s.entries)-2, 2; i >= 0; i, j = i-1, j+1 {
		fmt.Fprintf(&s.finalBuf, "\nWraps: (%d)", j)
		entry := s.entries[i]
		s.printEntry(entry)
	}

	// At the end, we link all the (N) references to the Go type of the
	// error.
	s.finalBuf.WriteString("\nError types:")
	for i, j := len(s.entries)-1, 1; i >= 0; i, j = i-1, j+1 {
		fmt.Fprintf(&s.finalBuf, " (%d) %T", j, s.entries[i].err)
	}
}

// printEntry renders the entry given as argument
// into s.finalBuf.
//
// If s.redactableOutput is set, then s.finalBuf is to contain
// a RedactableBytes, with redaction markers. In that
// case, we must be careful to escape (or not) the entry
// depending on entry.redactable.
//
// If s.redactableOutput is unset, then we are not caring about
// redactability. In that case entry.redactable is not set
// anyway and we can pass contents through.
func (s *state) printEntry(entry formatEntry) {
	if len(entry.head) > 0 {
		if entry.head[0] != '\n' {
			s.finalBuf.WriteByte(' ')
		}
		if len(entry.head) > 0 {
			if !s.redactableOutput || entry.redactable {
				// If we don't care about redaction, then we can pass the string
				// through.
				//
				// If we do care about redaction, and entry.redactable is true,
				// then entry.head is already a RedactableBytes. Then we can
				// also pass it through.
				s.finalBuf.Write(entry.head)
			} else {
				// We care about redaction, and the head is unsafe. Escape it
				// and enclose the result within redaction markers.
				s.finalBuf.Write([]byte(redact.EscapeBytes(entry.head)))
			}
		}
	}
	if len(entry.details) > 0 {
		if len(entry.head) == 0 {
			if entry.details[0] != '\n' {
				s.finalBuf.WriteByte(' ')
			}
		}
		if !s.redactableOutput || entry.redactable {
			// If we don't care about redaction, then we can pass the string
			// through.
			//
			// If we do care about redaction, and entry.redactable is true,
			// then entry.details is already a RedactableBytes. Then we can
			// also pass it through.
			s.finalBuf.Write(entry.details)
		} else {
			// We care about redaction, and the details are unsafe. Escape
			// them and enclose the result within redaction markers.
			s.finalBuf.Write([]byte(redact.EscapeBytes(entry.details)))
		}
	}
	if entry.stackTrace != nil {
		s.finalBuf.WriteString("\n  -- stack trace:")
		s.finalBuf.WriteString(strings.ReplaceAll(
			fmt.Sprintf("%+v", entry.stackTrace),
			"\n", string(detailSep)))
		if entry.elidedStackTrace {
			fmt.Fprintf(&s.finalBuf, "%s[...repeated from below...]", detailSep)
		}
	}
}

// formatSingleLineOutput prints the details extracted via
// formatRecursive() through the chain of errors as if .Error() has
// been called: it only prints the non-detail parts and prints them on
// one line with ": " separators.
//
// This function is used both when FormatError() is called indirectly
// from .Error(), e.g. in:
//      (e *myType) Error() { return fmt.Sprintf("%v", e) }
//      (e *myType) Format(s fmt.State, verb rune) { errors.FormatError(s, verb, e) }
//
// and also to print the first line in the output of a %+v format.
//
// It reads from s.entries and writes to s.finalBuf.
// s.buf is left untouched.
//
// Note that if s.redactableOutput is true, s.finalBuf is to contain a
// RedactableBytes. However, we are not using the helper facilities
// from redact.SafePrinter to do this, so care should be taken below
// to properly escape markers, etc.
func (s *state) formatSingleLineOutput() {
	for i := len(s.entries) - 1; i >= 0; i-- {
		entry := &s.entries[i]
		if entry.elideShort {
			continue
		}
		if s.finalBuf.Len() > 0 && len(entry.head) > 0 {
			s.finalBuf.WriteString(": ")
		}
		if len(entry.head) == 0 {
			// shortcut, to avoid the copy below.
			continue
		}
		if !s.redactableOutput || entry.redactable {
			// If we don't care about redaction, then we can pass the string
			// through.
			//
			// If we do care about redaction, and entry.redactable is true,
			// then entry.head is already a RedactableBytes. Then we can
			// also pass it through.
			s.finalBuf.Write(entry.head)
		} else {
			// We do care about redaction, but entry.redactable is unset.
			// This means entry.head is unsafe. We need to escape it.
			s.finalBuf.Write([]byte(redact.EscapeBytes(entry.head)))
		}
	}
}

// formatRecursive performs a post-order traversal on the chain of
// errors to collect error details from innermost to outermost.
//
// It uses s.buf as an intermediate buffer to collect strings.
// It populates s.entries as a result.
// Between each layer of error, s.buf is reset.
//
// s.finalBuf is untouched. The conversion of s.entries
// to s.finalBuf is done by formatSingleLineOutput() and/or
// formatEntries().
func (s *state) formatRecursive(err error, isOutermost, withDetail bool) {
	cause := UnwrapOnce(err)
	if cause != nil {
		// Recurse first.
		s.formatRecursive(cause, false /*isOutermost*/, withDetail)
	}

	// Reinitialize the state for this stage of wrapping.
	s.wantDetail = withDetail
	s.needSpace = false
	s.needNewline = 0
	s.multiLine = false
	s.notEmpty = false
	s.hasDetail = false
	s.headBuf = nil

	seenTrace := false

	bufIsRedactable := false

	switch v := err.(type) {
	case SafeFormatter:
		bufIsRedactable = true
		desiredShortening := v.SafeFormatError((*safePrinter)(s))
		if desiredShortening == nil {
			// The error wants to elide the short messages from inner
			// causes. Do it.
			for i := range s.entries {
				s.entries[i].elideShort = true
			}
		}

	case Formatter:
		desiredShortening := v.FormatError((*printer)(s))
		if desiredShortening == nil {
			// The error wants to elide the short messages from inner
			// causes. Do it.
			for i := range s.entries {
				s.entries[i].elideShort = true
			}
		}

	case fmt.Formatter:
		// We can only use a fmt.Formatter when both the following
		// conditions are true:
		// - when it is the leaf error, because a fmt.Formatter
		//   on a wrapper also recurses.
		// - when it is not the outermost wrapper, because
		//   the Format() method is likely to be calling FormatError()
		//   to do its job and we want to avoid an infinite recursion.
		if !isOutermost && cause == nil {
			v.Format(s, 'v')
			if st, ok := err.(StackTraceProvider); ok {
				// This is likely a leaf error from github/pkg/errors.
				// The thing probably printed its stack trace on its own.
				seenTrace = true
				// We'll subsequently simplify stack traces in wrappers.
				s.lastStack = st.StackTrace()
			}
		} else {
			s.formatSimple(err, cause)
		}

	default:
		// Handle the special case overrides for context.Canceled,
		// os.PathError, etc for which we know how to extract some safe
		// strings.
		//
		// We need to do this in the `default` branch, instead of doing
		// this above the switch, because the special handler could call a
		// .Error() that delegates its implementation to fmt.Formatter,
		// errors.Safeformatter or errors.Formattable, which brings us
		// back to this method in a call cycle. So we need to handle the
		// various interfaces first.
		printDone := false
		for _, fn := range specialCases {
			if handled, desiredShortening := fn(err, (*safePrinter)(s), cause == nil /* leaf */); handled {
				printDone = true
				bufIsRedactable = true
				if desiredShortening == nil {
					// The error wants to elide the short messages from inner
					// causes. Do it.
					for i := range s.entries {
						s.entries[i].elideShort = true
					}
				}
				break
			}
		}
		if !printDone {
			// If the error did not implement errors.Formatter nor
			// fmt.Formatter, but it is a wrapper, still attempt best effort:
			// print what we can at this level.
			s.formatSimple(err, cause)
		}
	}

	// Collect the result.
	entry := s.collectEntry(err, bufIsRedactable)

	// If there's an embedded stack trace, also collect it.
	// This will get either a stack from pkg/errors, or ours.
	if !seenTrace {
		if st, ok := err.(StackTraceProvider); ok {
			entry.stackTrace, entry.elidedStackTrace = ElideSharedStackTraceSuffix(s.lastStack, st.StackTrace())
			s.lastStack = entry.stackTrace
		}
	}

	// Remember the entry for later rendering.
	s.entries = append(s.entries, entry)
	s.buf = bytes.Buffer{}
}

func (s *state) collectEntry(err error, bufIsRedactable bool) formatEntry {
	entry := formatEntry{err: err}
	if s.wantDetail {
		// The buffer has been populated as a result of formatting with
		// %+v. In that case, if the printer has separated detail
		// from non-detail, we can use the split.
		if s.hasDetail {
			entry.head = s.headBuf
			entry.details = s.buf.Bytes()
		} else {
			entry.head = s.buf.Bytes()
		}
	} else {
		entry.head = s.headBuf
		if len(entry.head) > 0 && entry.head[len(entry.head)-1] != '\n' &&
			s.buf.Len() > 0 && s.buf.Bytes()[0] != '\n' {
			entry.head = append(entry.head, '\n')
		}
		entry.head = append(entry.head, s.buf.Bytes()...)
	}

	if bufIsRedactable {
		// In this case, we've produced entry.head/entry.details using a
		// SafeFormatError() invocation. The strings in
		// entry.head/entry.detail contain redaction markers at this
		// point.
		if s.redactableOutput {
			// Redaction markers desired in the final output. Keep the
			// redaction markers.
			entry.redactable = true
		} else {
			// Markers not desired in the final output: strip the markers.
			entry.head = redact.RedactableBytes(entry.head).StripMarkers()
			entry.details = redact.RedactableBytes(entry.details).StripMarkers()
		}
	}

	return entry
}

// safeErrorPrinterFn is the type of a function that can take
// over the safe printing of an error. This is used to inject special
// cases into the formatting in errutil. We need this machinery to
// prevent import cycles.
type safeErrorPrinterFn = func(err error, p Printer, isLeaf bool) (handled bool, next error)

// specialCases is a list of functions to apply for special cases.
var specialCases []safeErrorPrinterFn

// RegisterSpecialCasePrinter registers a handler.
func RegisterSpecialCasePrinter(fn safeErrorPrinterFn) {
	specialCases = append(specialCases, fn)
}

// formatSimple performs a best effort at extracting the details at a
// given level of wrapping when the error object does not implement
// the Formatter interface.
func (s *state) formatSimple(err, cause error) {
	var pref string
	if cause != nil {
		pref = extractPrefix(err, cause)
	} else {
		pref = err.Error()
	}
	if len(pref) > 0 {
		s.Write([]byte(pref))
	}
}

// finishDisplay renders s.finalBuf into s.State.
func (p *state) finishDisplay(verb rune) {
	if p.redactableOutput {
		// If we're rendering in redactable form, then s.finalBuf contains
		// a RedactableBytes. We can emit that directly.
		sp := p.State.(redact.SafePrinter)
		sp.Print(redact.RedactableBytes(p.finalBuf.Bytes()))
		return
	}
	// Not redactable: render depending on flags and verb.

	width, okW := p.Width()
	_, okP := p.Precision()

	// If `direct` is set to false, then the buffer is always
	// passed through fmt.Printf regardless of the width and alignment
	// settings. This is important for e.g. %q where quotes must be added
	// in any case.
	// If `direct` is set to true, then the detour via
	// fmt.Printf only occurs if there is a width or alignment
	// specifier.
	direct := verb == 'v' || verb == 's'

	if !direct || (okW && width > 0) || okP {
		_, format := redact.MakeFormat(p, verb)
		fmt.Fprintf(p.State, format, p.finalBuf.String())
	} else {
		io.Copy(p.State, &p.finalBuf)
	}
}

var detailSep = []byte("\n  | ")

// state tracks error printing state. It implements fmt.State.
type state struct {
	// state inherits fmt.State.
	//
	// If we are rendering with redactableOutput=true, then fmt.State
	// can be downcasted to redact.SafePrinter.
	fmt.State

	// redactableOutput indicates whether we want the output
	// to use redaction markers. When set to true,
	// the fmt.State above is actually a redact.SafePrinter.
	redactableOutput bool

	// finalBuf contains the final rendered string, prior to being
	// copied to the fmt.State above.
	//
	// If redactableOutput is true, then finalBuf contains a RedactableBytes
	// and safe redaction markers. Otherwise, it can be considered
	// an unsafe string.
	finalBuf bytes.Buffer

	// entries collect the result of formatRecursive(). They are
	// consumed by formatSingleLineOutput() and formatEntries() to
	// procude the contents of finalBuf.
	entries []formatEntry

	// buf collects the details of the current error object at a given
	// stage of recursion in formatRecursive().
	//
	// At each stage of recursion (level of wrapping), buf contains
	// successively:
	//
	// - at the beginning, the "simple" part of the error message --
	//   either the pre-Detail() string if the error implements Formatter,
	//   or the result of Error().
	//
	// - after the first call to Detail(), buf is copied to headBuf,
	//   then reset, then starts collecting the "advanced" part of the
	//   error message.
	//
	// At the end of an error layer, the contents of buf and headBuf
	// are collected into a formatEntry by collectEntry().
	// This collection does not touch finalBuf above.
	//
	// The entries are later consumed by formatSingleLineOutput() or
	// formatEntries() to produce the contents of finalBuf.
	//
	//
	// Notes regarding redaction markers and string safety. Throughout a
	// single "level" of error, there are three cases to consider:
	//
	// - the error level implements SafeErrorFormatter and
	//   s.redactableOutput is set. In that case, the error's
	//   SafeErrorFormat() is used to produce a RedactableBytes in
	//   buf/headBuf via safePrinter{}, and an entry is collected at the
	//   end of that with the redactable bit set on the entry.
	//
	// - the error level implements SafeErrorFormatter
	//   and s.redactableOutput is *not* set. In this case,
	//   for convenience we implement non-redactable output by using
	//   SafeErrorFormat() to generate a RedactableBytes into
	//   buf/headBuf via safePrinter{}, and then stripping the redaction
	//   markers to produce the entry. The entry is not marked as
	//   redactable.
	//
	// - in the remaining case (s.redactableOutput is not set or the
	//   error only implements Formatter), then we use FormatError()
	//   to produce a non-redactable string into buf/headBuf,
	//   and mark the resulting entry as non-redactable.
	buf bytes.Buffer
	// When an error's FormatError() calls Detail(), the current
	// value of buf above is copied to headBuf, and a new
	// buf is initialized.
	headBuf []byte

	// lastStack tracks the last stack trace observed when looking at
	// the errors from innermost to outermost. This is used to elide
	// redundant stack trace entries.
	lastStack StackTrace

	// ---------------------------------------------------------------
	// The following attributes organize the synchronization of writes
	// to buf and headBuf, during the rendering of a single error
	// layer. They get reset between layers.

	// hasDetail becomes true at each level of the formatRecursive()
	// recursion after the first call to .Detail(). It is used to
	// determine how to translate buf/headBuf into a formatEntry.
	hasDetail bool

	// wantDetail is set to true when the error is formatted via %+v.
	// When false, printer.Detail() will always return false and the
	// error's .FormatError() method can perform less work. (This is an
	// optimization for the common case when an error's .Error() method
	// delegates its work to its .FormatError() via fmt.Format and
	// errors.FormatError().)
	wantDetail bool

	// collectingRedactableString is true iff the data being accumulated
	// into buf comes from a redact string. It ensures that newline
	// characters are not included inside redaction markers.
	collectingRedactableString bool

	// notEmpty tracks, at each level of recursion of formatRecursive(),
	// whether there were any details printed by an error's
	// .FormatError() method. It is used to properly determine whether
	// the printout should start with a newline and padding.
	notEmpty bool
	// needSpace tracks whether the next character displayed should pad
	// using a space character.
	needSpace bool
	// needNewline tracks whether the next character displayed should
	// pad using a newline and indentation.
	needNewline int
	// multiLine tracks whether the details so far contain multiple
	// lines. It is used to determine whether an enclosed stack trace,
	// if any, should be introduced with a separator.
	multiLine bool
}

// formatEntry collects the textual details about one level of
// wrapping or the leaf error in an error chain.
type formatEntry struct {
	err error
	// redactable is true iff the data in head and details
	// are RedactableBytes. See the explanatory comments
	// on (state).buf for when this is set.
	redactable bool
	// head is the part of the text that is suitable for printing in the
	// one-liner summary, or when producing the output of .Error().
	head []byte
	// details is the part of the text produced in the advanced output
	// included for `%+v` formats.
	details []byte
	// elideShort, if true, elides the value of 'head' from concatenated
	// "short" messages produced by formatSingleLineOutput().
	elideShort bool

	// stackTrace, if non-nil, reports the stack trace embedded at this
	// level of error.
	stackTrace StackTrace
	// elidedStackTrace, if true, indicates that the stack trace was
	// truncated to avoid duplication of entries. This is used to
	// display a truncation indicator during verbose rendering.
	elidedStackTrace bool
}

// String is used for debugging only.
func (e formatEntry) String() string {
	return fmt.Sprintf("formatEntry{%T, %q, %q}", e.err, e.head, e.details)
}

// Write implements io.Writer.
func (s *state) Write(b []byte) (n int, err error) {
	if len(b) == 0 {
		return 0, nil
	}
	k := 0

	sep := detailSep
	if !s.wantDetail {
		sep = []byte("\n")
	}

	for i, c := range b {
		if c == '\n' {
			// Flush all the bytes seen so far.
			s.buf.Write(b[k:i])
			// Don't print the newline itself; instead, prepare the state so
			// that the _next_ character encountered will pad with a newline.
			// This algorithm avoids terminating error details with excess
			// newline characters.
			k = i + 1
			s.needNewline++
			s.needSpace = false
			s.multiLine = true
			if s.wantDetail {
				s.switchOver()
			}
		} else {
			if s.needNewline > 0 && s.notEmpty {
				// If newline chars were pending, display them now.
				for i := 0; i < s.needNewline-1; i++ {
					s.buf.Write(detailSep[:len(sep)-1])
				}
				s.buf.Write(sep)
				s.needNewline = 0
				s.needSpace = false
			} else if s.needSpace {
				s.buf.WriteByte(' ')
				s.needSpace = false
			}
			s.notEmpty = true
		}
	}
	s.buf.Write(b[k:])
	return len(b), nil
}

// printer wraps a state to implement an xerrors.Printer.
type printer state

func (p *state) detail() bool {
	if !p.wantDetail {
		return false
	}
	if p.notEmpty {
		p.needNewline = 1
	}
	p.switchOver()
	return true
}

func (p *state) switchOver() {
	if p.hasDetail {
		return
	}
	p.headBuf = p.buf.Bytes()
	p.buf = bytes.Buffer{}
	p.notEmpty = false
	p.hasDetail = true
}

func (s *printer) Detail() bool {
	return ((*state)(s)).detail()
}

func (s *printer) Print(args ...interface{}) {
	s.enhanceArgs(args)
	fmt.Fprint((*state)(s), args...)
}

func (s *printer) Printf(format string, args ...interface{}) {
	s.enhanceArgs(args)
	fmt.Fprintf((*state)(s), format, args...)
}

func (s *printer) enhanceArgs(args []interface{}) {
	prevStack := s.lastStack
	lastSeen := prevStack
	for i := range args {
		if st, ok := args[i].(pkgErr.StackTrace); ok {
			args[i], _ = ElideSharedStackTraceSuffix(prevStack, st)
			lastSeen = st
		}
		if err, ok := args[i].(error); ok {
			args[i] = &errorFormatter{err}
		}
	}
	s.lastStack = lastSeen
}

// safePrinter is a variant to printer used when the current error
// level implements SafeFormatter.
//
// In any case, it uses the error's SafeFormatError() method to
// prepare a RedactableBytes into s.buf / s.headBuf.
// The the explanation for `buf` in the state struct.
type safePrinter state

func (s *safePrinter) Detail() bool {
	return ((*state)(s)).detail()
}

func (s *safePrinter) Print(args ...interface{}) {
	s.enhanceArgs(args)
	redact.Fprint((*state)(s), args...)
}

func (s *safePrinter) Printf(format string, args ...interface{}) {
	s.enhanceArgs(args)
	redact.Fprintf((*state)(s), format, args...)
}

func (s *safePrinter) enhanceArgs(args []interface{}) {
	prevStack := s.lastStack
	lastSeen := prevStack
	for i := range args {
		if st, ok := args[i].(pkgErr.StackTrace); ok {
			thisStack, _ := ElideSharedStackTraceSuffix(prevStack, st)
			// Stack traces are safe strings.
			args[i] = redact.Safe(thisStack)
			lastSeen = st
		}
		// In contrast with (*printer).enhanceArgs(), we dont use a
		// special case for `error` here, because the redact package
		// already helps us recursing into a safe print for
		// error objects.
	}
	s.lastStack = lastSeen
}

type errorFormatter struct{ err error }

// Format implements the fmt.Formatter interface.
func (ef *errorFormatter) Format(s fmt.State, verb rune) { FormatError(ef.err, s, verb) }

// Error implements error, so that `redact` knows what to do with it.
func (ef *errorFormatter) Error() string { return ef.err.Error() }

// Unwrap makes it a wrapper.
func (ef *errorFormatter) Unwrap() error { return ef.err }

// Cause makes it a wrapper.
func (ef *errorFormatter) Cause() error { return ef.err }

// ElideSharedStackTraceSuffix removes the suffix of newStack that's already
// present in prevStack. The function returns true if some entries
// were elided.
func ElideSharedStackTraceSuffix(prevStack, newStack StackTrace) (StackTrace, bool) {
	if len(prevStack) == 0 {
		return newStack, false
	}
	if len(newStack) == 0 {
		return newStack, false
	}

	// Skip over the common suffix.
	var i, j int
	for i, j = len(newStack)-1, len(prevStack)-1; i > 0 && j > 0; i, j = i-1, j-1 {
		if newStack[i] != prevStack[j] {
			break
		}
	}
	if i == 0 {
		// Keep at least one entry.
		i = 1
	}
	return newStack[:i], i < len(newStack)-1
}

// StackTrace is the type of the data for a call stack.
// This mirrors the type of the same name in github.com/pkg/errors.
type StackTrace = pkgErr.StackTrace

// StackFrame is the type of a single call frame entry.
// This mirrors the type of the same name in github.com/pkg/errors.
type StackFrame = pkgErr.Frame

// StackTraceProvider is a provider of StackTraces.
// This is, intendedly, defined to be implemented by pkg/errors.stack.
type StackTraceProvider interface {
	StackTrace() StackTrace
}
