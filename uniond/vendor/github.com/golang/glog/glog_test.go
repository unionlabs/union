package glog

import (
	"bytes"
	"context"
	"flag"
	"fmt"
	"io/ioutil"
	stdLog "log"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/golang/glog/internal/logsink"
)

// Test that shortHostname works as advertised.
func TestShortHostname(t *testing.T) {
	for hostname, expect := range map[string]string{
		"":                     "",
		"host":                 "host",
		"host.google.com":      "host",
		"host.corp.google.com": "host",
	} {
		if got := shortHostname(hostname); expect != got {
			t.Errorf("shortHostname(%q): expected %q, got %q", hostname, expect, got)
		}
	}
}

// flushBuffer wraps a bytes.Buffer to satisfy flushSyncWriter.
type flushBuffer struct {
	bytes.Buffer
}

func (f *flushBuffer) Flush() error {
	f.Buffer.Reset()
	return nil
}

func (f *flushBuffer) Sync() error {
	return nil
}

func (f *flushBuffer) filenames() []string {
	return []string{"<local name>"}
}

// swap sets the log writers and returns the old array.
func (s *fileSink) swap(writers severityWriters) (old severityWriters) {
	s.mu.Lock()
	defer s.mu.Unlock()
	old = s.file
	for i, w := range writers {
		s.file[i] = w
	}
	return
}

// newBuffers sets the log writers to all new byte buffers and returns the old array.
func (s *fileSink) newBuffers() severityWriters {
	return s.swap(severityWriters{new(flushBuffer), new(flushBuffer), new(flushBuffer), new(flushBuffer)})
}

func (s *fileSink) resetBuffers() {
	s.mu.Lock()
	defer s.mu.Unlock()
	for _, buf := range s.file {
		if buf != nil {
			buf.Flush()
		}
	}
}

// contents returns the specified log value as a string.
func contents(s logsink.Severity) string {
	return sinks.file.file[s].(*flushBuffer).String()
}

// contains reports whether the string is contained in the log.
func contains(s logsink.Severity, str string, t *testing.T) bool {
	return strings.Contains(contents(s), str)
}

// setFlags configures the logging flags how the test expects them.
func setFlags() {
	toStderr = false
}

// Test that Info works as advertised.
func TestInfo(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())
	funcs := []func(args ...any){
		Info,
		func(args ...any) { InfoContext(context.Background(), args) },
	}

	for _, f := range funcs {
		sinks.file.resetBuffers()
		f("test")
		if !contains(logsink.Info, "I", t) {
			t.Errorf("Info has wrong character: %q", contents(logsink.Info))
		}
		if !contains(logsink.Info, "test", t) {
			t.Error("Info failed")
		}
	}
}

func TestInfoDepth(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())

	funcs := []func(d int, args ...any){
		InfoDepth,
		func(d int, args ...any) { InfoContextDepth(context.Background(), d+1, args) },
	}

	for _, infoDepth := range funcs {
		sinks.file.resetBuffers()
		f := func() { infoDepth(1, "depth-test1") }

		// The next three lines must stay together
		_, _, wantLine, _ := runtime.Caller(0)
		infoDepth(0, "depth-test0")
		f()

		msgs := strings.Split(strings.TrimSuffix(contents(logsink.Info), "\n"), "\n")
		if len(msgs) != 2 {
			t.Fatalf("Got %d lines, expected 2", len(msgs))
		}

		for i, m := range msgs {
			if !strings.HasPrefix(m, "I") {
				t.Errorf("InfoDepth[%d] has wrong character: %q", i, m)
			}
			w := fmt.Sprintf("depth-test%d", i)
			if !strings.Contains(m, w) {
				t.Errorf("InfoDepth[%d] missing %q: %q", i, w, m)
			}

			// pull out the line number (between : and ])
			msg := m[strings.LastIndex(m, ":")+1:]
			x := strings.Index(msg, "]")
			if x < 0 {
				t.Errorf("InfoDepth[%d]: missing ']': %q", i, m)
				continue
			}
			line, err := strconv.Atoi(msg[:x])
			if err != nil {
				t.Errorf("InfoDepth[%d]: bad line number: %q", i, m)
				continue
			}
			wantLine++
			if wantLine != line {
				t.Errorf("InfoDepth[%d]: got line %d, want %d", i, line, wantLine)
			}
		}
	}
}

