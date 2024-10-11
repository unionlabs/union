// Copyright 2020 The Cockroach Authors.
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

package fmttests

import (
	"bytes"
	"context"
	goErr "errors"
	"flag"
	"fmt"
	"net"
	"os"
	"regexp"
	"sort"
	"strings"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors/barriers"
	"github.com/cockroachdb/errors/contexttags"
	"github.com/cockroachdb/errors/domains"
	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/errutil"
	"github.com/cockroachdb/errors/hintdetail"
	"github.com/cockroachdb/errors/issuelink"
	"github.com/cockroachdb/errors/join"
	"github.com/cockroachdb/errors/report"
	"github.com/cockroachdb/errors/safedetails"
	"github.com/cockroachdb/errors/secondary"
	"github.com/cockroachdb/errors/telemetrykeys"
	"github.com/cockroachdb/errors/withstack"
	"github.com/cockroachdb/logtags"
	"github.com/cockroachdb/redact"
	"github.com/getsentry/sentry-go"
	"github.com/kr/pretty"
	pkgErr "github.com/pkg/errors"
)

// The -generate flag generates the test input files from scratch.
//
// NB: it is advised to not use -generate if the test suite as already generated
// does not pass. Consider examining failure and/or using -rewrite until
// the tests pass, then use -generate.
//
// This is because investigating test failures during generation makes
// troubleshooting errors more difficult.
var generateTestFiles = flag.Bool(
	"generate", false,
	"generate the error formatting tests",
)

var leafCommands = map[string]commandFn{
	"goerr": func(_ error, args []arg) error { return goErr.New(strfy(args)) },

	"os-invalid":    func(_ error, _ []arg) error { return os.ErrInvalid },
	"os-permission": func(_ error, _ []arg) error { return os.ErrPermission },
	"os-exist":      func(_ error, _ []arg) error { return os.ErrExist },
	"os-notexist":   func(_ error, _ []arg) error { return os.ErrNotExist },
	"os-closed":     func(_ error, _ []arg) error { return os.ErrClosed },
	"ctx-canceled":  func(_ error, _ []arg) error { return context.Canceled },
	"ctx-deadline":  func(_ error, _ []arg) error { return context.DeadlineExceeded },

	"pkgerr": func(err error, args []arg) error { return pkgErr.New(strfy(args)) },
	// errNoFmt does neither implement Format() nor FormatError().
	"nofmt": func(_ error, args []arg) error { return &errNoFmt{strfy(args)} },
	// errFmto has just a Format() method that does everything, and does
	// not know about neither FormatError() nor errors.Formatter.
	// This is the "old style" support for formatting, e.g. used
	// in github.com/pkg/errors.
	"fmt-old": func(_ error, args []arg) error { return &errFmto{strfy(args)} },
	// errFmtoDelegate is like errFmto but the Error() method delegates to
	// Format().
	"fmt-old-delegate": func(_ error, args []arg) error { return &errFmtoDelegate{strfy(args)} },
	// errFmtp implements Format() that forwards to FormatError(),
	// but does not implement errors.Formatter. It is used
	// to check that FormatError() does the right thing.
	"fmt-partial": func(_ error, args []arg) error { return &errFmtp{strfy(args)} },
	// errFmt has both Format() and FormatError(),
	// and demonstrates the common case of "rich" errors.
	"fmt": func(_ error, args []arg) error { return &errFmt{strfy(args)} },
	// errSafeFormat has SafeFormatError(), and has Error() and Format()
	// redirect to that.
	"safefmt": func(_ error, args []arg) error { return &errSafeFormat{strfy(args)} },

	// errutil.New implements multi-layer errors.
	"newf": func(_ error, args []arg) error { return errutil.Newf("new-style %s", strfy(args)) },
	// assertions mask their cause from the barriers, but otherwise format as-is.
	"assertion": func(_ error, args []arg) error { return errutil.AssertionFailedf("assertmsg %s", strfy(args)) },
	// newf-attached embeds its error argument as extra payload.
	"newf-attached": func(_ error, args []arg) error {
		return errutil.Newf("new-style %s: %v", strfy(args), errutil.New("payload"))
	},

	"unimplemented": func(_ error, args []arg) error {
		return issuelink.UnimplementedErrorf(issuelink.IssueLink{IssueURL: "https://mysite", Detail: "issuedetails"}, strfy(args))
	},
}

var leafOnlyExceptions = map[string]string{}

