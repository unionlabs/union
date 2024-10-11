package async

import (
	"errors"
	"fmt"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestParallel(t *testing.T) {
	// Create tasks.
	counter := new(int32)
	tasks := make([]Task, 100*1000)
	for i := 0; i < len(tasks); i++ {
		tasks[i] = func(i int) (res any, abort bool, err error) {
			atomic.AddInt32(counter, 1)
			return -1 * i, false, nil
		}
	}

	// Run in parallel.
	trs, ok := Parallel(tasks...)
	assert.True(t, ok)

	// Verify.
	assert.Len(t, tasks, int(*counter), "Each task should have incremented the counter already")
	var failedTasks int
	for i := 0; i < len(tasks); i++ {
		taskResult, ok := trs.LatestResult(i)
		switch {
		case !ok:
			assert.Fail(t, "Task #%v did not complete.", i)
			failedTasks++
		case taskResult.Error != nil:
			assert.Fail(t, "Task should not have errored but got %v", taskResult.Error)
			failedTasks++
		case !assert.Equal(t, -1*i, taskResult.Value.(int)):
			assert.Fail(t, "Task should have returned %v but got %v", -1*i, taskResult.Value.(int))
			failedTasks++
		}
		// else {
		// Good!
		// }
	}
	assert.Equal(t, 0, failedTasks, "No task should have failed")
	require.NoError(t, trs.FirstError(), "There should be no errors")
	assert.Equal(t, 0, trs.FirstValue(), "First value should be 0")
}

func TestParallelAbort(t *testing.T) {
	flow1 := make(chan struct{}, 1)
	flow2 := make(chan struct{}, 1)
	flow3 := make(chan struct{}, 1) // Cap must be > 0 to prevent blocking.
	flow4 := make(chan struct{}, 1)

	// Create tasks.
	tasks := []Task{
		func(i int) (res any, abort bool, err error) {
			assert.Equal(t, 0, i)
			flow1 <- struct{}{}
			return 0, false, nil
		},
		func(i int) (res any, abort bool, err error) {
			assert.Equal(t, 1, i)
			flow2 <- <-flow1
			return 1, false, errors.New("some error")
		},
		func(i int) (res any, abort bool, err error) {
			assert.Equal(t, 2, i)
			flow3 <- <-flow2
			return 2, true, nil
		},
		func(i int) (res any, abort bool, err error) {
			assert.Equal(t, 3, i)
			<-flow4
			return 3, false, nil
		},
	}

	// Run in parallel.
	taskResultSet, ok := Parallel(tasks...)
	assert.False(t, ok, "ok should be false since we aborted task #2.")

	// Verify task #3.
	// Initially taskResultSet.chz[3] sends nothing since flow4 didn't send.
	waitTimeout(t, taskResultSet.chz[3], "Task #3")

	// Now let the last task (#3) complete after abort.
	flow4 <- <-flow3

	// Wait until all tasks have returned or panic'd.
	taskResultSet.Wait()

	// Verify task #0, #1, #2.
	checkResult(t, taskResultSet, 0, 0, nil, nil)
	checkResult(t, taskResultSet, 1, 1, errors.New("some error"), nil)
	checkResult(t, taskResultSet, 2, 2, nil, nil)
	checkResult(t, taskResultSet, 3, 3, nil, nil)
}

func TestParallelRecover(t *testing.T) {
	// Create tasks.
	tasks := []Task{
		func(_ int) (res any, abort bool, err error) {
			return 0, false, nil
		},
		func(_ int) (res any, abort bool, err error) {
			return 1, false, errors.New("some error")
		},
		func(_ int) (res any, abort bool, err error) {
			panic(2)
		},
	}

	// Run in parallel.
	taskResultSet, ok := Parallel(tasks...)
	assert.False(t, ok, "ok should be false since we panic'd in task #2.")

	// Verify task #0, #1, #2.
	checkResult(t, taskResultSet, 0, 0, nil, nil)
	checkResult(t, taskResultSet, 1, 1, errors.New("some error"), nil)
	checkResult(t, taskResultSet, 2, nil, nil, fmt.Errorf("panic in task %v", 2).Error())
}

// Wait for result.
func checkResult(t *testing.T, taskResultSet *TaskResultSet, index int,
	val any, err error, pnk any,
) {
	t.Helper()
	taskResult, ok := taskResultSet.LatestResult(index)
	taskName := fmt.Sprintf("Task #%v", index)
	assert.True(t, ok, "TaskResultCh unexpectedly closed for %v", taskName)
	assert.Equal(t, val, taskResult.Value, taskName)
	switch {
	case err != nil:
		assert.Equal(t, err.Error(), taskResult.Error.Error(), taskName)
	case pnk != nil:
		assert.Contains(t, taskResult.Error.Error(), pnk, taskName)
	default:
		require.NoError(t, taskResult.Error, taskName)
	}
}

// Wait for timeout (no result).
func waitTimeout(t *testing.T, taskResultCh TaskResultCh, taskName string) {
	t.Helper()
	select {
	case _, ok := <-taskResultCh:
		if !ok {
			assert.Fail(t, "TaskResultCh unexpectedly closed (%v)", taskName)
		} else {
			assert.Fail(t, "TaskResultCh unexpectedly returned for %v", taskName)
		}
	case <-time.After(1 * time.Second): // TODO use deterministic time?
		// Good!
	}
}
