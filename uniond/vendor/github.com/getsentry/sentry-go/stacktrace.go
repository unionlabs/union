package sentry

import (
	"go/build"
	"reflect"
	"runtime"
	"strings"
)

const unknown string = "unknown"

// The module download is split into two parts: downloading the go.mod and downloading the actual code.
// If you have dependencies only needed for tests, then they will show up in your go.mod,
// and go get will download their go.mods, but it will not download their code.
// The test-only dependencies get downloaded only when you need it, such as the first time you run go test.
//
// https://github.com/golang/go/issues/26913#issuecomment-411976222

// Stacktrace holds information about the frames of the stack.
type Stacktrace struct {
	Frames        []Frame `json:"frames,omitempty"`
	FramesOmitted []uint  `json:"frames_omitted,omitempty"`
}

// NewStacktrace creates a stacktrace using runtime.Callers.
func NewStacktrace() *Stacktrace {
	pcs := make([]uintptr, 100)
	n := runtime.Callers(1, pcs)

	if n == 0 {
		return nil
	}

	runtimeFrames := extractFrames(pcs[:n])
	frames := createFrames(runtimeFrames)

	stacktrace := Stacktrace{
		Frames: frames,
	}

	return &stacktrace
}

// TODO: Make it configurable so that anyone can provide their own implementation?
// Use of reflection allows us to not have a hard dependency on any given
// package, so we don't have to import it.

// ExtractStacktrace creates a new Stacktrace based on the given error.
func ExtractStacktrace(err error) *Stacktrace {
	method := extractReflectedStacktraceMethod(err)

	var pcs []uintptr

	if method.IsValid() {
		pcs = extractPcs(method)
	} else {
		pcs = extractXErrorsPC(err)
	}

	if len(pcs) == 0 {
		return nil
	}

	runtimeFrames := extractFrames(pcs)
	frames := createFrames(runtimeFrames)

	stacktrace := Stacktrace{
		Frames: frames,
	}

	return &stacktrace
}

func extractReflectedStacktraceMethod(err error) reflect.Value {
	errValue := reflect.ValueOf(err)

	// https://github.com/go-errors/errors
	methodStackFrames := errValue.MethodByName("StackFrames")
	if methodStackFrames.IsValid() {
		return methodStackFrames
	}

	// https://github.com/pkg/errors
	methodStackTrace := errValue.MethodByName("StackTrace")
	if methodStackTrace.IsValid() {
		return methodStackTrace
	}

	// https://github.com/pingcap/errors
	methodGetStackTracer := errValue.MethodByName("GetStackTracer")
	if methodGetStackTracer.IsValid() {
		stacktracer := methodGetStackTracer.Call(nil)[0]
		stacktracerStackTrace := reflect.ValueOf(stacktracer).MethodByName("StackTrace")

		if stacktracerStackTrace.IsValid() {
			return stacktracerStackTrace
		}
	}

	return reflect.Value{}
}

func extractPcs(method reflect.Value) []uintptr {
	var pcs []uintptr

	stacktrace := method.Call(nil)[0]

	if stacktrace.Kind() != reflect.Slice {
		return nil
	}

	for i := 0; i < stacktrace.Len(); i++ {
		pc := stacktrace.Index(i)

		switch pc.Kind() {
		case reflect.Uintptr:
			pcs = append(pcs, uintptr(pc.Uint()))
		case reflect.Struct:
			for _, fieldName := range []string{"ProgramCounter", "PC"} {
				field := pc.FieldByName(fieldName)
				if !field.IsValid() {
					continue
				}
				if field.Kind() == reflect.Uintptr {
					pcs = append(pcs, uintptr(field.Uint()))
					break
				}
			}
		}
	}

	return pcs
}