func init() {
	for leafName := range leafCommands {
		// The following leaf types don't implement FormatError(), so they don't get
		// enhanced displays via %+v automatically. Formattable() adds it to them.
		// So it is expected that %+v via Formattable() is different (better) than
		// %+v on its own.
		//
		if leafName == "nofmt" || strings.HasPrefix(leafName, "os-") || strings.HasPrefix(leafName, "ctx-") {
			leafOnlyExceptions[leafName] = `accept %\+v via Formattable.*IRREGULAR: not same as %\+v`
		}
	}

	//
	// Additionally, the Go idea of %#v is somewhat broken: this is specified
	// in printf() docstrings to always emit a "Go representation". However,
	// if the object implements Formatter(), this takes over the display of %#v
	// too. Most implementation of Format() are incomplete and unable to
	// emit a "Go representation", so this breaks.
	//
	for _, v := range []string{"goerr", "fmt-old", "fmt-old-delegate"} {
		leafOnlyExceptions[v] = `
accept %\+v via Formattable.*IRREGULAR: not same as %\+v
accept %\#v via Formattable.*IRREGULAR: not same as %\#v
`
	}

	// The simple leaf type from github/pkg/errors does not properly implement
	// Format(). So it does not know how to format %x/%X properly.
	leafOnlyExceptions[`pkgerr`] = `
accept %x.*IRREGULAR: not same as hex Error
accept %X.*IRREGULAR: not same as HEX Error
accept %\#v via Formattable.*IRREGULAR: not same as %\#v
accept %\+v via Formattable.*IRREGULAR: not same as %\+v`
}

