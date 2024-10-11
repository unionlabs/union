/*Package poll provides tools for testing asynchronous code.
 */
package poll // import "gotest.tools/v3/poll"

import (
	"fmt"
	"strings"
	"time"

	"gotest.tools/v3/assert/cmp"
	"gotest.tools/v3/internal/assert"
)

// TestingT is the subset of [testing.T] used by [WaitOn]
type TestingT interface {
	LogT
	Fatalf(format string, args ...interface{})
}

// LogT is a logging interface that is passed to the [WaitOn] check function
type LogT interface {
	Log(args ...interface{})
	Logf(format string, args ...interface{})
}

type helperT interface {
	Helper()
}

// Settings are used to configure the behaviour of [WaitOn]
type Settings struct {
	// Timeout is the maximum time to wait for the condition. Defaults to 10s.
	Timeout time.Duration
	// Delay is the time to sleep between checking the condition. Defaults to
	// 100ms.
	Delay time.Duration
}

func defaultConfig() *Settings {
	return &Settings{Timeout: 10 * time.Second, Delay: 100 * time.Millisecond}
}

// SettingOp is a function which accepts and modifies Settings
type SettingOp func(config *Settings)

// WithDelay sets the delay to wait between polls
func WithDelay(delay time.Duration) SettingOp {
	return func(config *Settings) {
		config.Delay = delay
	}
}

// WithTimeout sets the timeout
func WithTimeout(timeout time.Duration) SettingOp {
	return func(config *Settings) {
		config.Timeout = timeout
	}
}

// Result of a check performed by [WaitOn]
type Result interface {
	// Error indicates that the check failed and polling should stop, and the
	// the has failed
	Error() error
	// Done indicates that polling should stop, and the test should proceed
	Done() bool
	// Message provides the most recent state when polling has not completed
	Message() string
}

type result struct {
	done    bool
	message string
	err     error
}

func (r result) Done() bool {
	return r.done
}

func (r result) Message() string {
	return r.message
}

func (r result) Error() error {
	return r.err
}

// Continue returns a [Result] that indicates to [WaitOn] that it should continue
// polling. The message text will be used as the failure message if the timeout
// is reached.
func Continue(message string, args ...interface{}) Result {
	return result{message: fmt.Sprintf(message, args...)}
}

// Success returns a [Result] where Done() returns true, which indicates to [WaitOn]
// that it should stop polling and exit without an error.
func Success() Result {
	return result{done: true}
}

// Error returns a [Result] that indicates to [WaitOn] that it should fail the test
// and stop polling.
func Error(err error) Result {
	return result{err: err}
}

// WaitOn a condition or until a timeout. Poll by calling check and exit when
// check returns a done Result. To fail a test and exit polling with an error
// return a error result.
func WaitOn(t TestingT, check Check, pollOps ...SettingOp) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	config := defaultConfig()
	for _, pollOp := range pollOps {
		pollOp(config)
	}

	var lastMessage string
	after := time.After(config.Timeout)
	chResult := make(chan Result)
	for {
		go func() {
			chResult <- check(t)
		}()
		select {
		case <-after:
			if lastMessage == "" {
				lastMessage = "first check never completed"
			}
			t.Fatalf("timeout hit after %s: %s", config.Timeout, lastMessage)
		case result := <-chResult:
			switch {
			case result.Error() != nil:
				t.Fatalf("polling check failed: %s", result.Error())
			case result.Done():
				return
			}
			time.Sleep(config.Delay)
			lastMessage = result.Message()
		}
	}
}

// Compare values using the [cmp.Comparison]. If the comparison fails return a
// result which indicates to WaitOn that it should continue waiting.
// If the comparison is successful then [WaitOn] stops polling.
func Compare(compare cmp.Comparison) Result {
	buf := new(logBuffer)
	if assert.RunComparison(buf, assert.ArgsAtZeroIndex, compare) {
		return Success()
	}
	return Continue(buf.String())
}

type logBuffer struct {
	log [][]interface{}
}

func (c *logBuffer) Log(args ...interface{}) {
	c.log = append(c.log, args)
}

func (c *logBuffer) String() string {
	b := new(strings.Builder)
	for _, item := range c.log {
		b.WriteString(fmt.Sprint(item...) + " ")
	}
	return b.String()
}