// extractXErrorsPC extracts program counters from error values compatible with
// the error types from golang.org/x/xerrors.
//
// It returns nil if err is not compatible with errors from that package or if
// no program counters are stored in err.
func extractXErrorsPC(err error) []uintptr {
	// This implementation uses the reflect package to avoid a hard dependency
	// on third-party packages.

	// We don't know if err matches the expected type. For simplicity, instead
	// of trying to account for all possible ways things can go wrong, some
	// assumptions are made and if they are violated the code will panic. We
	// recover from any panic and ignore it, returning nil.
	//nolint: errcheck
	defer func() { recover() }()

	field := reflect.ValueOf(err).Elem().FieldByName("frame") // type Frame struct{ frames [3]uintptr }
	field = field.FieldByName("frames")
	field = field.Slice(1, field.Len()) // drop first pc pointing to xerrors.New
	pc := make([]uintptr, field.Len())
	for i := 0; i < field.Len(); i++ {
		pc[i] = uintptr(field.Index(i).Uint())
	}
	return pc
}

// Frame represents a function call and it's metadata. Frames are associated
// with a Stacktrace.
type Frame struct {
	Function string `json:"function,omitempty"`
	Symbol   string `json:"symbol,omitempty"`
	// Module is, despite the name, the Sentry protocol equivalent of a Go
	// package's import path.
	Module      string                 `json:"module,omitempty"`
	Filename    string                 `json:"filename,omitempty"`
	AbsPath     string                 `json:"abs_path,omitempty"`
	Lineno      int                    `json:"lineno,omitempty"`
	Colno       int                    `json:"colno,omitempty"`
	PreContext  []string               `json:"pre_context,omitempty"`
	ContextLine string                 `json:"context_line,omitempty"`
	PostContext []string               `json:"post_context,omitempty"`
	InApp       bool                   `json:"in_app"`
	Vars        map[string]interface{} `json:"vars,omitempty"`
	// Package and the below are not used for Go stack trace frames.  In
	// other platforms it refers to a container where the Module can be
	// found.  For example, a Java JAR, a .NET Assembly, or a native
	// dynamic library.  They exists for completeness, allowing the
	// construction and reporting of custom event payloads.
	Package         string `json:"package,omitempty"`
	InstructionAddr string `json:"instruction_addr,omitempty"`
	AddrMode        string `json:"addr_mode,omitempty"`
	SymbolAddr      string `json:"symbol_addr,omitempty"`
	ImageAddr       string `json:"image_addr,omitempty"`
	Platform        string `json:"platform,omitempty"`
	StackStart      bool   `json:"stack_start,omitempty"`
}

// NewFrame assembles a stacktrace frame out of runtime.Frame.
func NewFrame(f runtime.Frame) Frame {
	function := f.Function
	var pkg string

	if function != "" {
		pkg, function = splitQualifiedFunctionName(function)
	}

	return newFrame(pkg, function, f.File, f.Line)
}

// Like filepath.IsAbs() but doesn't care what platform you run this on.
// I.e. it also recognizies `/path/to/file` when run on Windows.
func isAbsPath(path string) bool {
	if len(path) == 0 {
		return false
	}

	// If the volume name starts with a double slash, this is an absolute path.
	if len(path) >= 1 && (path[0] == '/' || path[0] == '\\') {
		return true
	}

	// Windows absolute path, see https://learn.microsoft.com/en-us/dotnet/standard/io/file-path-formats
	if len(path) >= 3 && path[1] == ':' && (path[2] == '/' || path[2] == '\\') {
		return true
	}

	return false
}

func newFrame(module string, function string, file string, line int) Frame {
	frame := Frame{
		Lineno:   line,
		Module:   module,
		Function: function,
	}

	switch {
	case len(file) == 0:
		frame.Filename = unknown
		// Leave abspath as the empty string to be omitted when serializing event as JSON.
	case isAbsPath(file):
		frame.AbsPath = file
		// TODO: in the general case, it is not trivial to come up with a
		// "project relative" path with the data we have in run time.
		// We shall not use filepath.Base because it creates ambiguous paths and
		// affects the "Suspect Commits" feature.
		// For now, leave relpath empty to be omitted when serializing the event
		// as JSON. Improve this later.
	default:
		// f.File is a relative path. This may happen when the binary is built
		// with the -trimpath flag.
		frame.Filename = file
		// Omit abspath when serializing the event as JSON.
	}

	setInAppFrame(&frame)

	return frame
}