var wrapCommands = map[string]commandFn{
	"goerr": func(err error, args []arg) error { return fmt.Errorf("%s: %w", strfy(args), err) },
	// errorf constructs a wrapper that encodes as an opaqueWrapper with
	// `ownsErrorString` set to true. This is because the error string of the
	// wrapper is not a strict prefix with `: ` of the wrapped error.
	"go-errorf": func(e error, args []arg) error {
		return fmt.Errorf("%s - %w", strfy(args), e)
	},
	"go-errorf-suffix": func(e error, args []arg) error {
		return fmt.Errorf("%w - %s", e, strfy(args))
	},
	"go-errorf-multi": func(err error, args []arg) error {
		return fmt.Errorf("%s - %w %w", strfy(args), err, pkgErr.New("sibling error in wrapper"))
	},
	"opaque": func(err error, _ []arg) error {
		return errbase.DecodeError(context.Background(),
			errbase.EncodeError(context.Background(), err))
	},
	"os-syscall": func(err error, _ []arg) error { return os.NewSyscallError("open", err) },
	"os-link": func(err error, _ []arg) error {
		return &os.LinkError{Op: "link", Old: "/path/to/file", New: "/path/to/newfile", Err: err}
	},
	"os-path": func(err error, _ []arg) error { return &os.PathError{Op: "link", Path: "/path/to/file", Err: err} },
	"os-netop": func(err error, _ []arg) error {
		return &net.OpError{Op: "send", Net: "tcp", Addr: &net.UnixAddr{Name: "unixhello", Net: "unixgram"}, Err: err}
	},
	"pkgmsg":   func(err error, args []arg) error { return pkgErr.WithMessage(err, strfy(args)) },
	"pkgstack": func(err error, _ []arg) error { return pkgErr.WithStack(err) },
	// werrNoFmt does neither implement Format() nor FormatError().
	"nofmt": func(err error, args []arg) error { return &werrNoFmt{err, strfy(args)} },
	// werrFmto has just a Format() method that does everything, and does
	// not know about neither FormatError() nor errors.Formatter.
	// This is the "old style" support for formatting, e.g. used
	// in github.com/pkg/errors.
	"fmt-old": func(err error, args []arg) error { return &werrFmto{err, strfy(args)} },
	// werrFmtoDelegate is like errFmto but the Error() method delegates to
	// Format().
	"fmt-old-delegate": func(err error, args []arg) error { return &werrFmtoDelegate{err, strfy(args)} },
	// werrFmtp implements Format() that forwards to FormatError(),
	// but does not implement errors.Formatter. It is used
	// to check that FormatError() does the right thing.
	"fmt-partial": func(err error, args []arg) error { return &werrFmtp{err, strfy(args)} },
	// werrFmt has both Format() and FormatError(),
	// and demonstrates the common case of "rich" errors.
	"fmt": func(err error, args []arg) error { return &werrFmt{err, strfy(args)} },
	// werrSafeFormat has SafeFormatError(), and has Error() and Format()
	// redirect to that.
	"safefmt": func(err error, args []arg) error { return &werrSafeFormat{cause: err, msg: strfy(args)} },
	// werrEmpty has no message of its own. Its Error() is implemented via its cause.
	"empty": func(err error, _ []arg) error { return &werrEmpty{err} },
	// werrDelegate delegates its Error() behavior to FormatError().
	"delegate": func(err error, args []arg) error { return &werrDelegate{err, strfy(args)} },
	// werrDelegate-noprefix delegates its Error() behavior to FormatError() via fmt.Format, has
	// no prefix of its own in its short message but has a detail field.
	"delegate-noprefix": func(err error, _ []arg) error { return &werrDelegateNoPrefix{err} },
	// werrDelegateEmpty implements Error via fmt.Formatter using FormatError,
	// and has no message nor detail of its own.
	"delegate-empty": func(err error, _ []arg) error { return &werrDelegateEmpty{err} },
	// werrWithElidedClause overrides its cause's Error() from its own
	// short message.
	"elided-cause": func(err error, args []arg) error { return &werrWithElidedCause{err, strfy(args)} },
	"multi-cause": func(err error, args []arg) error {
		return newMultiCause("A", false, /* elide */
			newMultiCause("C", false /* elide */, err, errutil.New(strfy(args))),
			newMultiCause("B", false /* elide */, errutil.New("included 1"), errutil.New("included 2")),
		)
	},
	// This iteration elides the causes in the second child error,
	// which omits them from the format string.
	"multi-elided-cause": func(err error, args []arg) error {
		return newMultiCause("A", false, /* elide */
			newMultiCause("C", false /* elide */, err, errutil.New(strfy(args))),
			newMultiCause("B", true /* elide */, errutil.New("elided 1"), errutil.New("elided 2")),
		)
	},

	// stack attaches a simple stack trace.
	"stack": func(err error, _ []arg) error { return withstack.WithStack(err) },

	// msg is our own public message wrapper, which implements a proper
	// FormatError() method.
	"msg": func(err error, args []arg) error { return errutil.WithMessage(err, strfy(args)) },

	// newfw is errors.Newf("%w") which is the fmt-standard way to wrap an error.
	"newfw": func(err error, args []arg) error { return errutil.Newf("new-style (%s) :: %w ::", strfy(args), err) },

	// newfw-suffix is like newfw, except the cause is printed before
	// the prefix.
	"newfw-suffix": func(err error, args []arg) error { return errutil.Newf(":: %w :: new-style (%s)", err, strfy(args)) },

	// errutil.Wrap implements multi-layer wrappers.
	"wrapf": func(err error, args []arg) error { return errutil.Wrapf(err, "new-stylew %s", strfy(args)) },
	// assertions mask their cause from the barriers, but otherwise format as-is.
	"assertion": func(err error, args []arg) error { return errutil.HandleAsAssertionFailure(err) },
	// assertions mask their cause from the barriers, with a custom assertion message.
	"assertwrap": func(err error, args []arg) error {
		return errutil.NewAssertionErrorWithWrappedErrf(err, "assertmsg: %s", strfy(args))
	},
	// barrier is a simpler barrier
	"barrier": func(err error, _ []arg) error { return barriers.Handled(err) },
	// domains are hidden annotations. Tested here for sentry reporting.
	"domain": func(err error, _ []arg) error { return domains.WithDomain(err, "mydomain") },
	// handled-domain wraps the error behind a barrier in its implicit domain.
	"handled-domain": func(err error, _ []arg) error { return domains.Handled(err) },
	// hint and detail add user annotations.
	"hint":   func(err error, args []arg) error { return hintdetail.WithHint(err, strfy(args)) },
	"detail": func(err error, args []arg) error { return hintdetail.WithDetail(err, strfy(args)) },
	// migrated changes the path to mimic a type that has migrated packages.
	"migrated": func(err error, _ []arg) error { return &werrMigrated{err} },
	// safedetails adds non-visible safe details
	"safedetails": func(err error, args []arg) error { return safedetails.WithSafeDetails(err, "safe %s", strfy(args)) },
	// secondary adds an error annotation
	"secondary": func(err error, args []arg) error { return secondary.WithSecondaryError(err, errutil.New(strfy(args))) },
	// wrapf-attached embeds its error argument as extra payload.
	"wrapf-attached": func(err error, args []arg) error {
		return errutil.Wrapf(err, "new-style %s (%v)", strfy(args), errutil.New("payload"))
	},
	// issuelink attaches a link and detail.
	"issuelink": func(err error, args []arg) error {
		return issuelink.WithIssueLink(err, issuelink.IssueLink{IssueURL: "https://mysite", Detail: strfy(args)})
	},
	// telemetry adds telemetry keys
	"telemetry": func(err error, args []arg) error { return telemetrykeys.WithTelemetry(err, "somekey", strfy(args)) },
	// tags captures context tags
	"tags": func(err error, args []arg) error {
		ctx := context.Background()
		ctx = logtags.AddTag(ctx, "k", 123)
		ctx = logtags.AddTag(ctx, "safe", redact.Safe(456))
		return contexttags.WithContextTags(err, ctx)
	},
	"join": func(err error, args []arg) error {
		return join.Join(err, errutil.New(strfy(args)))
	},
}