func init() {
	CopyStandardLogTo("INFO")
}

// Test that CopyStandardLogTo panics on bad input.
func TestCopyStandardLogToPanic(t *testing.T) {
	defer func() {
		if s, ok := recover().(string); !ok || !strings.Contains(s, "LOG") {
			t.Errorf(`CopyStandardLogTo("LOG") should have panicked: %v`, s)
		}
	}()
	CopyStandardLogTo("LOG")
}

// Test that using the standard log package logs to INFO.
func TestStandardLog(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())
	stdLog.Print("test")
	if !contains(logsink.Info, "I", t) {
		t.Errorf("Info has wrong character: %q", contents(logsink.Info))
	}
	if !contains(logsink.Info, "test", t) {
		t.Error("Info failed")
	}
}

// Test that the header has the correct format.
func TestHeader(t *testing.T) {
	setFlags()
	defer func(previous func() time.Time) { timeNow = previous }(timeNow)
	timeNow = func() time.Time {
		return time.Date(2006, 1, 2, 15, 4, 5, .067890e9, time.Local)
	}

	oldPID := pid
	defer func() { pid = oldPID }()
	pid = 1234

	defer sinks.file.swap(sinks.file.newBuffers())

	Info("testHeader")
	var line int
	format := "I0102 15:04:05.067890 %7d glog_test.go:%d] testHeader\n"
	var gotPID int64
	n, err := fmt.Sscanf(contents(logsink.Info), format, &gotPID, &line)
	if n != 2 || err != nil {
		t.Errorf("log format error: %d elements, error %s:\n%s", n, err, contents(logsink.Info))
	}

	if want := int64(pid); gotPID != want {
		t.Errorf("expected log line to be logged with process ID %d, got %d", want, gotPID)
	}

	// Scanf treats multiple spaces as equivalent to a single space,
	// so check for correct space-padding also.
	want := fmt.Sprintf(format, gotPID, line)
	if contents(logsink.Info) != want {
		t.Errorf("log format error: got:\n\t%q\nwant:\n\t%q", contents(logsink.Info), want)
	}

}

// Test that an Error log goes to Warning and Info.
// Even in the Info log, the source character will be E, so the data should
// all be identical.
func TestError(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())

	funcs := []func(args ...any){
		Error,
		func(args ...any) { ErrorContext(context.Background(), args) },
	}

	for _, error := range funcs {
		sinks.file.resetBuffers()
		error("test")
		if !contains(logsink.Error, "E", t) {
			t.Errorf("Error has wrong character: %q", contents(logsink.Error))
		}
		if !contains(logsink.Error, "test", t) {
			t.Error("Error failed")
		}
		str := contents(logsink.Error)
		if !contains(logsink.Warning, str, t) {
			t.Error("Warning failed")
		}
		if !contains(logsink.Info, str, t) {
			t.Error("Info failed")
		}
	}
}

// Test that a Warning log goes to Info.
// Even in the Info log, the source character will be W, so the data should
// all be identical.
func TestWarning(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())

	funcs := []func(args ...any){
		Warning,
		func(args ...any) { WarningContext(context.Background(), args) },
	}

	for _, warning := range funcs {
		sinks.file.resetBuffers()
		warning("test")
		if !contains(logsink.Warning, "W", t) {
			t.Errorf("Warning has wrong character: %q", contents(logsink.Warning))
		}
		if !contains(logsink.Warning, "test", t) {
			t.Error("Warning failed")
		}
		str := contents(logsink.Warning)
		if !contains(logsink.Info, str, t) {
			t.Error("Info failed")
		}
	}
}