// splitQualifiedFunctionName splits a package path-qualified function name into
// package name and function name. Such qualified names are found in
// runtime.Frame.Function values.
func splitQualifiedFunctionName(name string) (pkg string, fun string) {
	pkg = packageName(name)
	if len(pkg) > 0 {
		fun = name[len(pkg)+1:]
	}
	return
}

func extractFrames(pcs []uintptr) []runtime.Frame {
	var frames = make([]runtime.Frame, 0, len(pcs))
	callersFrames := runtime.CallersFrames(pcs)

	for {
		callerFrame, more := callersFrames.Next()

		frames = append(frames, callerFrame)

		if !more {
			break
		}
	}

	// TODO don't append and reverse, put in the right place from the start.
	// reverse
	for i, j := 0, len(frames)-1; i < j; i, j = i+1, j-1 {
		frames[i], frames[j] = frames[j], frames[i]
	}

	return frames
}

// createFrames creates Frame objects while filtering out frames that are not
// meant to be reported to Sentry, those are frames internal to the SDK or Go.
func createFrames(frames []runtime.Frame) []Frame {
	if len(frames) == 0 {
		return nil
	}

	result := make([]Frame, 0, len(frames))

	for _, frame := range frames {
		function := frame.Function
		var pkg string
		if function != "" {
			pkg, function = splitQualifiedFunctionName(function)
		}

		if !shouldSkipFrame(pkg) {
			result = append(result, newFrame(pkg, function, frame.File, frame.Line))
		}
	}

	return result
}

// TODO ID: why do we want to do this?
// I'm not aware of other SDKs skipping all Sentry frames, regardless of their position in the stactrace.
// For example, in the .NET SDK, only the first frames are skipped until the call to the SDK.
// As is, this will also hide any intermediate frames in the stack and make debugging issues harder.
func shouldSkipFrame(module string) bool {
	// Skip Go internal frames.
	if module == "runtime" || module == "testing" {
		return true
	}

	// Skip Sentry internal frames, except for frames in _test packages (for testing).
	if strings.HasPrefix(module, "github.com/getsentry/sentry-go") &&
		!strings.HasSuffix(module, "_test") {
		return true
	}

	return false
}

// On Windows, GOROOT has backslashes, but we want forward slashes.
var goRoot = strings.ReplaceAll(build.Default.GOROOT, "\\", "/")

func setInAppFrame(frame *Frame) {
	if strings.HasPrefix(frame.AbsPath, goRoot) ||
		strings.Contains(frame.Module, "vendor") ||
		strings.Contains(frame.Module, "third_party") {
		frame.InApp = false
	} else {
		frame.InApp = true
	}
}

func callerFunctionName() string {
	pcs := make([]uintptr, 1)
	runtime.Callers(3, pcs)
	callersFrames := runtime.CallersFrames(pcs)
	callerFrame, _ := callersFrames.Next()
	return baseName(callerFrame.Function)
}

// packageName returns the package part of the symbol name, or the empty string
// if there is none.
// It replicates https://golang.org/pkg/debug/gosym/#Sym.PackageName, avoiding a
// dependency on debug/gosym.
func packageName(name string) string {
	if isCompilerGeneratedSymbol(name) {
		return ""
	}

	pathend := strings.LastIndex(name, "/")
	if pathend < 0 {
		pathend = 0
	}

	if i := strings.Index(name[pathend:], "."); i != -1 {
		return name[:pathend+i]
	}
	return ""
}

// baseName returns the symbol name without the package or receiver name.
// It replicates https://golang.org/pkg/debug/gosym/#Sym.BaseName, avoiding a
// dependency on debug/gosym.
func baseName(name string) string {
	if i := strings.LastIndex(name, "."); i != -1 {
		return name[i+1:]
	}
	return name
}