var noPrefixWrappers = map[string]bool{
	"assertion":         true,
	"barrier":           true,
	"delegate-empty":    true,
	"delegate-noprefix": true,
	"detail":            true,
	"domain":            true,
	"empty":             true,
	"handled-domain":    true,
	"hint":              true,
	"issuelink":         true,
	"migrated":          true,
	"os-link":           true,
	"os-netop":          true,
	"os-path":           true,
	"os-syscall":        true,
	"pkgstack":          true,
	"safedetails":       true,
	"secondary":         true,
	"stack":             true,
	"tags":              true,
	"telemetry":         true,
	"join":              true,
}

var wrapOnlyExceptions = map[string]string{}

func init() {
	for _, v := range []string{
		// The following wrapper types don't implement FormatError(), so they don't get
		// enhanced displays via %+v automatically. Formattable() adds it to them.
		// So it is expected that %+v via Formattable() is different (better) than
		// %+v on its own.
		//
		// Additionally, the Go idea of %#v is somewhat broken: this is specified
		// in printf() docstrings to always emit a "Go representation". However,
		// if the object implements Formatter(), this takes over the display of %#v
		// too. Most implementation of Format() are incomplete and unable to
		// emit a "Go representation", so this breaks.
		//
		"goerr", "go-errorf", "go-errorf-suffix", "go-errorf-multi", "fmt-old", "fmt-old-delegate",
		"os-syscall",
		"os-link",
		"os-path",
		"os-netop",
		// In the case of werrNoFmt{}, there is no Format() method, so the
		// stdlib Fprintf is able to emit a Go representation.  However it's
		// a bit dumb doing so, and only reports members by address. Our
		// Formattable() implementation is able to report more, but that
		// means they don't match.
		"nofmt",
		"errorf",
		"join",
	} {
		wrapOnlyExceptions[v] = `
accept %\+v via Formattable.*IRREGULAR: not same as %\+v
accept %\#v via Formattable.*IRREGULAR: not same as %\#v
`
	}

	// The wrapper types from github/pkg/errors does not properly implement
	// Format(). So they do not know how to format %x/%X/%q properly.
	for _, v := range []string{"pkgmsg", "pkgstack"} {
		wrapOnlyExceptions[v] = `
accept %x.*IRREGULAR: not same as hex Error
accept %q.*IRREGULAR: not same as quoted Error
accept %X.*IRREGULAR: not same as HEX Error
accept %\#v via Formattable.*IRREGULAR: not same as %\#v
accept %\+v via Formattable.*IRREGULAR: not same as %\+v`
	}
}

const testPath = "testdata/format"