// Test that a V log goes to Info.
func TestV(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())
	if err := flag.Lookup("v").Value.Set("2"); err != nil {
		t.Fatalf("Failed to set -v=2: %v", err)
	}
	defer flag.Lookup("v").Value.Set("0")

	funcs := []func(args ...any){
		V(2).Info,
		func(args ...any) { V(2).InfoContext(context.Background(), args) },
	}
	for _, info := range funcs {
		sinks.file.resetBuffers()
		info("test")
		if !contains(logsink.Info, "I", t) {
			t.Errorf("Info has wrong character: %q", contents(logsink.Info))
		}
		if !contains(logsink.Info, "test", t) {
			t.Error("Info failed")
		}
	}
}

// Test that updating -v at runtime, while -vmodule is set to a non-empty
// value, resets the modules cache correctly.
func TestVFlagUpdates(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())
	// Set -vmodule to some arbitrary value to make values read from cache.
	// See log_flags.go:/func .* enabled/.
	if err := flag.Lookup("vmodule").Value.Set("non_existent_module=3"); err != nil {
		t.Fatalf("Failed to set -vmodule=log_test=3: %v", err)
	}
	defer flag.Lookup("vmodule").Value.Set("")
	if err := flag.Lookup("v").Value.Set("3"); err != nil {
		t.Fatalf("Failed to set -v=3: %v", err)
	}
	defer flag.Lookup("v").Value.Set("0")

	if !V(2) {
		t.Error("V(2) not enabled for 2")
	}
	if !V(3) {
		t.Error("V(3) not enabled for 3")
	}

	// Setting a lower level should reset the modules cache.
	if err := flag.Lookup("v").Value.Set("2"); err != nil {
		t.Fatalf("Failed to set -v=2: %v", err)
	}
	if !V(2) {
		t.Error("V(2) not enabled for 2")
	}
	if V(3) {
		t.Error("V(3) enabled for 3")
	}
}

// Test that an arbitrary log.Level value does not modify -v.
func TestLevel(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())
	if err := flag.Lookup("v").Value.Set("3"); err != nil {
		t.Fatalf("Failed to set -v=3: %v", err)
	}
	defer flag.Lookup("v").Value.Set("0")

	var l Level
	if got, want := l.String(), "0"; got != want {
		t.Errorf("l.String() = %q, want %q", got, want)
	}
	if err := l.Set("2"); err != nil {
		t.Fatalf("l.Set(2) failed: %v", err)
	}
	if got, want := l.String(), "2"; got != want {
		t.Errorf("l.String() = %q, want %q", got, want)
	}
	// -v flag should still be "3".
	if got, want := flag.Lookup("v").Value.String(), "3"; got != want {
		t.Errorf("-v=%v, want %v", got, want)
	}
}

// Test that a vmodule enables a log in this file.
func TestVmoduleOn(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())
	if err := flag.Lookup("vmodule").Value.Set("glog_test=2"); err != nil {
		t.Fatalf("Failed to set -vmodule=log_test=2: %v", err)
	}
	defer flag.Lookup("vmodule").Value.Set("")

	if !V(1) {
		t.Error("V not enabled for 1")
	}
	if !V(2) {
		t.Error("V not enabled for 2")
	}
	if V(3) {
		t.Error("V enabled for 3")
	}
	V(2).Info("test")
	if !contains(logsink.Info, "I", t) {
		t.Errorf("Info has wrong character: %q", contents(logsink.Info))
	}
	if !contains(logsink.Info, "test", t) {
		t.Error("Info failed")
	}
}

