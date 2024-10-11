// stackdump_test checks that the heuristics the stackdump package applies to
// prune frames work as expected in production Go compilers.

package stackdump_test

import (
	"bytes"
	"fmt"
	"regexp"
	"runtime"
	"testing"

	"github.com/golang/glog/internal/stackdump"
)

var file string

func init() {
	_, file, _, _ = runtime.Caller(0)
}

func TestCallerText(t *testing.T) {
	stack := stackdump.CallerText(0)
	_, _, line, _ := runtime.Caller(0)
	line--

	wantRE := regexp.MustCompile(fmt.Sprintf(
		`^goroutine \d+ \[running\]:
github.com/golang/glog/internal/stackdump_test\.TestCallerText(\([^)]*\))?
	%v:%v.*
`, file, line))
	if !wantRE.Match(stack) {
		t.Errorf("Stack dump:\n%s\nwant matching regexp:\n%s", stack, wantRE.String())

		buf := make([]byte, len(stack)*2)
		origStack := buf[:runtime.Stack(buf, false)]
		t.Logf("Unpruned stack:\n%s", origStack)
	}
}

func callerAt(calls int, depth int) (stack []byte) {
	if calls == 1 {
		return stackdump.CallerText(depth)
	}
	return callerAt(calls-1, depth)
}

func TestCallerTextSkip(t *testing.T) {
	const calls = 3
	cases := []struct {
		depth          int
		callerAtFrames int
		wantEndOfStack bool
	}{
		{depth: 0, callerAtFrames: calls},
		{depth: calls - 1, callerAtFrames: 1},
		{depth: calls, callerAtFrames: 0},
		{depth: calls + 1, callerAtFrames: 0},
		{depth: calls + 100, wantEndOfStack: true},
	}

	for _, tc := range cases {
		stack := callerAt(calls, tc.depth)

		wantREBuf := bytes.NewBuffer(nil)
		fmt.Fprintf(wantREBuf, `^goroutine \d+ \[running\]:
`)
		if tc.wantEndOfStack {
			fmt.Fprintf(wantREBuf, "\n|$")
		} else {
			for n := tc.callerAtFrames; n > 0; n-- {
				fmt.Fprintf(wantREBuf, `github.com/golang/glog/internal/stackdump_test\.callerAt(\([^)]*\))?
	%v:\d+.*
`, file)
			}

			if tc.depth <= calls {
				fmt.Fprintf(wantREBuf, `github.com/golang/glog/internal/stackdump_test\.TestCallerTextSkip(\([^)]*\))?
	%v:\d+.*
`, file)
			}
		}

		wantRE := regexp.MustCompile(wantREBuf.String())

		if !wantRE.Match(stack) {
			t.Errorf("for %v calls, stackdump.CallerText(%v) =\n%s\n\nwant matching regexp:\n%s", calls, tc.depth, stack, wantRE.String())
		}
	}
}

func pcAt(calls int, depth int) (stack []uintptr) {
	if calls == 1 {
		return stackdump.CallerPC(depth)
	}
	stack = pcAt(calls-1, depth)
	runtime.Gosched() // Thwart tail-call optimization.
	return stack
}

func TestCallerPC(t *testing.T) {
	const calls = 3
	cases := []struct {
		depth          int
		pcAtFrames     int
		wantEndOfStack bool
	}{
		{depth: 0, pcAtFrames: calls},
		{depth: calls - 1, pcAtFrames: 1},
		{depth: calls, pcAtFrames: 0},
		{depth: calls + 1, pcAtFrames: 0},
		{depth: calls + 100, wantEndOfStack: true},
	}

	for _, tc := range cases {
		stack := pcAt(calls, tc.depth)
		if tc.wantEndOfStack {
			if len(stack) != 0 {
				t.Errorf("for %v calls, stackdump.CallerPC(%v) =\n%q\nwant []", calls, tc.depth, stack)
			}
			continue
		}

		wantFuncs := []string{}
		for n := tc.pcAtFrames; n > 0; n-- {
			wantFuncs = append(wantFuncs, `github.com/golang/glog/internal/stackdump_test\.pcAt$`)
		}
		if tc.depth <= calls {
			wantFuncs = append(wantFuncs, `^github.com/golang/glog/internal/stackdump_test\.TestCallerPC$`)
		}

		gotFuncs := []string{}
		for _, pc := range stack {
			gotFuncs = append(gotFuncs, runtime.FuncForPC(pc).Name())
		}
		if len(gotFuncs) > len(wantFuncs) {
			gotFuncs = gotFuncs[:len(wantFuncs)]
		}

		ok := true
		for i, want := range wantFuncs {
			re := regexp.MustCompile(want)
			if i >= len(gotFuncs) || !re.MatchString(gotFuncs[i]) {
				ok = false
				break
			}
		}
		if !ok {
			t.Errorf("for %v calls, stackdump.CallerPC(%v) =\n%q\nwant %q", calls, tc.depth, gotFuncs, wantFuncs)
		}
	}
}