func generateFiles() {
	// Make the leaf and wrapper names sorted for determinism.
	var leafNames []string
	for leafName := range leafCommands {
		leafNames = append(leafNames, leafName)
	}
	sort.Strings(leafNames)
	var wrapNames []string
	for wrapName := range wrapCommands {
		wrapNames = append(wrapNames, wrapName)
	}
	sort.Strings(wrapNames)

	// Generate the "leaves" input file, which tests formatting for
	// leaf-only error types.
	var leafTests bytes.Buffer
	for _, leafName := range leafNames {
		fmt.Fprintf(&leafTests, "run\n"+
			// The leaf error being examined.
			"%s oneline twoline\n"+
			// accepted irregularities, if any, follow.
			"%s\n",
			leafName, leafOnlyExceptions[leafName])
		if !strings.HasPrefix(leafName, "os-") && !strings.HasPrefix(leafName, "ctx-") {
			// All renderings need to contain at least the words 'oneline' and 'twolines'.
			leafTests.WriteString("require (?s)oneline.*twoline\n")
		}
		leafTests.WriteString("----\n\n")
	}
	os.WriteFile(testPath+"/leaves", leafTests.Bytes(), 0666)

	// Generate the "leaves-via-network" input file, which tests
	// formatting for leaf-only error types after being brought over
	// the network.
	leafTests.Reset()
	for _, leafName := range leafNames {
		fmt.Fprintf(&leafTests, "run\n"+
			// The leaf error being examined.
			"%s oneline twoline\n"+
			"opaque\n"+
			// accepted irregularities, if any, follow.
			"%s\n",
			leafName, leafOnlyExceptions[leafName])
		if !strings.HasPrefix(leafName, "os-") && !strings.HasPrefix(leafName, "ctx-") {
			// All renderings need to contain at least the words 'oneline' and 'twolines'.
			leafTests.WriteString("require (?s)oneline.*twoline\n")
		}
		leafTests.WriteString("----\n\n")
	}
	os.WriteFile(testPath+"/leaves-via-network", leafTests.Bytes(), 0666)

	// Leaf types for which we want to test all wrappers:
	wrapperLeafTypes := []string{"fmt", "goerr", "nofmt", "pkgerr", "newf"}
	// Generate the direct wrapper tests.
	for _, leafName := range wrapperLeafTypes {
		var wrapTests bytes.Buffer
		for _, wrapName := range wrapNames {
			if wrapName == "opaque" {
				// opaque wrappers get their own tests in separate files.
				continue
			}

			fmt.Fprintf(&wrapTests, "run\n"+
				// The leaf error being examined.
				"%s innerone innertwo\n"+
				// The wrappererror being examined.
				"%s outerthree outerfour\n"+
				// accepted irregularities, if any, follow.
				"%s\n",
				leafName, wrapName, wrapOnlyExceptions[wrapName])

			expectedLeafString := "innerone.*innertwo"
			if !strings.HasPrefix(leafName, "os-") && !strings.HasPrefix(leafName, "ctx-") {
				expectedLeafString = ""
			}

			if noPrefixWrappers[wrapName] {
				// No prefix in wrapper. Only expect the leaf (inner) error.
				fmt.Fprintf(&wrapTests, "require (?s)%s\n", expectedLeafString)
			} else if wrapName == "elided-cause" {
				// This wrapper type hides the inner error.
				wrapTests.WriteString("require (?s)outerthree.*outerfour\n")
			} else if strings.HasSuffix(wrapName, "-suffix") {
				// Wrapper with message override: the test cases place
				// the leaf before the prefix.
				fmt.Fprintf(&wrapTests, "require (?s)%s.*outerthree.*outerfour\n", expectedLeafString)
			} else {
				// Wrapper with prefix: all renderings need to contain at
				// least the words from the leaf and the wrapper.
				fmt.Fprintf(&wrapTests, "require (?s)outerthree.*outerfour.*%s\n", expectedLeafString)
			}
			wrapTests.WriteString("----\n\n")
		}
		os.WriteFile(
			fmt.Sprintf(testPath+"/wrap-%s", leafName),
			wrapTests.Bytes(), 0666)
	}

	// Generate the wrappers-via-network tests.
	for _, leafName := range wrapperLeafTypes {
		var wrapTests bytes.Buffer
		for _, wrapName := range wrapNames {
			if wrapName == "opaque" {
				// opaque wrappers get their own tests in separate files.
				continue
			}

			fmt.Fprintf(&wrapTests, "run\n"+
				// The leaf error being examined.
				"%s innerone innertwo\n"+
				// The wrappererror being examined.
				"%s outerthree outerfour\n"+
				// Via network.
				"opaque\n"+
				// accepted irregularities, if any, follow.
				"%s\n",
				leafName, wrapName, wrapOnlyExceptions[wrapName])
			if noPrefixWrappers[wrapName] {
				// No prefix in wrapper. Only expect the leaf (inner) error.
				wrapTests.WriteString("require (?s)innerone.*innertwo\n")
			} else if wrapName == "elided-cause" {
				// This wrapper type hides the inner error.
				wrapTests.WriteString("require (?s)outerthree.*outerfour\n")
			} else if strings.HasSuffix(wrapName, "-suffix") {
				// Wrapper with message override: the test cases place
				// the leaf before the prefix.
				wrapTests.WriteString("require (?s)innerone.*innertwo.*outerthree.*outerfour\n")
			} else {
				// Wrapper with prefix: all renderings need to contain at
				// least the words from the leaf and the wrapper.
				wrapTests.WriteString("require (?s)outerthree.*outerfour.*innerone.*innertwo\n")
			}
			wrapTests.WriteString("----\n\n")
		}
		os.WriteFile(
			fmt.Sprintf(testPath+"/wrap-%s-via-network", leafName),
			wrapTests.Bytes(), 0666)
	}
}

