// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package testenv

import (
	"context"
	"os"
	"os/exec"
	"reflect"
	"strconv"
	"testing"
	"time"
)

// CommandContext is like exec.CommandContext, but:
//   - skips t if the platform does not support os/exec,
//   - sends SIGQUIT (if supported by the platform) instead of SIGKILL
//     in its Cancel function
//   - if the test has a deadline, adds a Context timeout and WaitDelay
//     for an arbitrary grace period before the test's deadline expires,
//   - fails the test if the command does not complete before the test's deadline, and
//   - sets a Cleanup function that verifies that the test did not leak a subprocess.
func CommandContext(t testing.TB, ctx context.Context, name string, args ...string) *exec.Cmd {
	t.Helper()

	var (
		cancelCtx   context.CancelFunc
		gracePeriod time.Duration // unlimited unless the test has a deadline (to allow for interactive debugging)
	)

	if t, ok := t.(interface {
		testing.TB
		Deadline() (time.Time, bool)
	}); ok {
		if td, ok := t.Deadline(); ok {
			// Start with a minimum grace period, just long enough to consume the
			// output of a reasonable program after it terminates.
			gracePeriod = 100 * time.Millisecond
			if s := os.Getenv("GO_TEST_TIMEOUT_SCALE"); s != "" {
				scale, err := strconv.Atoi(s)
				if err != nil {
					t.Fatalf("invalid GO_TEST_TIMEOUT_SCALE: %v", err)
				}
				gracePeriod *= time.Duration(scale)
			}

			// If time allows, increase the termination grace period to 5% of the
			// test's remaining time.
			testTimeout := time.Until(td)
			if gp := testTimeout / 20; gp > gracePeriod {
				gracePeriod = gp
			}

			// When we run commands that execute subprocesses, we want to reserve two
			// grace periods to clean up: one for the delay between the first
			// termination signal being sent (via the Cancel callback when the Context
			// expires) and the process being forcibly terminated (via the WaitDelay
			// field), and a second one for the delay between the process being
			// terminated and the test logging its output for debugging.
			//
			// (We want to ensure that the test process itself has enough time to
			// log the output before it is also terminated.)
			cmdTimeout := testTimeout - 2*gracePeriod

			if cd, ok := ctx.Deadline(); !ok || time.Until(cd) > cmdTimeout {
				// Either ctx doesn't have a deadline, or its deadline would expire
				// after (or too close before) the test has already timed out.
				// Add a shorter timeout so that the test will produce useful output.
				ctx, cancelCtx = context.WithTimeout(ctx, cmdTimeout)
			}
		}
	}

	cmd := exec.CommandContext(ctx, name, args...)
	// Set the Cancel and WaitDelay fields only if present (go 1.20 and later).
	// TODO: When Go 1.19 is no longer supported, remove this use of reflection
	// and instead set the fields directly.
	if cmdCancel := reflect.ValueOf(cmd).Elem().FieldByName("Cancel"); cmdCancel.IsValid() {
		cmdCancel.Set(reflect.ValueOf(func() error {
			if cancelCtx != nil && ctx.Err() == context.DeadlineExceeded {
				// The command timed out due to running too close to the test's deadline.
				// There is no way the test did that intentionally — it's too close to the
				// wire! — so mark it as a test failure. That way, if the test expects the
				// command to fail for some other reason, it doesn't have to distinguish
				// between that reason and a timeout.
				t.Errorf("test timed out while running command: %v", cmd)
			} else {
				// The command is being terminated due to ctx being canceled, but
				// apparently not due to an explicit test deadline that we added.
				// Log that information in case it is useful for diagnosing a failure,
				// but don't actually fail the test because of it.
				t.Logf("%v: terminating command: %v", ctx.Err(), cmd)
			}
			return cmd.Process.Signal(Sigquit)
		}))
	}
	if cmdWaitDelay := reflect.ValueOf(cmd).Elem().FieldByName("WaitDelay"); cmdWaitDelay.IsValid() {
		cmdWaitDelay.Set(reflect.ValueOf(gracePeriod))
	}

	t.Cleanup(func() {
		if cancelCtx != nil {
			cancelCtx()
		}
		if cmd.Process != nil && cmd.ProcessState == nil {
			t.Errorf("command was started, but test did not wait for it to complete: %v", cmd)
		}
	})

	return cmd
}

// Command is like exec.Command, but applies the same changes as
// testenv.CommandContext (with a default Context).
func Command(t testing.TB, name string, args ...string) *exec.Cmd {
	t.Helper()
	return CommandContext(t, context.Background(), name, args...)
}
