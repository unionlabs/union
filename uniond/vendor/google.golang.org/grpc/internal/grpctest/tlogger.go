/*
 *
 * Copyright 2020 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package grpctest

import (
	"errors"
	"fmt"
	"os"
	"path"
	"regexp"
	"runtime"
	"strconv"
	"sync"
	"testing"
	"time"

	"google.golang.org/grpc/grpclog"
)

// TLogger serves as the grpclog logger and is the interface through which
// expected errors are declared in tests.
var TLogger *tLogger

const callingFrame = 4

type logType int

func (l logType) String() string {
	switch l {
	case infoLog:
		return "INFO"
	case warningLog:
		return "WARNING"
	case errorLog:
		return "ERROR"
	case fatalLog:
		return "FATAL"
	}
	return "UNKNOWN"
}

const (
	infoLog logType = iota
	warningLog
	errorLog
	fatalLog
)

type tLogger struct {
	v           int
	initialized bool

	mu     sync.Mutex // guards t, start, and errors
	t      *testing.T
	start  time.Time
	errors map[*regexp.Regexp]int
}

func init() {
	TLogger = &tLogger{errors: map[*regexp.Regexp]int{}}
	vLevel := os.Getenv("GRPC_GO_LOG_VERBOSITY_LEVEL")
	if vl, err := strconv.Atoi(vLevel); err == nil {
		TLogger.v = vl
	}
}

// getCallingPrefix returns the <file:line> at the given depth from the stack.
func getCallingPrefix(depth int) (string, error) {
	_, file, line, ok := runtime.Caller(depth)
	if !ok {
		return "", errors.New("frame request out-of-bounds")
	}
	return fmt.Sprintf("%s:%d", path.Base(file), line), nil
}

// log logs the message with the specified parameters to the tLogger.
func (g *tLogger) log(ltype logType, depth int, format string, args ...any) {
	g.mu.Lock()
	defer g.mu.Unlock()
	prefix, err := getCallingPrefix(callingFrame + depth)
	if err != nil {
		g.t.Error(err)
		return
	}
	args = append([]any{ltype.String() + " " + prefix}, args...)
	args = append(args, fmt.Sprintf(" (t=+%s)", time.Since(g.start)))

	if format == "" {
		switch ltype {
		case errorLog:
			// fmt.Sprintln is used rather than fmt.Sprint because t.Log uses fmt.Sprintln behavior.
			if g.expected(fmt.Sprintln(args...)) {
				g.t.Log(args...)
			} else {
				g.t.Error(args...)
			}
		case fatalLog:
			panic(fmt.Sprint(args...))
		default:
			g.t.Log(args...)
		}
	} else {
		// Add formatting directives for the callingPrefix and timeSuffix.
		format = "%v " + format + "%s"
		switch ltype {
		case errorLog:
			if g.expected(fmt.Sprintf(format, args...)) {
				g.t.Logf(format, args...)
			} else {
				g.t.Errorf(format, args...)
			}
		case fatalLog:
			panic(fmt.Sprintf(format, args...))
		default:
			g.t.Logf(format, args...)
		}
	}
}

// Update updates the testing.T that the testing logger logs to. Should be done
// before every test. It also initializes the tLogger if it has not already.
func (g *tLogger) Update(t *testing.T) {
	g.mu.Lock()
	defer g.mu.Unlock()
	if !g.initialized {
		grpclog.SetLoggerV2(TLogger)
		g.initialized = true
	}
	g.t = t
	g.start = time.Now()
	g.errors = map[*regexp.Regexp]int{}
}

// ExpectError declares an error to be expected. For the next test, the first
// error log matching the expression (using FindString) will not cause the test
// to fail. "For the next test" includes all the time until the next call to
// Update(). Note that if an expected error is not encountered, this will cause
// the test to fail.
func (g *tLogger) ExpectError(expr string) {
	g.ExpectErrorN(expr, 1)
}

// ExpectErrorN declares an error to be expected n times.
func (g *tLogger) ExpectErrorN(expr string, n int) {
	g.mu.Lock()
	defer g.mu.Unlock()
	re, err := regexp.Compile(expr)
	if err != nil {
		g.t.Error(err)
		return
	}
	g.errors[re] += n
}

// EndTest checks if expected errors were not encountered.
func (g *tLogger) EndTest(t *testing.T) {
	g.mu.Lock()
	defer g.mu.Unlock()
	for re, count := range g.errors {
		if count > 0 {
			t.Errorf("Expected error '%v' not encountered", re.String())
		}
	}
	g.errors = map[*regexp.Regexp]int{}
}

// expected determines if the error string is protected or not.
func (g *tLogger) expected(s string) bool {
	for re, count := range g.errors {
		if re.FindStringIndex(s) != nil {
			g.errors[re]--
			if count <= 1 {
				delete(g.errors, re)
			}
			return true
		}
	}
	return false
}

func (g *tLogger) Info(args ...any) {
	g.log(infoLog, 0, "", args...)
}

func (g *tLogger) Infoln(args ...any) {
	g.log(infoLog, 0, "", args...)
}

func (g *tLogger) Infof(format string, args ...any) {
	g.log(infoLog, 0, format, args...)
}

func (g *tLogger) InfoDepth(depth int, args ...any) {
	g.log(infoLog, depth, "", args...)
}

func (g *tLogger) Warning(args ...any) {
	g.log(warningLog, 0, "", args...)
}

func (g *tLogger) Warningln(args ...any) {
	g.log(warningLog, 0, "", args...)
}

func (g *tLogger) Warningf(format string, args ...any) {
	g.log(warningLog, 0, format, args...)
}

func (g *tLogger) WarningDepth(depth int, args ...any) {
	g.log(warningLog, depth, "", args...)
}

func (g *tLogger) Error(args ...any) {
	g.log(errorLog, 0, "", args...)
}

func (g *tLogger) Errorln(args ...any) {
	g.log(errorLog, 0, "", args...)
}

func (g *tLogger) Errorf(format string, args ...any) {
	g.log(errorLog, 0, format, args...)
}

func (g *tLogger) ErrorDepth(depth int, args ...any) {
	g.log(errorLog, depth, "", args...)
}

func (g *tLogger) Fatal(args ...any) {
	g.log(fatalLog, 0, "", args...)
}

func (g *tLogger) Fatalln(args ...any) {
	g.log(fatalLog, 0, "", args...)
}

func (g *tLogger) Fatalf(format string, args ...any) {
	g.log(fatalLog, 0, format, args...)
}

func (g *tLogger) FatalDepth(depth int, args ...any) {
	g.log(fatalLog, depth, "", args...)
}

func (g *tLogger) V(l int) bool {
	return l <= g.v
}