// TestDatadriven exercises error formatting and Sentry report
// formatting using the datadriven package.
//
// The test DSL accepts a single directive "run" with a sub-DSL
// for each test. The sub-DSL accepts 3 types of directive:
//
//	accept <regexp>
//	      Tells the test that a "problem" or "irregularity"
//	      is not to be considered a test failure if it matches
//	      the provided <regexp>
//
//	require <regexp>
//	      Requires the result of both Error() and %+v formatting
//	      to match <regexp>
//
//	<error constructor>
//	      The remaining directives in the sub-DSL construct
//	      an error object to format using a stack: the first directive
//	      creates a leaf error; the 2nd one wraps it a first time,
//	      the 3rd one wraps it a second time, and so forth.
func TestDatadriven(t *testing.T) {
	if *generateTestFiles {
		generateFiles()
	}

	// events collected the emitted sentry report on each test.
	var events []*sentry.Event

	client, err := sentry.NewClient(
		sentry.ClientOptions{
			// Install a Transport that locally records events rather than
			// sending them to Sentry over HTTP.
			Transport: interceptingTransport{
				SendFunc: func(event *sentry.Event) {
					events = append(events, event)
				},
			},
		},
	)
	if err != nil {
		t.Fatal(err)
	}
	sentry.CurrentHub().BindClient(client)

	datadriven.Walk(t, testPath, func(t *testing.T, path string) {
		datadriven.RunTest(t, path,
			func(t *testing.T, d *datadriven.TestData) string {
				if d.Cmd != "run" {
					d.Fatalf(t, "unknown directive: %s", d.Cmd)
				}
				pos := d.Pos
				var resultErr error
				var acceptList []*regexp.Regexp
				var acceptDirectives []string
				var requiredMsgRe *regexp.Regexp

				// We're going to execute the input line-by-line.
				lines := strings.Split(d.Input, "\n")
				for i, line := range lines {
					if short := strings.TrimSpace(line); short == "" || strings.HasPrefix(short, "#") {
						// Comment or empty line. Do nothing.
						continue
					}
					// Compute a line prefix, to clarify error message. We
					// prefix a newline character because some text editor do
					// not know how to jump to the location of an error if
					// there are multiple file:line prefixes on the same line.
					d.Pos = fmt.Sprintf("\n%s: (+%d)", pos, i+1)

					switch {
					case strings.HasPrefix(line, "accept "):
						acceptDirectives = append(acceptDirectives, line)
						acceptList = append(acceptList, regexp.MustCompile(line[7:]))

					case strings.HasPrefix(line, "require "):
						requiredMsgRe = regexp.MustCompile(line[8:])

					default:
						var err error
						d.Cmd, d.CmdArgs, err = datadriven.ParseLine(line)
						if err != nil {
							d.Fatalf(t, "%v", err)
						}
						var c commandFn
						if resultErr == nil {
							c = leafCommands[d.Cmd]
						} else {
							c = wrapCommands[d.Cmd]
						}
						if c == nil {
							d.Fatalf(t, "unknown command: %s", d.Cmd)
						}
						resultErr = c(resultErr, d.CmdArgs)
					}
				}

				accepted := func(irregular string) bool {
					for _, re := range acceptList {
						if re.MatchString(irregular) {
							return true
						}
					}
					return false
				}

				// Result buffer.
				var buf bytes.Buffer
				reportIrregular := func(format string, args ...interface{}) {
					s := fmt.Sprintf(format, args...)
					fmt.Fprint(&buf, s)
					if accepted(s) {
					} else {
						d.Fatalf(t, "unexpected:\n%s\n\naccepted irregularities:\n%s",
							buf.String(), strings.Join(acceptDirectives, "\n"))
					}
				}

				fmt.Fprintf(&buf, "%# v\n", pretty.Formatter(resultErr))

				buf.WriteString("=====\n===== non-redactable formats\n=====\n")
				hErr := fmt.Sprintf("%#v", resultErr)
				fmt.Fprintf(&buf, "== %%#v\n%s\n", hErr)

				refStr := resultErr.Error()
				fmt.Fprintf(&buf, "== Error()\n%s\n", refStr)

				if requiredMsgRe != nil && !requiredMsgRe.MatchString(refStr) {
					d.Fatalf(t, "unexpected:\n%s\n\nerror string does not match required regexp %q",
						buf.String(), requiredMsgRe)
				}

				vErr := fmt.Sprintf("%v", resultErr)
				if vErr == refStr {
					fmt.Fprintf(&buf, "== %%v = Error(), good\n")
				} else {
					reportIrregular("== %%v (IRREGULAR: not same as Error())\n%s", vErr)
				}
				sErr := fmt.Sprintf("%s", resultErr)
				if sErr == refStr {
					fmt.Fprintf(&buf, "== %%s = Error(), good\n")
				} else {
					reportIrregular("== %%s (IRREGULAR: not same as Error())\n%s\n", sErr)
				}
				qErr := fmt.Sprintf("%q", resultErr)
				if qref := fmt.Sprintf("%q", refStr); qErr == qref {
					fmt.Fprintf(&buf, "== %%q = quoted Error(), good\n")
				} else {
					reportIrregular("== %%q (IRREGULAR: not same as quoted Error())\n%s\n", qErr)
				}
				xErr := fmt.Sprintf("%x", resultErr)
				if xref := fmt.Sprintf("%x", refStr); xErr == xref {
					fmt.Fprintf(&buf, "== %%x = hex Error(), good\n")
				} else {
					if xErr == "" {
						xErr = "(EMPTY STRING)"
					}
					reportIrregular("== %%x (IRREGULAR: not same as hex Error())\n%s\n", xErr)
				}
				xxErr := fmt.Sprintf("%X", resultErr)
				if xxref := fmt.Sprintf("%X", refStr); xxErr == xxref {
					fmt.Fprintf(&buf, "== %%X = HEX Error(), good\n")
				} else {
					if xxErr == "" {
						xxErr = "(EMPTY STRING)"
					}
					reportIrregular("== %%X (IRREGULAR: not same as HEX Error())\n%s\n", xxErr)
				}

				vpErr := fmt.Sprintf("%+v", resultErr)
				if vpErr == vErr {
					fmt.Fprintf(&buf, "== %%+v = Error(), ok\n")
				} else {
					fmt.Fprintf(&buf, "== %%+v\n%s\n", vpErr)
				}

				hfErr := fmt.Sprintf("%#v", errbase.Formattable(resultErr))
				if hfErr == hErr {
					fmt.Fprintf(&buf, "== %%#v via Formattable() = %%#v, good\n")
				} else {
					reportIrregular("== %%#v via Formattable() (IRREGULAR: not same as %%#v)\n%s\n", hfErr)
				}

				vfErr := fmt.Sprintf("%v", errbase.Formattable(resultErr))
				if vfErr == refStr {
					fmt.Fprintf(&buf, "== %%v via Formattable() = Error(), good\n")
				} else {
					reportIrregular("== %%v via Formattable() (IRREGULAR: not same as Error())\n%s\n", vfErr)
				}
				sfErr := fmt.Sprintf("%s", errbase.Formattable(resultErr))
				if sfErr == vfErr {
					fmt.Fprintf(&buf, "== %%s via Formattable() = %%v via Formattable(), good\n")
				} else {
					reportIrregular("== %%s via Formattable() (IRREGULAR: not same as Error())\n%s\n", sfErr)
				}
				qfErr := fmt.Sprintf("%q", errbase.Formattable(resultErr))
				if qfref := fmt.Sprintf("%q", vfErr); qfErr == qfref {
					fmt.Fprintf(&buf, "== %%q via Formattable() = quoted %%v via Formattable(), good\n")
				} else {
					reportIrregular("== %%q via Formattable() (IRREGULAR: not same as quoted %%v via Formattable())\n%s\n", qfErr)
				}

				vpfErr := fmt.Sprintf("%+v", errbase.Formattable(resultErr))
				if vpfErr == vpErr {
					fmt.Fprintf(&buf, "== %%+v via Formattable() == %%+v, good\n")
				} else {
					reportIrregular("== %%+v via Formattable() (IRREGULAR: not same as %%+v)\n%s\n", vpfErr)
				}

				buf.WriteString("=====\n===== redactable formats\n=====\n")

				rErr := redact.Sprint(resultErr)
				annot := " (IRREGULAR: not congruent)"
				stripped := rErr.StripMarkers()
				if stripped == vErr {
					annot = ", ok - congruent with %v"
				} else if stripped == vfErr {
					annot = ", ok - congruent with %v via Formattable()"
				}
				if strings.HasPrefix(annot, ", ok") {
					fmt.Fprintf(&buf, "== printed via redact Print()%s\n%s\n", annot, string(rErr))
				} else {
					reportIrregular("== printed via redact Print()%s\n%s\n", annot, string(rErr))
				}
				checkMarkers(&buf, d, t, string(rErr))

				vrErr := string(redact.Sprintf("%v", resultErr))
				if vrErr == string(rErr) {
					fmt.Fprintf(&buf, "== printed via redact Printf() %%v = Print(), good\n")
				} else {
					reportIrregular("== printed via redact Printf() %%v (IRREGULAR: not same as Print())\n%s\n", vrErr)
				}
				checkMarkers(&buf, d, t, vrErr)

				srErr := string(redact.Sprintf("%s", resultErr))
				if srErr == string(rErr) {
					fmt.Fprintf(&buf, "== printed via redact Printf() %%s = Print(), good\n")
				} else {
					reportIrregular("== printed via redact Printf() %%s (IRREGULAR: not same as Print())\n%s\n", vrErr)
				}
				checkMarkers(&buf, d, t, srErr)

				qrErr := string(redact.Sprintf("%q", resultErr))
				m := string(redact.StartMarker())
				if strings.HasPrefix(qrErr, m+"%!q(") {
					fmt.Fprintf(&buf, "== printed via redact Printf() %%q, refused - good\n")
				} else {
					reportIrregular("== printed via redact Printf() %%q - UNEXPECTED\n%s\n", qrErr)
				}
				checkMarkers(&buf, d, t, qrErr)

				xrErr := string(redact.Sprintf("%x", resultErr))
				if strings.HasPrefix(xrErr, m+"%!x(") {
					fmt.Fprintf(&buf, "== printed via redact Printf() %%x, refused - good\n")
				} else {
					reportIrregular("== printed via redact Printf() %%x - UNEXPECTED\n%s\n", xrErr)
				}
				checkMarkers(&buf, d, t, xrErr)

				xxrErr := string(redact.Sprintf("%X", resultErr))
				if strings.HasPrefix(xxrErr, m+"%!X(") {
					fmt.Fprintf(&buf, "== printed via redact Printf() %%X, refused - good\n")
				} else {
					reportIrregular("== printed via redact Printf() %%X - UNEXPECTED\n%s\n", xxrErr)
				}
				checkMarkers(&buf, d, t, xxrErr)

				vprErr := redact.Sprintf("%+v", resultErr)
				annot = " (IRREGULAR: not congruent)"
				stripped = vprErr.StripMarkers()
				if stripped == vpErr {
					annot = ", ok - congruent with %+v"
				} else if stripped == vpfErr {
					annot = ", ok - congruent with %+v via Formattable()"
				}
				if strings.HasPrefix(annot, ", ok") {
					fmt.Fprintf(&buf, "== printed via redact Printf() %%+v%s\n%s\n", annot, string(vprErr))
				} else {
					reportIrregular("== printed via redact Printf()%%+v%s\n%s\n", annot, string(vprErr))
				}
				checkMarkers(&buf, d, t, string(vprErr))

				buf.WriteString("=====\n===== Sentry reporting\n=====\n")

				events = nil
				if eventID := report.ReportError(resultErr); eventID == "" {
					d.Fatalf(t, "Sentry eventID is empty")
				}
				// t.Logf("received events: %# v", pretty.Formatter(events))
				if len(events) != 1 {
					d.Fatalf(t, "more than one event received")
				}
				se := events[0]

				fmt.Fprintf(&buf, "== Message payload\n%s\n", se.Message)

				// Make the extra key deterministic.
				extraNames := make([]string, 0, len(se.Extra))
				for ek := range se.Extra {
					extraNames = append(extraNames, ek)
				}
				sort.Strings(extraNames)
				for _, ek := range extraNames {
					extraS := fmt.Sprintf("%v", se.Extra[ek])
					fmt.Fprintf(&buf, "== Extra %q\n%s\n", ek, strings.TrimSpace(extraS))
				}

				for i, exc := range se.Exception {
					fmt.Fprintf(&buf, "== Exception %d (Module: %q)\nType: %q\nTitle: %q\n", i+1, exc.Module, exc.Type, exc.Value)
					if exc.Stacktrace == nil {
						fmt.Fprintf(&buf, "(NO STACKTRACE)\n")
					} else {
						for _, f := range exc.Stacktrace.Frames {
							fmt.Fprintf(&buf, "%s:%d:\n", f.Filename, f.Lineno)
							fmt.Fprintf(&buf, "   (%s) %s()\n", f.Module, f.Function)
						}
					}
				}

				return fmtClean(buf.String())
			})
	})
}