// Test that a VDepth calculates the depth correctly.
func TestVDepth(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())
	if err := flag.Lookup("vmodule").Value.Set("glog_test=3"); err != nil {
		t.Fatalf("Failed to set -vmodule=glog_test=3: %v", err)
	}
	defer flag.Lookup("vmodule").Value.Set("")

	if !V(3) {
		t.Error("V not enabled for 3")
	}
	if !VDepth(0, 2) {
		t.Error("VDepth(0) not enabled for 2")
	}
	if !VDepth(0, 3) {
		t.Error("VDepth(0) not enabled for 3")
	}
	if VDepth(0, 4) {
		t.Error("VDepth(0) enabled for 4")
	}

	// Since vmodule is set to glog_test=3, V(3) is true only for frames in
	// glog_test. runInAnotherModule's stack frame is in log_vmodule_test, whereas
	// this test and the provided closures are in glog_test. Therefore VDepth(0, 3)
	// and VDepth(2, 3) are true, while VDepth(1, 3) is false.
	if !runInAnotherModule(func() bool { return bool(VDepth(0, 3)) }) {
		t.Error("VDepth(0) in closure not enabled for 3")
	}
	if runInAnotherModule(func() bool { return bool(VDepth(1, 3)) }) {
		t.Error("VDepth(1) in closure enabled for 3")
	}
	if !runInAnotherModule(func() bool { return bool(VDepth(2, 3)) }) {
		t.Error("VDepth(2) in closure not enabled for 3")
	}
}

// Test that a vmodule of another file does not enable a log in this file.
func TestVmoduleOff(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())
	if err := flag.Lookup("vmodule").Value.Set("notthisfile=2"); err != nil {
		t.Fatalf("Failed to set -vmodule=notthisfile=2: %v", err)
	}
	defer flag.Lookup("vmodule").Value.Set("")

	for i := 1; i <= 3; i++ {
		if V(Level(i)) {
			t.Errorf("V enabled for %d", i)
		}
	}
	V(2).Info("test")
	if contents(logsink.Info) != "" {
		t.Error("V logged incorrectly")
	}
}

// vGlobs are patterns that match/don't match this file at V=2.
var vGlobs = map[string]bool{
	// Easy to test the numeric match here.
	"glog_test=1": false, // If -vmodule sets V to 1, V(2) will fail.
	"glog_test=2": true,
	"glog_test=3": true, // If -vmodule sets V to 1, V(3) will succeed.
	// These all use 2 and check the patterns. All are true.
	"*=2":           true,
	"?l*=2":         true,
	"????_*=2":      true,
	"??[mno]?_*t=2": true,
	// These all use 2 and check the patterns. All are false.
	"*x=2":         false,
	"m*=2":         false,
	"??_*=2":       false,
	"?[abc]?_*t=2": false,
}

// Test that vmodule globbing works as advertised.
func testVmoduleGlob(pat string, match bool, t *testing.T) {
	t.Helper()
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())
	if err := flag.Lookup("vmodule").Value.Set(pat); err != nil {
		t.Errorf("Failed to set -vmodule=%s: %v", pat, err)
	}
	defer flag.Lookup("vmodule").Value.Set("")

	if V(2) != Verbose(match) {
		t.Errorf("incorrect match for %q: got %t expected %t", pat, V(2), match)
	}
}

// Test that a vmodule globbing works as advertised.
func TestVmoduleGlob(t *testing.T) {
	for glob, match := range vGlobs {
		testVmoduleGlob(glob, match, t)
	}
}

// Test that a vmodule globbing on a full path works as advertised.
func TestVmoduleFullGlob(t *testing.T) {
	_, file, _, _ := runtime.Caller(0)
	for glob, match := range vGlobs {
		testVmoduleGlob(filepath.Join(filepath.Dir(file), glob), match, t)
	}
}

// Test that a vmodule globbing across multiple directories works as advertised.
func TestVmoduleFullGlobMultipleDirectories(t *testing.T) {
	// Note: only covering here what
	// TestVmoduleGlob does not.
	_, file, _, _ := runtime.Caller(0)
	dir := filepath.Dir(filepath.Dir(file))
	testVmoduleGlob(filepath.Join(dir, "*/glog_test=2"), true, t)
	testVmoduleGlob(filepath.Join(dir, "*/glog_????=2"), true, t)
}