func checkMarkers(buf *bytes.Buffer, d *datadriven.TestData, t *testing.T, s string) {
	sm := string(redact.StartMarker())
	em := string(redact.EndMarker())
	anyMarker := regexp.MustCompile("((?s)[" + sm + em + "].*)")
	expectOpen := true
	for {
		s = anyMarker.FindString(s)
		if s == "" {
			break
		}
		if expectOpen {
			if strings.HasPrefix(s, em) {
				d.Fatalf(t, "unexpected closing redaction marker:\n%s\n\n(suffix: %q)", buf.String(), s)
			}
			s = strings.TrimPrefix(s, sm)
		} else {
			if strings.HasPrefix(s, sm) {
				d.Fatalf(t, "unexpected open redaction marker:\n%s\n\n(suffix: %q)", buf.String(), s)
			}
			s = strings.TrimPrefix(s, em)
		}
		expectOpen = !expectOpen
	}
	if !expectOpen {
		d.Fatalf(t, "unclosed redaction marker:\n%s\n\n(suffix: %q)", buf.String(), s)
	}
}

type arg = datadriven.CmdArg

type commandFn func(inputErr error, args []arg) (resultErr error)

func strfy(args []arg) string {
	var out strings.Builder
	sp := ""
	for _, arg := range args {
		out.WriteString(sp)
		if len(arg.Vals) == 0 {
			out.WriteString(arg.Key)
		} else {
			out.WriteString(strings.Join(arg.Vals, " "))
		}
		sp = "\n"
	}
	return out.String()
}

// interceptingTransport is an implementation of sentry.Transport that
// delegates calls to the SendEvent method to the send function contained
// within.
type interceptingTransport struct {
	// SendFunc is the send callback.
	SendFunc func(event *sentry.Event)
}

var _ sentry.Transport = &interceptingTransport{}

// Flush implements the sentry.Transport interface.
func (it interceptingTransport) Flush(time.Duration) bool {
	return true
}

// Configure implements the sentry.Transport interface.
func (it interceptingTransport) Configure(sentry.ClientOptions) {
}

// SendEvent implements the sentry.Transport interface.
func (it interceptingTransport) SendEvent(event *sentry.Event) {
	it.SendFunc(event)
}