func logAtVariousLevels() {
	V(3).Infof("level 3 message")
	V(2).Infof("level 2 message")
	V(1).Infof("level 1 message")
	Infof("default level message")
}

func TestRollover(t *testing.T) {
	setFlags()

	Info("x") // Be sure we have a file.
	info, ok := sinks.file.file[logsink.Info].(*syncBuffer)
	if !ok {
		t.Fatal("info wasn't created")
	}

	// Make sure the next log file gets a file name with a different
	// time stamp.
	//
	// TODO: determine whether we need to support subsecond log
	// rotation.  C++ does not appear to handle this case (nor does it
	// handle Daylight Savings Time properly).
	time.Sleep(1 * time.Second)

	// Measure the current size of the log file.
	info.Flush()
	fi, err := info.file.Stat()
	if err != nil {
		t.Fatalf("Unable to stat log file %s: %v", info.file.Name(), err)
	}

	// Set MaxSize to a value that will accept one longMessage, but not two.
	longMessage := strings.Repeat("x", 1024)
	defer func(previous uint64) { MaxSize = previous }(MaxSize)
	MaxSize = uint64(fi.Size()) + uint64(2*len(longMessage)) - 1

	fname0 := info.file.Name()

	// Force a rotation.
	Info(longMessage)
	Info(longMessage)
	info.Flush()

	fname1 := info.file.Name()
	if fname0 == fname1 {
		t.Errorf("info.f.Name did not change: %v", fname0)
	}
	if info.nbytes >= MaxSize {
		t.Errorf("file size was not reset: %d", info.nbytes)
	}

	// Check to see if the original file has the continued footer.
	f0, err := ioutil.ReadFile(fname0)
	if err != nil {
		t.Fatalf("Unable to read file %s: %v", fname0, err)
	}
	if !bytes.HasSuffix(f0, []byte(footer)) {
		t.Errorf("%v: Missing footer %q", fname0, footer)
	}
	found := false
	for _, l := range bytes.Split(f0, []byte("\n")) {
		var file string
		_, err = fmt.Sscanf(string(l), "Next log: %s\n", &file)
		if err != nil {
			continue
		}
		if file != fname1 {
			t.Errorf("%v: Wanted next filename %s, got %s", fname0, fname1, file)
		}
		found = true
	}
	if !found {
		t.Errorf("%v: Next log footer not found", fname0)
	}

	// Check to see if the previous file header is there in the new file
	f1, err := ioutil.ReadFile(fname1)
	if err != nil {
		t.Fatalf("Unable to read file %s: %v", fname1, err)
	}
	found = false
	for _, l := range bytes.Split(f1, []byte("\n")) {
		var file string
		_, err = fmt.Sscanf(string(l), "Previous log: %s\n", &file)
		if err != nil {
			continue
		}
		if file != fname0 {
			t.Errorf("%v: Wanted previous filename %s, got %s", fname1, fname0, file)
		}
		found = true
	}
	if !found {
		t.Errorf("%v: Previous log header not found", fname1)
	}

	// Make sure Names returned the right names.
	n, err := Names("INFO")
	if len(n) != 2 && err != nil && n[0] != fname0 && n[1] != fname1 {
		t.Errorf("Names(INFO) wanted [%s, %s]/nil, got %v/%v", fname0, fname1, n, err)
	}

	if t.Failed() {
		t.Logf("%v:\n%q", fname0, f0)
		t.Logf("%v:\n%q", fname1, f1)
	}
}

func TestLogBacktraceAt(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())
	// The peculiar style of this code simplifies line counting and maintenance of the
	// tracing block below.
	var infoLine string
	setTraceLocation := func(file string, line int, ok bool, delta int) {
		if !ok {
			t.Fatal("could not get file:line")
		}
		_, file = filepath.Split(file)
		infoLine = fmt.Sprintf("%s:%d", file, line+delta)
		err := logBacktraceAt.Set(infoLine)
		if err != nil {
			t.Fatal("error setting log_backtrace_at: ", err)
		}
	}
	{
		// Start of tracing block. These lines know about each other's relative position.
		_, file, line, ok := runtime.Caller(0)
		setTraceLocation(file, line, ok, +2) // Two lines between Caller and Info calls.
		Info("we want a stack trace here")
	}
	numAppearances := strings.Count(contents(logsink.Info), infoLine)
	if numAppearances < 2 {
		// Need 2 appearances, one in the log header and one in the trace:
		//   log_test.go:281: I0511 16:36:06.952398 02238 log_test.go:280] we want a stack trace here
		//   ...
		//   .../glog/glog_test.go:280 (0x41ba91)
		//   ...
		// We could be more precise but that would require knowing the details
		// of the traceback format, which may not be dependable.
		t.Fatal("got no trace back; log is ", contents(logsink.Info))
	}
}

func TestNewStandardLoggerLogBacktraceAt(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())
	s := NewStandardLogger("INFO")
	// The peculiar style of this code simplifies line counting and maintenance of the
	// tracing block below.
	var infoLine string
	setTraceLocation := func(file string, line int, ok bool, delta int) {
		if !ok {
			t.Fatal("could not get file:line")
		}
		_, file = filepath.Split(file)
		infoLine = fmt.Sprintf("%s:%d", file, line+delta)
		err := logBacktraceAt.Set(infoLine)
		if err != nil {
			t.Fatal("error setting log_backtrace_at: ", err)
		}
	}
	{
		// Start of tracing block. These lines know about each other's relative position.
		_, file, line, ok := runtime.Caller(0)
		setTraceLocation(file, line, ok, +2) // Two lines between Caller and Info calls.
		s.Printf("we want a stack trace here")
	}
	infoContents := contents(logsink.Info)
	if strings.Contains(infoContents, infoLine+"] [") {
		t.Fatal("got extra bracketing around log line contents; log is ", infoContents)
	}
	numAppearances := strings.Count(infoContents, infoLine)
	if numAppearances < 2 {
		// Need 2 appearances, one in the log header and one in the trace:
		//   log_test.go:281: I0511 16:36:06.952398 02238 log_test.go:280] we want a stack trace here
		//   ...
		//   .../glog/glog_test.go:280 (0x41ba91)
		//   ...
		// We could be more precise but that would require knowing the details
		// of the traceback format, which may not be dependable.
		t.Fatal("got no trace back; log is ", infoContents)
	}
}

// Test to make sure the log naming function works properly.
func TestLogNames(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())
	n, e := Names("FOO")
	if e == nil {
		t.Errorf("Names(FOO) was %v/nil, should be []/error", n)
	}

	// Set the infoLog to nil to simulate "log not yet written to"
	h := sinks.file.file[logsink.Info]
	sinks.file.file[logsink.Info] = nil
	n, e = Names("INFO")
	if e != ErrNoLog {
		t.Errorf("Names(INFO) was %v/%v, should be [], ErrNoLog", n, e)
	}
	sinks.file.file[logsink.Info] = h

	// Get the name; testing has a fixed fake name for these.
	Info("test")
	n, e = Names("INFO")
	if len(n) != 1 && n[0] != "<local name>" {
		t.Errorf("Names(INFO) got %s, want <local name>", n)
	}
}

func TestLogLength(t *testing.T) {
	setFlags()
	defer sinks.file.swap(sinks.file.newBuffers())
	Info(strings.Repeat("X", logsink.MaxLogMessageLen*2))
	if c := contents(logsink.Info); len(c) != logsink.MaxLogMessageLen {
		t.Errorf("Info was not truncated: got length %d, want %d, contents %q",
			len(c), logsink.MaxLogMessageLen, c)
	}
}
